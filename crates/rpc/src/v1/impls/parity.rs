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

//! Parity-specific rpc implementation.
use std::{collections::BTreeMap, str::FromStr, sync::Arc};

use crypto::{publickey::ecies, DEFAULT_MAC};
use ethcore::{
    client::{BlockChainClient, Call, EngineInfo, StateClient},
    miner::{self, MinerService, TransactionFilter},
    snapshot::{RestorationStatus, SnapshotService},
    state::StateInfo,
};
use ethcore_logger::RotatingLogger;
use ethereum_types::{Address, H160, H256, H512, H64, U256, U64};
use ethkey::Brain;
use ethstore::random_phrase;
use jsonrpc_core::{futures::future, BoxFuture, Result};
use stats::PrometheusMetrics;
use sync::{ManageNetwork, SyncProvider};
use types::ids::BlockId;
use v1::{
    helpers::{
        self,
        block_import::is_major_importing,
        errors,
        external_signer::{SignerService, SigningQueue},
        fake_sign, verify_signature, NetworkSettings,
    },
    traits::Parity,
    types::{
        block_number_to_id, BlockNumber, Bytes, CallRequest, ChainStatus, Header, Histogram,
        LocalTransactionStatus, Peers, Receipt, RecoveredAccount, RichHeader, RpcSettings,
        Transaction, TransactionStats,
    },
};
use version::version_data;
use Host;

/// Parity implementation.
pub struct ParityClient<C, M>
where
    C: PrometheusMetrics,
{
    client: Arc<C>,
    miner: Arc<M>,
    sync: Arc<dyn SyncProvider>,
    net: Arc<dyn ManageNetwork>,
    logger: Arc<RotatingLogger>,
    settings: Arc<NetworkSettings>,
    signer: Option<Arc<SignerService>>,
    ws_address: Option<Host>,
    snapshot: Option<Arc<dyn SnapshotService>>,
}

impl<C, M> ParityClient<C, M>
where
    C: BlockChainClient + PrometheusMetrics + EngineInfo,
{
    /// Creates new `ParityClient`.
    pub fn new(
        client: Arc<C>,
        miner: Arc<M>,
        sync: Arc<dyn SyncProvider>,
        net: Arc<dyn ManageNetwork>,
        logger: Arc<RotatingLogger>,
        settings: Arc<NetworkSettings>,
        signer: Option<Arc<SignerService>>,
        ws_address: Option<Host>,
        snapshot: Option<Arc<dyn SnapshotService>>,
    ) -> Self {
        ParityClient {
            client,
            miner,
            sync,
            net,
            logger,
            settings,
            signer,
            ws_address,
            snapshot,
        }
    }
}

impl<C, M, S> Parity for ParityClient<C, M>
where
    S: StateInfo + 'static,
    C: miner::BlockChainClient
        + BlockChainClient
        + PrometheusMetrics
        + StateClient<State = S>
        + Call<State = S>
        + EngineInfo
        + 'static,
    M: MinerService<State = S> + 'static,
{
    fn transactions_limit(&self) -> Result<usize> {
        Ok(self.miner.queue_status().limits.max_count)
    }

    fn min_gas_price(&self) -> Result<U256> {
        Ok(self.miner.queue_status().options.minimal_gas_price)
    }

    fn extra_data(&self) -> Result<Bytes> {
        Ok(Bytes::new(self.miner.authoring_params().extra_data))
    }

    fn gas_floor_target(&self) -> Result<U256> {
        Ok(self.miner.authoring_params().gas_range_target.0)
    }

    fn gas_ceil_target(&self) -> Result<U256> {
        Ok(self.miner.authoring_params().gas_range_target.1)
    }

    fn dev_logs(&self) -> Result<Vec<String>> {
        warn!("This method is deprecated and will be removed in future. See PR #10102");
        let logs = self.logger.logs();
        Ok(logs.as_slice().to_owned())
    }

    fn dev_logs_levels(&self) -> Result<String> {
        Ok(self.logger.levels().to_owned())
    }

    fn net_chain(&self) -> Result<String> {
        Ok(self.settings.chain.clone())
    }

    fn chain(&self) -> Result<String> {
        Ok(self.client.spec_name())
    }

    fn net_peers(&self) -> Result<Peers> {
        let sync_status = self.sync.status();
        let num_peers_range = self.net.num_peers_range();
        debug_assert!(num_peers_range.end() >= num_peers_range.start());
        let peers = self.sync.peers().into_iter().map(Into::into).collect();

        Ok(Peers {
            active: sync_status.num_active_peers,
            connected: sync_status.num_peers,
            max: sync_status.current_max_peers(*num_peers_range.start(), *num_peers_range.end()),
            peers,
        })
    }

    fn net_port(&self) -> Result<u16> {
        Ok(self.settings.network_port)
    }

    fn node_name(&self) -> Result<String> {
        Ok(self.settings.name.clone())
    }

    fn registry_address(&self) -> Result<Option<H160>> {
        Ok(self
            .client
            .additional_params()
            .get("registrar")
            .and_then(|s| Address::from_str(s).ok()))
    }

    fn rpc_settings(&self) -> Result<RpcSettings> {
        Ok(RpcSettings {
            enabled: self.settings.rpc_enabled,
            interface: self.settings.rpc_interface.clone(),
            port: u64::from(self.settings.rpc_port),
        })
    }

    fn default_extra_data(&self) -> Result<Bytes> {
        Ok(Bytes::new(version_data()))
    }

    fn gas_price_histogram(&self) -> BoxFuture<Histogram> {
        Box::new(future::done(
            self.client
                .gas_price_corpus(100)
                .histogram(10)
                .ok_or_else(errors::not_enough_data)
                .map(Into::into),
        ))
    }

    fn unsigned_transactions_count(&self) -> Result<usize> {
        match self.signer {
            None => Err(errors::signer_disabled()),
            Some(ref signer) => Ok(signer.len()),
        }
    }

    fn generate_secret_phrase(&self) -> Result<String> {
        Ok(random_phrase(12))
    }

    fn phrase_to_address(&self, phrase: String) -> Result<H160> {
        Ok(Brain::new(phrase).generate().address())
    }

    fn list_accounts(
        &self,
        count: u64,
        after: Option<H160>,
        block_number: Option<BlockNumber>,
    ) -> Result<Option<Vec<H160>>> {
        let number = match block_number.unwrap_or_default() {
            BlockNumber::Pending => {
                warn!("BlockNumber::Pending is unsupported");
                return Ok(None);
            }

            num => block_number_to_id(num),
        };

        Ok(self
            .client
            .list_accounts(number, after.as_ref(), count)
            .map(|a| a.into_iter().collect()))
    }

    fn list_storage_keys(
        &self,
        address: H160,
        count: u64,
        after: Option<H256>,
        block_number: Option<BlockNumber>,
    ) -> Result<Option<Vec<H256>>> {
        let number = match block_number.unwrap_or_default() {
            BlockNumber::Pending => {
                warn!("BlockNumber::Pending is unsupported");
                return Ok(None);
            }

            num => block_number_to_id(num),
        };

        Ok(self
            .client
            .list_storage(number, &address, after.as_ref(), count)
            .map(|a| a.into_iter().collect()))
    }

    fn encrypt_message(&self, key: H512, phrase: Bytes) -> Result<Bytes> {
        ecies::encrypt(&key, &DEFAULT_MAC, &phrase.0)
            .map_err(errors::encryption)
            .map(Into::into)
    }

    fn pending_transactions(
        &self,
        limit: Option<usize>,
        filter: Option<TransactionFilter>,
    ) -> Result<Vec<Transaction>> {
        let ready_transactions = self.miner.ready_transactions_filtered(
            &*self.client,
            limit.unwrap_or_else(usize::max_value),
            filter,
            miner::PendingOrdering::Priority,
        );

        Ok(ready_transactions
            .into_iter()
            .map(|t| Transaction::from_pending(t.pending().clone()))
            .collect())
    }

    fn all_transactions(&self) -> Result<Vec<Transaction>> {
        let all_transactions = self.miner.queued_transactions();

        Ok(all_transactions
            .into_iter()
            .map(|t| Transaction::from_pending(t.pending().clone()))
            .collect())
    }

    fn all_transaction_hashes(&self) -> Result<Vec<H256>> {
        Ok(self.miner.queued_transaction_hashes())
    }

    fn future_transactions(&self) -> Result<Vec<Transaction>> {
        Err(errors::deprecated("Use `parity_allTransaction` instead."))
    }

    fn pending_transactions_stats(&self) -> Result<BTreeMap<H256, TransactionStats>> {
        let stats = self.sync.pending_transactions_stats();
        Ok(stats
            .into_iter()
            .map(|(hash, stats)| (hash, stats.into()))
            .collect())
    }

    fn new_transactions_stats(&self) -> Result<BTreeMap<H256, TransactionStats>> {
        let stats = self.sync.new_transactions_stats();
        Ok(stats
            .into_iter()
            .map(|(hash, stats)| (hash, stats.into()))
            .collect())
    }

    fn local_transactions(&self) -> Result<BTreeMap<H256, LocalTransactionStatus>> {
        let transactions = self.miner.local_transactions();
        Ok(transactions
            .into_iter()
            .map(|(hash, status)| (hash, LocalTransactionStatus::from(status)))
            .collect())
    }

    fn ws_url(&self) -> Result<String> {
        helpers::to_url(&self.ws_address).ok_or_else(errors::ws_disabled)
    }

    fn next_nonce(&self, address: H160) -> BoxFuture<U256> {
        Box::new(future::ok(self.miner.next_nonce(&*self.client, &address)))
    }

    fn mode(&self) -> Result<String> {
        Ok(self.client.mode().to_string())
    }

    fn enode(&self) -> Result<String> {
        self.sync.enode().ok_or_else(errors::network_disabled)
    }

    fn chain_status(&self) -> Result<ChainStatus> {
        let chain_info = self.client.chain_info();

        let gap = chain_info
            .ancient_block_number
            .map(|x| U256::from(x + 1))
            .and_then(|first| {
                chain_info
                    .first_block_number
                    .map(|last| (first, U256::from(last)))
            });

        Ok(ChainStatus { block_gap: gap })
    }

    fn node_kind(&self) -> Result<::v1::types::NodeKind> {
        use v1::types::{Availability, Capability, NodeKind};

        Ok(NodeKind {
            availability: Availability::Personal,
            capability: Capability::Full,
        })
    }

    fn block_header(&self, number: Option<BlockNumber>) -> BoxFuture<RichHeader> {
        const EXTRA_INFO_PROOF: &str = "Object exists in blockchain (fetched earlier), extra_info is always available if object exists; qed";
        let number = number.unwrap_or_default();

        let (header, extra) = if number == BlockNumber::Pending {
            let info = self.client.chain_info();
            let header = try_bf!(self
                .miner
                .pending_block_header(info.best_block_number)
                .ok_or_else(errors::unknown_block));

            (header.encoded(), None)
        } else {
            let id = match number {
                BlockNumber::Hash { hash, .. } => BlockId::Hash(hash),
                BlockNumber::Num(num) => BlockId::Number(num),
                BlockNumber::Earliest => BlockId::Earliest,
                BlockNumber::Latest => BlockId::Latest,
                BlockNumber::Pending => unreachable!(), // Already covered
            };

            let header = try_bf!(self
                .client
                .block_header(id)
                .ok_or_else(errors::unknown_block));
            let info = self.client.block_extra_info(id).expect(EXTRA_INFO_PROOF);

            (header, Some(info))
        };

        Box::new(future::ok(RichHeader {
            inner: Header::new(&header, self.client.engine().params().eip1559_transition),
            extra_info: extra.unwrap_or_default(),
        }))
    }

    fn block_receipts(&self, number: Option<BlockNumber>) -> BoxFuture<Vec<Receipt>> {
        let number = number.unwrap_or_default();

        let id = match number {
            BlockNumber::Pending => {
                let info = self.client.chain_info();
                let receipts = try_bf!(self
                    .miner
                    .pending_receipts(info.best_block_number)
                    .ok_or_else(errors::unknown_block));
                return Box::new(future::ok(receipts.into_iter().map(Into::into).collect()));
            }
            BlockNumber::Hash { hash, .. } => BlockId::Hash(hash),
            BlockNumber::Num(num) => BlockId::Number(num),
            BlockNumber::Earliest => BlockId::Earliest,
            BlockNumber::Latest => BlockId::Latest,
        };
        let receipts = try_bf!(self
            .client
            .localized_block_receipts(id)
            .ok_or_else(errors::unknown_block));
        Box::new(future::ok(receipts.into_iter().map(Into::into).collect()))
    }

    fn call(&self, requests: Vec<CallRequest>, num: Option<BlockNumber>) -> Result<Vec<Bytes>> {
        let requests = requests
            .into_iter()
            .map(|request| Ok((fake_sign::sign_call(request.into())?, Default::default())))
            .collect::<Result<Vec<_>>>()?;

        let num = num.unwrap_or_default();

        let (mut state, header) = if num == BlockNumber::Pending {
            let info = self.client.chain_info();
            let state = self
                .miner
                .pending_state(info.best_block_number)
                .ok_or_else(errors::state_pruned)?;
            let header = self
                .miner
                .pending_block_header(info.best_block_number)
                .ok_or_else(errors::state_pruned)?;

            (state, header)
        } else {
            let id = match num {
                BlockNumber::Hash { hash, .. } => BlockId::Hash(hash),
                BlockNumber::Num(num) => BlockId::Number(num),
                BlockNumber::Earliest => BlockId::Earliest,
                BlockNumber::Latest => BlockId::Latest,
                BlockNumber::Pending => unreachable!(), // Already covered
            };

            let state = self.client.state_at(id).ok_or_else(errors::state_pruned)?;
            let header = self
                .client
                .block_header(id)
                .ok_or_else(errors::state_pruned)?
                .decode(self.client.engine().params().eip1559_transition)
                .map_err(errors::decode)?;

            (state, header)
        };

        self.client
            .call_many(&requests, &mut state, &header)
            .map(|res| res.into_iter().map(|res| res.output.into()).collect())
            .map_err(errors::call)
    }

    fn submit_work_detail(&self, nonce: H64, pow_hash: H256, mix_hash: H256) -> Result<H256> {
        helpers::submit_work_detail(&self.client, &self.miner, nonce, pow_hash, mix_hash)
    }

    fn status(&self) -> Result<()> {
        let has_peers = self.settings.is_dev_chain || self.sync.status().num_peers > 0;
        let is_warping = match self.snapshot.as_ref().map(|s| s.restoration_status()) {
            Some(RestorationStatus::Ongoing { .. }) => true,
            _ => false,
        };
        let is_not_syncing = !is_warping
            && !is_major_importing(Some(self.sync.status().state), self.client.queue_info());

        if has_peers && is_not_syncing {
            Ok(())
        } else {
            Err(errors::status_error(has_peers))
        }
    }

    fn verify_signature(
        &self,
        is_prefixed: bool,
        message: Bytes,
        r: H256,
        s: H256,
        v: U64,
    ) -> Result<RecoveredAccount> {
        verify_signature(
            is_prefixed,
            message,
            r,
            s,
            v,
            self.client.signing_chain_id(),
        )
    }
}
