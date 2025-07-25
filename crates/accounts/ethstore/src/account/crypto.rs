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

use account::{Aes128Ctr, Cipher, Kdf, Pbkdf2, Prf};
use crypto::{self, publickey::Secret, Keccak256};
use ethkey::Password;
use json;
use random::Random;
use smallvec::SmallVec;
use std::{num::NonZeroU32, str};
use Error;

/// Encrypted data
#[derive(Debug, PartialEq, Clone)]
pub struct Crypto {
    /// Encryption parameters
    pub cipher: Cipher,
    /// Encrypted data buffer
    pub ciphertext: Vec<u8>,
    /// Key derivation function parameters
    pub kdf: Kdf,
    /// Message authentication code
    pub mac: [u8; 32],
}

impl From<json::Crypto> for Crypto {
    fn from(json: json::Crypto) -> Self {
        Crypto {
            cipher: json.cipher.into(),
            ciphertext: json.ciphertext.into(),
            kdf: json.kdf.into(),
            mac: json.mac.into(),
        }
    }
}

impl From<Crypto> for json::Crypto {
    fn from(c: Crypto) -> Self {
        json::Crypto {
            cipher: c.cipher.into(),
            ciphertext: c.ciphertext.into(),
            kdf: c.kdf.into(),
            mac: c.mac.into(),
        }
    }
}

impl str::FromStr for Crypto {
    type Err = <json::Crypto as str::FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<json::Crypto>().map(Into::into)
    }
}

impl From<Crypto> for String {
    fn from(c: Crypto) -> Self {
        json::Crypto::from(c).into()
    }
}

impl Crypto {
    /// Encrypt account secret
    pub fn with_secret(
        secret: &Secret,
        password: &Password,
        iterations: NonZeroU32,
    ) -> Result<Self, crypto::Error> {
        Crypto::with_plain(secret.as_bytes(), password, iterations)
    }

    /// Encrypt custom plain data
    pub fn with_plain(
        plain: &[u8],
        password: &Password,
        iterations: NonZeroU32,
    ) -> Result<Self, crypto::Error> {
        let salt: [u8; 32] = Random::random();
        let iv: [u8; 16] = Random::random();

        // two parts of derived key
        // DK = [ DK[0..15] DK[16..31] ] = [derived_left_bits, derived_right_bits]
        let (derived_left_bits, derived_right_bits) =
            crypto::derive_key_iterations(password.as_bytes(), &salt, iterations.get());

        // preallocated (on-stack in case of `Secret`) buffer to hold cipher
        // length = length(plain) as we are using CTR-approach
        let plain_len = plain.len();
        let mut ciphertext: SmallVec<[u8; 32]> = SmallVec::from_vec(vec![0; plain_len]);

        // aes-128-ctr with initial vector of iv
        crypto::aes::encrypt_128_ctr(&derived_left_bits, &iv, plain, &mut ciphertext)?;

        // KECCAK(DK[16..31] ++ <ciphertext>), where DK[16..31] - derived_right_bits
        let mac = crypto::derive_mac(&derived_right_bits, &ciphertext).keccak256();

        Ok(Crypto {
            cipher: Cipher::Aes128Ctr(Aes128Ctr { iv }),
            ciphertext: ciphertext.into_vec(),
            kdf: Kdf::Pbkdf2(Pbkdf2 {
                dklen: crypto::KEY_LENGTH as u32,
                salt: salt.to_vec(),
                c: iterations,
                prf: Prf::HmacSha256,
            }),
            mac,
        })
    }

    /// Try to decrypt and convert result to account secret
    pub fn secret(&self, password: &Password) -> Result<Secret, Error> {
        if self.ciphertext.len() > 32 {
            return Err(Error::InvalidSecret);
        }

        let secret = self.do_decrypt(password, 32)?;
        Ok(Secret::import_key(&secret)?)
    }

    /// Try to decrypt and return result as is
    pub fn decrypt(&self, password: &Password) -> Result<Vec<u8>, Error> {
        let expected_len = self.ciphertext.len();
        self.do_decrypt(password, expected_len)
    }

    fn do_decrypt(&self, password: &Password, expected_len: usize) -> Result<Vec<u8>, Error> {
        let (derived_left_bits, derived_right_bits) = match self.kdf {
            Kdf::Pbkdf2(ref params) => {
                crypto::derive_key_iterations(password.as_bytes(), &params.salt, params.c.get())
            }
            Kdf::Scrypt(ref params) => crypto::scrypt::derive_key(
                password.as_bytes(),
                &params.salt,
                params.n,
                params.p,
                params.r,
            )?,
        };

        let mac = crypto::derive_mac(&derived_right_bits, &self.ciphertext).keccak256();

        if !crypto::is_equal(&mac, &self.mac) {
            return Err(Error::InvalidPassword);
        }

        let mut plain: SmallVec<[u8; 32]> = SmallVec::from_vec(vec![0; expected_len]);

        match self.cipher {
            Cipher::Aes128Ctr(ref params) => {
                // checker by callers
                debug_assert!(expected_len >= self.ciphertext.len());

                let from = expected_len - self.ciphertext.len();
                crypto::aes::decrypt_128_ctr(
                    &derived_left_bits,
                    &params.iv,
                    &self.ciphertext,
                    &mut plain[from..],
                )?;
                Ok(plain.into_iter().collect())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Crypto, Error, NonZeroU32};
    use crypto::publickey::{Generator, Random};

    lazy_static! {
        static ref ITERATIONS: NonZeroU32 = NonZeroU32::new(10240).expect("10240 > 0; qed");
    }

    #[test]
    fn crypto_with_secret_create() {
        let keypair = Random.generate();
        let passwd = "this is sparta".into();
        let crypto = Crypto::with_secret(keypair.secret(), &passwd, *ITERATIONS).unwrap();
        let secret = crypto.secret(&passwd).unwrap();
        assert_eq!(keypair.secret(), &secret);
    }

    #[test]
    fn crypto_with_secret_invalid_password() {
        let keypair = Random.generate();
        let crypto =
            Crypto::with_secret(keypair.secret(), &"this is sparta".into(), *ITERATIONS).unwrap();
        assert_matches!(
            crypto.secret(&"this is sparta!".into()),
            Err(Error::InvalidPassword)
        )
    }

    #[test]
    fn crypto_with_null_plain_data() {
        let original_data = b"";
        let passwd = "this is sparta".into();
        let crypto = Crypto::with_plain(&original_data[..], &passwd, *ITERATIONS).unwrap();
        let decrypted_data = crypto.decrypt(&passwd).unwrap();
        assert_eq!(original_data[..], *decrypted_data);
    }

    #[test]
    fn crypto_with_tiny_plain_data() {
        let original_data = b"{}";
        let passwd = "this is sparta".into();
        let crypto = Crypto::with_plain(&original_data[..], &passwd, *ITERATIONS).unwrap();
        let decrypted_data = crypto.decrypt(&passwd).unwrap();
        assert_eq!(original_data[..], *decrypted_data);
    }

    #[test]
    fn crypto_with_huge_plain_data() {
        let original_data: Vec<_> = (1..65536).map(|i| (i % 256) as u8).collect();
        let passwd = "this is sparta".into();
        let crypto = Crypto::with_plain(&original_data, &passwd, *ITERATIONS).unwrap();
        let decrypted_data = crypto.decrypt(&passwd).unwrap();
        assert_eq!(&original_data, &decrypted_data);
    }
}
