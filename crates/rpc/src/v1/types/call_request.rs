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

use ethereum_types::{H160, U256, U64};
use v1::{
    helpers::CallRequest as Request,
    types::{AccessList, Bytes},
};

/// Call request
#[derive(Debug, Default, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct CallRequest {
    /// transaction type. Defaults to legacy type.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub transaction_type: Option<U64>,
    /// From
    pub from: Option<H160>,
    /// To
    pub to: Option<H160>,
    /// Gas Price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_price: Option<U256>,
    /// Max fee per gas
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_fee_per_gas: Option<U256>,
    /// Gas
    pub gas: Option<U256>,
    /// Value
    pub value: Option<U256>,
    /// Data
    pub data: Option<Bytes>,
    /// Nonce
    pub nonce: Option<U256>,
    /// Access list
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_list: Option<AccessList>,
    /// Miner bribe
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_priority_fee_per_gas: Option<U256>,
}

impl From<CallRequest> for Request {
    fn from(val: CallRequest) -> Self {
        Request {
            transaction_type: val.transaction_type,
            from: val.from,
            to: val.to,
            gas_price: val.gas_price,
            max_fee_per_gas: val.max_fee_per_gas,
            gas: val.gas,
            value: val.value,
            data: val.data.map(Into::into),
            nonce: val.nonce,
            access_list: val.access_list,
            max_priority_fee_per_gas: val.max_priority_fee_per_gas,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CallRequest;
    use ethereum_types::{H160, U256};
    use rustc_hex::FromHex;
    use serde_json;
    use std::str::FromStr;

    #[test]
    fn call_request_deserialize() {
        let s = r#"{
			"from":"0x0000000000000000000000000000000000000001",
			"to":"0x0000000000000000000000000000000000000002",
			"gasPrice":"0x1",
			"gas":"0x2",
			"value":"0x3",
			"data":"0x123456",
			"nonce":"0x4"
		}"#;
        let deserialized: CallRequest = serde_json::from_str(s).unwrap();

        assert_eq!(
            deserialized,
            CallRequest {
                transaction_type: Default::default(),
                from: Some(H160::from_low_u64_be(1)),
                to: Some(H160::from_low_u64_be(2)),
                gas_price: Some(U256::from(1)),
                max_fee_per_gas: None,
                gas: Some(U256::from(2)),
                value: Some(U256::from(3)),
                data: Some(vec![0x12, 0x34, 0x56].into()),
                nonce: Some(U256::from(4)),
                access_list: None,
                max_priority_fee_per_gas: None,
            }
        );
    }

    #[test]
    fn call_request_deserialize2() {
        let s = r#"{
			"from": "0xb60e8dd61c5d32be8058bb8eb970870f07233155",
			"to": "0xd46e8dd67c5d32be8058bb8eb970870f07244567",
			"gas": "0x76c0",
			"gasPrice": "0x9184e72a000",
			"value": "0x9184e72a",
			"data": "0xd46e8dd67c5d32be8d46e8dd67c5d32be8058bb8eb970870f072445675058bb8eb970870f072445675"
		}"#;
        let deserialized: CallRequest = serde_json::from_str(s).unwrap();

        assert_eq!(deserialized, CallRequest {
            transaction_type: Default::default(),
			from: Some(H160::from_str("b60e8dd61c5d32be8058bb8eb970870f07233155").unwrap()),
			to: Some(H160::from_str("d46e8dd67c5d32be8058bb8eb970870f07244567").unwrap()),
			gas_price: Some(U256::from_str("9184e72a000").unwrap()),
			max_fee_per_gas: None,
			gas: Some(U256::from_str("76c0").unwrap()),
			value: Some(U256::from_str("9184e72a").unwrap()),
			data: Some("d46e8dd67c5d32be8d46e8dd67c5d32be8058bb8eb970870f072445675058bb8eb970870f072445675".from_hex().unwrap().into()),
			nonce: None,
			access_list: None,
			max_priority_fee_per_gas: None,
		});
    }

    #[test]
    fn call_request_deserialize_empty() {
        let s = r#"{"from":"0x0000000000000000000000000000000000000001"}"#;
        let deserialized: CallRequest = serde_json::from_str(s).unwrap();

        assert_eq!(
            deserialized,
            CallRequest {
                transaction_type: Default::default(),
                from: Some(H160::from_low_u64_be(1)),
                to: None,
                gas_price: None,
                max_fee_per_gas: None,
                gas: None,
                value: None,
                data: None,
                nonce: None,
                access_list: None,
                max_priority_fee_per_gas: None,
            }
        );
    }
}
