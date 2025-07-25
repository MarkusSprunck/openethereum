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

#![allow(dead_code)]
#![allow(unused_variables)]

use std::{
    cmp::PartialEq,
    collections::{BTreeMap, HashSet},
    str::FromStr,
    sync::Arc,
};

pub use parity_rpc::signer::SignerService;

use crate::{
    account_utils::{self, AccountProvider},
    miner::external::ExternalMiner,
    sync::{ManageNetwork, SyncProvider},
};
use ethcore::{client::Client, miner::Miner, snapshot::SnapshotService};
use ethcore_logger::RotatingLogger;
use fetch::Client as FetchClient;
use jsonrpc_core::{self as core, MetaIoHandler};
use parity_rpc::{
    dispatch::FullDispatcher,
    informant::{ActivityNotifier, ClientNotifier},
    Host, Metadata, NetworkSettings,
};
use parity_runtime::Executor;
use parking_lot::Mutex;

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum Api {
    /// Web3 (Safe)
    Web3,
    /// Net (Safe)
    Net,
    /// Eth (Safe)
    Eth,
    /// Eth Pub-Sub (Safe)
    EthPubSub,
    /// Geth-compatible "personal" API (DEPRECATED; only used in `--geth` mode.)
    Personal,
    /// Signer - Confirm transactions in Signer (UNSAFE: Passwords, List of transactions)
    Signer,
    /// Parity - Custom extensions (Safe)
    Parity,
    /// Traces (Safe)
    Traces,
    /// Rpc (Safe)
    Rpc,
    /// Parity PubSub - Generic Publish-Subscriber (Safety depends on other APIs exposed).
    ParityPubSub,
    /// Parity Accounts extensions (UNSAFE: Passwords, Side Effects (new account))
    ParityAccounts,
    /// Parity - Set methods (UNSAFE: Side Effects affecting node operation)
    ParitySet,
    /// SecretStore (UNSAFE: arbitrary hash signing)
    SecretStore,
    /// Geth-compatible (best-effort) debug API (Potentially UNSAFE)
    /// NOTE We don't aim to support all methods, only the ones that are useful.
    Debug,
}

impl FromStr for Api {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use self::Api::*;

        match s {
            "debug" => Ok(Debug),
            "eth" => Ok(Eth),
            "net" => Ok(Net),
            "parity" => Ok(Parity),
            "parity_accounts" => Ok(ParityAccounts),
            "parity_pubsub" => Ok(ParityPubSub),
            "parity_set" => Ok(ParitySet),
            "personal" => Ok(Personal),
            "pubsub" => Ok(EthPubSub),
            "rpc" => Ok(Rpc),
            "secretstore" => Ok(SecretStore),
            "signer" => Ok(Signer),
            "traces" => Ok(Traces),
            "web3" => Ok(Web3),
            api => Err(format!("Unknown api: {api}")),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub enum ApiSet {
    // Unsafe context (like jsonrpc over http)
    #[default]
    UnsafeContext,
    // All possible APIs (safe context like token-protected WS interface)
    All,
    // Local "unsafe" context and accounts access
    IpcContext,
    // APIs for Parity Generic Pub-Sub
    PubSub,
    // Fixed list of APis
    List(HashSet<Api>),
}

impl PartialEq for ApiSet {
    fn eq(&self, other: &Self) -> bool {
        self.list_apis() == other.list_apis()
    }
}

impl FromStr for ApiSet {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut apis = HashSet::new();

        for api in s.split(',') {
            match api {
                "all" => {
                    apis.extend(ApiSet::All.list_apis());
                }
                "safe" => {
                    // Safe APIs are those that are safe even in UnsafeContext.
                    apis.extend(ApiSet::UnsafeContext.list_apis());
                }
                // Remove the API
                api if api.starts_with("-") => {
                    let api = api[1..].parse()?;
                    apis.remove(&api);
                }
                api => {
                    let api = api.parse()?;
                    apis.insert(api);
                }
            }
        }

        Ok(ApiSet::List(apis))
    }
}

fn to_modules(apis: &HashSet<Api>) -> BTreeMap<String, String> {
    let mut modules = BTreeMap::new();
    for api in apis {
        let (name, version) = match *api {
            Api::Debug => ("debug", "1.0"),
            Api::Eth => ("eth", "1.0"),
            Api::EthPubSub => ("pubsub", "1.0"),
            Api::Net => ("net", "1.0"),
            Api::Parity => ("parity", "1.0"),
            Api::ParityAccounts => ("parity_accounts", "1.0"),
            Api::ParityPubSub => ("parity_pubsub", "1.0"),
            Api::ParitySet => ("parity_set", "1.0"),
            Api::Personal => ("personal", "1.0"),
            Api::Rpc => ("rpc", "1.0"),
            Api::SecretStore => ("secretstore", "1.0"),
            Api::Signer => ("signer", "1.0"),
            Api::Traces => ("traces", "1.0"),
            Api::Web3 => ("web3", "1.0"),
        };
        modules.insert(name.into(), version.into());
    }
    modules
}

macro_rules! add_signing_methods {
    ($namespace:ident, $handler:expr, $deps:expr, $dispatch:expr) => {{
        let deps = &$deps;
        let (dispatcher, accounts) = $dispatch;
        if deps.signer_service.is_enabled() {
            $handler.extend_with($namespace::to_delegate(SigningQueueClient::new(
                &deps.signer_service,
                dispatcher.clone(),
                deps.executor.clone(),
                accounts,
            )))
        } else {
            $handler.extend_with($namespace::to_delegate(SigningUnsafeClient::new(
                accounts,
                dispatcher.clone(),
            )))
        }
    }};
}

/// RPC dependencies can be used to initialize RPC endpoints from APIs.
pub trait Dependencies {
    type Notifier: ActivityNotifier;

    /// Create the activity notifier.
    fn activity_notifier(&self) -> Self::Notifier;

    /// Extend the given I/O handler with endpoints for each API.
    fn extend_with_set<S>(&self, handler: &mut MetaIoHandler<Metadata, S>, apis: &HashSet<Api>)
    where
        S: core::Middleware<Metadata>;
}

/// RPC dependencies for a full node.
pub struct FullDependencies {
    pub signer_service: Arc<SignerService>,
    pub client: Arc<Client>,
    pub snapshot: Arc<dyn SnapshotService>,
    pub sync: Arc<dyn SyncProvider>,
    pub net: Arc<dyn ManageNetwork>,
    pub accounts: Arc<AccountProvider>,
    pub miner: Arc<Miner>,
    pub external_miner: Arc<ExternalMiner>,
    pub logger: Arc<RotatingLogger>,
    pub settings: Arc<NetworkSettings>,
    pub net_service: Arc<dyn ManageNetwork>,
    pub experimental_rpcs: bool,
    pub ws_address: Option<Host>,
    pub fetch: FetchClient,
    pub executor: Executor,
    pub gas_price_percentile: usize,
    pub poll_lifetime: u32,
    pub allow_missing_blocks: bool,
    pub no_ancient_blocks: bool,
}

impl FullDependencies {
    fn extend_api<S>(
        &self,
        handler: &mut MetaIoHandler<Metadata, S>,
        apis: &HashSet<Api>,
        for_generic_pubsub: bool,
    ) where
        S: core::Middleware<Metadata>,
    {
        use parity_rpc::v1::*;

        let nonces = Arc::new(Mutex::new(dispatch::Reservations::new(
            self.executor.clone(),
        )));
        let dispatcher = FullDispatcher::new(
            self.client.clone(),
            self.miner.clone(),
            nonces.clone(),
            self.gas_price_percentile,
        );
        let account_signer = Arc::new(dispatch::Signer::new(self.accounts.clone())) as _;
        let accounts = account_utils::accounts_list(self.accounts.clone());

        for api in apis {
            match *api {
                Api::Debug => {
                    handler.extend_with(DebugClient::new(self.client.clone()).to_delegate());
                }
                Api::Web3 => {
                    handler.extend_with(Web3Client.to_delegate());
                }
                Api::Net => {
                    handler.extend_with(NetClient::new(&self.sync).to_delegate());
                }
                Api::Eth => {
                    let client = EthClient::new(
                        &self.client,
                        &self.snapshot,
                        &self.sync,
                        &accounts,
                        &self.miner,
                        &self.external_miner,
                        EthClientOptions {
                            gas_price_percentile: self.gas_price_percentile,
                            allow_missing_blocks: self.allow_missing_blocks,
                            allow_experimental_rpcs: self.experimental_rpcs,
                            no_ancient_blocks: self.no_ancient_blocks,
                        },
                    );
                    handler.extend_with(client.to_delegate());

                    if !for_generic_pubsub {
                        let filter_client = EthFilterClient::new(
                            self.client.clone(),
                            self.miner.clone(),
                            self.poll_lifetime,
                        );
                        handler.extend_with(filter_client.to_delegate());

                        add_signing_methods!(
                            EthSigning,
                            handler,
                            self,
                            (&dispatcher, &account_signer)
                        );
                    }
                }
                Api::EthPubSub => {
                    if !for_generic_pubsub {
                        let client =
                            EthPubSubClient::new(self.client.clone(), self.executor.clone());
                        let h = client.handler();
                        self.miner
                            .add_transactions_listener(Box::new(move |hashes| {
                                if let Some(h) = h.upgrade() {
                                    h.notify_new_transactions(hashes);
                                }
                            }));

                        if let Some(h) = client.handler().upgrade() {
                            self.client.add_notify(h);
                        }
                        handler.extend_with(client.to_delegate());
                    }
                }
                Api::Personal => {
                    #[cfg(feature = "accounts")]
                    handler.extend_with(
                        PersonalClient::new(
                            &self.accounts,
                            dispatcher.clone(),
                            self.experimental_rpcs,
                        )
                        .to_delegate(),
                    );
                }
                Api::Signer => {
                    handler.extend_with(
                        SignerClient::new(
                            account_signer.clone(),
                            dispatcher.clone(),
                            &self.signer_service,
                            self.executor.clone(),
                        )
                        .to_delegate(),
                    );
                }
                Api::Parity => {
                    let signer = match self.signer_service.is_enabled() {
                        true => Some(self.signer_service.clone()),
                        false => None,
                    };
                    handler.extend_with(
                        ParityClient::new(
                            self.client.clone(),
                            self.miner.clone(),
                            self.sync.clone(),
                            self.net_service.clone(),
                            self.logger.clone(),
                            self.settings.clone(),
                            signer,
                            self.ws_address.clone(),
                            self.snapshot.clone().into(),
                        )
                        .to_delegate(),
                    );
                    #[cfg(feature = "accounts")]
                    handler.extend_with(ParityAccountsInfo::to_delegate(
                        ParityAccountsClient::new(&self.accounts),
                    ));

                    if !for_generic_pubsub {
                        add_signing_methods!(
                            ParitySigning,
                            handler,
                            self,
                            (&dispatcher, &account_signer)
                        );
                    }
                }
                Api::ParityPubSub => {
                    if !for_generic_pubsub {
                        let mut rpc = MetaIoHandler::default();
                        let apis = ApiSet::List(apis.clone())
                            .retain(ApiSet::PubSub)
                            .list_apis();
                        self.extend_api(&mut rpc, &apis, true);
                        handler.extend_with(
                            PubSubClient::new(rpc, self.executor.clone()).to_delegate(),
                        );
                    }
                }
                Api::ParityAccounts => {
                    #[cfg(feature = "accounts")]
                    handler.extend_with(ParityAccounts::to_delegate(ParityAccountsClient::new(
                        &self.accounts,
                    )));
                }
                Api::ParitySet => {
                    handler.extend_with(
                        ParitySetClient::new(
                            &self.client,
                            &self.miner,
                            &self.net_service,
                            self.fetch.clone(),
                        )
                        .to_delegate(),
                    );
                    #[cfg(feature = "accounts")]
                    handler.extend_with(
                        ParitySetAccountsClient::new(&self.accounts, &self.miner).to_delegate(),
                    );
                }
                Api::Traces => handler.extend_with(TracesClient::new(&self.client).to_delegate()),
                Api::Rpc => {
                    let modules = to_modules(apis);
                    handler.extend_with(RpcClient::new(modules).to_delegate());
                }
                Api::SecretStore => {
                    #[cfg(feature = "accounts")]
                    handler.extend_with(SecretStoreClient::new(&self.accounts).to_delegate());
                }
            }
        }
    }
}

impl Dependencies for FullDependencies {
    type Notifier = ClientNotifier;

    fn activity_notifier(&self) -> ClientNotifier {
        ClientNotifier {
            client: self.client.clone(),
        }
    }

    fn extend_with_set<S>(&self, handler: &mut MetaIoHandler<Metadata, S>, apis: &HashSet<Api>)
    where
        S: core::Middleware<Metadata>,
    {
        self.extend_api(handler, apis, false)
    }
}

impl ApiSet {
    /// Retains only APIs in given set.
    pub fn retain(self, set: Self) -> Self {
        ApiSet::List(&self.list_apis() & &set.list_apis())
    }

    pub fn list_apis(&self) -> HashSet<Api> {
        let mut public_list: HashSet<Api> = [
            Api::Web3,
            Api::Net,
            Api::Eth,
            Api::EthPubSub,
            Api::Parity,
            Api::Rpc,
        ]
        .iter()
        .cloned()
        .collect();

        match *self {
            ApiSet::List(ref apis) => apis.clone(),
            ApiSet::UnsafeContext => {
                public_list.insert(Api::Traces);
                public_list.insert(Api::ParityPubSub);
                public_list
            }
            ApiSet::IpcContext => {
                public_list.insert(Api::Traces);
                public_list.insert(Api::ParityPubSub);
                public_list.insert(Api::ParityAccounts);
                public_list
            }
            ApiSet::All => {
                public_list.insert(Api::Debug);
                public_list.insert(Api::Traces);
                public_list.insert(Api::ParityPubSub);
                public_list.insert(Api::ParityAccounts);
                public_list.insert(Api::ParitySet);
                public_list.insert(Api::Signer);
                public_list.insert(Api::Personal);
                public_list.insert(Api::SecretStore);
                public_list
            }
            ApiSet::PubSub => [
                Api::Eth,
                Api::Parity,
                Api::ParityAccounts,
                Api::ParitySet,
                Api::Traces,
            ]
            .iter()
            .cloned()
            .collect(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::{Api, ApiSet};

    #[test]
    fn test_api_parsing() {
        assert_eq!(Api::Debug, "debug".parse().unwrap());
        assert_eq!(Api::Web3, "web3".parse().unwrap());
        assert_eq!(Api::Net, "net".parse().unwrap());
        assert_eq!(Api::Eth, "eth".parse().unwrap());
        assert_eq!(Api::EthPubSub, "pubsub".parse().unwrap());
        assert_eq!(Api::Personal, "personal".parse().unwrap());
        assert_eq!(Api::Signer, "signer".parse().unwrap());
        assert_eq!(Api::Parity, "parity".parse().unwrap());
        assert_eq!(Api::ParityAccounts, "parity_accounts".parse().unwrap());
        assert_eq!(Api::ParitySet, "parity_set".parse().unwrap());
        assert_eq!(Api::Traces, "traces".parse().unwrap());
        assert_eq!(Api::Rpc, "rpc".parse().unwrap());
        assert_eq!(Api::SecretStore, "secretstore".parse().unwrap());
        assert!("rp".parse::<Api>().is_err());
    }

    #[test]
    fn test_api_set_default() {
        assert_eq!(ApiSet::UnsafeContext, ApiSet::default());
    }

    #[test]
    fn test_api_set_parsing() {
        assert_eq!(
            ApiSet::List(vec![Api::Web3, Api::Eth].into_iter().collect()),
            "web3,eth".parse().unwrap()
        );
    }

    #[test]
    fn test_api_set_unsafe_context() {
        let expected = vec![
            // make sure this list contains only SAFE methods
            Api::Web3,
            Api::Net,
            Api::Eth,
            Api::EthPubSub,
            Api::Parity,
            Api::ParityPubSub,
            Api::Traces,
            Api::Rpc,
        ]
        .into_iter()
        .collect();
        assert_eq!(ApiSet::UnsafeContext.list_apis(), expected);
    }

    #[test]
    fn test_api_set_ipc_context() {
        let expected = vec![
            // safe
            Api::Web3,
            Api::Net,
            Api::Eth,
            Api::EthPubSub,
            Api::Parity,
            Api::ParityPubSub,
            Api::Traces,
            Api::Rpc,
            // semi-safe
            Api::ParityAccounts,
        ]
        .into_iter()
        .collect();
        assert_eq!(ApiSet::IpcContext.list_apis(), expected);
    }

    #[test]
    fn test_all_apis() {
        assert_eq!(
            "all".parse::<ApiSet>().unwrap(),
            ApiSet::List(
                vec![
                    Api::Web3,
                    Api::Net,
                    Api::Eth,
                    Api::EthPubSub,
                    Api::Parity,
                    Api::ParityPubSub,
                    Api::Traces,
                    Api::Rpc,
                    Api::SecretStore,
                    Api::ParityAccounts,
                    Api::ParitySet,
                    Api::Signer,
                    Api::Personal,
                    Api::Debug,
                ]
                .into_iter()
                .collect()
            )
        );
    }

    #[test]
    fn test_all_without_personal_apis() {
        assert_eq!(
            "personal,all,-personal".parse::<ApiSet>().unwrap(),
            ApiSet::List(
                vec![
                    Api::Web3,
                    Api::Net,
                    Api::Eth,
                    Api::EthPubSub,
                    Api::Parity,
                    Api::ParityPubSub,
                    Api::Traces,
                    Api::Rpc,
                    Api::SecretStore,
                    Api::ParityAccounts,
                    Api::ParitySet,
                    Api::Signer,
                    Api::Debug,
                ]
                .into_iter()
                .collect()
            )
        );
    }

    #[test]
    fn test_safe_parsing() {
        assert_eq!(
            "safe".parse::<ApiSet>().unwrap(),
            ApiSet::List(
                vec![
                    Api::Web3,
                    Api::Net,
                    Api::Eth,
                    Api::EthPubSub,
                    Api::Parity,
                    Api::ParityPubSub,
                    Api::Traces,
                    Api::Rpc,
                ]
                .into_iter()
                .collect()
            )
        );
    }
}
