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

//! Account management (personal) rpc implementation
use std::sync::Arc;

use accounts::AccountProvider;
use bytes::Bytes;
use crypto::publickey::{public_to_address, recover, Signature};
use eip_712::{hash_structured_data, EIP712};
use ethereum_types::{Address, H160, H256, H520, U128};
use types::transaction::{PendingTransaction, SignedTransaction};

use jsonrpc_core::{
    futures::{future, Future},
    types::Value,
    BoxFuture, Result,
};
use v1::{
    helpers::{
        deprecated::{self, DeprecationNotice},
        dispatch::{self, eth_data_hash, Dispatcher, PostSign, SignWith, WithToken},
        eip191, errors,
    },
    metadata::Metadata,
    traits::Personal,
    types::{
        Bytes as RpcBytes, ConfirmationPayload as RpcConfirmationPayload,
        ConfirmationResponse as RpcConfirmationResponse, EIP191Version,
        RichRawTransaction as RpcRichRawTransaction, TransactionRequest,
    },
};

/// Account management (personal) rpc implementation.
pub struct PersonalClient<D: Dispatcher> {
    accounts: Arc<AccountProvider>,
    dispatcher: D,
    allow_experimental_rpcs: bool,
    deprecation_notice: DeprecationNotice,
}

impl<D: Dispatcher> PersonalClient<D> {
    /// Creates new `PersonalClient`
    pub fn new(
        accounts: &Arc<AccountProvider>,
        dispatcher: D,
        allow_experimental_rpcs: bool,
    ) -> Self {
        PersonalClient {
            accounts: accounts.clone(),
            dispatcher,
            allow_experimental_rpcs,
            deprecation_notice: DeprecationNotice::default(),
        }
    }
}

impl<D: Dispatcher + 'static> PersonalClient<D> {
    fn do_sign_transaction<P>(
        &self,
        _meta: Metadata,
        request: TransactionRequest,
        password: String,
        post_sign: P,
    ) -> BoxFuture<P::Item>
    where
        P: PostSign + 'static,
        <P::Out as futures::future::IntoFuture>::Future: Send,
    {
        let dispatcher = self.dispatcher.clone();
        let accounts = self.accounts.clone();

        let default = match request.from.as_ref() {
            Some(account) => Ok(*account),
            None => accounts
                .default_account()
                .map_err(|e| errors::account("Cannot find default account.", e)),
        };

        let default = match default {
            Ok(default) => default,
            Err(e) => return Box::new(future::err(e)),
        };

        let accounts = Arc::new(dispatch::Signer::new(accounts)) as _;
        Box::new(
            dispatcher
                .fill_optional_fields(request.into(), default, false)
                .and_then(move |filled| {
                    dispatcher.sign(
                        filled,
                        &accounts,
                        SignWith::Password(password.into()),
                        post_sign,
                    )
                }),
        )
    }
}

impl<D: Dispatcher + 'static> Personal for PersonalClient<D> {
    type Metadata = Metadata;

    fn accounts(&self) -> Result<Vec<H160>> {
        self.deprecation_notice
            .print("personal_accounts", deprecated::msgs::ACCOUNTS);
        let accounts = self
            .accounts
            .accounts()
            .map_err(|e| errors::account("Could not fetch accounts.", e))?;
        Ok(accounts.into_iter().collect::<Vec<H160>>())
    }

    fn new_account(&self, pass: String) -> Result<H160> {
        self.deprecation_notice
            .print("personal_newAccount", deprecated::msgs::ACCOUNTS);
        self.accounts
            .new_account(&pass.into())
            .map_err(|e| errors::account("Could not create account.", e))
    }

    fn unlock_account(
        &self,
        account: H160,
        account_pass: String,
        duration: Option<U128>,
    ) -> Result<bool> {
        self.deprecation_notice
            .print("personal_unlockAccount", deprecated::msgs::ACCOUNTS);
        let account: Address = account;
        let store = self.accounts.clone();
        let duration = match duration {
            None => None,
            Some(duration) => {
                let duration: U128 = duration;
                let v = duration.low_u64() as u32;
                if duration == v.into() {
                    Some(v)
                } else {
                    return Err(errors::invalid_params("Duration", "Invalid Number"));
                }
            }
        };

        let r = match duration {
            None => store.unlock_account_temporarily(account, account_pass.into()),
            _ => {
                return Err(errors::unsupported(
                    "Time-unlocking is not supported when permanent unlock is disabled.",
                    Some("Use personal_sendTransaction instead."),
                ))
            }
        };
        match r {
            Ok(()) => Ok(true),
            Err(err) => Err(errors::account("Unable to unlock the account.", err)),
        }
    }

    fn sign(&self, data: RpcBytes, account: H160, password: String) -> BoxFuture<H520> {
        self.deprecation_notice
            .print("personal_sign", deprecated::msgs::ACCOUNTS);
        let dispatcher = self.dispatcher.clone();
        let accounts = Arc::new(dispatch::Signer::new(self.accounts.clone())) as _;

        let payload = RpcConfirmationPayload::EthSignMessage((account, data).into());

        Box::new(
            dispatch::from_rpc(payload, account, &dispatcher)
                .and_then(move |payload| {
                    dispatch::execute(
                        dispatcher,
                        &accounts,
                        payload,
                        dispatch::SignWith::Password(password.into()),
                    )
                })
                .map(super::super::helpers::dispatch::WithToken::into_value)
                .then(|res| match res {
                    Ok(RpcConfirmationResponse::Signature(signature)) => Ok(signature),
                    Err(e) => Err(e),
                    e => Err(errors::internal("Unexpected result", e)),
                }),
        )
    }

    fn sign_191(
        &self,
        version: EIP191Version,
        data: Value,
        account: H160,
        password: String,
    ) -> BoxFuture<H520> {
        self.deprecation_notice
            .print("personal_sign191", deprecated::msgs::ACCOUNTS);
        try_bf!(errors::require_experimental(
            self.allow_experimental_rpcs,
            "191"
        ));

        let data = try_bf!(eip191::hash_message(version, data));
        let dispatcher = self.dispatcher.clone();
        let accounts = Arc::new(dispatch::Signer::new(self.accounts.clone())) as _;

        let payload = RpcConfirmationPayload::EIP191SignMessage((account, data).into());

        Box::new(
            dispatch::from_rpc(payload, account, &dispatcher)
                .and_then(move |payload| {
                    dispatch::execute(
                        dispatcher,
                        &accounts,
                        payload,
                        dispatch::SignWith::Password(password.into()),
                    )
                })
                .map(super::super::helpers::dispatch::WithToken::into_value)
                .then(|res| match res {
                    Ok(RpcConfirmationResponse::Signature(signature)) => Ok(signature),
                    Err(e) => Err(e),
                    e => Err(errors::internal("Unexpected result", e)),
                }),
        )
    }

    fn sign_typed_data(
        &self,
        typed_data: EIP712,
        account: H160,
        password: String,
    ) -> BoxFuture<H520> {
        self.deprecation_notice
            .print("personal_signTypedData", deprecated::msgs::ACCOUNTS);
        try_bf!(errors::require_experimental(
            self.allow_experimental_rpcs,
            "712"
        ));

        let data = match hash_structured_data(typed_data) {
            Ok(d) => d,
            Err(err) => return Box::new(future::err(errors::invalid_call_data(err.kind()))),
        };
        let dispatcher = self.dispatcher.clone();
        let accounts = Arc::new(dispatch::Signer::new(self.accounts.clone())) as _;

        let payload = RpcConfirmationPayload::EIP191SignMessage((account, data).into());

        Box::new(
            dispatch::from_rpc(payload, account, &dispatcher)
                .and_then(move |payload| {
                    dispatch::execute(
                        dispatcher,
                        &accounts,
                        payload,
                        dispatch::SignWith::Password(password.into()),
                    )
                })
                .map(super::super::helpers::dispatch::WithToken::into_value)
                .then(|res| match res {
                    Ok(RpcConfirmationResponse::Signature(signature)) => Ok(signature),
                    Err(e) => Err(e),
                    e => Err(errors::internal("Unexpected result", e)),
                }),
        )
    }

    fn ec_recover(&self, data: RpcBytes, signature: H520) -> BoxFuture<H160> {
        let signature: H520 = signature;
        let signature = Signature::from_electrum(signature.as_bytes());
        let data: Bytes = data.into();

        let hash = eth_data_hash(data);
        let account = recover(&signature, &hash)
            .map_err(errors::encryption)
            .map(|public| public_to_address(&public));

        Box::new(future::done(account))
    }

    fn sign_transaction(
        &self,
        meta: Metadata,
        request: TransactionRequest,
        password: String,
    ) -> BoxFuture<RpcRichRawTransaction> {
        self.deprecation_notice
            .print("personal_signTransaction", deprecated::msgs::ACCOUNTS);

        let condition = request.condition.clone().map(Into::into);
        let dispatcher = self.dispatcher.clone();
        Box::new(
            self.do_sign_transaction(meta, request, password, ())
                .map(move |tx| PendingTransaction::new(tx.into_value(), condition))
                .map(move |pending_tx| dispatcher.enrich(pending_tx.transaction)),
        )
    }

    fn send_transaction(
        &self,
        meta: Metadata,
        request: TransactionRequest,
        password: String,
    ) -> BoxFuture<H256> {
        self.deprecation_notice
            .print("personal_sendTransaction", deprecated::msgs::ACCOUNTS);
        let condition = request.condition.clone().map(Into::into);
        let dispatcher = self.dispatcher.clone();
        Box::new(self.do_sign_transaction(
            meta,
            request,
            password,
            move |signed: WithToken<SignedTransaction>| {
                dispatcher
                    .dispatch_transaction(PendingTransaction::new(signed.into_value(), condition))
            },
        ))
    }

    fn sign_and_send_transaction(
        &self,
        meta: Metadata,
        request: TransactionRequest,
        password: String,
    ) -> BoxFuture<H256> {
        self.deprecation_notice.print(
            "personal_signAndSendTransaction",
            Some("use personal_sendTransaction instead."),
        );
        warn!("Using deprecated personal_signAndSendTransaction, use personal_sendTransaction instead.");
        self.send_transaction(meta, request, password)
    }
}
