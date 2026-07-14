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

//! Signing RPC implementation.

use parking_lot::Mutex;
use std::sync::Arc;
use transient_hashmap::TransientHashMap;

use ethereum_types::{H160, H256, H520, U256};

use jsonrpc_core::{
    BoxFuture, Error, Result,
};

use crate::v1::{
    helpers::{
        deprecated::{self, DeprecationNotice},
        dispatch::{self, Dispatcher},
        errors,
        external_signer::{
            ConfirmationReceiver as RpcConfirmationReceiver,
            ConfirmationResult as RpcConfirmationResult, SignerService, SigningQueue,
        },
    },
    metadata::Metadata,
    traits::{EthSigning, ParitySigning},
    types::{
        Bytes as RpcBytes, ConfirmationPayload as RpcConfirmationPayload,
        ConfirmationResponse as RpcConfirmationResponse, Either as RpcEither, Origin,
        RichRawTransaction as RpcRichRawTransaction, TransactionRequest as RpcTransactionRequest,
    },
};

use parity_runtime::Executor;

/// After 60s entries that are not queried with `check_request` will get garbage collected.
const MAX_PENDING_DURATION_SEC: u32 = 60;

#[must_use = "futures do nothing unless polled"]
enum DispatchResult {
    Future(U256, RpcConfirmationReceiver),
    Value(RpcConfirmationResponse),
}

impl DispatchResult {
    async fn resolve(self) -> std::result::Result<RpcConfirmationResponse, Error> {
        match self {
            DispatchResult::Value(response) => Ok(response),
            DispatchResult::Future(_uid, future) => {
                // future: Receiver<ConfirmationResult>
                // .await gives Result<ConfirmationResult, Error>
                // ConfirmationResult = Result<ConfirmationResponse, Error>
                let confirmation_result = future.await?;
                confirmation_result
            }
        }
    }
}

fn schedule(
    executor: Executor,
    confirmations: Arc<Mutex<TransientHashMap<U256, Option<RpcConfirmationResult>>>>,
    id: U256,
    future: RpcConfirmationReceiver,
) {
    {
        let mut confirmations = confirmations.lock();
        confirmations.insert(id, None);
    }

    executor.spawn_03(async move {
        // future.await → Result<ConfirmationResult, Error>
        // ConfirmationResult = Result<ConfirmationResponse, Error>
        let result = match future.await {
            Ok(confirmation_result) => confirmation_result,
            Err(e) => Err(e),
        };
        let mut confirmations = confirmations.lock();
        confirmations.prune();
        confirmations.insert(id, Some(result));
    });
}

/// Implementation of functions that require signing when no trusted signer is used.
pub struct SigningQueueClient<D> {
    signer: Arc<SignerService>,
    accounts: Arc<dyn dispatch::Accounts>,
    dispatcher: D,
    executor: Executor,
    // None here means that the request hasn't yet been confirmed
    confirmations: Arc<Mutex<TransientHashMap<U256, Option<RpcConfirmationResult>>>>,
    deprecation_notice: DeprecationNotice,
}

impl<D: Dispatcher + 'static> SigningQueueClient<D> {
    /// Creates a new signing queue client given shared signing queue.
    pub fn new(
        signer: &Arc<SignerService>,
        dispatcher: D,
        executor: Executor,
        accounts: &Arc<dyn dispatch::Accounts>,
    ) -> Self {
        SigningQueueClient {
            signer: signer.clone(),
            accounts: accounts.clone(),
            dispatcher,
            executor,
            confirmations: Arc::new(Mutex::new(TransientHashMap::new(MAX_PENDING_DURATION_SEC))),
            deprecation_notice: Default::default(),
        }
    }

    fn dispatch(
        &self,
        payload: RpcConfirmationPayload,
        origin: Origin,
    ) -> BoxFuture<Result<DispatchResult>> {
        let default_account = self.accounts.default_account();
        let accounts = self.accounts.clone();
        let dispatcher = self.dispatcher.clone();
        let signer = self.signer.clone();
        Box::pin(async move {
            let payload = dispatch::from_rpc(payload, default_account, &dispatcher).await?;
            let sender = payload.sender();
            if accounts.is_unlocked(&sender) {
                let result =
                    dispatch::execute(dispatcher, &accounts, payload, dispatch::SignWith::Nothing)
                        .await?;
                Ok(DispatchResult::Value(result.into_value()))
            } else {
                signer
                    .add_request(payload, origin)
                    .map(|(id, future)| DispatchResult::Future(id, future))
                    .map_err(|_| errors::request_rejected_limit())
            }
        })
    }
}

impl<D: Dispatcher + 'static> ParitySigning for SigningQueueClient<D> {
    type Metadata = Metadata;

    fn compose_transaction(
        &self,
        _meta: Metadata,
        transaction: RpcTransactionRequest,
    ) -> BoxFuture<Result<RpcTransactionRequest>> {
        let default_account = self.accounts.default_account();
        let fut = self
            .dispatcher
            .fill_optional_fields(transaction.into(), default_account, true);
        Box::pin(async move { fut.await.map(Into::into) })
    }

    fn post_sign(
        &self,
        meta: Metadata,
        address: H160,
        data: RpcBytes,
    ) -> BoxFuture<Result<RpcEither<U256, RpcConfirmationResponse>>> {
        self.deprecation_notice
            .print("parity_postSign", deprecated::msgs::ACCOUNTS);
        let executor = self.executor.clone();
        let confirmations = self.confirmations.clone();
        let fut = self.dispatch(
            RpcConfirmationPayload::EthSignMessage((address, data).into()),
            meta.origin,
        );
        Box::pin(async move {
            let result = fut.await?;
            Ok(match result {
                DispatchResult::Value(v) => RpcEither::Or(v),
                DispatchResult::Future(id, future) => {
                    schedule(executor, confirmations, id, future);
                    RpcEither::Either(id)
                }
            })
        })
    }

    fn post_transaction(
        &self,
        meta: Metadata,
        request: RpcTransactionRequest,
    ) -> BoxFuture<Result<RpcEither<U256, RpcConfirmationResponse>>> {
        self.deprecation_notice
            .print("parity_postTransaction", deprecated::msgs::ACCOUNTS);
        let executor = self.executor.clone();
        let confirmations = self.confirmations.clone();
        let fut = self.dispatch(RpcConfirmationPayload::SendTransaction(request), meta.origin);
        Box::pin(async move {
            let result = fut.await?;
            Ok(match result {
                DispatchResult::Value(v) => RpcEither::Or(v),
                DispatchResult::Future(id, future) => {
                    schedule(executor, confirmations, id, future);
                    RpcEither::Either(id)
                }
            })
        })
    }

    fn check_request(&self, id: U256) -> Result<Option<RpcConfirmationResponse>> {
        self.deprecation_notice
            .print("parity_checkRequest", deprecated::msgs::ACCOUNTS);
        match self.confirmations.lock().get(&id) {
            None => Err(errors::request_not_found()), // Request info has been dropped, or even never been there
            Some(&None) => Ok(None), // No confirmation yet, request is known, confirmation is pending
            Some(Some(confirmation)) => confirmation.clone().map(Some), // Confirmation is there
        }
    }

    fn decrypt_message(
        &self,
        meta: Metadata,
        address: H160,
        data: RpcBytes,
    ) -> BoxFuture<Result<RpcBytes>> {
        self.deprecation_notice
            .print("parity_decryptMessage", deprecated::msgs::ACCOUNTS);
        let fut = self.dispatch(
            RpcConfirmationPayload::Decrypt((address, data).into()),
            meta.origin,
        );
        Box::pin(async move {
            match fut.await?.resolve().await? {
                RpcConfirmationResponse::Decrypt(data) => Ok(data),
                e => Err(errors::internal("Unexpected result.", e)),
            }
        })
    }
}

impl<D: Dispatcher + 'static> EthSigning for SigningQueueClient<D> {
    type Metadata = Metadata;

    fn sign(&self, meta: Metadata, address: H160, data: RpcBytes) -> BoxFuture<Result<H520>> {
        self.deprecation_notice
            .print("eth_sign", deprecated::msgs::ACCOUNTS);
        let fut = self.dispatch(
            RpcConfirmationPayload::EthSignMessage((address, data).into()),
            meta.origin,
        );
        Box::pin(async move {
            match fut.await?.resolve().await? {
                RpcConfirmationResponse::Signature(sig) => Ok(sig),
                e => Err(errors::internal("Unexpected result.", e)),
            }
        })
    }

    fn send_transaction(&self, meta: Metadata, request: RpcTransactionRequest) -> BoxFuture<Result<H256>> {
        self.deprecation_notice
            .print("eth_sendTransaction", deprecated::msgs::ACCOUNTS);
        let fut = self.dispatch(RpcConfirmationPayload::SendTransaction(request), meta.origin);
        Box::pin(async move {
            match fut.await?.resolve().await? {
                RpcConfirmationResponse::SendTransaction(hash) => Ok(hash),
                e => Err(errors::internal("Unexpected result.", e)),
            }
        })
    }

    fn sign_transaction(
        &self,
        meta: Metadata,
        request: RpcTransactionRequest,
    ) -> BoxFuture<Result<RpcRichRawTransaction>> {
        self.deprecation_notice
            .print("eth_signTransaction", deprecated::msgs::ACCOUNTS);
        let fut = self.dispatch(RpcConfirmationPayload::SignTransaction(request), meta.origin);
        Box::pin(async move {
            match fut.await?.resolve().await? {
                RpcConfirmationResponse::SignTransaction(tx) => Ok(tx),
                e => Err(errors::internal("Unexpected result.", e)),
            }
        })
    }
}
