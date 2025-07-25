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

//! Transaction Verifier
//!
//! Responsible for verifying a transaction before importing to the pool.
//! Should make sure that the transaction is structuraly valid.
//!
//! May have some overlap with `Readiness` since we don't want to keep around
//! stalled transactions.

use std::{
    cmp,
    sync::{
        atomic::{self, AtomicUsize},
        Arc,
    },
};

use ethereum_types::{H256, U256};
use hash::KECCAK_EMPTY;
use txpool;
use types::transaction;

use super::{
    client::{Client, TransactionType},
    VerifiedTransaction,
};

/// Verification options.
#[derive(Debug, Clone, PartialEq)]
pub struct Options {
    /// Minimal allowed gas price (actually minimal block producer reward = effective_priority_fee).
    pub minimal_gas_price: U256,
    /// Current block gas limit.
    pub block_gas_limit: U256,
    /// Block base fee. Exists if the EIP 1559 is activated.
    pub block_base_fee: Option<U256>,
    /// Maximal gas limit for a single transaction.
    pub tx_gas_limit: U256,
    /// Skip checks for early rejection, to make sure that local transactions are always imported.
    pub no_early_reject: bool,
    /// Accept transactions from non EOAs (see EIP-3607)
    pub allow_non_eoa_sender: bool,
}

#[cfg(test)]
impl Default for Options {
    fn default() -> Self {
        Options {
            minimal_gas_price: 0.into(),
            block_gas_limit: U256::max_value(),
            block_base_fee: None,
            tx_gas_limit: U256::max_value(),
            no_early_reject: false,
            allow_non_eoa_sender: false,
        }
    }
}

/// Transaction to verify.
#[cfg_attr(test, derive(Clone))]
pub enum Transaction {
    /// Fresh, never verified transaction.
    ///
    /// We need to do full verification of such transactions
    Unverified(transaction::UnverifiedTransaction),

    /// Transaction from retracted block.
    ///
    /// We could skip some parts of verification of such transactions
    Retracted(transaction::UnverifiedTransaction),

    /// Locally signed or retracted transaction.
    ///
    /// We can skip consistency verifications and just verify readiness.
    Local(transaction::PendingTransaction),
}

impl Transaction {
    /// Return transaction hash
    pub fn hash(&self) -> H256 {
        match *self {
            Transaction::Unverified(ref tx) => tx.hash(),
            Transaction::Retracted(ref tx) => tx.hash(),
            Transaction::Local(ref tx) => tx.hash(),
        }
    }

    /// Return transaction gas price for non 1559 transactions or maxFeePerGas for 1559 transactions.
    pub fn gas_price(&self) -> &U256 {
        match *self {
            Transaction::Unverified(ref tx) => &tx.tx().gas_price,
            Transaction::Retracted(ref tx) => &tx.tx().gas_price,
            Transaction::Local(ref tx) => &tx.tx().gas_price,
        }
    }

    fn gas(&self) -> &U256 {
        match *self {
            Transaction::Unverified(ref tx) => &tx.tx().gas,
            Transaction::Retracted(ref tx) => &tx.tx().gas,
            Transaction::Local(ref tx) => &tx.tx().gas,
        }
    }

    /// Return actual gas price of the transaction depending on the current block base fee
    pub fn effective_gas_price(&self, block_base_fee: Option<U256>) -> U256 {
        match *self {
            Transaction::Unverified(ref tx) => tx.effective_gas_price(block_base_fee),
            Transaction::Retracted(ref tx) => tx.effective_gas_price(block_base_fee),
            Transaction::Local(ref tx) => tx.effective_gas_price(block_base_fee),
        }
    }

    /// Return effective fee - part of the transaction fee that goes to the miner
    pub fn effective_priority_fee(&self, block_base_fee: Option<U256>) -> U256 {
        match *self {
            Transaction::Unverified(ref tx) => tx.effective_priority_fee(block_base_fee),
            Transaction::Retracted(ref tx) => tx.effective_priority_fee(block_base_fee),
            Transaction::Local(ref tx) => tx.effective_priority_fee(block_base_fee),
        }
    }

    /// Return maximum transaction fee that may go to the miner:
    /// transaction gas price for non 1559 transactions or maxPriorityFeePerGas for 1559 transactions.
    pub fn max_priority_fee(&self) -> U256 {
        match *self {
            Transaction::Unverified(ref tx) => tx.max_priority_fee_per_gas(),
            Transaction::Retracted(ref tx) => tx.max_priority_fee_per_gas(),
            Transaction::Local(ref tx) => tx.max_priority_fee_per_gas(),
        }
    }

    /// Check if transaction has zero gas price
    pub fn has_zero_gas_price(&self) -> bool {
        match *self {
            Transaction::Unverified(ref tx) => tx.has_zero_gas_price(),
            Transaction::Retracted(ref tx) => tx.has_zero_gas_price(),
            Transaction::Local(ref tx) => tx.has_zero_gas_price(),
        }
    }

    fn transaction(&self) -> &transaction::TypedTransaction {
        match *self {
            Transaction::Unverified(ref tx) => tx,
            Transaction::Retracted(ref tx) => tx,
            Transaction::Local(ref tx) => tx,
        }
    }

    fn is_local(&self) -> bool {
        match *self {
            Transaction::Local(..) => true,
            _ => false,
        }
    }

    fn is_retracted(&self) -> bool {
        match *self {
            Transaction::Retracted(..) => true,
            _ => false,
        }
    }
}

/// Transaction verifier.
///
/// Verification can be run in parallel for all incoming transactions.
#[derive(Debug)]
pub struct Verifier<C, S, V> {
    client: C,
    options: Options,
    id: Arc<AtomicUsize>,
    transaction_to_replace: Option<(S, Arc<V>)>,
}

impl<C, S, V> Verifier<C, S, V> {
    /// Creates new transaction verfier with specified options.
    pub fn new(
        client: C,
        options: Options,
        id: Arc<AtomicUsize>,
        transaction_to_replace: Option<(S, Arc<V>)>,
    ) -> Self {
        Verifier {
            client,
            options,
            id,
            transaction_to_replace,
        }
    }
}

impl<C: Client> txpool::Verifier<Transaction>
    for Verifier<C, ::pool::scoring::NonceAndGasPrice, VerifiedTransaction>
{
    type Error = transaction::Error;
    type VerifiedTransaction = VerifiedTransaction;

    fn verify_transaction(
        &self,
        tx: Transaction,
    ) -> Result<Self::VerifiedTransaction, Self::Error> {
        // The checks here should be ordered by cost/complexity.
        // Cheap checks should be done as early as possible to discard unneeded transactions early.

        let hash = tx.hash();

        if self.client.transaction_already_included(&hash) {
            trace!(target: "txqueue", "[{hash:?}] Rejected tx already in the blockchain");
            bail!(transaction::Error::AlreadyImported)
        }

        let gas_limit = cmp::min(self.options.tx_gas_limit, self.options.block_gas_limit);
        if tx.gas() > &gas_limit {
            debug!(
                target: "txqueue",
                "[{:?}] Rejected transaction above gas limit: {} > min({}, {})",
                hash,
                tx.gas(),
                self.options.block_gas_limit,
                self.options.tx_gas_limit,
            );
            bail!(transaction::Error::GasLimitExceeded {
                limit: gas_limit,
                got: *tx.gas(),
            });
        }

        let minimal_gas = self.client.required_gas(tx.transaction().tx());
        if tx.gas() < &minimal_gas {
            trace!(target: "txqueue",
                "[{:?}] Rejected transaction with insufficient gas: {} < {}",
                hash,
                tx.gas(),
                minimal_gas,
            );

            bail!(transaction::Error::InsufficientGas {
                minimal: minimal_gas,
                got: *tx.gas(),
            })
        }

        let is_own = tx.is_local();
        let has_zero_gas_price = tx.has_zero_gas_price();
        // Quick exit for non-service and non-local transactions
        //
        // We're checking if the transaction is below configured minimal gas price
        // or the effective minimal gas price in case the pool is full.

        if !has_zero_gas_price && !is_own {
            let max_priority_fee = tx.max_priority_fee();

            if max_priority_fee < self.options.minimal_gas_price {
                trace!(
                    target: "txqueue",
                    "[{:?}] Rejected tx below minimal gas price threshold: {} < {}",
                    hash,
                    max_priority_fee,
                    self.options.minimal_gas_price,
                );
                bail!(transaction::Error::InsufficientGasPrice {
                    minimal: self.options.minimal_gas_price,
                    got: max_priority_fee,
                });
            }

            if let Some((ref scoring, ref vtx)) = self.transaction_to_replace {
                if scoring.should_reject_early(vtx, &tx) {
                    trace!(
                        target: "txqueue",
                        "[{:?}] Rejected tx early, cause it doesn't have any chance to get to the pool: (gas price: {} < {})",
                        hash,
                        tx.effective_gas_price(self.options.block_base_fee),
                        vtx.transaction.effective_gas_price(self.options.block_base_fee),
                    );
                    return Err(transaction::Error::TooCheapToReplace {
                        prev: Some(
                            vtx.transaction
                                .effective_gas_price(self.options.block_base_fee),
                        ),
                        new: Some(tx.effective_gas_price(self.options.block_base_fee)),
                    });
                }
            }
        }

        // Some more heavy checks below.
        // Actually recover sender and verify that transaction
        let is_retracted = tx.is_retracted();
        let transaction = match tx {
            Transaction::Retracted(tx) | Transaction::Unverified(tx) => {
                match self.client.verify_transaction(tx) {
                    Ok(signed) => signed.into(),
                    Err(err) => {
                        debug!(target: "txqueue", "[{hash:?}] Rejected tx {err:?}");
                        bail!(err)
                    }
                }
            }
            Transaction::Local(tx) => match self.client.verify_transaction_basic(&tx) {
                Ok(()) => tx,
                Err(err) => {
                    warn!(target: "txqueue", "[{hash:?}] Rejected local tx {err:?}");
                    return Err(err);
                }
            },
        };

        // Verify RLP payload
        if let Err(err) = self.client.decode_transaction(&transaction.encode()) {
            debug!(target: "txqueue", "[{err:?}] Rejected transaction's rlp payload");
            bail!(err)
        }

        let sender = transaction.sender();
        let account_details = self.client.account_details(&sender);

        if !self.options.allow_non_eoa_sender {
            if let Some(code_hash) = account_details.code_hash {
                if code_hash != KECCAK_EMPTY {
                    debug!(
                        target: "txqueue",
                        "[{hash:?}] Rejected tx, sender is not an EOA: {code_hash}"
                    );
                    bail!(transaction::Error::SenderIsNotEOA);
                }
            }
        }

        let max_priority_fee = transaction.max_priority_fee_per_gas();

        if max_priority_fee < self.options.minimal_gas_price {
            let transaction_type = self.client.transaction_type(&transaction);
            if let TransactionType::Service = transaction_type {
                debug!(target: "txqueue", "Service tx {hash:?} below minimal gas price accepted");
            } else if is_own || account_details.is_local {
                info!(target: "own_tx", "Local tx {hash:?} below minimal gas price accepted");
            } else {
                trace!(
                    target: "txqueue",
                    "[{:?}] Rejected tx below minimal gas price threshold: {} < {}",
                    hash,
                    max_priority_fee,
                    self.options.minimal_gas_price,
                );
                bail!(transaction::Error::InsufficientGasPrice {
                    minimal: self.options.minimal_gas_price,
                    got: max_priority_fee,
                });
            }
        }

        let gas_price = transaction.tx().gas_price;

        if gas_price < transaction.max_priority_fee_per_gas() {
            bail!(transaction::Error::InsufficientGasPrice {
                minimal: transaction.max_priority_fee_per_gas(),
                got: gas_price,
            });
        }

        let (full_gas_price, overflow_1) = gas_price.overflowing_mul(transaction.tx().gas);
        let (cost, overflow_2) = transaction.tx().value.overflowing_add(full_gas_price);
        if overflow_1 || overflow_2 {
            trace!(
                target: "txqueue",
                "[{hash:?}] Rejected tx, price overflow"
            );
            bail!(transaction::Error::InsufficientBalance {
                cost: U256::max_value(),
                balance: account_details.balance,
            });
        }
        if account_details.balance < cost {
            debug!(
                target: "txqueue",
                "[{:?}] Rejected tx with not enough balance: {} < {}",
                hash,
                account_details.balance,
                cost,
            );
            bail!(transaction::Error::InsufficientBalance {
                cost,
                balance: account_details.balance,
            });
        }

        if transaction.tx().nonce < account_details.nonce {
            debug!(
                target: "txqueue",
                "[{:?}] Rejected tx with old nonce ({} < {})",
                hash,
                transaction.tx().nonce,
                account_details.nonce,
            );
            bail!(transaction::Error::Old);
        }

        let priority = match (is_own || account_details.is_local, is_retracted) {
            (true, _) => super::Priority::Local,
            (false, false) => super::Priority::Regular,
            (false, true) => super::Priority::Retracted,
        };
        Ok(VerifiedTransaction {
            transaction,
            priority,
            hash,
            sender,
            insertion_id: self.id.fetch_add(1, atomic::Ordering::AcqRel),
        })
    }
}
