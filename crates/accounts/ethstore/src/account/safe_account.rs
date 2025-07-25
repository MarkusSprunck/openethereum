// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of OpenEthereum.

// OpenEthereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// OpenEthereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with OpenEthereum.  If not, see <http://www.gnu.org/licenses/>.

use super::crypto::Crypto;
use account::Version;
use crypto::{
    self,
    publickey::{ecdh::agree, sign, Address, KeyPair, Message, Public, Secret, Signature},
};
use ethkey::Password;
use json;
use std::num::NonZeroU32;
use Error;

/// Account representation.
#[derive(Debug, PartialEq, Clone)]
pub struct SafeAccount {
    /// Account ID
    pub id: [u8; 16],
    /// Account version
    pub version: Version,
    /// Account address
    pub address: Address,
    /// Account private key derivation definition.
    pub crypto: Crypto,
    /// Account filename
    pub filename: Option<String>,
    /// Account name
    pub name: String,
    /// Account metadata
    pub meta: String,
}

impl From<SafeAccount> for json::KeyFile {
    fn from(val: SafeAccount) -> Self {
        json::KeyFile {
            id: From::from(val.id),
            version: val.version.into(),
            address: Some(val.address.into()),
            crypto: val.crypto.into(),
            name: Some(val.name),
            meta: Some(val.meta),
        }
    }
}

impl SafeAccount {
    /// Create a new account
    pub fn create(
        keypair: &KeyPair,
        id: [u8; 16],
        password: &Password,
        iterations: NonZeroU32,
        name: String,
        meta: String,
    ) -> Result<Self, crypto::Error> {
        Ok(SafeAccount {
            id,
            version: Version::V3,
            crypto: Crypto::with_secret(keypair.secret(), password, iterations)?,
            address: keypair.address(),
            filename: None,
            name,
            meta,
        })
    }

    /// Create a new `SafeAccount` from the given `json`; if it was read from a
    /// file, the `filename` should be `Some` name. If it is as yet anonymous, then it
    /// can be left `None`.
    /// In case `password` is provided, we will attempt to read the secret from the keyfile
    /// and derive the address from it instead of reading it directly.
    /// Providing password is required for `json::KeyFile`s with no address.
    pub fn from_file(
        json: json::KeyFile,
        filename: Option<String>,
        password: &Option<Password>,
    ) -> Result<Self, Error> {
        let crypto = Crypto::from(json.crypto);
        let address = match (password, &json.address) {
            (None, Some(json_address)) => json_address.into(),
            (None, None) => Err(Error::Custom(
                "This keystore does not contain address. You need to provide password to import it"
                    .into(),
            ))?,
            (Some(password), json_address) => {
                let derived_address = KeyPair::from_secret(
                    crypto
                        .secret(password)
                        .map_err(|_| Error::InvalidPassword)?,
                )?
                .address();

                if let Some(json_address) = json_address {
                    let json_address = json_address.into();
                    if derived_address != json_address {
                        warn!("Detected address mismatch when opening an account. Derived: {derived_address:?}, in json got: {json_address:?}");
                    }
                }
                derived_address
            }
        };

        Ok(SafeAccount {
            id: json.id.into(),
            version: json.version.into(),
            address,
            crypto,
            filename,
            name: json.name.unwrap_or_default(),
            meta: json.meta.unwrap_or("{}".to_owned()),
        })
    }

    /// Create a new `SafeAccount` from the given vault `json`; if it was read from a
    /// file, the `filename` should be `Some` name. If it is as yet anonymous, then it
    /// can be left `None`.
    pub fn from_vault_file(
        password: &Password,
        json: json::VaultKeyFile,
        filename: Option<String>,
    ) -> Result<Self, Error> {
        let meta_crypto: Crypto = json.metacrypto.into();
        let meta_plain = meta_crypto.decrypt(password)?;
        let meta_plain =
            json::VaultKeyMeta::load(&meta_plain).map_err(|e| Error::Custom(format!("{e:?}")))?;

        SafeAccount::from_file(
            json::KeyFile {
                id: json.id,
                version: json.version,
                crypto: json.crypto,
                address: Some(meta_plain.address),
                name: meta_plain.name,
                meta: meta_plain.meta,
            },
            filename,
            &None,
        )
    }

    /// Create a new `VaultKeyFile` from the given `self`
    pub fn into_vault_file(
        self,
        iterations: NonZeroU32,
        password: &Password,
    ) -> Result<json::VaultKeyFile, Error> {
        let meta_plain = json::VaultKeyMeta {
            address: self.address.into(),
            name: Some(self.name),
            meta: Some(self.meta),
        };
        let meta_plain = meta_plain
            .write()
            .map_err(|e| Error::Custom(format!("{e:?}")))?;
        let meta_crypto = Crypto::with_plain(&meta_plain, password, iterations)?;

        Ok(json::VaultKeyFile {
            id: self.id.into(),
            version: self.version.into(),
            crypto: self.crypto.into(),
            metacrypto: meta_crypto.into(),
        })
    }

    /// Sign a message.
    pub fn sign(&self, password: &Password, message: &Message) -> Result<Signature, Error> {
        let secret = self.crypto.secret(password)?;
        sign(&secret, message).map_err(From::from)
    }

    /// Decrypt a message.
    pub fn decrypt(
        &self,
        password: &Password,
        shared_mac: &[u8],
        message: &[u8],
    ) -> Result<Vec<u8>, Error> {
        let secret = self.crypto.secret(password)?;
        crypto::publickey::ecies::decrypt(&secret, shared_mac, message).map_err(From::from)
    }

    /// Agree on shared key.
    pub fn agree(&self, password: &Password, other: &Public) -> Result<Secret, Error> {
        let secret = self.crypto.secret(password)?;
        agree(&secret, other).map_err(From::from)
    }

    /// Derive public key.
    pub fn public(&self, password: &Password) -> Result<Public, Error> {
        let secret = self.crypto.secret(password)?;
        Ok(*KeyPair::from_secret(secret)?.public())
    }

    /// Change account's password.
    pub fn change_password(
        &self,
        old_password: &Password,
        new_password: &Password,
        iterations: NonZeroU32,
    ) -> Result<Self, Error> {
        let secret = self.crypto.secret(old_password)?;
        let result = SafeAccount {
            id: self.id,
            version: self.version.clone(),
            crypto: Crypto::with_secret(&secret, new_password, iterations)?,
            address: self.address,
            filename: self.filename.clone(),
            name: self.name.clone(),
            meta: self.meta.clone(),
        };
        Ok(result)
    }

    /// Check if password matches the account.
    pub fn check_password(&self, password: &Password) -> bool {
        self.crypto.secret(password).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::{NonZeroU32, SafeAccount};
    use crypto::publickey::{verify_public, Generator, Random};

    lazy_static! {
        static ref ITERATIONS: NonZeroU32 = NonZeroU32::new(10240).expect("10240 > 0; qed");
    }

    #[test]
    fn sign_and_verify_public() {
        let keypair = Random.generate();
        let password = "hello world".into();
        let message = [1u8; 32].into();
        let account = SafeAccount::create(
            &keypair,
            [0u8; 16],
            &password,
            *ITERATIONS,
            "Test".to_owned(),
            "{}".to_owned(),
        );
        let signature = account.unwrap().sign(&password, &message).unwrap();
        assert!(verify_public(keypair.public(), &signature, &message).unwrap());
    }

    #[test]
    fn change_password() {
        let keypair = Random.generate();
        let first_password = "hello world".into();
        let sec_password = "this is sparta".into();
        let message = [1u8; 32].into();
        let account = SafeAccount::create(
            &keypair,
            [0u8; 16],
            &first_password,
            *ITERATIONS,
            "Test".to_owned(),
            "{}".to_owned(),
        )
        .unwrap();
        let new_account = account
            .change_password(&first_password, &sec_password, *ITERATIONS)
            .unwrap();
        assert!(account.sign(&first_password, &message).is_ok());
        assert!(account.sign(&sec_password, &message).is_err());
        assert!(new_account.sign(&first_password, &message).is_err());
        assert!(new_account.sign(&sec_password, &message).is_ok());
    }
}
