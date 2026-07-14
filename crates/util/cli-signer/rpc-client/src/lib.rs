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

pub mod client;
pub mod signer_client;

extern crate ethereum_types;
extern crate futures;
extern crate jsonrpc_core;
extern crate jsonrpc_ws_server as ws;
extern crate keccak_hash as hash;
extern crate parity_rpc as rpc;
extern crate parking_lot;
extern crate serde;
extern crate serde_json;
extern crate url;

#[macro_use]
extern crate log;
