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

//! EIP712 structs
use ethereum_types::{Address, H256, U256};
use lazy_static::lazy_static;
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use validator::{Validate, ValidationErrors};
use validator_derive::Validate;

pub(crate) type MessageTypes = HashMap<String, Vec<FieldType>>;

lazy_static! {
    // match solidity identifier with the addition of '[(\d)*]*'
    static ref TYPE_REGEX: Regex = Regex::new(r"^[a-zA-Z_$][a-zA-Z_$0-9]*(\[([1-9]\d*)*\])*$").unwrap();
    static ref IDENT_REGEX: Regex = Regex::new(r"^[a-zA-Z_$][a-zA-Z_$0-9]*$").unwrap();
}

#[derive(Deserialize, Serialize, Validate, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub(crate) struct EIP712Domain {
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) chain_id: U256,
    pub(crate) verifying_contract: Address,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) salt: Option<H256>,
}
/// EIP-712 struct
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct EIP712 {
    pub(crate) types: MessageTypes,
    pub(crate) primary_type: String,
    pub(crate) message: Value,
    pub(crate) domain: EIP712Domain,
}

impl Validate for EIP712 {
    fn validate(&self) -> Result<(), ValidationErrors> {
        for field_types in self.types.values() {
            for field_type in field_types {
                field_type.validate()?;
            }
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone)]
pub(crate) struct FieldType {
    #[validate(regex(path = "*IDENT_REGEX"))]
    pub name: String,
    #[serde(rename = "type")]
    #[validate(regex(path = "*TYPE_REGEX"))]
    pub type_: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::from_str;

    #[test]
    fn test_regex() {
        let test_cases = vec![
            "unint bytes32",
            "Seun\\[]",
            "byte[]uint",
            "byte[7[]uint][]",
            "Person[0]",
        ];
        for case in test_cases {
            assert!(!TYPE_REGEX.is_match(case))
        }

        let test_cases = vec![
            "bytes32",
            "Foo[]",
            "bytes1",
            "bytes32[][]",
            "byte[9]",
            "contents",
        ];
        for case in test_cases {
            assert!(TYPE_REGEX.is_match(case))
        }
    }

    #[test]
    fn test_deserialization() {
        let string = r#"{
			"primaryType": "Mail",
			"domain": {
				"name": "Ether Mail",
				"version": "1",
				"chainId": "0x1",
				"verifyingContract": "0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"
			},
			"message": {
				"from": {
					"name": "Cow",
					"wallet": "0xCD2a3d9F938E13CD947Ec05AbC7FE734Df8DD826"
				},
				"to": {
					"name": "Bob",
					"wallet": "0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB"
				},
				"contents": "Hello, Bob!"
			},
			"types": {
				"EIP712Domain": [
					{ "name": "name", "type": "string" },
					{ "name": "version", "type": "string" },
					{ "name": "chainId", "type": "uint256" },
					{ "name": "verifyingContract", "type": "address" }
				],
				"Person": [
					{ "name": "name", "type": "string" },
					{ "name": "wallet", "type": "address" }
				],
				"Mail": [
					{ "name": "from", "type": "Person" },
					{ "name": "to", "type": "Person" },
					{ "name": "contents", "type": "string" }
				]
			}
		}"#;
        let _ = from_str::<EIP712>(string).unwrap();
    }

    #[test]
    fn test_failing_deserialization() {
        let string = r#"{
			"primaryType": "Mail",
			"domain": {
				"name": "Ether Mail",
				"version": "1",
				"chainId": "0x1",
				"verifyingContract": "0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"
			},
			"message": {
				"from": {
					"name": "Cow",
					"wallet": "0xCD2a3d9F938E13CD947Ec05AbC7FE734Df8DD826"
				},
				"to": {
					"name": "Bob",
					"wallet": "0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB"
				},
				"contents": "Hello, Bob!"
			},
			"types": {
				"EIP712Domain": [
					{ "name": "name", "type": "string" },
					{ "name": "version", "type": "string" },
					{ "name": "chainId", "type": "7uint256[x] Seun" },
					{ "name": "verifyingContract", "type": "address" }
				],
				"Person": [
					{ "name": "name", "type": "string" },
					{ "name": "wallet amen", "type": "address" }
				],
				"Mail": [
					{ "name": "from", "type": "Person" },
					{ "name": "to", "type": "Person" },
					{ "name": "contents", "type": "string" }
				]
			}
		}"#;
        let data = from_str::<EIP712>(string).unwrap();
        assert!(data.validate().is_err());
    }
}
