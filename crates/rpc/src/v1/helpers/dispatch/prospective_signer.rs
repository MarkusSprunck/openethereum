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

use jsonrpc_core::{BoxFuture, Error, Result};
use crate::types::transaction::SignedTransaction;

use super::{Accounts, PostSign, SignWith, WithToken};
use crate::v1::helpers::{errors, nonce, FilledTransactionRequest};

/// Create a prospective-signing future: attempts to sign early with a prospective
/// nonce while waiting for the real one, then executes `post_sign`.
pub fn new<P>(
    signer: Arc<dyn Accounts>,
    filled: FilledTransactionRequest,
    chain_id: Option<u64>,
    reserved: nonce::Reserved,
    password: SignWith,
    post_sign: P,
) -> BoxFuture<Result<P::Item>>
where
    P: PostSign + 'static,
    P::Out: Send + 'static,
    P::Item: Send,
{
    let prospective_value = *reserved.prospective_value();
    let supports_prospective = signer.supports_prospective_signing(&filled.from, &password);

    Box::pin(async move {
        // Optionally pre-sign with prospective nonce while waiting.
        let prospective_signed: Option<std::result::Result<WithToken<SignedTransaction>, Error>> =
            if supports_prospective {
                Some(signer.sign_transaction(
                    filled.clone(),
                    chain_id,
                    prospective_value,
                    password.clone(),
                ))
            } else {
                None
            };

        let nonce = reserved
            .await
            .map_err(|()| errors::internal("Nonce reservation failure", ""))?;

        let signed = if supports_prospective && nonce.matches_prospective() {
            // Reuse prospective signature if nonce matches.
            prospective_signed
                .expect("prospective_signed is Some when supports_prospective; qed")?
        } else {
            signer.sign_transaction(filled, chain_id, *nonce.value(), password)?
        };

        let result = post_sign.execute(signed).await?;
        nonce.mark_used();
        Ok(result)
    })
}
