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

use std::fmt::{self, Display};
use validator::{ValidationErrors, ValidationErrorsKind};

pub(crate) type Result<T> = ::std::result::Result<T, Error>;

/// Error type
#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

impl Error {
    /// Creates a new `Error` with the specified `ErrorKind`.
    pub fn new(kind: ErrorKind) -> Self {
        Error { kind }
    }

    /// extract the error kind
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl std::error::Error for Error {}

/// Possible errors encountered while hashing/encoding an EIP-712 compliant data structure
#[derive(Clone, Debug, PartialEq)]
pub enum ErrorKind {
    /// if we fail to deserialize from a serde::Value as a type specified in message types
    UnexpectedType(String, String),
    /// the primary type supplied doesn't exist in the MessageTypes
    NonExistentType,
    /// an invalid address was encountered during encoding
    InvalidAddressLength(usize),
    /// a hex parse error occurred
    HexParseError(String),
    /// the field was declared with an unknown type
    UnknownType(String, String),
    /// Unexpected token
    UnexpectedToken(String, String),
    /// the user has attempted to define a typed array with a depth > 10
    UnsupportedArrayDepth,
    /// FieldType validation error
    ValidationError(String),
    /// the typed array defined in message types was declared with a fixed length
    UnequalArrayItems(u64, String, u64),
    /// Typed array length doesn't fit into a u64
    InvalidArraySize(String),
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::UnexpectedType(expected, field) => {
                write!(f, "Expected type '{}' for field '{}'", expected, field)
            }
            ErrorKind::NonExistentType => {
                write!(f, "The given primaryType wasn't found in the types field")
            }
            ErrorKind::InvalidAddressLength(len) => write!(
                f,
                "Address string should be a 0x-prefixed 40 character string, got length {}",
                len
            ),
            ErrorKind::HexParseError(hex) => write!(f, "Failed to parse hex '{}'", hex),
            ErrorKind::UnknownType(field, ty) => {
                write!(f, "The field '{}' has an unknown type '{}'", field, ty)
            }
            ErrorKind::UnexpectedToken(token, typename) => write!(
                f,
                "Unexpected token '{}' while parsing typename '{}'",
                token, typename
            ),
            ErrorKind::UnsupportedArrayDepth => write!(f, "Maximum depth for nested arrays is 10"),
            ErrorKind::ValidationError(msg) => write!(f, "{}", msg),
            ErrorKind::UnequalArrayItems(expected, ty, got) => write!(
                f,
                "Expected {} items for array type {}, got {} items",
                expected, ty, got
            ),
            ErrorKind::InvalidArraySize(size) => {
                write!(f, "Attempted to declare fixed size with length {}", size)
            }
        }
    }
}

pub(crate) fn serde_error(expected: &str, field: Option<&str>) -> ErrorKind {
    ErrorKind::UnexpectedType(expected.to_owned(), field.unwrap_or("").to_owned())
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Error::new(kind)
    }
}

impl From<ValidationErrors> for Error {
    fn from(error: ValidationErrors) -> Self {
        let mut string: String = "".into();
        for (field_name, error_kind) in error.errors() {
            match error_kind {
                ValidationErrorsKind::Field(validation_errors) => {
                    for error in validation_errors {
                        let str_error = format!(
                            "the field '{}', has an invalid value {}",
                            field_name, error.params["value"]
                        );
                        string.push_str(&str_error);
                    }
                }
                _ => unreachable!(
                    "#[validate] is only used on fields for regex;\
				its impossible to get any other	ErrorKind; qed"
                ),
            }
        }
        ErrorKind::ValidationError(string).into()
    }
}
