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

use ethcore::{
    client::BlockChainClient,
    miner::{self, MinerService},
};
use ethereum_types::{Address, H256, U256};
use parking_lot::Mutex;
use crate::types::transaction::{PendingTransaction, SignedTransaction};

use jsonrpc_core::{
    futures::future,
    BoxFuture, Result,
};
use crate::v1::{
    helpers::{errors, nonce, FilledTransactionRequest, TransactionRequest},
    types::RichRawTransaction as RpcRichRawTransaction,
};

use super::{
    default_gas_price, prospective_signer, Accounts, Dispatcher, PostSign,
    SignWith,
};

/// A dispatcher which uses references to a client and miner in order to sign
/// requests locally.
#[derive(Debug)]
pub struct FullDispatcher<C, M> {
    client: Arc<C>,
    miner: Arc<M>,
    nonces: Arc<Mutex<nonce::Reservations>>,
    gas_price_percentile: usize,
}

impl<C, M> FullDispatcher<C, M> {
    /// Create a `FullDispatcher` from Arc references to a client and miner.
    pub fn new(
        client: Arc<C>,
        miner: Arc<M>,
        nonces: Arc<Mutex<nonce::Reservations>>,
        gas_price_percentile: usize,
    ) -> Self {
        FullDispatcher {
            client,
            miner,
            nonces,
            gas_price_percentile,
        }
    }
}

impl<C, M> Clone for FullDispatcher<C, M> {
    fn clone(&self) -> Self {
        FullDispatcher {
            client: self.client.clone(),
            miner: self.miner.clone(),
            nonces: self.nonces.clone(),
            gas_price_percentile: self.gas_price_percentile,
        }
    }
}

impl<C: miner::BlockChainClient, M: MinerService> FullDispatcher<C, M> {
    fn state_nonce(&self, from: &Address) -> U256 {
        self.miner.next_nonce(&*self.client, from)
    }

    /// Post transaction to the network.
    ///
    /// If transaction is trusted we are more likely to assume it is coming from a local account.
    pub fn dispatch_transaction(
        client: &C,
        miner: &M,
        signed_transaction: PendingTransaction,
        trusted: bool,
    ) -> Result<H256> {
        let hash = signed_transaction.transaction.hash();

        // use `import_claimed_local_transaction` so we can decide (based on config flags) if we want to treat
        // it as local or not. Nodes with public RPC interfaces will want these transactions to be treated like
        // external transactions.
        miner
            .import_claimed_local_transaction(client, signed_transaction, trusted)
            .map_err(errors::transaction)
            .map(|()| hash)
    }
}

impl<C: miner::BlockChainClient + BlockChainClient, M: MinerService> Dispatcher
    for FullDispatcher<C, M>
{
    fn fill_optional_fields(
        &self,
        request: TransactionRequest,
        default_sender: Address,
        force_nonce: bool,
    ) -> BoxFuture<Result<FilledTransactionRequest>> {
        let request = request;
        let from = request.from.unwrap_or(default_sender);
        let nonce = if force_nonce {
            request.nonce.or_else(|| Some(self.state_nonce(&from)))
        } else {
            request.nonce
        };

        Box::pin(future::ready(Ok(FilledTransactionRequest {
            transaction_type: request.transaction_type,
            from,
            used_default_from: request.from.is_none(),
            to: request.to,
            nonce,
            gas_price: Some(request.gas_price.unwrap_or_else(|| {
                default_gas_price(&*self.client, &*self.miner, self.gas_price_percentile)
            })),
            max_fee_per_gas: request.max_fee_per_gas,
            gas: request
                .gas
                .unwrap_or_else(|| self.miner.sensible_gas_limit()),
            value: request.value.unwrap_or_else(|| 0.into()),
            data: request.data.unwrap_or_default(),
            condition: request.condition,
            access_list: request.access_list,
            max_priority_fee_per_gas: request.max_priority_fee_per_gas,
        })))
    }

    fn sign<P>(
        &self,
        filled: FilledTransactionRequest,
        signer: &Arc<dyn Accounts>,
        password: SignWith,
        post_sign: P,
    ) -> BoxFuture<Result<P::Item>>
    where
        P: PostSign + 'static,
        P::Out: Send + 'static,
        P::Item: Send,
    {
        let chain_id = self.client.signing_chain_id();

        if let Some(nonce) = filled.nonce {
            let signer = signer.clone();
            Box::pin(async move {
                let signed =
                    signer.sign_transaction(filled, chain_id, nonce, password)?;
                post_sign.execute(signed).await
            })
        } else {
            let state = self.state_nonce(&filled.from);
            let reserved = self.nonces.lock().reserve(filled.from, state);
            prospective_signer::new(signer.clone(), filled, chain_id, reserved, password, post_sign)
        }
    }

    fn enrich(&self, signed_transaction: SignedTransaction) -> RpcRichRawTransaction {
        RpcRichRawTransaction::from_signed(signed_transaction)
    }

    fn dispatch_transaction(&self, signed_transaction: PendingTransaction) -> Result<H256> {
        Self::dispatch_transaction(&*self.client, &*self.miner, signed_transaction, true)
    }
}
