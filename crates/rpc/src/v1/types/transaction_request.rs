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

//! `TransactionRequest` type

use ansi_term::Colour;
use ethereum_types::{H160, U256, U64};
use v1::{
    helpers,
    types::{AccessList, Bytes, TransactionCondition},
};

use std::fmt;

/// Transaction request coming from RPC
#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct TransactionRequest {
    /// type of transaction. Defaults to legacy type.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub transaction_type: Option<U64>,
    /// Sender
    pub from: Option<H160>,
    /// Recipient
    pub to: Option<H160>,
    /// Gas Price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_price: Option<U256>,
    /// Max fee per gas
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_fee_per_gas: Option<U256>,
    /// Gas
    pub gas: Option<U256>,
    /// Value of transaction in wei
    pub value: Option<U256>,
    /// Additional data sent with transaction
    pub data: Option<Bytes>,
    /// Transaction's nonce
    pub nonce: Option<U256>,
    /// Delay until this block condition.
    pub condition: Option<TransactionCondition>,
    /// Access list
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_list: Option<AccessList>,
    /// Miner bribe
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_priority_fee_per_gas: Option<U256>,
}

pub fn format_ether(i: U256) -> String {
    let mut string = format!("{i}");
    let idx = string.len() as isize - 18;
    if idx <= 0 {
        let mut prefix = String::from("0.");
        for _ in 0..idx.abs() {
            prefix.push('0');
        }
        string = prefix + &string;
    } else {
        string.insert(idx as usize, '.');
    }
    String::from(string.trim_end_matches('0').trim_end_matches('.'))
}

impl fmt::Display for TransactionRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let eth = self.value.unwrap_or_default();
        match self.to {
            Some(ref to) => write!(
                f,
                "{} ETH from {} to 0x{:?}",
                Colour::White.bold().paint(format_ether(eth)),
                Colour::White.bold().paint(
                    self.from
                        .as_ref()
                        .map_or_else(|| "?".to_string(), |f| format!("0x{f:?}"))
                ),
                to
            ),
            None => write!(
                f,
                "{} ETH from {} for contract creation",
                Colour::White.bold().paint(format_ether(eth)),
                Colour::White.bold().paint(
                    self.from
                        .as_ref()
                        .map_or_else(|| "?".to_string(), |f| format!("0x{f:?}"))
                ),
            ),
        }
    }
}

impl From<helpers::TransactionRequest> for TransactionRequest {
    fn from(r: helpers::TransactionRequest) -> Self {
        TransactionRequest {
            transaction_type: r.transaction_type,
            from: r.from,
            to: r.to,
            gas_price: r.gas_price,
            max_fee_per_gas: r.max_fee_per_gas,
            gas: r.gas,
            value: r.value,
            data: r.data.map(Into::into),
            nonce: r.nonce,
            condition: r.condition,
            access_list: r.access_list,
            max_priority_fee_per_gas: r.max_priority_fee_per_gas,
        }
    }
}

impl From<helpers::FilledTransactionRequest> for TransactionRequest {
    fn from(r: helpers::FilledTransactionRequest) -> Self {
        TransactionRequest {
            transaction_type: r.transaction_type,
            from: Some(r.from),
            to: r.to,
            gas_price: r.gas_price,
            max_fee_per_gas: r.max_fee_per_gas,
            gas: Some(r.gas),
            value: Some(r.value),
            data: Some(r.data.into()),
            nonce: r.nonce,
            condition: r.condition,
            access_list: r.access_list,
            max_priority_fee_per_gas: r.max_priority_fee_per_gas,
        }
    }
}

impl From<TransactionRequest> for helpers::TransactionRequest {
    fn from(val: TransactionRequest) -> Self {
        helpers::TransactionRequest {
            transaction_type: val.transaction_type,
            from: val.from,
            to: val.to,
            gas_price: val.gas_price,
            max_fee_per_gas: val.max_fee_per_gas,
            gas: val.gas,
            value: val.value,
            data: val.data.map(Into::into),
            nonce: val.nonce,
            condition: val.condition,
            access_list: val.access_list,
            max_priority_fee_per_gas: val.max_priority_fee_per_gas,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethereum_types::{H160, U256};
    use rustc_hex::FromHex;
    use serde_json;
    use std::str::FromStr;
    use v1::types::TransactionCondition;

    #[test]
    fn transaction_request_deserialize() {
        let s = r#"{
			"from":"0x0000000000000000000000000000000000000001",
			"to":"0x0000000000000000000000000000000000000002",
			"gasPrice":"0x1",
			"gas":"0x2",
			"value":"0x3",
			"data":"0x123456",
			"nonce":"0x4",
			"condition": { "block": 19 }
		}"#;
        let deserialized: TransactionRequest = serde_json::from_str(s).unwrap();

        assert_eq!(
            deserialized,
            TransactionRequest {
                transaction_type: Default::default(),
                from: Some(H160::from_low_u64_be(1)),
                to: Some(H160::from_low_u64_be(2)),
                gas_price: Some(U256::from(1)),
                max_fee_per_gas: None,
                gas: Some(U256::from(2)),
                value: Some(U256::from(3)),
                data: Some(vec![0x12, 0x34, 0x56].into()),
                nonce: Some(U256::from(4)),
                condition: Some(TransactionCondition::Number(0x13)),
                access_list: None,
                max_priority_fee_per_gas: None,
            }
        );
    }

    #[test]
    fn transaction_request_deserialize_1559() {
        let s = r#"{
            "type":"0x02",
			"from":"0x0000000000000000000000000000000000000001",
			"to":"0x0000000000000000000000000000000000000002",
			"maxFeePerGas":"0x01",
			"maxPriorityFeePerGas":"0x01",
			"gas":"0x2",
			"value":"0x3",
			"data":"0x123456",
			"nonce":"0x4",
			"condition": { "block": 19 }
		}"#;
        let deserialized: TransactionRequest = serde_json::from_str(s).unwrap();

        assert_eq!(
            deserialized,
            TransactionRequest {
                transaction_type: Some(U64::from(2)),
                from: Some(H160::from_low_u64_be(1)),
                to: Some(H160::from_low_u64_be(2)),
                gas_price: None,
                max_fee_per_gas: Some(U256::from(1)),
                gas: Some(U256::from(2)),
                value: Some(U256::from(3)),
                data: Some(vec![0x12, 0x34, 0x56].into()),
                nonce: Some(U256::from(4)),
                condition: Some(TransactionCondition::Number(0x13)),
                access_list: None,
                max_priority_fee_per_gas: Some(U256::from(1)),
            }
        );
    }

    #[test]
    fn transaction_request_deserialize2() {
        let s = r#"{
			"from": "0xb60e8dd61c5d32be8058bb8eb970870f07233155",
			"to": "0xd46e8dd67c5d32be8058bb8eb970870f07244567",
			"gas": "0x76c0",
			"gasPrice": "0x9184e72a000",
			"value": "0x9184e72a",
			"data": "0xd46e8dd67c5d32be8d46e8dd67c5d32be8058bb8eb970870f072445675058bb8eb970870f072445675"
		}"#;
        let deserialized: TransactionRequest = serde_json::from_str(s).unwrap();

        assert_eq!(deserialized, TransactionRequest {
            transaction_type: Default::default(),
			from: Some(H160::from_str("b60e8dd61c5d32be8058bb8eb970870f07233155").unwrap()),
			to: Some(H160::from_str("d46e8dd67c5d32be8058bb8eb970870f07244567").unwrap()),
			gas_price: Some(U256::from_str("9184e72a000").unwrap()),
			max_fee_per_gas: None,
			gas: Some(U256::from_str("76c0").unwrap()),
			value: Some(U256::from_str("9184e72a").unwrap()),
			data: Some("d46e8dd67c5d32be8d46e8dd67c5d32be8058bb8eb970870f072445675058bb8eb970870f072445675".from_hex().unwrap().into()),
			nonce: None,
			condition: None,
			access_list: None,
			max_priority_fee_per_gas: None,
		});
    }

    #[test]
    fn transaction_request_deserialize_empty() {
        let s = r#"{"from":"0x0000000000000000000000000000000000000001"}"#;
        let deserialized: TransactionRequest = serde_json::from_str(s).unwrap();

        assert_eq!(
            deserialized,
            TransactionRequest {
                transaction_type: Default::default(),
                from: Some(H160::from_low_u64_be(1)),
                to: None,
                gas_price: None,
                max_fee_per_gas: None,
                gas: None,
                value: None,
                data: None,
                nonce: None,
                condition: None,
                access_list: None,
                max_priority_fee_per_gas: None,
            }
        );
    }

    #[test]
    fn transaction_request_deserialize_test() {
        let s = r#"{
			"from":"0xb5f7502a2807cb23615c7456055e1d65b2508625",
			"to":"0x895d32f2db7d01ebb50053f9e48aacf26584fe40",
			"data":"0x8595bab1",
			"gas":"0x2fd618",
			"gasPrice":"0x0ba43b7400"
		}"#;

        let deserialized: TransactionRequest = serde_json::from_str(s).unwrap();

        assert_eq!(
            deserialized,
            TransactionRequest {
                transaction_type: Default::default(),
                from: Some(H160::from_str("b5f7502a2807cb23615c7456055e1d65b2508625").unwrap()),
                to: Some(H160::from_str("895d32f2db7d01ebb50053f9e48aacf26584fe40").unwrap()),
                gas_price: Some(U256::from_str("0ba43b7400").unwrap()),
                max_fee_per_gas: None,
                gas: Some(U256::from_str("2fd618").unwrap()),
                value: None,
                data: Some(vec![0x85, 0x95, 0xba, 0xb1].into()),
                nonce: None,
                condition: None,
                access_list: None,
                max_priority_fee_per_gas: None,
            }
        );
    }

    #[test]
    fn transaction_request_deserialize_error() {
        let s = r#"{
			"from":"0xb5f7502a2807cb23615c7456055e1d65b2508625",
			"to":"",
			"data":"0x8595bab1",
			"gas":"0x2fd618",
			"gasPrice":"0x0ba43b7400"
		}"#;

        let deserialized = serde_json::from_str::<TransactionRequest>(s);

        assert!(deserialized.is_err(), "Should be error because to is empty");
    }

    #[test]
    fn test_format_ether() {
        assert_eq!(&format_ether(U256::from(1000000000000000000u64)), "1");
        assert_eq!(&format_ether(U256::from(500000000000000000u64)), "0.5");
        assert_eq!(&format_ether(U256::from(50000000000000000u64)), "0.05");
        assert_eq!(&format_ether(U256::from(5000000000000000u64)), "0.005");
        assert_eq!(&format_ether(U256::from(2000000000000000000u64)), "2");
        assert_eq!(&format_ether(U256::from(2500000000000000000u64)), "2.5");
        assert_eq!(&format_ether(U256::from(10000000000000000000u64)), "10");
    }
}
