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

//! VM errors module

use action_params::ActionParams;
use ethereum_types::Address;
use ethtrie;
use std::fmt;
use ResumeCall;
use ResumeCreate;

#[derive(Debug)]
pub enum TrapKind {
    Call(ActionParams),
    Create(ActionParams, Address),
}

pub enum TrapError<Call, Create> {
    Call(ActionParams, Call),
    Create(ActionParams, Address, Create),
}

/// VM errors.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// `OutOfGas` is returned when transaction execution runs out of gas.
    /// The state should be reverted to the state from before the
    /// transaction execution. But it does not mean that transaction
    /// was invalid. Balance still should be transfered and nonce
    /// should be increased.
    OutOfGas,
    /// `BadJumpDestination` is returned when execution tried to move
    /// to position that wasn't marked with JUMPDEST instruction
    BadJumpDestination {
        /// Position the code tried to jump to.
        destination: usize,
    },
    /// `BadInstructions` is returned when given instruction is not supported
    BadInstruction {
        /// Unrecognized opcode
        instruction: u8,
    },
    /// `StackUnderflow` when there is not enough stack elements to execute instruction
    StackUnderflow {
        /// Invoked instruction
        instruction: &'static str,
        /// How many stack elements was requested by instruction
        wanted: usize,
        /// How many elements were on stack
        on_stack: usize,
    },
    /// When execution would exceed defined Stack Limit
    OutOfStack {
        /// Invoked instruction
        instruction: &'static str,
        /// How many stack elements instruction wanted to push
        wanted: usize,
        /// What was the stack limit
        limit: usize,
    },
    /// When there is not enough subroutine stack elements to return from
    SubStackUnderflow {
        /// How many stack elements was requested by instruction
        wanted: usize,
        /// How many elements were on stack
        on_stack: usize,
    },
    /// When execution would exceed defined subroutine Stack Limit
    OutOfSubStack {
        /// How many stack elements instruction wanted to pop
        wanted: usize,
        /// What was the stack limit
        limit: usize,
    },
    /// When the code walks into a subroutine, that is not allowed
    InvalidSubEntry,
    /// Built-in contract failed on given input
    BuiltIn(&'static str),
    /// When execution tries to modify the state in static context
    MutableCallInStaticContext,
    /// Invalid code to deploy as a contract
    InvalidCode,
    /// Likely to cause consensus issues.
    Internal(String),
    /// Wasm runtime error
    Wasm(String),
    /// Out of bounds access in RETURNDATACOPY.
    OutOfBounds,
    /// Execution has been reverted with REVERT.
    Reverted,
}

impl From<Box<ethtrie::TrieError>> for Error {
    fn from(err: Box<ethtrie::TrieError>) -> Self {
        Error::Internal(format!("Internal error: {err}"))
    }
}

impl From<ethtrie::TrieError> for Error {
    fn from(err: ethtrie::TrieError) -> Self {
        Error::Internal(format!("Internal error: {err}"))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Error::*;
        match *self {
            OutOfGas => write!(f, "Out of gas"),
            BadJumpDestination { destination } => write!(
                f,
                "Bad jump destination {destination:x}  (trimmed to usize)"
            ),
            BadInstruction { instruction } => write!(f, "Bad instruction {instruction:x}"),
            StackUnderflow {
                instruction,
                wanted,
                on_stack,
            } => write!(f, "Stack underflow {instruction} {wanted}/{on_stack}"),
            OutOfStack {
                instruction,
                wanted,
                limit,
            } => write!(f, "Out of stack {instruction} {wanted}/{limit}"),
            SubStackUnderflow { wanted, on_stack } => {
                write!(f, "Subroutine stack underflow {wanted}/{on_stack}")
            }
            OutOfSubStack { wanted, limit } => {
                write!(f, "Out of subroutine stack {wanted}/{limit}")
            }
            InvalidSubEntry => write!(f, "Invalid subroutine entry"),
            BuiltIn(name) => write!(f, "Built-in failed: {name}"),
            Internal(ref msg) => write!(f, "Internal error: {msg}"),
            MutableCallInStaticContext => write!(f, "Mutable call in static context"),
            InvalidCode => write!(f, "Invalid code to deploy as a contract"),
            Wasm(ref msg) => write!(f, "Internal error: {msg}"),
            OutOfBounds => write!(f, "Out of bounds"),
            Reverted => write!(f, "Reverted"),
        }
    }
}

pub type Result<T> = ::std::result::Result<T, Error>;
pub type TrapResult<T, Call, Create> = ::std::result::Result<Result<T>, TrapError<Call, Create>>;

pub type ExecTrapResult<T> = TrapResult<T, Box<dyn ResumeCall>, Box<dyn ResumeCreate>>;
pub type ExecTrapError = TrapError<Box<dyn ResumeCall>, Box<dyn ResumeCreate>>;
