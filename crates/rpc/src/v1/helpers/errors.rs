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

//! RPC Error codes and error objects

use std::fmt;

use ethcore::{
    client::{BlockChainClient, BlockId},
    error::{CallError, Error as EthcoreError, ErrorKind},
};
use jsonrpc_core::{Error, ErrorCode, Result as RpcResult, Value};
use rlp::DecoderError;
use types::{blockchain_info::BlockChainInfo, transaction::Error as TransactionError};
use v1::{impls::EthClientOptions, types::BlockNumber};
use vm::Error as VMError;

mod codes {
    // NOTE [ToDr] Codes from [-32099, -32000]
    pub const UNSUPPORTED_REQUEST: i64 = -32000;
    pub const NO_WORK: i64 = -32001;
    pub const NO_AUTHOR: i64 = -32002;
    pub const NO_NEW_WORK: i64 = -32003;
    pub const NO_WORK_REQUIRED: i64 = -32004;
    pub const CANNOT_SUBMIT_WORK: i64 = -32005;
    pub const UNKNOWN_ERROR: i64 = -32009;
    pub const TRANSACTION_ERROR: i64 = -32010;
    pub const EXECUTION_ERROR: i64 = -32015;
    pub const EXCEPTION_ERROR: i64 = -32016;
    pub const DATABASE_ERROR: i64 = -32017;
    #[cfg(any(test, feature = "accounts"))]
    pub const ACCOUNT_LOCKED: i64 = -32020;
    #[cfg(any(test, feature = "accounts"))]
    pub const PASSWORD_INVALID: i64 = -32021;
    pub const ACCOUNT_ERROR: i64 = -32023;
    pub const REQUEST_REJECTED: i64 = -32040;
    pub const REQUEST_REJECTED_LIMIT: i64 = -32041;
    pub const REQUEST_NOT_FOUND: i64 = -32042;
    pub const ENCRYPTION_ERROR: i64 = -32055;
    #[cfg(any(test, feature = "accounts"))]
    pub const ENCODING_ERROR: i64 = -32058;
    pub const FETCH_ERROR: i64 = -32060;
    pub const NO_PEERS: i64 = -32066;
    pub const DEPRECATED: i64 = -32070;
    pub const EXPERIMENTAL_RPC: i64 = -32071;
    pub const CANNOT_RESTART: i64 = -32080;
}

pub fn unimplemented(details: Option<String>) -> Error {
    Error {
        code: ErrorCode::ServerError(codes::UNSUPPORTED_REQUEST),
        message: "This request is not implemented yet. Please create an issue on Github repo."
            .into(),
        data: details.map(Value::String),
    }
}

pub fn unsupported<T: Into<String>>(msg: T, details: Option<T>) -> Error {
    Error {
        code: ErrorCode::ServerError(codes::UNSUPPORTED_REQUEST),
        message: msg.into(),
        data: details.map(Into::into).map(Value::String),
    }
}

pub fn request_not_found() -> Error {
    Error {
        code: ErrorCode::ServerError(codes::REQUEST_NOT_FOUND),
        message: "Request not found.".into(),
        data: None,
    }
}

pub fn request_rejected() -> Error {
    Error {
        code: ErrorCode::ServerError(codes::REQUEST_REJECTED),
        message: "Request has been rejected.".into(),
        data: None,
    }
}

pub fn request_rejected_limit() -> Error {
    Error {
        code: ErrorCode::ServerError(codes::REQUEST_REJECTED_LIMIT),
        message: "Request has been rejected because of queue limit.".into(),
        data: None,
    }
}

pub fn account<T: fmt::Debug>(error: &str, details: T) -> Error {
    Error {
        code: ErrorCode::ServerError(codes::ACCOUNT_ERROR),
        message: error.into(),
        data: Some(Value::String(format!("{details:?}"))),
    }
}

pub fn cannot_restart() -> Error {
    Error {
		code: ErrorCode::ServerError(codes::CANNOT_RESTART),
		message: "OpenEthereum could not be restarted. This feature is disabled in development mode and if the binary name isn't openethereum.".into(),
		data: None,
	}
}

/// Internal error signifying a logic error in code.
/// Should not be used when function can just fail
/// because of invalid parameters or incomplete node state.
pub fn internal<T: fmt::Debug>(error: &str, data: T) -> Error {
    Error {
        code: ErrorCode::InternalError,
        message: format!("Internal error occurred: {error}"),
        data: Some(Value::String(format!("{data:?}"))),
    }
}

pub fn invalid_params<T: fmt::Debug>(param: &str, details: T) -> Error {
    Error {
        code: ErrorCode::InvalidParams,
        message: format!("Couldn't parse parameters: {param}"),
        data: Some(Value::String(format!("{details:?}"))),
    }
}

pub fn execution<T: fmt::Debug>(data: T) -> Error {
    Error {
        code: ErrorCode::ServerError(codes::EXECUTION_ERROR),
        message: "Transaction execution error.".into(),
        data: Some(Value::String(format!("{data:?}"))),
    }
}

pub fn state_pruned() -> Error {
    Error {
		code: ErrorCode::ServerError(codes::UNSUPPORTED_REQUEST),
		message: "This request is not supported because your node is running with state pruning. Run with --pruning=archive.".into(),
		data: None,
	}
}

pub fn state_corrupt() -> Error {
    internal("State corrupt", "")
}

pub fn exceptional<T: fmt::Display>(data: T) -> Error {
    Error {
        code: ErrorCode::ServerError(codes::EXCEPTION_ERROR),
        message: "The execution failed due to an exception.".into(),
        data: Some(Value::String(data.to_string())),
    }
}

pub fn no_work() -> Error {
    Error {
        code: ErrorCode::ServerError(codes::NO_WORK),
        message: "Still syncing.".into(),
        data: None,
    }
}

pub fn no_new_work() -> Error {
    Error {
        code: ErrorCode::ServerError(codes::NO_NEW_WORK),
        message: "Work has not changed.".into(),
        data: None,
    }
}

pub fn no_author() -> Error {
    Error {
        code: ErrorCode::ServerError(codes::NO_AUTHOR),
        message: "Author not configured. Run Parity with --author to configure.".into(),
        data: None,
    }
}

pub fn no_work_required() -> Error {
    Error {
        code: ErrorCode::ServerError(codes::NO_WORK_REQUIRED),
        message: "External work is only required for Proof of Work engines.".into(),
        data: None,
    }
}

pub fn cannot_submit_work(err: EthcoreError) -> Error {
    Error {
        code: ErrorCode::ServerError(codes::CANNOT_SUBMIT_WORK),
        message: "Cannot submit work.".into(),
        data: Some(Value::String(err.to_string())),
    }
}

pub fn unavailable_block(no_ancient_block: bool, by_hash: bool) -> Error {
    if no_ancient_block {
        Error {
			code: ErrorCode::ServerError(codes::UNSUPPORTED_REQUEST),
			message: "Looks like you disabled ancient block download, unfortunately the information you're \
			trying to fetch doesn't exist in the db and is probably in the ancient blocks.".into(),
			data: None,
		}
    } else if by_hash {
        Error {
			code: ErrorCode::ServerError(codes::UNSUPPORTED_REQUEST),
			message: "Block information is incomplete while ancient block sync is still in progress, before \
					it's finished we can't determine the existence of requested item.".into(),
			data: None,
		}
    } else {
        Error {
			code: ErrorCode::ServerError(codes::UNSUPPORTED_REQUEST),
			message: "Requested block number is in a range that is not available yet, because the ancient block sync is still in progress.".into(),
			data: None,
		}
    }
}

pub fn check_block_number_existence<T, C>(
    client: &C,
    num: BlockNumber,
    options: EthClientOptions,
) -> impl Fn(Option<T>) -> RpcResult<Option<T>> + '_
where
    C: BlockChainClient,
{
    move |response| {
        if response.is_none() {
            if let BlockNumber::Num(block_number) = num {
                // tried to fetch block number and got nothing even though the block number is
                // less than the latest block number
                if block_number < client.chain_info().best_block_number
                    && !options.allow_missing_blocks
                {
                    return Err(unavailable_block(options.no_ancient_blocks, false));
                }
            }
        }
        Ok(response)
    }
}

pub fn check_block_gap<T, C>(
    client: &C,
    options: EthClientOptions,
) -> impl Fn(Option<T>) -> RpcResult<Option<T>> + '_
where
    C: BlockChainClient,
{
    move |response| {
        if response.is_none() && !options.allow_missing_blocks {
            let BlockChainInfo {
                ancient_block_hash, ..
            } = client.chain_info();
            // block information was requested, but unfortunately we couldn't find it and there
            // are gaps in the database ethcore/src/blockchain/blockchain.rs
            if ancient_block_hash.is_some() {
                return Err(unavailable_block(options.no_ancient_blocks, true));
            }
        }
        Ok(response)
    }
}

pub fn eip1559_not_activated() -> Error {
    unsupported("EIP-1559 is not activated", None)
}

pub fn not_enough_data() -> Error {
    Error {
        code: ErrorCode::ServerError(codes::UNSUPPORTED_REQUEST),
        message: "The node does not have enough data to compute the given statistic.".into(),
        data: None,
    }
}

pub fn token(e: String) -> Error {
    Error {
        code: ErrorCode::ServerError(codes::UNKNOWN_ERROR),
        message: "There was an error when saving your authorization tokens.".into(),
        data: Some(Value::String(e)),
    }
}

pub fn signer_disabled() -> Error {
    Error {
        code: ErrorCode::ServerError(codes::UNSUPPORTED_REQUEST),
        message: "Trusted Signer is disabled. This API is not available.".into(),
        data: None,
    }
}

pub fn ws_disabled() -> Error {
    Error {
        code: ErrorCode::ServerError(codes::UNSUPPORTED_REQUEST),
        message: "WebSockets Server is disabled. This API is not available.".into(),
        data: None,
    }
}

pub fn network_disabled() -> Error {
    Error {
        code: ErrorCode::ServerError(codes::UNSUPPORTED_REQUEST),
        message: "Network is disabled or not yet up.".into(),
        data: None,
    }
}

pub fn encryption<T: fmt::Debug>(error: T) -> Error {
    Error {
        code: ErrorCode::ServerError(codes::ENCRYPTION_ERROR),
        message: "Encryption error.".into(),
        data: Some(Value::String(format!("{error:?}"))),
    }
}

pub fn database<T: fmt::Debug>(error: T) -> Error {
    Error {
        code: ErrorCode::ServerError(codes::DATABASE_ERROR),
        message: "Database error.".into(),
        data: Some(Value::String(format!("{error:?}"))),
    }
}

pub fn fetch<T: fmt::Debug>(error: T) -> Error {
    Error {
        code: ErrorCode::ServerError(codes::FETCH_ERROR),
        message: "Error while fetching content.".into(),
        data: Some(Value::String(format!("{error:?}"))),
    }
}

#[cfg(any(test, feature = "accounts"))]
pub fn invalid_call_data<T: fmt::Display>(error: T) -> Error {
    Error {
        code: ErrorCode::ServerError(codes::ENCODING_ERROR),
        message: format!("{error}"),
        data: None,
    }
}

#[cfg(any(test, feature = "accounts"))]
pub fn signing(error: ::accounts::SignError) -> Error {
    Error {
		code: ErrorCode::ServerError(codes::ACCOUNT_LOCKED),
		message: "Your account is locked. Unlock the account via CLI, personal_unlockAccount or use Trusted Signer.".into(),
		data: Some(Value::String(format!("{error:?}"))),
	}
}

#[cfg(any(test, feature = "accounts"))]
pub fn password(error: ::accounts::SignError) -> Error {
    Error {
        code: ErrorCode::ServerError(codes::PASSWORD_INVALID),
        message: "Account password is invalid or account does not exist.".into(),
        data: Some(Value::String(format!("{error:?}"))),
    }
}

pub fn transaction_message(error: &TransactionError) -> String {
    use self::TransactionError::{
        AlreadyImported, CodeBanned, GasLimitExceeded, GasPriceLowerThanBaseFee,
        InsufficientBalance, InsufficientGas, InsufficientGasPrice, InvalidChainId,
        InvalidGasLimit, InvalidRlp, InvalidSignature, LimitReached, NotAllowed, Old,
        RecipientBanned, SenderBanned, SenderIsNotEOA, TooBig, TooCheapToReplace,
        TransactionTypeNotEnabled,
    };

    match *error {
		AlreadyImported => "Transaction with the same hash was already imported.".into(),
		Old => "Transaction nonce is too low. Try incrementing the nonce.".into(),
		TooCheapToReplace { prev, new } => {
			format!("Transaction gas price {} is too low. There is another transaction with same nonce in the queue{}. Try increasing the gas price or incrementing the nonce.",
					new.map_or("supplied".into(), |gas| format!("{gas}wei")),
					prev.map_or(String::new(), |gas| format!(" with gas price: {gas}wei"))
			)
		}
		LimitReached => {
			"There are too many transactions in the queue. Your transaction was dropped due to limit. Try increasing the fee.".into()
		}
		InsufficientGas { minimal, got } => {
			format!("Transaction gas is too low. There is not enough gas to cover minimal cost of the transaction (minimal: {minimal}, got: {got}). Try increasing supplied gas.")
		}
		InsufficientGasPrice { minimal, got } => {
			format!("Transaction gas price is too low. It does not satisfy your node's minimal gas price (minimal: {minimal}, got: {got}). Try increasing the gas price.")
		}
		GasPriceLowerThanBaseFee { gas_price, base_fee} => {
			format!("Transaction max gas price is lower then the required base fee (gas_price: {gas_price}, base_fee: {base_fee}). Try increasing the max gas price.")
		}
		InsufficientBalance { balance, cost } => {
			format!("Insufficient funds. The account you tried to send transaction from does not have enough funds. Required {cost} and got: {balance}.")
		}
		GasLimitExceeded { limit, got } => {
			format!("Transaction cost exceeds current gas limit. Limit: {limit}, got: {got}. Try decreasing supplied gas.")
		}
		InvalidSignature(ref sig) => format!("Invalid signature: {sig}"),
		InvalidChainId => "Invalid chain id.".into(),
		InvalidGasLimit(_) => "Supplied gas is beyond limit.".into(),
		SenderBanned => "Sender is banned in local queue.".into(),
		RecipientBanned => "Recipient is banned in local queue.".into(),
		CodeBanned => "Code is banned in local queue.".into(),
		NotAllowed => "Transaction is not permitted.".into(),
		TooBig => "Transaction is too big, see chain specification for the limit.".into(),
        InvalidRlp(ref descr) => format!("Invalid RLP data: {descr}"),
        TransactionTypeNotEnabled => "Transaction type is not enabled for current block".to_string(),
        SenderIsNotEOA => "Transaction sender is not an EOA (see EIP-3607)".into(),
	}
}

pub fn transaction<T: Into<EthcoreError>>(error: T) -> Error {
    let error = error.into();
    if let ErrorKind::Transaction(ref e) = *error.kind() {
        Error {
            code: ErrorCode::ServerError(codes::TRANSACTION_ERROR),
            message: transaction_message(e),
            data: None,
        }
    } else {
        Error {
            code: ErrorCode::ServerError(codes::UNKNOWN_ERROR),
            message: "Unknown error when sending transaction.".into(),
            data: Some(Value::String(format!("{error:?}"))),
        }
    }
}

pub fn decode<T: Into<EthcoreError>>(error: T) -> Error {
    let error = error.into();
    match *error.kind() {
        ErrorKind::Decoder(ref dec_err) => rlp(dec_err.clone()),
        _ => Error {
            code: ErrorCode::InternalError,
            message: "decoding error".into(),
            data: None,
        },
    }
}

pub fn rlp(error: DecoderError) -> Error {
    Error {
        code: ErrorCode::InvalidParams,
        message: "Invalid RLP.".into(),
        data: Some(Value::String(format!("{error:?}"))),
    }
}

pub fn call(error: CallError) -> Error {
    match error {
        CallError::StatePruned => state_pruned(),
        CallError::StateCorrupt => state_corrupt(),
        CallError::Exceptional(e) => exceptional(e),
        CallError::Execution(e) => execution(e),
        CallError::TransactionNotFound => internal(
            "{}, this should not be the case with eth_call, most likely a bug.",
            CallError::TransactionNotFound,
        ),
    }
}

pub fn vm(error: &VMError, output: &[u8]) -> Error {
    use rustc_hex::ToHex;

    let data = match error {
        &VMError::Reverted => format!("{} 0x{}", VMError::Reverted, output.to_hex()),
        error => format!("{error}"),
    };

    Error {
        code: ErrorCode::ServerError(codes::EXECUTION_ERROR),
        message: "VM execution error.".into(),
        data: Some(Value::String(data)),
    }
}

pub fn unknown_block() -> Error {
    Error {
        code: ErrorCode::InvalidParams,
        message: "Unknown block number".into(),
        data: None,
    }
}

pub fn deprecated<S: Into<String>, T: Into<Option<S>>>(message: T) -> Error {
    Error {
        code: ErrorCode::ServerError(codes::DEPRECATED),
        message: "Method deprecated".into(),
        data: message.into().map(Into::into).map(Value::String),
    }
}

pub fn filter_not_found() -> Error {
    Error {
        code: ErrorCode::ServerError(codes::UNSUPPORTED_REQUEST),
        message: "Filter not found".into(),
        data: None,
    }
}

pub fn filter_block_not_found(id: BlockId) -> Error {
    Error {
		code: ErrorCode::ServerError(codes::UNSUPPORTED_REQUEST), // Specified in EIP-234.
		message: "One of the blocks specified in filter (fromBlock, toBlock or blockHash) cannot be found".into(),
		data: Some(Value::String(match id {
			BlockId::Hash(hash) => format!("0x{hash:x}"),
			BlockId::Number(number) => format!("0x{number:x}"),
			BlockId::Earliest => "earliest".to_string(),
			BlockId::Latest => "latest".to_string(),
		})),
	}
}

pub fn status_error(has_peers: bool) -> Error {
    if has_peers {
        no_work()
    } else {
        Error {
            code: ErrorCode::ServerError(codes::NO_PEERS),
            message: "Node is not connected to any peers.".into(),
            data: None,
        }
    }
}

/// Returns a descriptive error in case experimental RPCs are not enabled.
pub fn require_experimental(allow_experimental_rpcs: bool, eip: &str) -> Result<(), Error> {
    if allow_experimental_rpcs {
        Ok(())
    } else {
        Err(Error {
			code: ErrorCode::ServerError(codes::EXPERIMENTAL_RPC),
			message: format!("This method is not part of the official RPC API yet (EIP-{eip}). Run with `--jsonrpc-experimental` to enable it."),
			data: Some(Value::String(format!("See EIP: https://eips.ethereum.org/EIPS/eip-{eip}"))),
		})
    }
}

/// returns an error for when `require_canonical` was specified in RPC for EIP-1898
pub fn invalid_input() -> Error {
    Error {
        // UNSUPPORTED_REQUEST shares the same error code for EIP-1898
        code: ErrorCode::ServerError(codes::UNSUPPORTED_REQUEST),
        message: "Invalid input".into(),
        data: None,
    }
}
