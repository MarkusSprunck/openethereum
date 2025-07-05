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

//! A service to fetch any HTTP / HTTPS content.

#![warn(missing_docs)]

extern crate log;

extern crate futures;

extern crate http;
extern crate hyper;
extern crate hyper_tls;

extern crate bytes;
extern crate tokio;
extern crate url;

/// Fetch client implementation.
pub mod client;
#[cfg(feature = "compat")]
/// Compatibility layer for futures 0.1
pub mod compat;

pub use self::client::{Abort, BodyReader, Client, Error, Fetch, Request, Response};
pub use hyper::Method;
pub use url::Url;

#[cfg(feature = "compat")]
pub use self::compat::{ClientCompatExt, FetchResult01};
