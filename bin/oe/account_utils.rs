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

use std::sync::Arc;

use crypto::publickey;
use dir::Directories;
use ethereum_types::{Address, H160};
use ethkey::Password;

use crate::params::{AccountsConfig, SpecType};

#[cfg(not(feature = "accounts"))]
mod accounts {
    use super::*;

    /// Dummy AccountProvider
    pub struct AccountProvider;

    impl ::ethcore::miner::LocalAccounts for AccountProvider {
        fn is_local(&self, _address: &Address) -> bool {
            false
        }
    }

    pub fn prepare_account_provider(
        _spec: &SpecType,
        _dirs: &Directories,
        _data_dir: &str,
        _cfg: AccountsConfig,
        _passwords: &[Password],
    ) -> Result<AccountProvider, String> {
        warn!("Note: Your instance of OpenEthereum is running without account support. Some CLI options are ignored.");
        Ok(AccountProvider)
    }

    pub fn miner_local_accounts(_: Arc<AccountProvider>) -> AccountProvider {
        AccountProvider
    }

    pub fn miner_author(
        _spec: &SpecType,
        _dirs: &Directories,
        _account_provider: &Arc<AccountProvider>,
        _engine_signer: Address,
        _passwords: &[Password],
    ) -> Result<Option<::ethcore::miner::Author>, String> {
        Ok(None)
    }

    pub fn accounts_list(
        _account_provider: Arc<AccountProvider>,
    ) -> Arc<dyn Fn() -> Vec<Address> + Send + Sync> {
        Arc::new(|| vec![])
    }
}

#[cfg(feature = "accounts")]
mod accounts {
    use super::*;
    use crate::{ethereum_types::H256, upgrade::upgrade_key_location};
    use std::str::FromStr;

    pub use crate::accounts::AccountProvider;

    /// Pops along with error messages when a password is missing or invalid.
    const VERIFY_PASSWORD_HINT: &str = "Make sure valid password is present in files passed using `--password` or in the configuration file.";

    /// Initialize account provider
    pub fn prepare_account_provider(
        spec: &SpecType,
        dirs: &Directories,
        data_dir: &str,
        cfg: AccountsConfig,
        passwords: &[Password],
    ) -> Result<AccountProvider, String> {
        use crate::accounts::AccountProviderSettings;
        use ethstore::{accounts_dir::RootDiskDirectory, EthStore};

        let path = dirs.keys_path(data_dir);
        upgrade_key_location(&dirs.legacy_keys_path(cfg.testnet), &path);
        let dir = Box::new(
            RootDiskDirectory::create(&path)
                .map_err(|e| format!("Could not open keys directory: {e}"))?,
        );
        let account_settings = AccountProviderSettings {
            unlock_keep_secret: cfg.enable_fast_unlock,
            blacklisted_accounts: match *spec {
                SpecType::Morden
                | SpecType::Ropsten
                | SpecType::Kovan
                | SpecType::Goerli
                | SpecType::Sokol
                | SpecType::Dev => vec![],
                _ => vec![H160::from_str("00a329c0648769a73afac7f9381e08fb43dbea72")
                    .expect("the string is valid hex; qed")],
            },
        };

        let ethstore = EthStore::open_with_iterations(dir, cfg.iterations)
            .map_err(|e| format!("Could not open keys directory: {e}"))?;
        if cfg.refresh_time > 0 {
            ethstore.set_refresh_time(::std::time::Duration::from_secs(cfg.refresh_time));
        }
        let account_provider = AccountProvider::new(Box::new(ethstore), account_settings);

        // Add development account if running dev chain:
        if let SpecType::Dev = *spec {
            insert_dev_account(&account_provider);
        }

        for a in cfg.unlocked_accounts {
            // Check if the account exists
            if !account_provider.has_account(a) {
                return Err(format!(
                    "Account {} not found for the current chain. {}",
                    a,
                    build_create_account_hint(spec, &dirs.keys)
                ));
            }

            // Check if any passwords have been read from the password file(s)
            if passwords.is_empty() {
                return Err(format!(
                    "No password found to unlock account {a}. {VERIFY_PASSWORD_HINT}"
                ));
            }

            if !passwords.iter().any(|p| {
                account_provider
                    .unlock_account_permanently(a, (*p).clone())
                    .is_ok()
            }) {
                return Err(format!(
                    "No valid password to unlock account {a}. {VERIFY_PASSWORD_HINT}"
                ));
            }
        }

        Ok(account_provider)
    }

    pub struct LocalAccounts(Arc<AccountProvider>);
    impl ::ethcore::miner::LocalAccounts for LocalAccounts {
        fn is_local(&self, address: &Address) -> bool {
            self.0.has_account(*address)
        }
    }

    pub fn miner_local_accounts(account_provider: Arc<AccountProvider>) -> LocalAccounts {
        LocalAccounts(account_provider)
    }

    pub fn miner_author(
        spec: &SpecType,
        dirs: &Directories,
        account_provider: &Arc<AccountProvider>,
        engine_signer: Address,
        passwords: &[Password],
    ) -> Result<Option<::ethcore::miner::Author>, String> {
        use ethcore::engines::EngineSigner;

        // Check if engine signer exists
        if !account_provider.has_account(engine_signer) {
            return Err(format!(
                "Consensus signer account not found for the current chain. {}",
                build_create_account_hint(spec, &dirs.keys)
            ));
        }

        // Check if any passwords have been read from the password file(s)
        if passwords.is_empty() {
            return Err(format!(
                "No password found for the consensus signer {engine_signer}. {VERIFY_PASSWORD_HINT}"
            ));
        }

        let mut author = None;
        for password in passwords {
            let signer = parity_rpc::signer::EngineSigner::new(
                account_provider.clone(),
                engine_signer,
                password.clone(),
            );
            // sign dummy msg to check if password and account can be used.
            if signer.sign(H256::from_low_u64_be(1)).is_ok() {
                author = Some(::ethcore::miner::Author::Sealer(Box::new(signer)));
            }
        }
        if author.is_none() {
            return Err(format!(
                "No valid password for the consensus signer {engine_signer}. {VERIFY_PASSWORD_HINT}"
            ));
        }

        Ok(author)
    }

    pub fn accounts_list(
        account_provider: Arc<AccountProvider>,
    ) -> Arc<dyn Fn() -> Vec<Address> + Send + Sync> {
        Arc::new(move || account_provider.accounts().unwrap_or_default())
    }

    fn insert_dev_account(account_provider: &AccountProvider) {
        let secret = publickey::Secret::copy_from_str(
            "4d5db4107d237df6a3d58ee5f70ae63d73d7658d4026f2eefd2f204c81682cb7",
        )
        .expect("Valid account;qed");
        let dev_account = publickey::KeyPair::from_secret(secret.clone())
            .expect("Valid secret produces valid key;qed");
        if !account_provider.has_account(dev_account.address()) {
            match account_provider.insert_account(secret, &Password::from(String::new())) {
                Err(e) => warn!("Unable to add development account: {e}"),
                Ok(address) => {
                    let _ =
                        account_provider.set_account_name(address, "Development Account".into());
                    let _ = account_provider.set_account_meta(
                        address,
                        ::serde_json::to_string(
                            &(vec![
                                (
                                    "description",
                                    "Never use this account outside of development chain!",
                                ),
                                ("passwordHint", "Password is empty string"),
                            ]
                            .into_iter()
                            .collect::<::std::collections::HashMap<_, _>>()),
                        )
                        .expect("Serialization of hashmap does not fail."),
                    );
                }
            }
        }
    }

    // Construct an error `String` with an adaptive hint on how to create an account.
    fn build_create_account_hint(spec: &SpecType, keys: &str) -> String {
        format!("You can create an account via RPC, UI or `openethereum account new --chain {spec} --keys-path {keys}`.")
    }
}

pub use self::accounts::{
    accounts_list, miner_author, miner_local_accounts, prepare_account_provider, AccountProvider,
};
