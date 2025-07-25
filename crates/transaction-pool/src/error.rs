// Copyright 2015-2018 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

use std::{error, fmt, result};

/// Transaction Pool Error
#[derive(Debug)]
pub enum Error<Hash: fmt::Debug + fmt::LowerHex> {
    /// Transaction is already imported
    AlreadyImported(Hash),
    /// Transaction is too cheap to enter the queue
    TooCheapToEnter(Hash, String),
    /// Transaction is too cheap to replace existing transaction that occupies the same slot.
    TooCheapToReplace(Hash, Hash),
}

/// Transaction Pool Result
pub type Result<T, H> = result::Result<T, Error<H>>;

impl<H: fmt::Debug + fmt::LowerHex> fmt::Display for Error<H> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::AlreadyImported(h) => write!(f, "[{h:?}] already imported"),
            Error::TooCheapToEnter(hash, min_score) => write!(
                f,
                "[{hash:x}] too cheap to enter the pool. Min score: {min_score}"
            ),
            Error::TooCheapToReplace(old_hash, hash) => {
                write!(f, "[{hash:x}] too cheap to replace: {old_hash:x}")
            }
        }
    }
}

impl<H: fmt::Debug + fmt::LowerHex> error::Error for Error<H> {}

#[cfg(test)]
impl<H: fmt::Debug + fmt::LowerHex> PartialEq for Error<H>
where
    H: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        use self::Error::*;

        match (self, other) {
            (AlreadyImported(h1), AlreadyImported(h2)) => h1 == h2,
            (TooCheapToEnter(h1, s1), TooCheapToEnter(h2, s2)) => h1 == h2 && s1 == s2,
            (TooCheapToReplace(old1, new1), TooCheapToReplace(old2, new2)) => {
                old1 == old2 && new1 == new2
            }
            _ => false,
        }
    }
}
