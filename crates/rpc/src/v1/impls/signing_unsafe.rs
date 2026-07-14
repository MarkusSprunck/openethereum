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

//! Unsafe Signing RPC implementation.

use std::sync::Arc;

use ethereum_types::{Address, H160, H256, H520, U256};
use jsonrpc_core::{
    futures::future,
    BoxFuture, Result,
};
use crate::v1::{
    helpers::{
        deprecated::{self, DeprecationNotice},
        dispatch::{self, Dispatcher},
        errors,
    },
    metadata::Metadata,
    traits::{EthSigning, ParitySigning},
    types::{
        Bytes as RpcBytes, ConfirmationPayload as RpcConfirmationPayload,
        ConfirmationResponse as RpcConfirmationResponse, Either as RpcEither,
        RichRawTransaction as RpcRichRawTransaction, TransactionRequest as RpcTransactionRequest,
    },
};

/// Implementation of functions that require signing when no trusted signer is used.
pub struct SigningUnsafeClient<D> {
    accounts: Arc<dyn dispatch::Accounts>,
    dispatcher: D,
    deprecation_notice: DeprecationNotice,
}

impl<D: Dispatcher + 'static> SigningUnsafeClient<D> {
    /// Creates new `SigningUnsafeClient`.
    pub fn new(accounts: &Arc<dyn dispatch::Accounts>, dispatcher: D) -> Self {
        SigningUnsafeClient {
            accounts: accounts.clone(),
            dispatcher,
            deprecation_notice: Default::default(),
        }
    }

    fn handle(
        &self,
        payload: RpcConfirmationPayload,
        account: Address,
    ) -> BoxFuture<Result<RpcConfirmationResponse>> {
        let accounts = self.accounts.clone();
        let dis = self.dispatcher.clone();
        Box::pin(async move {
            let payload = dispatch::from_rpc(payload, account, &dis).await?;
            let result =
                dispatch::execute(dis, &accounts, payload, dispatch::SignWith::Nothing).await?;
            Ok(result.into_value())
        })
    }
}

impl<D: Dispatcher + 'static> EthSigning for SigningUnsafeClient<D> {
    type Metadata = Metadata;

    fn sign(&self, _: Metadata, address: H160, data: RpcBytes) -> BoxFuture<Result<H520>> {
        self.deprecation_notice
            .print("eth_sign", deprecated::msgs::ACCOUNTS);
        let fut = self.handle(
            RpcConfirmationPayload::EthSignMessage((address, data).into()),
            address,
        );
        Box::pin(async move {
            match fut.await? {
                RpcConfirmationResponse::Signature(signature) => Ok(signature),
                e => Err(errors::internal("Unexpected result", e)),
            }
        })
    }

    fn send_transaction(&self, _meta: Metadata, request: RpcTransactionRequest) -> BoxFuture<Result<H256>> {
        self.deprecation_notice
            .print("eth_sendTransaction", deprecated::msgs::ACCOUNTS);
        let fut = self.handle(
            RpcConfirmationPayload::SendTransaction(request),
            self.accounts.default_account(),
        );
        Box::pin(async move {
            match fut.await? {
                RpcConfirmationResponse::SendTransaction(hash) => Ok(hash),
                e => Err(errors::internal("Unexpected result", e)),
            }
        })
    }

    fn sign_transaction(
        &self,
        _meta: Metadata,
        request: RpcTransactionRequest,
    ) -> BoxFuture<Result<RpcRichRawTransaction>> {
        self.deprecation_notice
            .print("eth_signTransaction", deprecated::msgs::ACCOUNTS);
        let fut = self.handle(
            RpcConfirmationPayload::SignTransaction(request),
            self.accounts.default_account(),
        );
        Box::pin(async move {
            match fut.await? {
                RpcConfirmationResponse::SignTransaction(tx) => Ok(tx),
                e => Err(errors::internal("Unexpected result", e)),
            }
        })
    }
}

impl<D: Dispatcher + 'static> ParitySigning for SigningUnsafeClient<D> {
    type Metadata = Metadata;

    fn compose_transaction(
        &self,
        _meta: Metadata,
        transaction: RpcTransactionRequest,
    ) -> BoxFuture<Result<RpcTransactionRequest>> {
        let accounts = self.accounts.clone();
        let default_account = accounts.default_account();
        let fut = self
            .dispatcher
            .fill_optional_fields(transaction.into(), default_account, true);
        Box::pin(async move { fut.await.map(Into::into) })
    }

    fn decrypt_message(&self, _: Metadata, address: H160, data: RpcBytes) -> BoxFuture<Result<RpcBytes>> {
        self.deprecation_notice
            .print("parity_decryptMessage", deprecated::msgs::ACCOUNTS);
        let fut = self.handle(
            RpcConfirmationPayload::Decrypt((address, data).into()),
            address,
        );
        Box::pin(async move {
            match fut.await? {
                RpcConfirmationResponse::Decrypt(data) => Ok(data),
                e => Err(errors::internal("Unexpected result", e)),
            }
        })
    }

    fn post_sign(
        &self,
        _: Metadata,
        _: H160,
        _: RpcBytes,
    ) -> BoxFuture<Result<RpcEither<U256, RpcConfirmationResponse>>> {
        // We don't support this in non-signer mode.
        Box::pin(future::err(errors::signer_disabled()))
    }

    fn post_transaction(
        &self,
        _: Metadata,
        _: RpcTransactionRequest,
    ) -> BoxFuture<Result<RpcEither<U256, RpcConfirmationResponse>>> {
        // We don't support this in non-signer mode.
        Box::pin(future::err(errors::signer_disabled()))
    }

    fn check_request(&self, _: U256) -> Result<Option<RpcConfirmationResponse>> {
        // We don't support this in non-signer mode.
        Err(errors::signer_disabled())
    }
}
