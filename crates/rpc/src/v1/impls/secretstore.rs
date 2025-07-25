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

//! SecretStore-specific rpc implementation.

use std::{collections::BTreeSet, sync::Arc};

use accounts::AccountProvider;
use crypto::{publickey::Secret, DEFAULT_MAC};
use ethereum_types::{H160, H256, H512};

use ethkey::Password;
use jsonrpc_core::Result;
use v1::{
    helpers::{
        errors,
        secretstore::{
            decrypt_document, decrypt_document_with_shadow, encrypt_document,
            generate_document_key, ordered_servers_keccak,
        },
    },
    traits::SecretStore,
    types::{Bytes, EncryptedDocumentKey},
};

/// Parity implementation.
pub struct SecretStoreClient {
    accounts: Arc<AccountProvider>,
}

impl SecretStoreClient {
    /// Creates new `SecretStoreClient`
    pub fn new(store: &Arc<AccountProvider>) -> Self {
        SecretStoreClient {
            accounts: store.clone(),
        }
    }

    /// Decrypt public key using account' private key
    fn decrypt_key(&self, address: H160, password: Password, key: Bytes) -> Result<Vec<u8>> {
        self.accounts
            .decrypt(address, Some(password), &DEFAULT_MAC, &key.0)
            .map_err(|e| errors::account("Could not decrypt key.", e))
    }

    /// Decrypt secret key using account' private key
    fn decrypt_secret(&self, address: H160, password: Password, key: Bytes) -> Result<Secret> {
        self.decrypt_key(address, password, key)
            .and_then(|s| Secret::import_key(&s).map_err(|e| errors::account("invalid secret", e)))
    }
}

impl SecretStore for SecretStoreClient {
    fn generate_document_key(
        &self,
        address: H160,
        password: Password,
        server_key_public: H512,
    ) -> Result<EncryptedDocumentKey> {
        let account_public = self
            .accounts
            .account_public(address, &password)
            .map_err(|e| errors::account("Could not read account public.", e))?;
        generate_document_key(account_public, server_key_public)
    }

    fn encrypt(&self, address: H160, password: Password, key: Bytes, data: Bytes) -> Result<Bytes> {
        encrypt_document(self.decrypt_key(address, password, key)?, data.0).map(Into::into)
    }

    fn decrypt(&self, address: H160, password: Password, key: Bytes, data: Bytes) -> Result<Bytes> {
        decrypt_document(self.decrypt_key(address, password, key)?, data.0).map(Into::into)
    }

    fn shadow_decrypt(
        &self,
        address: H160,
        password: Password,
        decrypted_secret: H512,
        common_point: H512,
        decrypt_shadows: Vec<Bytes>,
        data: Bytes,
    ) -> Result<Bytes> {
        let mut shadows = Vec::with_capacity(decrypt_shadows.len());
        for decrypt_shadow in decrypt_shadows {
            shadows.push(self.decrypt_secret(address, password.clone(), decrypt_shadow)?);
        }

        decrypt_document_with_shadow(decrypted_secret, common_point, shadows, data.0)
            .map(Into::into)
    }

    fn servers_set_hash(&self, servers_set: BTreeSet<H512>) -> Result<H256> {
        Ok(ordered_servers_keccak(servers_set))
    }

    fn sign_raw_hash(&self, address: H160, password: Password, raw_hash: H256) -> Result<Bytes> {
        self.accounts
            .sign(address, Some(password), raw_hash)
            .map(|s| Bytes::new((*s).to_vec()))
            .map_err(|e| errors::account("Could not sign raw hash.", e))
    }
}
