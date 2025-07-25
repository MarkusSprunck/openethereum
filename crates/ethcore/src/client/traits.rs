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

//! Traits implemented by client.

use std::{collections::BTreeMap, sync::Arc};

use blockchain::{BlockReceipts, TreeRoute};
use bytes::Bytes;
use call_contract::{CallContract, RegistryInfo};
use ethcore_miner::pool::VerifiedTransaction;
use ethereum_types::{Address, H256, U256};
use evm::Schedule;
use itertools::Itertools;
use kvdb::DBValue;
use types::{
    basic_account::BasicAccount,
    block_status::BlockStatus,
    blockchain_info::BlockChainInfo,
    call_analytics::CallAnalytics,
    data_format::DataFormat,
    encoded,
    filter::Filter,
    header::Header,
    ids::*,
    log_entry::LocalizedLogEntry,
    pruning_info::PruningInfo,
    receipt::LocalizedReceipt,
    trace_filter::Filter as TraceFilter,
    transaction::{self, Action, LocalizedTransaction, SignedTransaction, TypedTxId},
    BlockNumber,
};
use vm::LastHashes;

use block::{ClosedBlock, OpenBlock, SealedBlock};
use client::Mode;
use engines::EthEngine;
use error::{Error, EthcoreResult};
use executed::CallError;
use executive::Executed;
use state::StateInfo;
use trace::LocalizedTrace;
use verification::queue::{kind::blocks::Unverified, QueueInfo as BlockQueueInfo};

/// State information to be used during client query
pub enum StateOrBlock {
    /// State to be used, may be pending
    State(Box<dyn StateInfo>),

    /// Id of an existing block from a chain to get state from
    Block(BlockId),
}

impl<S: StateInfo + 'static> From<S> for StateOrBlock {
    fn from(info: S) -> StateOrBlock {
        StateOrBlock::State(Box::new(info) as Box<_>)
    }
}

impl From<Box<dyn StateInfo>> for StateOrBlock {
    fn from(info: Box<dyn StateInfo>) -> StateOrBlock {
        StateOrBlock::State(info)
    }
}

impl From<BlockId> for StateOrBlock {
    fn from(id: BlockId) -> StateOrBlock {
        StateOrBlock::Block(id)
    }
}

/// Provides `nonce` and `latest_nonce` methods
pub trait Nonce {
    /// Attempt to get address nonce at given block.
    /// May not fail on BlockId::Latest.
    fn nonce(&self, address: &Address, id: BlockId) -> Option<U256>;

    /// Get address nonce at the latest block's state.
    fn latest_nonce(&self, address: &Address) -> U256 {
        self.nonce(address, BlockId::Latest).expect(
            "nonce will return Some when given BlockId::Latest. nonce was given BlockId::Latest. \
			Therefore nonce has returned Some; qed",
        )
    }
}

/// Provides `balance` and `latest_balance` methods
pub trait Balance {
    /// Get address balance at the given block's state.
    ///
    /// May not return None if given BlockId::Latest.
    /// Returns None if and only if the block's root hash has been pruned from the DB.
    fn balance(&self, address: &Address, state: StateOrBlock) -> Option<U256>;

    /// Get address balance at the latest block's state.
    fn latest_balance(&self, address: &Address) -> U256 {
        self.balance(address, BlockId::Latest.into()).expect(
            "balance will return Some if given BlockId::Latest. balance was given BlockId::Latest \
			Therefore balance has returned Some; qed",
        )
    }
}

/// Provides methods to access account info
pub trait AccountData: Nonce + Balance {}

/// Provides `chain_info` method
pub trait ChainInfo {
    /// Get blockchain information.
    fn chain_info(&self) -> BlockChainInfo;
}

/// Provides various information on a block by it's ID
pub trait BlockInfo {
    /// Get raw block header data by block id.
    fn block_header(&self, id: BlockId) -> Option<encoded::Header>;

    /// Get the best block header.
    fn best_block_header(&self) -> Header;

    /// Get raw block data by block header hash.
    fn block(&self, id: BlockId) -> Option<encoded::Block>;

    /// Get address code hash at given block's state.
    fn code_hash(&self, address: &Address, id: BlockId) -> Option<H256>;
}

/// Provides various information on a transaction by it's ID
pub trait TransactionInfo {
    /// Get the hash of block that contains the transaction, if any.
    fn transaction_block(&self, id: TransactionId) -> Option<H256>;
}

/// Provides methods to access chain state
pub trait StateClient {
    /// Type representing chain state
    type State: StateInfo;

    /// Get a copy of the best block's state and header.
    fn latest_state_and_header(&self) -> (Self::State, Header);

    /// Attempt to get a copy of a specific block's final state.
    ///
    /// This will not fail if given BlockId::Latest.
    /// Otherwise, this can fail (but may not) if the DB prunes state or the block
    /// is unknown.
    fn state_at(&self, id: BlockId) -> Option<Self::State>;
}

/// Provides various blockchain information, like block header, chain state etc.
pub trait BlockChain: ChainInfo + BlockInfo + TransactionInfo {}

// FIXME Why these methods belong to BlockChainClient and not MiningBlockChainClient?
/// Provides methods to import block into blockchain
pub trait ImportBlock {
    /// Import a block into the blockchain.
    fn import_block(&self, block: Unverified) -> EthcoreResult<H256>;
}

/// Provides `call` and `call_many` methods
pub trait Call {
    /// Type representing chain state
    type State: StateInfo;

    /// Makes a non-persistent transaction call.
    fn call(
        &self,
        tx: &SignedTransaction,
        analytics: CallAnalytics,
        state: &mut Self::State,
        header: &Header,
    ) -> Result<Executed, CallError>;

    /// Makes multiple non-persistent but dependent transaction calls.
    /// Returns a vector of successes or a failure if any of the transaction fails.
    fn call_many(
        &self,
        txs: &[(SignedTransaction, CallAnalytics)],
        state: &mut Self::State,
        header: &Header,
    ) -> Result<Vec<Executed>, CallError>;

    /// Estimates how much gas will be necessary for a call.
    fn estimate_gas(
        &self,
        t: &SignedTransaction,
        state: &Self::State,
        header: &Header,
    ) -> Result<U256, CallError>;
}

/// Provides `engine` method
pub trait EngineInfo {
    /// Get underlying engine object
    fn engine(&self) -> &dyn EthEngine;
}

/// IO operations that should off-load heavy work to another thread.
pub trait IoClient: Sync + Send {
    /// Queue transactions for importing.
    fn queue_transactions(&self, transactions: Vec<Bytes>, peer_id: usize);

    /// Queue block import with transaction receipts. Does no sealing and transaction validation.
    fn queue_ancient_block(
        &self,
        block_bytes: Unverified,
        receipts_bytes: Bytes,
    ) -> EthcoreResult<H256>;

    /// Return percentage of how full is queue that handles ancient blocks. 0 if empty, 1 if full.
    fn ancient_block_queue_fullness(&self) -> f32;

    /// Queue conensus engine message.
    fn queue_consensus_message(&self, message: Bytes);
}

/// Provides recently seen bad blocks.
pub trait BadBlocks {
    /// Returns a list of blocks that were recently not imported because they were invalid.
    fn bad_blocks(&self) -> Vec<(Unverified, String)>;
}

/// Blockchain database client. Owns and manages a blockchain and a block queue.
pub trait BlockChainClient:
    Sync
    + Send
    + AccountData
    + BlockChain
    + CallContract
    + RegistryInfo
    + ImportBlock
    + IoClient
    + BadBlocks
{
    /// Look up the block number for the given block ID.
    fn block_number(&self, id: BlockId) -> Option<BlockNumber>;

    /// Get raw block body data by block id.
    /// Block body is an RLP list of two items: uncles and transactions.
    fn block_body(&self, id: BlockId) -> Option<encoded::Body>;

    /// Get block status by block header hash.
    fn block_status(&self, id: BlockId) -> BlockStatus;

    /// Get block total difficulty.
    fn block_total_difficulty(&self, id: BlockId) -> Option<U256>;

    /// Attempt to get address storage root at given block.
    /// May not fail on BlockId::Latest.
    fn storage_root(&self, address: &Address, id: BlockId) -> Option<H256>;

    /// Get block hash.
    fn block_hash(&self, id: BlockId) -> Option<H256>;

    /// Get address code at given block's state.
    fn code(&self, address: &Address, state: StateOrBlock) -> Option<Option<Bytes>>;

    /// Get address code at the latest block's state.
    fn latest_code(&self, address: &Address) -> Option<Bytes> {
        self.code(address, BlockId::Latest.into())
            .expect("code will return Some if given BlockId::Latest; qed")
    }

    /// Returns true if the given block is known and in the canon chain.
    fn is_canon(&self, hash: &H256) -> bool;

    /// Get address code hash at given block's state.

    /// Get value of the storage at given position at the given block's state.
    ///
    /// May not return None if given BlockId::Latest.
    /// Returns None if and only if the block's root hash has been pruned from the DB.
    fn storage_at(&self, address: &Address, position: &H256, state: StateOrBlock) -> Option<H256>;

    /// Get value of the storage at given position at the latest block's state.
    fn latest_storage_at(&self, address: &Address, position: &H256) -> H256 {
        self.storage_at(address, position, BlockId::Latest.into())
			.expect("storage_at will return Some if given BlockId::Latest. storage_at was given BlockId::Latest. \
			Therefore storage_at has returned Some; qed")
    }

    /// Get a list of all accounts in the block `id`, if fat DB is in operation, otherwise `None`.
    /// If `after` is set the list starts with the following item.
    fn list_accounts(
        &self,
        id: BlockId,
        after: Option<&Address>,
        count: u64,
    ) -> Option<Vec<Address>>;

    /// Get a list of all storage keys in the block `id`, if fat DB is in operation, otherwise `None`.
    /// If `after` is set the list starts with the following item.
    fn list_storage(
        &self,
        id: BlockId,
        account: &Address,
        after: Option<&H256>,
        count: u64,
    ) -> Option<Vec<H256>>;

    /// Get transaction with given hash.
    fn block_transaction(&self, id: TransactionId) -> Option<LocalizedTransaction>;

    /// Get pool transaction with a given hash.
    fn queued_transaction(&self, hash: H256) -> Option<Arc<VerifiedTransaction>>;

    /// Get uncle with given id.
    fn uncle(&self, id: UncleId) -> Option<encoded::Header>;

    /// Get transaction receipt with given hash.
    fn transaction_receipt(&self, id: TransactionId) -> Option<LocalizedReceipt>;

    /// Get localized receipts for all transaction in given block.
    fn localized_block_receipts(&self, id: BlockId) -> Option<Vec<LocalizedReceipt>>;

    /// Get a tree route between `from` and `to`.
    /// See `BlockChain::tree_route`.
    fn tree_route(&self, from: &H256, to: &H256) -> Option<TreeRoute>;

    /// Get all possible uncle hashes for a block.
    fn find_uncles(&self, hash: &H256) -> Option<Vec<H256>>;

    /// Get latest state node
    fn state_data(&self, hash: &H256) -> Option<Bytes>;

    /// Get block receipts data by block header hash.
    fn block_receipts(&self, hash: &H256) -> Option<BlockReceipts>;

    /// Get block queue information.
    fn queue_info(&self) -> BlockQueueInfo;

    /// Returns true if block queue is empty.
    fn is_queue_empty(&self) -> bool {
        self.queue_info().is_empty()
    }

    /// Clear block queue and abort all import activity.
    fn clear_queue(&self);

    /// Get the registrar address, if it exists.
    fn additional_params(&self) -> BTreeMap<String, String>;

    /// Returns logs matching given filter. If one of the filtering block cannot be found, returns the block id that caused the error.
    fn logs(&self, filter: Filter) -> Result<Vec<LocalizedLogEntry>, BlockId>;

    /// Replays a given transaction for inspection.
    fn replay(&self, t: TransactionId, analytics: CallAnalytics) -> Result<Executed, CallError>;

    /// Replays all the transactions in a given block for inspection.
    fn replay_block_transactions(
        &self,
        block: BlockId,
        analytics: CallAnalytics,
    ) -> Result<Box<dyn Iterator<Item = (H256, Executed)>>, CallError>;

    /// Returns traces matching given filter.
    fn filter_traces(&self, filter: TraceFilter) -> Option<Vec<LocalizedTrace>>;

    /// Returns trace with given id.
    fn trace(&self, trace: TraceId) -> Option<LocalizedTrace>;

    /// Returns traces created by transaction.
    fn transaction_traces(&self, trace: TransactionId) -> Option<Vec<LocalizedTrace>>;

    /// Returns traces created by transaction from block.
    fn block_traces(&self, trace: BlockId) -> Option<Vec<LocalizedTrace>>;

    /// Get last hashes starting from best block.
    fn last_hashes(&self) -> LastHashes;

    /// List all ready transactions that should be propagated to other peers.
    fn transactions_to_propagate(&self) -> Vec<Arc<VerifiedTransaction>>;

    /// Get verified transaction with specified transaction hash.
    fn transaction(&self, tx_hash: &H256) -> Option<Arc<VerifiedTransaction>>;

    /// Sorted list of transaction gas prices from at least last sample_size blocks.
    fn gas_price_corpus(&self, sample_size: usize) -> ::stats::Corpus<U256> {
        let mut h = self.chain_info().best_block_hash;
        let mut corpus = Vec::new();
        while corpus.is_empty() {
            for _ in 0..sample_size {
                let block = match self.block(BlockId::Hash(h)) {
                    Some(block) => block,
                    None => return corpus.into(),
                };

                if block.number() == 0 {
                    return corpus.into();
                }
                block.transaction_views().iter().foreach(|t| {
                    corpus.push(t.effective_gas_price({
                        match t.transaction_type() {
                            TypedTxId::Legacy => None,
                            TypedTxId::AccessList => None,
                            TypedTxId::EIP1559Transaction => Some(block.header().base_fee()),
                        }
                    }))
                });
                h = block.parent_hash();
            }
        }
        corpus.into()
    }

    /// Sorted list of transaction priority gas prices from at least last sample_size blocks.
    fn priority_gas_price_corpus(
        &self,
        sample_size: usize,
        eip1559_transition: BlockNumber,
    ) -> ::stats::Corpus<U256> {
        let mut h = self.chain_info().best_block_hash;
        let mut corpus = Vec::new();
        while corpus.is_empty() {
            for _ in 0..sample_size {
                let block = match self.block(BlockId::Hash(h)) {
                    Some(block) => block,
                    None => return corpus.into(),
                };

                if block.number() == 0 || block.number() < eip1559_transition {
                    return corpus.into();
                }
                block
                    .transaction_views()
                    .iter()
                    .filter(
                        |t| t.gas_price() > 0.into(), /* filter zero cost transactions */
                    )
                    .foreach(|t| {
                        // As block.number() >= eip_1559_transition, the base_fee should exist
                        corpus.push(t.effective_priority_gas_price(Some(block.header().base_fee())))
                    });
                h = block.parent_hash();
            }
        }
        corpus.into()
    }

    /// Get the preferred chain ID to sign on
    fn signing_chain_id(&self) -> Option<u64>;

    /// Get the mode.
    fn mode(&self) -> Mode;

    /// Set the mode.
    fn set_mode(&self, mode: Mode);

    /// Get the chain spec name.
    fn spec_name(&self) -> String;

    /// Set the chain via a spec name.
    fn set_spec_name(&self, spec_name: String) -> Result<(), ()>;

    /// Disable the client from importing blocks. This cannot be undone in this session and indicates
    /// that a subsystem has reason to believe this executable incapable of syncing the chain.
    fn disable(&self);

    /// Returns engine-related extra info for `BlockId`.
    fn block_extra_info(&self, id: BlockId) -> Option<BTreeMap<String, String>>;

    /// Returns engine-related extra info for `UncleId`.
    fn uncle_extra_info(&self, id: UncleId) -> Option<BTreeMap<String, String>>;

    /// Returns information about pruning/data availability.
    fn pruning_info(&self) -> PruningInfo;

    /// Returns a transaction signed with the key configured in the engine signer.
    fn create_transaction(
        &self,
        tx_request: TransactionRequest,
    ) -> Result<SignedTransaction, transaction::Error>;

    /// Schedule state-altering transaction to be executed on the next pending
    /// block with the given gas and nonce parameters.
    fn transact(&self, tx_request: TransactionRequest) -> Result<(), transaction::Error>;

    /// Get the address of the registry itself.
    fn registrar_address(&self) -> Option<Address>;

    /// Returns true, if underlying import queue is processing possible fork at the moment
    fn is_processing_fork(&self) -> bool;
}

/// The data required for a `Client` to create a transaction.
///
/// Gas limit, gas price, or nonce can be set explicitly, e.g. to create service
/// transactions with zero gas price, or sequences of transactions with consecutive nonces.
/// Added for AuRa needs.
pub struct TransactionRequest {
    /// Transaction action
    pub action: Action,
    /// Transaction data
    pub data: Bytes,
    /// Transaction gas usage
    pub gas: Option<U256>,
    /// Transaction gas price
    pub gas_price: Option<U256>,
    /// Transaction nonce
    pub nonce: Option<U256>,
}

impl TransactionRequest {
    /// Creates a request to call a contract at `address` with the specified call data.
    pub fn call(address: Address, data: Bytes) -> TransactionRequest {
        TransactionRequest {
            action: Action::Call(address),
            data,
            gas: None,
            gas_price: None,
            nonce: None,
        }
    }

    /// Sets a gas limit. If this is not specified, a sensible default is used.
    pub fn gas(mut self, gas: U256) -> TransactionRequest {
        self.gas = Some(gas);
        self
    }

    /// Sets a gas price. If this is not specified or `None`, a sensible default is used.
    pub fn gas_price<T: Into<Option<U256>>>(mut self, gas_price: T) -> TransactionRequest {
        self.gas_price = gas_price.into();
        self
    }

    /// Sets a nonce. If this is not specified, the appropriate latest nonce for the author is used.
    pub fn nonce(mut self, nonce: U256) -> TransactionRequest {
        self.nonce = Some(nonce);
        self
    }
}

/// Provides `reopen_block` method
pub trait ReopenBlock {
    /// Reopens an OpenBlock and updates uncles.
    fn reopen_block(&self, block: ClosedBlock) -> OpenBlock;
}

/// Provides `prepare_open_block` method
pub trait PrepareOpenBlock {
    /// Returns OpenBlock prepared for closing.
    fn prepare_open_block(
        &self,
        author: Address,
        gas_range_target: (U256, U256),
        extra_data: Bytes,
    ) -> Result<OpenBlock, Error>;
}

/// Provides methods used for sealing new state
pub trait BlockProducer: PrepareOpenBlock + ReopenBlock {}

/// Provides `latest_schedule` method
pub trait ScheduleInfo {
    /// Returns latest schedule.
    fn latest_schedule(&self) -> Schedule;
}

///Provides `import_sealed_block` method
pub trait ImportSealedBlock {
    /// Import sealed block. Skips all verifications.
    fn import_sealed_block(&self, block: SealedBlock) -> EthcoreResult<H256>;
}

/// Provides `broadcast_proposal_block` method
pub trait BroadcastProposalBlock {
    /// Broadcast a block proposal.
    fn broadcast_proposal_block(&self, block: SealedBlock);
}

/// Provides methods to import sealed block and broadcast a block proposal
pub trait SealedBlockImporter: ImportSealedBlock + BroadcastProposalBlock {}

/// Do we want to force update sealing?
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ForceUpdateSealing {
    /// Ideally you want to use `No` at all times as `Yes` skips `reseal_required` checks.
    Yes,
    /// Don't skip `reseal_required` checks
    No,
}

/// Client facilities used by internally sealing Engines.
pub trait EngineClient: Sync + Send + ChainInfo {
    /// Make a new block and seal it.
    fn update_sealing(&self, force: ForceUpdateSealing);

    /// Submit a seal for a block in the mining queue.
    fn submit_seal(&self, block_hash: H256, seal: Vec<Bytes>);

    /// Broadcast a consensus message to the network.
    fn broadcast_consensus_message(&self, message: Bytes);

    /// Get the transition to the epoch the given parent hash is part of
    /// or transitions to.
    /// This will give the epoch that any children of this parent belong to.
    ///
    /// The block corresponding the the parent hash must be stored already.
    fn epoch_transition_for(&self, parent_hash: H256) -> Option<::engines::EpochTransition>;

    /// Attempt to cast the engine client to a full client.
    fn as_full_client(&self) -> Option<&dyn BlockChainClient>;

    /// Get a block number by ID.
    fn block_number(&self, id: BlockId) -> Option<BlockNumber>;

    /// Get raw block header data by block id.
    fn block_header(&self, id: BlockId) -> Option<encoded::Header>;
}

/// Extended client interface for providing proofs of the state.
pub trait ProvingBlockChainClient: BlockChainClient {
    /// Prove account storage at a specific block id.
    ///
    /// Both provided keys assume a secure trie.
    /// Returns a vector of raw trie nodes (in order from the root) proving the storage query.
    fn prove_storage(&self, key1: H256, key2: H256, id: BlockId) -> Option<(Vec<Bytes>, H256)>;

    /// Prove account existence at a specific block id.
    /// The key is the keccak hash of the account's address.
    /// Returns a vector of raw trie nodes (in order from the root) proving the query.
    fn prove_account(&self, key1: H256, id: BlockId) -> Option<(Vec<Bytes>, BasicAccount)>;

    /// Prove execution of a transaction at the given block.
    /// Returns the output of the call and a vector of database items necessary
    /// to reproduce it.
    fn prove_transaction(
        &self,
        transaction: SignedTransaction,
        id: BlockId,
    ) -> Option<(Bytes, Vec<DBValue>)>;

    /// Get an epoch change signal by block hash.
    fn epoch_signal(&self, hash: H256) -> Option<Vec<u8>>;
}

/// resets the blockchain
pub trait BlockChainReset {
    /// reset to best_block - n
    fn reset(&self, num: u32) -> Result<(), String>;
}

/// Provides a method for importing/exporting blocks
pub trait ImportExportBlocks {
    /// Export blocks to destination, with the given from, to and format argument.
    /// destination could be a file or stdout.
    /// If the format is hex, each block is written on a new line.
    /// For binary exports, all block data is written to the same line.
    fn export_blocks<'a>(
        &self,
        destination: Box<dyn std::io::Write + 'a>,
        from: BlockId,
        to: BlockId,
        format: Option<DataFormat>,
    ) -> Result<(), String>;

    /// Import blocks from destination, with the given format argument
    /// Source could be a file or stdout.
    /// For hex format imports, it attempts to read the blocks on a line by line basis.
    /// For binary format imports, reads the 8 byte RLP header in order to decode the block
    /// length to be read.
    fn import_blocks<'a>(
        &self,
        source: Box<dyn std::io::Read + 'a>,
        format: Option<DataFormat>,
    ) -> Result<(), String>;
}
