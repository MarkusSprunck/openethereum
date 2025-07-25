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

use crate::{
    bytes::Bytes,
    cli::{Args, ArgsError},
    hash::keccak,
    metrics::MetricsConfiguration,
    miner::pool,
    sync::{self, validate_node_url, NetworkConfiguration},
};
use ansi_term::Colour;

use crypto::publickey::{Public, Secret};
use ethcore::{
    client::VMType,
    miner::{stratum, MinerOptions},
    snapshot::SnapshotConfiguration,
    verification::queue::VerifierSettings,
};
use ethereum_types::{Address, H256, U256};

use parity_version::{version_data, version_short};
use std::{
    cmp,
    collections::{BTreeMap, HashSet},
    io::Read,
    iter::FromIterator,
    net::{SocketAddr, ToSocketAddrs},
    num::NonZeroU32,
    path::PathBuf,
    time::Duration,
};

use crate::{
    account::{AccountCmd, ImportAccounts, ListAccounts, NewAccount},
    blockchain::{
        BlockchainCmd, ExportBlockchain, ExportState, ImportBlockchain, KillBlockchain,
        ResetBlockchain,
    },
    cache::CacheConfig,
    helpers::{
        parity_ipc_path, to_address, to_addresses, to_block_id, to_bootnodes, to_duration, to_mode,
        to_pending_set, to_price, to_queue_penalization, to_queue_strategy, to_u256,
    },
    network::IpFilter,
    params::{AccountsConfig, GasPricerConfig, MinerExtras, ResealPolicy, SpecType},
    presale::ImportWallet,
    rpc::{HttpConfiguration, IpcConfiguration, WsConfiguration},
    run::RunCmd,
    secretstore::{
        Configuration as SecretStoreConfiguration, ContractAddress as SecretStoreContractAddress,
        NodeSecretKey,
    },
    snapshot::{self, SnapshotCommand},
    types::data_format::DataFormat,
};
use dir::{
    self, default_data_path, default_local_path,
    helpers::{replace_home, replace_home_and_local},
    Directories,
};
use ethcore_logger::Config as LogConfig;
use parity_rpc::NetworkSettings;

const DEFAULT_MAX_PEERS: u16 = 50;
const DEFAULT_MIN_PEERS: u16 = 25;
pub const ETHERSCAN_ETH_PRICE_ENDPOINT: &str =
    "https://api.etherscan.io/api?module=stats&action=ethprice";

#[derive(Debug, PartialEq)]
pub enum Cmd {
    Run(RunCmd),
    Version,
    Account(AccountCmd),
    ImportPresaleWallet(ImportWallet),
    Blockchain(BlockchainCmd),
    SignerToken(WsConfiguration, LogConfig),
    SignerSign {
        id: Option<usize>,
        pwfile: Option<PathBuf>,
        port: u16,
        authfile: PathBuf,
    },
    SignerList {
        port: u16,
        authfile: PathBuf,
    },
    SignerReject {
        id: Option<usize>,
        port: u16,
        authfile: PathBuf,
    },
    Snapshot(SnapshotCommand),
    Hash(Option<String>),
}

pub struct Execute {
    pub logger: LogConfig,
    pub cmd: Cmd,
}

/// Configuration for the OpenEthereum client.
#[derive(Debug, PartialEq)]
pub struct Configuration {
    /// Arguments to be interpreted.
    pub args: Args,
}

impl Configuration {
    /// Parses a configuration from a list of command line arguments.
    pub fn parse_cli<S: AsRef<str>>(command: &[S]) -> Result<Self, ArgsError> {
        let config = Configuration {
            args: Args::parse(command)?,
        };

        Ok(config)
    }

    pub(crate) fn into_command(self) -> Result<Execute, String> {
        let dirs = self.directories();
        let pruning = self.args.arg_pruning.parse()?;
        let pruning_history = self.args.arg_pruning_history;
        let vm_type = self.vm_type()?;
        let spec = self.chain()?;
        let mode = match self.args.arg_mode.as_ref() {
            "last" => None,
            mode => Some(to_mode(
                mode,
                self.args.arg_mode_timeout,
                self.args.arg_mode_alarm,
            )?),
        };
        let logger_config = self.logger_config();
        let ws_conf = self.ws_config()?;
        let snapshot_conf = self.snapshot_config()?;
        let http_conf = self.http_config()?;
        let ipc_conf = self.ipc_config()?;
        let net_conf = self.net_config()?;
        let network_id = self.network_id();
        let cache_config = self.cache_config();
        let tracing = self.args.arg_tracing.parse()?;
        let fat_db = self.args.arg_fat_db.parse()?;
        let compaction = self.args.arg_db_compaction.parse()?;
        let warp_sync = !self.args.flag_no_warp;
        let experimental_rpcs = self.args.flag_jsonrpc_experimental;
        let secretstore_conf = self.secretstore_config()?;
        let format = self.format()?;
        let metrics_conf = self.metrics_config()?;
        let keys_iterations = NonZeroU32::new(self.args.arg_keys_iterations)
            .ok_or("--keys-iterations must be non-zero")?;

        let cmd = if self.args.flag_version {
            Cmd::Version
        } else if self.args.cmd_signer {
            let authfile = crate::signer::codes_path(&ws_conf.signer_path);

            if self.args.cmd_signer_new_token {
                Cmd::SignerToken(ws_conf, logger_config.clone())
            } else if self.args.cmd_signer_sign {
                let pwfile = self
                    .accounts_config()?
                    .password_files
                    .first()
                    .map(PathBuf::from);
                Cmd::SignerSign {
                    id: self.args.arg_signer_sign_id,
                    pwfile,
                    port: ws_conf.port,
                    authfile,
                }
            } else if self.args.cmd_signer_reject {
                Cmd::SignerReject {
                    id: self.args.arg_signer_reject_id,
                    port: ws_conf.port,
                    authfile,
                }
            } else if self.args.cmd_signer_list {
                Cmd::SignerList {
                    port: ws_conf.port,
                    authfile,
                }
            } else {
                unreachable!();
            }
        } else if self.args.cmd_tools && self.args.cmd_tools_hash {
            Cmd::Hash(self.args.arg_tools_hash_file)
        } else if self.args.cmd_db && self.args.cmd_db_reset {
            Cmd::Blockchain(BlockchainCmd::Reset(ResetBlockchain {
                dirs,
                spec,
                pruning,
                pruning_history,
                pruning_memory: self.args.arg_pruning_memory,
                tracing,
                fat_db,
                compaction,
                cache_config,
                num: self.args.arg_db_reset_num,
            }))
        } else if self.args.cmd_db && self.args.cmd_db_kill {
            Cmd::Blockchain(BlockchainCmd::Kill(KillBlockchain {
                spec,
                dirs,
                pruning,
            }))
        } else if self.args.cmd_account {
            let account_cmd = if self.args.cmd_account_new {
                let new_acc = NewAccount {
                    iterations: keys_iterations,
                    path: dirs.keys,
                    spec,
                    password_file: self
                        .accounts_config()?
                        .password_files
                        .first()
                        .map(|x| x.to_owned()),
                };
                AccountCmd::New(new_acc)
            } else if self.args.cmd_account_list {
                let list_acc = ListAccounts {
                    path: dirs.keys,
                    spec,
                };
                AccountCmd::List(list_acc)
            } else if self.args.cmd_account_import {
                let import_acc = ImportAccounts {
                    from: self
                        .args
                        .arg_account_import_path
                        .expect("CLI argument is required; qed")
                        .clone(),
                    to: dirs.keys,
                    spec,
                };
                AccountCmd::Import(import_acc)
            } else {
                unreachable!();
            };
            Cmd::Account(account_cmd)
        } else if self.args.cmd_wallet {
            let presale_cmd = ImportWallet {
                iterations: keys_iterations,
                path: dirs.keys,
                spec,
                wallet_path: self.args.arg_wallet_import_path.clone().unwrap(),
                password_file: self
                    .accounts_config()?
                    .password_files
                    .first()
                    .map(|x| x.to_owned()),
            };
            Cmd::ImportPresaleWallet(presale_cmd)
        } else if self.args.cmd_import {
            let import_cmd = ImportBlockchain {
                spec,
                cache_config,
                dirs,
                file_path: self.args.arg_import_file.clone(),
                format,
                pruning,
                pruning_history,
                pruning_memory: self.args.arg_pruning_memory,
                compaction,
                tracing,
                fat_db,
                vm_type,
                check_seal: !self.args.flag_no_seal_check,
                with_color: logger_config.color,
                verifier_settings: self.verifier_settings(),
                max_round_blocks_to_import: self.args.arg_max_round_blocks_to_import,
            };
            Cmd::Blockchain(BlockchainCmd::Import(import_cmd))
        } else if self.args.cmd_export {
            if self.args.cmd_export_blocks {
                let export_cmd = ExportBlockchain {
                    spec,
                    cache_config,
                    dirs,
                    file_path: self.args.arg_export_blocks_file.clone(),
                    format,
                    pruning,
                    pruning_history,
                    pruning_memory: self.args.arg_pruning_memory,
                    compaction,
                    tracing,
                    fat_db,
                    from_block: to_block_id(&self.args.arg_export_blocks_from)?,
                    to_block: to_block_id(&self.args.arg_export_blocks_to)?,
                    check_seal: !self.args.flag_no_seal_check,
                    max_round_blocks_to_import: self.args.arg_max_round_blocks_to_import,
                };
                Cmd::Blockchain(BlockchainCmd::Export(export_cmd))
            } else if self.args.cmd_export_state {
                let export_cmd = ExportState {
                    spec,
                    cache_config,
                    dirs,
                    file_path: self.args.arg_export_state_file.clone(),
                    format,
                    pruning,
                    pruning_history,
                    pruning_memory: self.args.arg_pruning_memory,
                    compaction,
                    tracing,
                    fat_db,
                    at: to_block_id(&self.args.arg_export_state_at)?,
                    storage: !self.args.flag_export_state_no_storage,
                    code: !self.args.flag_export_state_no_code,
                    min_balance: self
                        .args
                        .arg_export_state_min_balance
                        .and_then(|s| to_u256(&s).ok()),
                    max_balance: self
                        .args
                        .arg_export_state_max_balance
                        .and_then(|s| to_u256(&s).ok()),
                    max_round_blocks_to_import: self.args.arg_max_round_blocks_to_import,
                };
                Cmd::Blockchain(BlockchainCmd::ExportState(export_cmd))
            } else {
                unreachable!();
            }
        } else if self.args.cmd_snapshot {
            let snapshot_cmd = SnapshotCommand {
                cache_config,
                dirs,
                spec,
                pruning,
                pruning_history,
                pruning_memory: self.args.arg_pruning_memory,
                tracing,
                fat_db,
                compaction,
                file_path: self.args.arg_snapshot_file.clone(),
                kind: snapshot::Kind::Take,
                block_at: to_block_id(&self.args.arg_snapshot_at)?,
                max_round_blocks_to_import: self.args.arg_max_round_blocks_to_import,
                snapshot_conf,
            };
            Cmd::Snapshot(snapshot_cmd)
        } else if self.args.cmd_restore {
            let restore_cmd = SnapshotCommand {
                cache_config,
                dirs,
                spec,
                pruning,
                pruning_history,
                pruning_memory: self.args.arg_pruning_memory,
                tracing,
                fat_db,
                compaction,
                file_path: self.args.arg_restore_file.clone(),
                kind: snapshot::Kind::Restore,
                block_at: to_block_id("latest")?, // unimportant.
                max_round_blocks_to_import: self.args.arg_max_round_blocks_to_import,
                snapshot_conf,
            };
            Cmd::Snapshot(restore_cmd)
        } else {
            let daemon = if self.args.cmd_daemon {
                Some(
                    self.args
                        .arg_daemon_pid_file
                        .clone()
                        .expect("CLI argument is required; qed"),
                )
            } else {
                None
            };

            let verifier_settings = self.verifier_settings();

            let run_cmd = RunCmd {
                cache_config,
                dirs,
                spec,
                pruning,
                pruning_history,
                pruning_memory: self.args.arg_pruning_memory,
                daemon,
                logger_config: logger_config.clone(),
                miner_options: self.miner_options()?,
                gas_price_percentile: self.args.arg_gas_price_percentile,
                poll_lifetime: self.args.arg_poll_lifetime,
                ws_conf,
                snapshot_conf,
                http_conf,
                ipc_conf,
                net_conf,
                network_id,
                acc_conf: self.accounts_config()?,
                gas_pricer_conf: self.gas_pricer_config()?,
                miner_extras: self.miner_extras()?,
                stratum: self.stratum_options()?,
                allow_missing_blocks: self.args.flag_jsonrpc_allow_missing_blocks,
                mode,
                tracing,
                fat_db,
                compaction,
                vm_type,
                warp_sync,
                warp_barrier: self.args.arg_warp_barrier,
                experimental_rpcs,
                net_settings: self.network_settings()?,
                secretstore_conf,
                name: self.args.arg_identity,
                custom_bootnodes: self.args.arg_bootnodes.is_some(),
                check_seal: !self.args.flag_no_seal_check,
                download_old_blocks: !self.args.flag_no_ancient_blocks,
                new_transactions_stats_period: self.args.arg_new_transactions_stats_period,
                verifier_settings,
                no_persistent_txqueue: self.args.flag_no_persistent_txqueue,
                max_round_blocks_to_import: self.args.arg_max_round_blocks_to_import,
                metrics_conf,
            };
            Cmd::Run(run_cmd)
        };

        Ok(Execute {
            logger: logger_config,
            cmd,
        })
    }

    fn vm_type(&self) -> Result<VMType, String> {
        Ok(VMType::Interpreter)
    }

    fn miner_extras(&self) -> Result<MinerExtras, String> {
        let floor = to_u256(&self.args.arg_gas_floor_target)?;
        let ceil = to_u256(&self.args.arg_gas_cap)?;
        let extras = MinerExtras {
            author: self.author()?,
            extra_data: self.extra_data()?,
            gas_range_target: (floor, ceil),
            engine_signer: self.engine_signer()?,
            work_notify: self.work_notify(),
            local_accounts: HashSet::from_iter(to_addresses(&self.args.arg_tx_queue_locals)?),
        };

        Ok(extras)
    }

    fn author(&self) -> Result<Address, String> {
        to_address(self.args.arg_author.clone())
    }

    fn engine_signer(&self) -> Result<Address, String> {
        to_address(self.args.arg_engine_signer.clone())
    }

    fn format(&self) -> Result<Option<DataFormat>, String> {
        match self
            .args
            .arg_import_format
            .clone()
            .or(self.args.arg_export_blocks_format.clone())
            .or(self.args.arg_export_state_format.clone())
        {
            Some(ref f) => Ok(Some(f.parse()?)),
            None => Ok(None),
        }
    }

    fn cache_config(&self) -> CacheConfig {
        match self.args.arg_cache_size {
            Some(size) => CacheConfig::new_with_total_cache_size(size),
            None => CacheConfig::new(
                self.args.arg_cache_size_db,
                self.args.arg_cache_size_blocks,
                self.args.arg_cache_size_queue,
                self.args.arg_cache_size_state,
            ),
        }
    }

    /// returns logger config
    pub fn logger_config(&self) -> LogConfig {
        LogConfig {
            mode: self.args.arg_logging.clone(),
            color: !self.args.flag_no_color,
            file: self
                .args
                .arg_log_file
                .as_ref()
                .map(|log_file| replace_home(&self.directories().base, log_file)),
            json: self.args.flag_json_logging,
        }
    }

    fn chain(&self) -> Result<SpecType, String> {
        self.args.arg_chain.parse()
    }

    fn is_dev_chain(&self) -> Result<bool, String> {
        Ok(self.chain()? == SpecType::Dev)
    }

    fn max_peers(&self) -> u32 {
        self.args
            .arg_max_peers
            .or(cmp::max(self.args.arg_min_peers, Some(DEFAULT_MAX_PEERS)))
            .unwrap_or(DEFAULT_MAX_PEERS) as u32
    }

    fn ip_filter(&self) -> Result<IpFilter, String> {
        IpFilter::parse(self.args.arg_allow_ips.as_str())
            .map_err(|_| "Invalid IP filter value".to_owned())
    }

    fn min_peers(&self) -> u32 {
        self.args
            .arg_min_peers
            .or(cmp::min(self.args.arg_max_peers, Some(DEFAULT_MIN_PEERS)))
            .unwrap_or(DEFAULT_MIN_PEERS) as u32
    }

    fn max_pending_peers(&self) -> u32 {
        self.args.arg_max_pending_peers as u32
    }

    fn snapshot_peers(&self) -> u32 {
        self.args.arg_snapshot_peers as u32
    }

    fn work_notify(&self) -> Vec<String> {
        self.args
            .arg_notify_work
            .as_ref()
            .map_or_else(Vec::new, |s| s.split(',').map(|s| s.to_owned()).collect())
    }

    fn accounts_config(&self) -> Result<AccountsConfig, String> {
        let keys_iterations = NonZeroU32::new(self.args.arg_keys_iterations)
            .ok_or("--keys-iterations must be non-zero")?;
        let cfg = AccountsConfig {
            iterations: keys_iterations,
            refresh_time: self.args.arg_accounts_refresh,
            testnet: false,
            password_files: self
                .args
                .arg_password
                .iter()
                .map(|s| replace_home(&self.directories().base, s))
                .collect(),
            unlocked_accounts: to_addresses(&self.args.arg_unlock)?,
            enable_fast_unlock: self.args.flag_fast_unlock,
        };

        Ok(cfg)
    }

    fn stratum_options(&self) -> Result<Option<stratum::Options>, String> {
        if self.args.flag_stratum {
            Ok(Some(stratum::Options {
                io_path: self.directories().db,
                listen_addr: self.stratum_interface(),
                port: self.args.arg_ports_shift + self.args.arg_stratum_port,
                secret: self
                    .args
                    .arg_stratum_secret
                    .as_ref()
                    .map(|s| s.parse::<H256>().unwrap_or_else(|_| keccak(s))),
            }))
        } else {
            Ok(None)
        }
    }

    fn miner_options(&self) -> Result<MinerOptions, String> {
        let is_dev_chain = self.is_dev_chain()?;
        if is_dev_chain && self.args.flag_force_sealing && self.args.arg_reseal_min_period == 0 {
            return Err("Force sealing can't be used with reseal_min_period = 0".into());
        }

        let reseal = self.args.arg_reseal_on_txs.parse::<ResealPolicy>()?;

        let options = MinerOptions {
            force_sealing: self.args.flag_force_sealing,
            reseal_on_external_tx: reseal.external,
            reseal_on_own_tx: reseal.own,
            reseal_on_uncle: self.args.flag_reseal_on_uncle,
            reseal_min_period: Duration::from_millis(self.args.arg_reseal_min_period),
            reseal_max_period: Duration::from_millis(self.args.arg_reseal_max_period),

            pending_set: to_pending_set(&self.args.arg_relay_set)?,
            work_queue_size: self.args.arg_work_queue_size,
            enable_resubmission: !self.args.flag_remove_solved,
            infinite_pending_block: self.args.flag_infinite_pending_block,

            tx_queue_penalization: to_queue_penalization(self.args.arg_tx_time_limit)?,
            tx_queue_strategy: to_queue_strategy(&self.args.arg_tx_queue_strategy)?,
            tx_queue_no_unfamiliar_locals: self.args.flag_tx_queue_no_unfamiliar_locals,
            refuse_service_transactions: self.args.flag_refuse_service_transactions,

            pool_limits: self.pool_limits()?,
            pool_verification_options: self.pool_verification_options()?,
        };

        Ok(options)
    }

    fn pool_limits(&self) -> Result<pool::Options, String> {
        let max_count = self.args.arg_tx_queue_size;

        Ok(pool::Options {
            max_count,
            max_per_sender: self
                .args
                .arg_tx_queue_per_sender
                .unwrap_or_else(|| cmp::max(16, max_count / 100)),
            max_mem_usage: if self.args.arg_tx_queue_mem_limit > 0 {
                self.args.arg_tx_queue_mem_limit as usize * 1024 * 1024
            } else {
                usize::max_value()
            },
        })
    }

    fn pool_verification_options(&self) -> Result<pool::verifier::Options, String> {
        Ok(pool::verifier::Options {
            // NOTE min_gas_price,block_gas_limit block_base_fee, and allow_non_eoa_sender will be overwritten right after start.
            minimal_gas_price: U256::from(20_000_000) * 1_000u32,
            block_gas_limit: U256::max_value(),
            block_base_fee: None,
            tx_gas_limit: match self.args.arg_tx_gas_limit {
                Some(ref d) => to_u256(d)?,
                None => U256::max_value(),
            },
            no_early_reject: self.args.flag_tx_queue_no_early_reject,
            allow_non_eoa_sender: false,
        })
    }

    fn secretstore_config(&self) -> Result<SecretStoreConfiguration, String> {
        Ok(SecretStoreConfiguration {
            enabled: self.secretstore_enabled(),
            http_enabled: self.secretstore_http_enabled(),
            auto_migrate_enabled: self.secretstore_auto_migrate_enabled(),
            acl_check_contract_address: self.secretstore_acl_check_contract_address()?,
            service_contract_address: self.secretstore_service_contract_address()?,
            service_contract_srv_gen_address: self
                .secretstore_service_contract_srv_gen_address()?,
            service_contract_srv_retr_address: self
                .secretstore_service_contract_srv_retr_address()?,
            service_contract_doc_store_address: self
                .secretstore_service_contract_doc_store_address()?,
            service_contract_doc_sretr_address: self
                .secretstore_service_contract_doc_sretr_address()?,
            self_secret: self.secretstore_self_secret()?,
            nodes: self.secretstore_nodes()?,
            key_server_set_contract_address: self.secretstore_key_server_set_contract_address()?,
            interface: self.secretstore_interface(),
            port: self.args.arg_ports_shift + self.args.arg_secretstore_port,
            http_interface: self.secretstore_http_interface(),
            http_port: self.args.arg_ports_shift + self.args.arg_secretstore_http_port,
            data_path: self.directories().secretstore,
            admin_public: self.secretstore_admin_public()?,
        })
    }

    fn gas_pricer_config(&self) -> Result<GasPricerConfig, String> {
        fn wei_per_gas(usd_per_tx: f32, usd_per_eth: f32) -> U256 {
            let wei_per_usd: f32 = 1.0e18 / usd_per_eth;
            let gas_per_tx: f32 = 21000.0;
            let wei_per_gas: f32 = wei_per_usd * usd_per_tx / gas_per_tx;
            U256::from_dec_str(&format!("{wei_per_gas:.0}")).unwrap()
        }

        if let Some(dec) = self.args.arg_min_gas_price {
            return Ok(GasPricerConfig::Fixed(U256::from(dec)));
        } else if self.chain()? != SpecType::Foundation {
            return Ok(GasPricerConfig::Fixed(U256::zero()));
        }

        let usd_per_tx = to_price(&self.args.arg_usd_per_tx)?;

        if "auto" == self.args.arg_usd_per_eth {
            Ok(GasPricerConfig::Calibrated {
                usd_per_tx,
                recalibration_period: to_duration(self.args.arg_price_update_period.as_str())?,
                api_endpoint: ETHERSCAN_ETH_PRICE_ENDPOINT.to_string(),
            })
        } else if let Ok(usd_per_eth_parsed) = to_price(&self.args.arg_usd_per_eth) {
            let wei_per_gas = wei_per_gas(usd_per_tx, usd_per_eth_parsed);

            info!(
                "Using a fixed conversion rate of Ξ1 = {} ({} wei/gas)",
                Colour::White
                    .bold()
                    .paint(format!("US${usd_per_eth_parsed:.2}")),
                Colour::Yellow.bold().paint(format!("{wei_per_gas}"))
            );

            Ok(GasPricerConfig::Fixed(wei_per_gas))
        } else {
            Ok(GasPricerConfig::Calibrated {
                usd_per_tx,
                recalibration_period: to_duration(self.args.arg_price_update_period.as_str())?,
                api_endpoint: self.args.arg_usd_per_eth.clone(),
            })
        }
    }

    fn extra_data(&self) -> Result<Bytes, String> {
        match &self.args.arg_extra_data {
            Some(x) if x.len() <= 32 => Ok(x.as_bytes().to_owned()),
            None => Ok(version_data()),
            Some(_) => Err("Extra data must be at most 32 characters".into()),
        }
    }

    fn init_reserved_nodes(&self) -> Result<Vec<String>, String> {
        use std::fs::File;

        match self.args.arg_reserved_peers {
            Some(ref path) => {
                let path = replace_home(&self.directories().base, path);

                let mut buffer = String::new();
                let mut node_file = File::open(&path)
                    .map_err(|e| format!("Error opening reserved nodes file: {e}"))?;
                node_file
                    .read_to_string(&mut buffer)
                    .map_err(|_| "Error reading reserved node file")?;
                let lines = buffer
                    .lines()
                    .map(|s| s.trim().to_owned())
                    .filter(|s| !s.is_empty() && !s.starts_with("#"))
                    .collect::<Vec<_>>();

                for line in &lines {
                    match validate_node_url(line).map(Into::into) {
                        None => continue,
                        Some(sync::ErrorKind::AddressResolve(_)) => {
                            return Err(format!(
                                "Failed to resolve hostname of a boot node: {line}"
                            ))
                        }
                        Some(_) => {
                            return Err(format!(
                                "Invalid node address format given for a boot node: {line}"
                            ))
                        }
                    }
                }

                Ok(lines)
            }
            None => Ok(Vec::new()),
        }
    }

    fn net_addresses(&self) -> Result<(SocketAddr, Option<SocketAddr>), String> {
        let port = self.args.arg_ports_shift + self.args.arg_port;
        let listen_address = SocketAddr::new(
            self.interface(&self.args.arg_interface).parse().unwrap(),
            port,
        );
        let public_address = if self.args.arg_nat.starts_with("extip:") {
            let host = self.args.arg_nat[6..]
                .split(':')
                .next()
                .expect("split has at least one part; qed");
            let host = format!("{host}:{port}");
            match host.to_socket_addrs() {
                Ok(mut addr_iter) => {
                    if let Some(addr) = addr_iter.next() {
                        Some(addr)
                    } else {
                        return Err(format!(
                            "Invalid host given with `--nat extip:{}`",
                            &self.args.arg_nat[6..]
                        ));
                    }
                }
                Err(_) => {
                    return Err(format!(
                        "Invalid host given with `--nat extip:{}`",
                        &self.args.arg_nat[6..]
                    ))
                }
            }
        } else {
            None
        };
        Ok((listen_address, public_address))
    }

    fn net_config(&self) -> Result<NetworkConfiguration, String> {
        let mut ret = NetworkConfiguration::new();
        ret.nat_enabled = self.args.arg_nat == "any" || self.args.arg_nat == "upnp";
        ret.boot_nodes = to_bootnodes(&self.args.arg_bootnodes)?;
        let (listen, public) = self.net_addresses()?;
        ret.listen_address = Some(format!("{listen}"));
        ret.public_address = public.map(|p| format!("{p}"));
        ret.use_secret = match self.args.arg_node_key.as_ref().map(|s| {
            s.parse::<Secret>()
                .or_else(|_| Secret::import_key(keccak(s).as_bytes()))
                .map_err(|e| format!("Invalid key: {e:?}"))
        }) {
            None => None,
            Some(Ok(key)) => Some(key),
            Some(Err(err)) => return Err(err),
        };
        ret.discovery_enabled = !self.args.flag_no_discovery;
        ret.max_peers = self.max_peers();
        ret.min_peers = self.min_peers();
        ret.snapshot_peers = self.snapshot_peers();
        ret.ip_filter = self.ip_filter()?;
        ret.max_pending_peers = self.max_pending_peers();
        let mut net_path = PathBuf::from(self.directories().base);
        net_path.push("network");
        ret.config_path = Some(net_path.to_str().unwrap().to_owned());
        ret.reserved_nodes = self.init_reserved_nodes()?;
        ret.allow_non_reserved = !self.args.flag_reserved_only;
        ret.client_version = {
            let mut client_version = version_short();
            if !self.args.arg_identity.is_empty() {
                // Insert name after the "Parity-Ethereum/" at the beginning of version string.
                let idx = client_version.find('/').unwrap_or(client_version.len());
                client_version.insert_str(idx, &format!("/{}", self.args.arg_identity));
            }
            client_version
        };
        Ok(ret)
    }

    fn network_id(&self) -> Option<u64> {
        self.args.arg_network_id
    }

    fn rpc_apis(&self) -> String {
        self.args.arg_jsonrpc_apis.clone()
    }

    fn cors(cors: &str) -> Option<Vec<String>> {
        match cors {
            "none" => return Some(Vec::new()),
            "*" | "all" | "any" => return None,
            _ => {}
        }

        Some(cors.split(',').map(Into::into).collect())
    }

    fn rpc_cors(&self) -> Option<Vec<String>> {
        let cors = self.args.arg_jsonrpc_cors.to_owned();
        Self::cors(&cors)
    }

    fn hosts(&self, hosts: &str, interface: &str) -> Option<Vec<String>> {
        if self.args.flag_unsafe_expose {
            return None;
        }

        if interface == "0.0.0.0" && hosts == "none" {
            return None;
        }

        Self::parse_hosts(hosts)
    }

    fn parse_hosts(hosts: &str) -> Option<Vec<String>> {
        match hosts {
            "none" => return Some(Vec::new()),
            "*" | "all" | "any" => return None,
            _ => {}
        }
        let hosts = hosts.split(',').map(Into::into).collect();
        Some(hosts)
    }

    fn rpc_hosts(&self) -> Option<Vec<String>> {
        self.hosts(&self.args.arg_jsonrpc_hosts, &self.rpc_interface())
    }

    fn ws_hosts(&self) -> Option<Vec<String>> {
        self.hosts(&self.args.arg_ws_hosts, &self.ws_interface())
    }

    fn ws_origins(&self) -> Option<Vec<String>> {
        if self.args.flag_unsafe_expose {
            return None;
        }

        Self::parse_hosts(&self.args.arg_ws_origins)
    }

    fn ipc_config(&self) -> Result<IpcConfiguration, String> {
        let conf = IpcConfiguration {
            enabled: !self.args.flag_no_ipc,
            socket_addr: self.ipc_path(),
            apis: self.args.arg_ipc_apis.parse()?,
        };

        Ok(conf)
    }

    fn http_config(&self) -> Result<HttpConfiguration, String> {
        let conf = HttpConfiguration {
            enabled: self.rpc_enabled(),
            interface: self.rpc_interface(),
            port: self.args.arg_ports_shift + self.args.arg_jsonrpc_port,
            apis: self.rpc_apis().parse()?,
            hosts: self.rpc_hosts(),
            cors: self.rpc_cors(),
            server_threads: match self.args.arg_jsonrpc_server_threads {
                Some(threads) if threads > 0 => threads,
                _ => 1,
            },
            processing_threads: self.args.arg_jsonrpc_threads,
            max_payload: match self.args.arg_jsonrpc_max_payload {
                Some(max) if max > 0 => max,
                _ => 5usize,
            },
            keep_alive: !self.args.flag_jsonrpc_no_keep_alive,
        };

        Ok(conf)
    }

    fn ws_config(&self) -> Result<WsConfiguration, String> {
        let support_token_api =
            // enabled when not unlocking
            self.args.arg_unlock.is_none();

        let conf = WsConfiguration {
            enabled: self.ws_enabled(),
            interface: self.ws_interface(),
            port: self.args.arg_ports_shift + self.args.arg_ws_port,
            apis: self.args.arg_ws_apis.parse()?,
            hosts: self.ws_hosts(),
            origins: self.ws_origins(),
            signer_path: self.directories().signer.into(),
            support_token_api,
            max_connections: self.args.arg_ws_max_connections,
            max_payload: self.args.arg_ws_max_payload,
        };

        Ok(conf)
    }

    fn metrics_config(&self) -> Result<MetricsConfiguration, String> {
        let conf = MetricsConfiguration {
            enabled: self.metrics_enabled(),
            prefix: self.metrics_prefix(),
            interface: self.metrics_interface(),
            port: self.args.arg_ports_shift + self.args.arg_metrics_port,
        };
        Ok(conf)
    }

    fn snapshot_config(&self) -> Result<SnapshotConfiguration, String> {
        let conf = SnapshotConfiguration {
            enable: self.args.flag_enable_snapshotting,
            processing_threads: match self.args.arg_snapshot_threads {
                Some(threads) if threads > 0 => threads,
                _ => ::std::cmp::max(1, num_cpus::get_physical() / 2),
            },
        };

        Ok(conf)
    }

    fn network_settings(&self) -> Result<NetworkSettings, String> {
        let http_conf = self.http_config()?;
        let net_addresses = self.net_addresses()?;
        Ok(NetworkSettings {
            name: self.args.arg_identity.clone(),
            chain: format!("{}", self.chain()?),
            is_dev_chain: self.is_dev_chain()?,
            network_port: net_addresses.0.port(),
            rpc_enabled: http_conf.enabled,
            rpc_interface: http_conf.interface,
            rpc_port: http_conf.port,
        })
    }

    fn directories(&self) -> Directories {
        let local_path = default_local_path();
        let base_path = self
            .args
            .arg_base_path
            .as_ref()
            .map_or_else(default_data_path, |s| s.clone());
        let data_path = replace_home("", &base_path);
        let is_using_base_path = self.args.arg_base_path.is_some();
        // If base_path is set and db_path is not we default to base path subdir instead of LOCAL.
        let base_db_path = if is_using_base_path && self.args.arg_db_path.is_none() {
            "$BASE/chains"
        } else {
            self.args
                .arg_db_path
                .as_ref()
                .map_or(dir::CHAINS_PATH, |s| s)
        };
        let cache_path = if is_using_base_path {
            "$BASE/cache"
        } else {
            dir::CACHE_PATH
        };

        let db_path = replace_home_and_local(&data_path, &local_path, base_db_path);
        let cache_path = replace_home_and_local(&data_path, &local_path, cache_path);
        let keys_path = replace_home(&data_path, &self.args.arg_keys_path);
        let secretstore_path = replace_home(&data_path, &self.args.arg_secretstore_path);
        let ui_path = replace_home(&data_path, &self.args.arg_ui_path);

        Directories {
            keys: keys_path,
            base: data_path,
            cache: cache_path,
            db: db_path,
            signer: ui_path,
            secretstore: secretstore_path,
        }
    }

    fn ipc_path(&self) -> String {
        parity_ipc_path(
            &self.directories().base,
            &self.args.arg_ipc_path,
            self.args.arg_ports_shift,
        )
    }

    fn interface(&self, interface: &str) -> String {
        if self.args.flag_unsafe_expose {
            return "0.0.0.0".into();
        }

        match interface {
            "all" => "0.0.0.0",
            "local" => "127.0.0.1",
            x => x,
        }
        .into()
    }

    fn rpc_interface(&self) -> String {
        self.interface(&self.args.arg_jsonrpc_interface)
    }

    fn ws_interface(&self) -> String {
        self.interface(&self.args.arg_ws_interface)
    }

    fn metrics_interface(&self) -> String {
        self.interface(&self.args.arg_metrics_interface)
    }

    fn secretstore_interface(&self) -> String {
        self.interface(&self.args.arg_secretstore_interface)
    }

    fn secretstore_http_interface(&self) -> String {
        self.interface(&self.args.arg_secretstore_http_interface)
    }

    fn secretstore_self_secret(&self) -> Result<Option<NodeSecretKey>, String> {
        match self.args.arg_secretstore_secret {
            Some(ref s) if s.len() == 64 => Ok(Some(NodeSecretKey::Plain(s.parse()
                .map_err(|e| format!("Invalid secret store secret: {s}. Error: {e:?}"))?))),
            #[cfg(feature = "accounts")]
            Some(ref s) if s.len() == 40 => Ok(Some(NodeSecretKey::KeyStore(s.parse()
                .map_err(|e| format!("Invalid secret store secret address: {s}. Error: {e:?}"))?))),
            Some(_) => Err("Invalid secret store secret. Must be either existing account address, or hex-encoded private key".to_string()),
            None => Ok(None),
        }
    }

    fn secretstore_admin_public(&self) -> Result<Option<Public>, String> {
        match self.args.arg_secretstore_admin_public.as_ref() {
            Some(admin_public) => {
                Ok(Some(admin_public.parse().map_err(|e| {
                    format!("Invalid secret store admin public: {e}")
                })?))
            }
            None => Ok(None),
        }
    }

    fn secretstore_nodes(&self) -> Result<BTreeMap<Public, (String, u16)>, String> {
        let mut nodes = BTreeMap::new();
        for node in self
            .args
            .arg_secretstore_nodes
            .split(',')
            .filter(|n| n != &"")
        {
            let public_and_addr: Vec<_> = node.split('@').collect();
            if public_and_addr.len() != 2 {
                return Err(format!("Invalid secret store node: {node}"));
            }

            let ip_and_port: Vec<_> = public_and_addr[1].split(':').collect();
            if ip_and_port.len() != 2 {
                return Err(format!("Invalid secret store node: {node}"));
            }

            let public = public_and_addr[0].parse().map_err(|e| {
                format!(
                    "Invalid public key in secret store node: {}. Error: {:?}",
                    public_and_addr[0], e
                )
            })?;
            let port = ip_and_port[1].parse().map_err(|e| {
                format!(
                    "Invalid port in secret store node: {}. Error: {:?}",
                    ip_and_port[1], e
                )
            })?;

            nodes.insert(public, (ip_and_port[0].into(), port));
        }

        Ok(nodes)
    }

    fn stratum_interface(&self) -> String {
        self.interface(&self.args.arg_stratum_interface)
    }

    fn rpc_enabled(&self) -> bool {
        !self.args.flag_no_jsonrpc
    }

    fn ws_enabled(&self) -> bool {
        !self.args.flag_no_ws
    }

    fn metrics_enabled(&self) -> bool {
        self.args.flag_metrics
    }

    fn metrics_prefix(&self) -> String {
        self.args.arg_metrics_prefix.clone()
    }

    fn secretstore_enabled(&self) -> bool {
        !self.args.flag_no_secretstore && cfg!(feature = "secretstore")
    }

    fn secretstore_http_enabled(&self) -> bool {
        !self.args.flag_no_secretstore_http && cfg!(feature = "secretstore")
    }

    fn secretstore_auto_migrate_enabled(&self) -> bool {
        !self.args.flag_no_secretstore_auto_migrate
    }

    fn secretstore_acl_check_contract_address(
        &self,
    ) -> Result<Option<SecretStoreContractAddress>, String> {
        into_secretstore_service_contract_address(self.args.arg_secretstore_acl_contract.as_ref())
    }

    fn secretstore_service_contract_address(
        &self,
    ) -> Result<Option<SecretStoreContractAddress>, String> {
        into_secretstore_service_contract_address(self.args.arg_secretstore_contract.as_ref())
    }

    fn secretstore_service_contract_srv_gen_address(
        &self,
    ) -> Result<Option<SecretStoreContractAddress>, String> {
        into_secretstore_service_contract_address(
            self.args.arg_secretstore_srv_gen_contract.as_ref(),
        )
    }

    fn secretstore_service_contract_srv_retr_address(
        &self,
    ) -> Result<Option<SecretStoreContractAddress>, String> {
        into_secretstore_service_contract_address(
            self.args.arg_secretstore_srv_retr_contract.as_ref(),
        )
    }

    fn secretstore_service_contract_doc_store_address(
        &self,
    ) -> Result<Option<SecretStoreContractAddress>, String> {
        into_secretstore_service_contract_address(
            self.args.arg_secretstore_doc_store_contract.as_ref(),
        )
    }

    fn secretstore_service_contract_doc_sretr_address(
        &self,
    ) -> Result<Option<SecretStoreContractAddress>, String> {
        into_secretstore_service_contract_address(
            self.args.arg_secretstore_doc_sretr_contract.as_ref(),
        )
    }

    fn secretstore_key_server_set_contract_address(
        &self,
    ) -> Result<Option<SecretStoreContractAddress>, String> {
        into_secretstore_service_contract_address(
            self.args.arg_secretstore_server_set_contract.as_ref(),
        )
    }

    fn verifier_settings(&self) -> VerifierSettings {
        let mut settings = VerifierSettings::default();
        settings.scale_verifiers = self.args.flag_scale_verifiers;
        if let Some(num_verifiers) = self.args.arg_num_verifiers {
            settings.num_verifiers = num_verifiers;
        }

        settings
    }
}

fn into_secretstore_service_contract_address(
    s: Option<&String>,
) -> Result<Option<SecretStoreContractAddress>, String> {
    match s.map(String::as_str) {
        None | Some("none") => Ok(None),
        Some("registry") => Ok(Some(SecretStoreContractAddress::Registry)),
        Some(a) => Ok(Some(SecretStoreContractAddress::Address(
            a.parse().map_err(|e| format!("{e}"))?,
        ))),
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Write, str::FromStr};

    use crate::{
        account::{AccountCmd, ImportAccounts, ListAccounts, NewAccount},
        blockchain::{BlockchainCmd, ExportBlockchain, ExportState, ImportBlockchain},
        cli::Args,
        helpers::default_network_config,
        miner::pool::PrioritizationStrategy,
        params::SpecType,
        presale::ImportWallet,
        rpc::WsConfiguration,
        rpc_apis::ApiSet,
        run::RunCmd,
        types::{data_format::DataFormat, ids::BlockId},
    };
    use dir::Directories;
    use ethcore::{client::VMType, miner::MinerOptions};
    use parity_rpc::NetworkSettings;
    use tempdir::TempDir;

    use crate::network::{AllowIP, IpFilter};

    extern crate ipnetwork;
    use self::ipnetwork::IpNetwork;

    use super::*;

    lazy_static! {
        static ref ITERATIONS: NonZeroU32 = NonZeroU32::new(10240).expect("10240 > 0; qed");
    }

    #[derive(Debug, PartialEq)]
    struct TestPasswordReader(&'static str);

    fn parse(args: &[&str]) -> Configuration {
        Configuration {
            args: Args::parse_without_config(args).unwrap(),
        }
    }

    #[test]
    fn test_command_version() {
        let args = vec!["openethereum", "--version"];
        let conf = parse(&args);
        assert_eq!(conf.into_command().unwrap().cmd, Cmd::Version);
    }

    #[test]
    fn test_command_account_new() {
        let args = vec!["openethereum", "account", "new"];
        let conf = parse(&args);
        assert_eq!(
            conf.into_command().unwrap().cmd,
            Cmd::Account(AccountCmd::New(NewAccount {
                iterations: *ITERATIONS,
                path: Directories::default().keys,
                password_file: None,
                spec: SpecType::default(),
            }))
        );
    }

    #[test]
    fn test_command_account_list() {
        let args = vec!["openethereum", "account", "list"];
        let conf = parse(&args);
        assert_eq!(
            conf.into_command().unwrap().cmd,
            Cmd::Account(AccountCmd::List(ListAccounts {
                path: Directories::default().keys,
                spec: SpecType::default(),
            }))
        );
    }

    #[test]
    fn test_command_account_import() {
        let args = vec!["openethereum", "account", "import", "my_dir", "another_dir"];
        let conf = parse(&args);
        assert_eq!(
            conf.into_command().unwrap().cmd,
            Cmd::Account(AccountCmd::Import(ImportAccounts {
                from: vec!["my_dir".into(), "another_dir".into()],
                to: Directories::default().keys,
                spec: SpecType::default(),
            }))
        );
    }

    #[test]
    fn test_command_wallet_import() {
        let args = vec![
            "openethereum",
            "wallet",
            "import",
            "my_wallet.json",
            "--password",
            "pwd",
        ];
        let conf = parse(&args);
        assert_eq!(
            conf.into_command().unwrap().cmd,
            Cmd::ImportPresaleWallet(ImportWallet {
                iterations: *ITERATIONS,
                path: Directories::default().keys,
                wallet_path: "my_wallet.json".into(),
                password_file: Some("pwd".into()),
                spec: SpecType::default(),
            })
        );
    }

    #[test]
    fn test_command_blockchain_import() {
        let args = vec!["openethereum", "import", "blockchain.json"];
        let conf = parse(&args);
        assert_eq!(
            conf.into_command().unwrap().cmd,
            Cmd::Blockchain(BlockchainCmd::Import(ImportBlockchain {
                spec: Default::default(),
                cache_config: Default::default(),
                dirs: Default::default(),
                file_path: Some("blockchain.json".into()),
                format: Default::default(),
                pruning: Default::default(),
                pruning_history: 64,
                pruning_memory: 32,
                compaction: Default::default(),
                tracing: Default::default(),
                fat_db: Default::default(),
                vm_type: VMType::Interpreter,
                check_seal: true,
                with_color: true,
                verifier_settings: Default::default(),
                max_round_blocks_to_import: 1,
            }))
        );
    }

    #[test]
    fn test_command_blockchain_export() {
        let args = vec!["openethereum", "export", "blocks", "blockchain.json"];
        let conf = parse(&args);
        assert_eq!(
            conf.into_command().unwrap().cmd,
            Cmd::Blockchain(BlockchainCmd::Export(ExportBlockchain {
                spec: Default::default(),
                cache_config: Default::default(),
                dirs: Default::default(),
                file_path: Some("blockchain.json".into()),
                pruning: Default::default(),
                pruning_history: 64,
                pruning_memory: 32,
                format: Default::default(),
                compaction: Default::default(),
                tracing: Default::default(),
                fat_db: Default::default(),
                from_block: BlockId::Number(1),
                to_block: BlockId::Latest,
                check_seal: true,
                max_round_blocks_to_import: 1,
            }))
        );
    }

    #[test]
    fn test_command_state_export() {
        let args = vec!["openethereum", "export", "state", "state.json"];
        let conf = parse(&args);
        assert_eq!(
            conf.into_command().unwrap().cmd,
            Cmd::Blockchain(BlockchainCmd::ExportState(ExportState {
                spec: Default::default(),
                cache_config: Default::default(),
                dirs: Default::default(),
                file_path: Some("state.json".into()),
                pruning: Default::default(),
                pruning_history: 64,
                pruning_memory: 32,
                format: Default::default(),
                compaction: Default::default(),
                tracing: Default::default(),
                fat_db: Default::default(),
                at: BlockId::Latest,
                storage: true,
                code: true,
                min_balance: None,
                max_balance: None,
                max_round_blocks_to_import: 1,
            }))
        );
    }

    #[test]
    fn test_command_blockchain_export_with_custom_format() {
        let args = vec![
            "openethereum",
            "export",
            "blocks",
            "--format",
            "hex",
            "blockchain.json",
        ];
        let conf = parse(&args);
        assert_eq!(
            conf.into_command().unwrap().cmd,
            Cmd::Blockchain(BlockchainCmd::Export(ExportBlockchain {
                spec: Default::default(),
                cache_config: Default::default(),
                dirs: Default::default(),
                file_path: Some("blockchain.json".into()),
                pruning: Default::default(),
                pruning_history: 64,
                pruning_memory: 32,
                format: Some(DataFormat::Hex),
                compaction: Default::default(),
                tracing: Default::default(),
                fat_db: Default::default(),
                from_block: BlockId::Number(1),
                to_block: BlockId::Latest,
                check_seal: true,
                max_round_blocks_to_import: 1,
            }))
        );
    }

    #[test]
    fn test_command_signer_new_token() {
        let args = vec!["openethereum", "signer", "new-token"];
        let conf = parse(&args);
        let expected = Directories::default().signer;
        assert_eq!(
            conf.into_command().unwrap().cmd,
            Cmd::SignerToken(
                WsConfiguration {
                    enabled: true,
                    interface: "127.0.0.1".into(),
                    port: 8546,
                    apis: ApiSet::UnsafeContext,
                    origins: Some(vec![
                        "parity://*".into(),
                        "chrome-extension://*".into(),
                        "moz-extension://*".into(),
                    ]),
                    hosts: Some(vec![]),
                    signer_path: expected.into(),
                    support_token_api: true,
                    max_connections: 100,
                    max_payload: 5,
                },
                LogConfig {
                    color: true,
                    mode: None,
                    file: None,
                    json: false,
                },
            )
        );
    }

    #[test]
    fn test_ws_max_connections() {
        let args = vec!["openethereum", "--ws-max-connections", "1"];
        let conf = parse(&args);

        assert_eq!(
            conf.ws_config().unwrap(),
            WsConfiguration {
                max_connections: 1,
                ..Default::default()
            }
        );
    }

    #[test]
    fn test_run_cmd() {
        let args = vec!["openethereum"];
        let conf = parse(&args);
        let mut expected = RunCmd {
            allow_missing_blocks: false,
            cache_config: Default::default(),
            dirs: Default::default(),
            spec: Default::default(),
            pruning: Default::default(),
            pruning_history: 64,
            pruning_memory: 32,
            daemon: None,
            logger_config: LogConfig {
                color: true,
                mode: None,
                file: None,
                json: false,
            },
            miner_options: Default::default(),
            gas_price_percentile: 50,
            poll_lifetime: 60,
            ws_conf: Default::default(),
            http_conf: Default::default(),
            ipc_conf: Default::default(),
            net_conf: default_network_config(),
            network_id: None,
            warp_sync: true,
            warp_barrier: None,
            acc_conf: Default::default(),
            gas_pricer_conf: Default::default(),
            miner_extras: Default::default(),
            mode: Default::default(),
            tracing: Default::default(),
            compaction: Default::default(),
            vm_type: Default::default(),
            experimental_rpcs: false,
            net_settings: Default::default(),
            secretstore_conf: Default::default(),
            name: "".into(),
            custom_bootnodes: false,
            fat_db: Default::default(),
            snapshot_conf: Default::default(),
            stratum: None,
            check_seal: true,
            download_old_blocks: true,
            new_transactions_stats_period: 0,
            verifier_settings: Default::default(),
            no_persistent_txqueue: false,
            max_round_blocks_to_import: 1,
            metrics_conf: MetricsConfiguration::default(),
        };
        expected.secretstore_conf.enabled = cfg!(feature = "secretstore");
        expected.secretstore_conf.http_enabled = cfg!(feature = "secretstore");
        assert_eq!(conf.into_command().unwrap().cmd, Cmd::Run(expected));
    }

    #[test]
    fn should_parse_mining_options() {
        // given
        let mut mining_options = MinerOptions::default();

        // when
        let conf0 = parse(&["openethereum"]);
        let conf2 = parse(&["openethereum", "--tx-queue-strategy", "gas_price"]);

        // then
        assert_eq!(conf0.miner_options().unwrap(), mining_options);
        mining_options.tx_queue_strategy = PrioritizationStrategy::GasPriceOnly;
        assert_eq!(conf2.miner_options().unwrap(), mining_options);
    }

    #[test]
    fn should_fail_on_force_reseal_and_reseal_min_period() {
        let conf = parse(&[
            "openethereum",
            "--chain",
            "dev",
            "--force-sealing",
            "--reseal-min-period",
            "0",
        ]);

        assert!(conf.miner_options().is_err());
    }

    #[test]
    fn should_parse_network_settings() {
        // given

        // when
        let conf = parse(&[
            "openethereum",
            "--chain",
            "goerli",
            "--identity",
            "testname",
        ]);

        // then
        assert_eq!(
            conf.network_settings(),
            Ok(NetworkSettings {
                name: "testname".to_owned(),
                chain: "goerli".to_owned(),
                is_dev_chain: false,
                network_port: 30303,
                rpc_enabled: true,
                rpc_interface: "127.0.0.1".to_owned(),
                rpc_port: 8545,
            })
        );
    }

    #[test]
    fn should_parse_rpc_hosts() {
        // given

        // when
        let conf0 = parse(&["openethereum"]);
        let conf1 = parse(&["openethereum", "--jsonrpc-hosts", "none"]);
        let conf2 = parse(&["openethereum", "--jsonrpc-hosts", "all"]);
        let conf3 = parse(&["openethereum", "--jsonrpc-hosts", "parity.io,something.io"]);

        // then
        assert_eq!(conf0.rpc_hosts(), Some(Vec::new()));
        assert_eq!(conf1.rpc_hosts(), Some(Vec::new()));
        assert_eq!(conf2.rpc_hosts(), None);
        assert_eq!(
            conf3.rpc_hosts(),
            Some(vec!["parity.io".into(), "something.io".into()])
        );
    }

    #[test]
    fn should_parse_ui_configuration() {
        // given

        // when
        let conf0 = parse(&["openethereum", "--ui-path=signer"]);

        // then
        assert_eq!(conf0.directories().signer, "signer".to_owned());
    }

    #[test]
    fn should_not_bail_on_empty_line_in_reserved_peers() {
        let tempdir = TempDir::new("").unwrap();
        let filename = tempdir.path().join("peers");
        File::create(&filename)
            .unwrap()
            .write_all(b"  \n\t\n")
            .unwrap();
        let args = vec![
            "openethereum",
            "--reserved-peers",
            filename.to_str().unwrap(),
        ];
        let conf = Configuration::parse_cli(&args).unwrap();
        assert!(conf.init_reserved_nodes().is_ok());
    }

    #[test]
    fn should_ignore_comments_in_reserved_peers() {
        let tempdir = TempDir::new("").unwrap();
        let filename = tempdir.path().join("peers_comments");
        File::create(&filename).unwrap().write_all(b"# Sample comment\nenode://6f8a80d14311c39f35f516fa664deaaaa13e85b2f7493f37f6144d86991ec012937307647bd3b9a82abe2974e1407241d54947bbb39763a4cac9f77166ad92a0@172.0.0.1:30303\n").unwrap();
        let args = vec![
            "openethereum",
            "--reserved-peers",
            filename.to_str().unwrap(),
        ];
        let conf = Configuration::parse_cli(&args).unwrap();
        let reserved_nodes = conf.init_reserved_nodes();
        assert!(reserved_nodes.is_ok());
        assert_eq!(reserved_nodes.unwrap().len(), 1);
    }

    #[test]
    fn test_dev_preset() {
        let args = vec!["openethereum", "--config", "dev"];
        let conf = Configuration::parse_cli(&args).unwrap();
        match conf.into_command().unwrap().cmd {
            Cmd::Run(c) => {
                assert_eq!(c.net_settings.chain, "dev");
                assert_eq!(c.gas_pricer_conf, GasPricerConfig::Fixed(0.into()));
                assert_eq!(c.miner_options.reseal_min_period, Duration::from_millis(0));
            }
            _ => panic!("Should be Cmd::Run"),
        }
    }

    #[test]
    fn test_mining_preset() {
        let args = vec!["openethereum", "--config", "mining"];
        let conf = Configuration::parse_cli(&args).unwrap();
        match conf.into_command().unwrap().cmd {
            Cmd::Run(c) => {
                assert_eq!(c.net_conf.min_peers, 50);
                assert_eq!(c.net_conf.max_peers, 100);
                assert!(!c.ipc_conf.enabled);
                assert!(c.miner_options.force_sealing);
                assert!(c.miner_options.reseal_on_external_tx);
                assert!(c.miner_options.reseal_on_own_tx);
                assert_eq!(
                    c.miner_options.reseal_min_period,
                    Duration::from_millis(4000)
                );
                assert_eq!(c.miner_options.pool_limits.max_count, 8192);
                assert_eq!(c.cache_config, CacheConfig::new_with_total_cache_size(1024));
                assert_eq!(c.logger_config.mode.unwrap(), "miner=trace,own_tx=trace");
            }
            _ => panic!("Should be Cmd::Run"),
        }
    }

    #[test]
    fn test_non_standard_ports_preset() {
        let args = vec!["openethereum", "--config", "non-standard-ports"];
        let conf = Configuration::parse_cli(&args).unwrap();
        match conf.into_command().unwrap().cmd {
            Cmd::Run(c) => {
                assert_eq!(c.net_settings.network_port, 30305);
                assert_eq!(c.net_settings.rpc_port, 8645);
            }
            _ => panic!("Should be Cmd::Run"),
        }
    }

    #[test]
    fn test_insecure_preset() {
        let args = vec!["openethereum", "--config", "insecure"];
        let conf = Configuration::parse_cli(&args).unwrap();
        match conf.into_command().unwrap().cmd {
            Cmd::Run(c) => {
                assert_eq!(c.net_settings.rpc_interface, "0.0.0.0");
                match c.http_conf.apis {
                    ApiSet::List(set) => assert_eq!(set, ApiSet::All.list_apis()),
                    _ => panic!("Incorrect rpc apis"),
                }
                // "web3,eth,net,personal,parity,parity_set,traces,rpc,parity_accounts");
                assert_eq!(c.http_conf.hosts, None);
            }
            _ => panic!("Should be Cmd::Run"),
        }
    }

    #[test]
    fn test_dev_insecure_preset() {
        let args = vec!["openethereum", "--config", "dev-insecure"];
        let conf = Configuration::parse_cli(&args).unwrap();
        match conf.into_command().unwrap().cmd {
            Cmd::Run(c) => {
                assert_eq!(c.net_settings.chain, "dev");
                assert_eq!(c.gas_pricer_conf, GasPricerConfig::Fixed(0.into()));
                assert_eq!(c.miner_options.reseal_min_period, Duration::from_millis(0));
                assert_eq!(c.net_settings.rpc_interface, "0.0.0.0");
                match c.http_conf.apis {
                    ApiSet::List(set) => assert_eq!(set, ApiSet::All.list_apis()),
                    _ => panic!("Incorrect rpc apis"),
                }
                // "web3,eth,net,personal,parity,parity_set,traces,rpc,parity_accounts");
                assert_eq!(c.http_conf.hosts, None);
            }
            _ => panic!("Should be Cmd::Run"),
        }
    }

    #[test]
    fn test_override_preset() {
        let args = vec!["openethereum", "--config", "mining", "--min-peers=99"];
        let conf = Configuration::parse_cli(&args).unwrap();
        match conf.into_command().unwrap().cmd {
            Cmd::Run(c) => {
                assert_eq!(c.net_conf.min_peers, 99);
            }
            _ => panic!("Should be Cmd::Run"),
        }
    }

    #[test]
    fn test_identity_arg() {
        let args = vec!["openethereum", "--identity", "Somebody"];
        let conf = Configuration::parse_cli(&args).unwrap();
        match conf.into_command().unwrap().cmd {
            Cmd::Run(c) => {
                assert_eq!(c.name, "Somebody");
                assert!(c
                    .net_conf
                    .client_version
                    .starts_with("OpenEthereum/Somebody/"));
            }
            _ => panic!("Should be Cmd::Run"),
        }
    }

    #[test]
    fn should_apply_ports_shift() {
        // given

        // when
        let conf0 = parse(&["openethereum", "--ports-shift", "1", "--stratum"]);
        let conf1 = parse(&[
            "openethereum",
            "--ports-shift",
            "1",
            "--jsonrpc-port",
            "8544",
        ]);

        // then
        assert_eq!(conf0.net_addresses().unwrap().0.port(), 30304);
        assert_eq!(conf0.network_settings().unwrap().network_port, 30304);
        assert_eq!(conf0.network_settings().unwrap().rpc_port, 8546);
        assert_eq!(conf0.http_config().unwrap().port, 8546);
        assert_eq!(conf0.ws_config().unwrap().port, 8547);
        assert_eq!(conf0.secretstore_config().unwrap().port, 8084);
        assert_eq!(conf0.secretstore_config().unwrap().http_port, 8083);
        assert_eq!(conf0.stratum_options().unwrap().unwrap().port, 8009);

        assert_eq!(conf1.net_addresses().unwrap().0.port(), 30304);
        assert_eq!(conf1.network_settings().unwrap().network_port, 30304);
        assert_eq!(conf1.network_settings().unwrap().rpc_port, 8545);
        assert_eq!(conf1.http_config().unwrap().port, 8545);
        assert_eq!(conf1.ws_config().unwrap().port, 8547);
        assert_eq!(conf1.secretstore_config().unwrap().port, 8084);
        assert_eq!(conf1.secretstore_config().unwrap().http_port, 8083);
    }

    #[test]
    fn should_resolve_external_nat_hosts() {
        // Ip works
        let conf = parse(&["openethereum", "--nat", "extip:1.1.1.1"]);
        assert_eq!(
            conf.net_addresses().unwrap().1.unwrap().ip().to_string(),
            "1.1.1.1"
        );
        assert_eq!(conf.net_addresses().unwrap().1.unwrap().port(), 30303);

        // Ip with port works, port is discarded
        let conf = parse(&["openethereum", "--nat", "extip:192.168.1.1:123"]);
        assert_eq!(
            conf.net_addresses().unwrap().1.unwrap().ip().to_string(),
            "192.168.1.1"
        );
        assert_eq!(conf.net_addresses().unwrap().1.unwrap().port(), 30303);

        // Hostname works
        let conf = parse(&["openethereum", "--nat", "extip:ethereum.org"]);
        assert!(conf.net_addresses().unwrap().1.is_some());
        assert_eq!(conf.net_addresses().unwrap().1.unwrap().port(), 30303);

        // Hostname works, garbage at the end is discarded
        let conf = parse(&[
            "openethereum",
            "--nat",
            "extip:ethereum.org:whatever bla bla 123",
        ]);
        assert!(conf.net_addresses().unwrap().1.is_some());
        assert_eq!(conf.net_addresses().unwrap().1.unwrap().port(), 30303);

        // Garbage is error
        let conf = parse(&["openethereum", "--nat", "extip:blabla"]);
        assert!(conf.net_addresses().is_err());
    }

    #[test]
    fn should_expose_all_servers() {
        // given

        // when
        let conf0 = parse(&["openethereum", "--unsafe-expose"]);

        // then
        assert_eq!(&conf0.network_settings().unwrap().rpc_interface, "0.0.0.0");
        assert_eq!(&conf0.http_config().unwrap().interface, "0.0.0.0");
        assert_eq!(conf0.http_config().unwrap().hosts, None);
        assert_eq!(&conf0.ws_config().unwrap().interface, "0.0.0.0");
        assert_eq!(conf0.ws_config().unwrap().hosts, None);
        assert_eq!(conf0.ws_config().unwrap().origins, None);
        assert_eq!(&conf0.secretstore_config().unwrap().interface, "0.0.0.0");
        assert_eq!(
            &conf0.secretstore_config().unwrap().http_interface,
            "0.0.0.0"
        );
    }

    #[test]
    fn allow_ips() {
        let all = parse(&["openethereum", "--allow-ips", "all"]);
        let private = parse(&["openethereum", "--allow-ips", "private"]);
        let block_custom = parse(&["openethereum", "--allow-ips", "-10.0.0.0/8"]);
        let combo = parse(&[
            "openethereum",
            "--allow-ips",
            "public 10.0.0.0/8 -1.0.0.0/8",
        ]);
        let ipv6_custom_public = parse(&["openethereum", "--allow-ips", "public fc00::/7"]);
        let ipv6_custom_private = parse(&["openethereum", "--allow-ips", "private -fc00::/7"]);

        assert_eq!(
            all.ip_filter().unwrap(),
            IpFilter {
                predefined: AllowIP::All,
                custom_allow: vec![],
                custom_block: vec![],
            }
        );

        assert_eq!(
            private.ip_filter().unwrap(),
            IpFilter {
                predefined: AllowIP::Private,
                custom_allow: vec![],
                custom_block: vec![],
            }
        );

        assert_eq!(
            block_custom.ip_filter().unwrap(),
            IpFilter {
                predefined: AllowIP::All,
                custom_allow: vec![],
                custom_block: vec![IpNetwork::from_str("10.0.0.0/8").unwrap()],
            }
        );

        assert_eq!(
            combo.ip_filter().unwrap(),
            IpFilter {
                predefined: AllowIP::Public,
                custom_allow: vec![IpNetwork::from_str("10.0.0.0/8").unwrap()],
                custom_block: vec![IpNetwork::from_str("1.0.0.0/8").unwrap()],
            }
        );

        assert_eq!(
            ipv6_custom_public.ip_filter().unwrap(),
            IpFilter {
                predefined: AllowIP::Public,
                custom_allow: vec![IpNetwork::from_str("fc00::/7").unwrap()],
                custom_block: vec![],
            }
        );

        assert_eq!(
            ipv6_custom_private.ip_filter().unwrap(),
            IpFilter {
                predefined: AllowIP::Private,
                custom_allow: vec![],
                custom_block: vec![IpNetwork::from_str("fc00::/7").unwrap()],
            }
        );
    }

    #[test]
    fn should_use_correct_cache_path_if_base_is_set() {
        use std::path;

        let std = parse(&["openethereum"]);
        let base = parse(&["openethereum", "--base-path", "/test"]);

        let base_path = ::dir::default_data_path();
        let local_path = ::dir::default_local_path();
        assert_eq!(
            std.directories().cache,
            dir::helpers::replace_home_and_local(&base_path, &local_path, ::dir::CACHE_PATH)
        );
        assert_eq!(
            path::Path::new(&base.directories().cache),
            path::Path::new("/test/cache")
        );
    }

    #[test]
    fn should_respect_only_max_peers_and_default() {
        let args = vec!["openethereum", "--max-peers=50"];
        let conf = Configuration::parse_cli(&args).unwrap();
        match conf.into_command().unwrap().cmd {
            Cmd::Run(c) => {
                assert_eq!(c.net_conf.min_peers, 25);
                assert_eq!(c.net_conf.max_peers, 50);
            }
            _ => panic!("Should be Cmd::Run"),
        }
    }

    #[test]
    fn should_respect_only_max_peers_less_than_default() {
        let args = vec!["openethereum", "--max-peers=5"];
        let conf = Configuration::parse_cli(&args).unwrap();
        match conf.into_command().unwrap().cmd {
            Cmd::Run(c) => {
                assert_eq!(c.net_conf.min_peers, 5);
                assert_eq!(c.net_conf.max_peers, 5);
            }
            _ => panic!("Should be Cmd::Run"),
        }
    }

    #[test]
    fn should_respect_only_min_peers_and_default() {
        let args = vec!["openethereum", "--min-peers=5"];
        let conf = Configuration::parse_cli(&args).unwrap();
        match conf.into_command().unwrap().cmd {
            Cmd::Run(c) => {
                assert_eq!(c.net_conf.min_peers, 5);
                assert_eq!(c.net_conf.max_peers, 50);
            }
            _ => panic!("Should be Cmd::Run"),
        }
    }

    #[test]
    fn should_respect_only_min_peers_and_greater_than_default() {
        let args = vec!["openethereum", "--min-peers=500"];
        let conf = Configuration::parse_cli(&args).unwrap();
        match conf.into_command().unwrap().cmd {
            Cmd::Run(c) => {
                assert_eq!(c.net_conf.min_peers, 500);
                assert_eq!(c.net_conf.max_peers, 500);
            }
            _ => panic!("Should be Cmd::Run"),
        }
    }
}
