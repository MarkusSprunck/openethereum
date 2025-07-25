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

use json;

#[derive(Debug, PartialEq, Clone)]
pub enum Version {
    V3,
}

impl From<json::Version> for Version {
    fn from(json: json::Version) -> Self {
        match json {
            json::Version::V3 => Version::V3,
        }
    }
}

impl From<Version> for json::Version {
    fn from(val: Version) -> Self {
        match val {
            Version::V3 => json::Version::V3,
        }
    }
}
