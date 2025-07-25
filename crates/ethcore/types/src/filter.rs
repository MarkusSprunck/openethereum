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

//! Blockchain filter

use crate::{ids::BlockId, log_entry::LogEntry};
use ethereum_types::{Address, Bloom, BloomInput, H256};

/// Blockchain Filter.
#[derive(Debug, PartialEq)]
pub struct Filter {
    /// Blockchain will be searched from this block.
    pub from_block: BlockId,

    /// Till this block.
    pub to_block: BlockId,

    /// Search addresses.
    ///
    /// If None, match all.
    /// If specified, log must be produced by one of these addresses.
    pub address: Option<Vec<Address>>,

    /// Search topics.
    ///
    /// If None, match all.
    /// If specified, log must contain one of these topics.
    pub topics: Vec<Option<Vec<H256>>>,

    /// Logs limit
    ///
    /// If None, return all logs
    /// If specified, should only return *last* `n` logs.
    pub limit: Option<usize>,
}

impl Clone for Filter {
    fn clone(&self) -> Self {
        let mut topics = [None, None, None, None];
        for i in 0..4 {
            topics[i] = self.topics[i].clone();
        }

        Filter {
            from_block: self.from_block,
            to_block: self.to_block,
            address: self.address.clone(),
            topics: topics[..].to_vec(),
            limit: self.limit,
        }
    }
}

impl Filter {
    /// Returns combinations of each address and topic.
    pub fn bloom_possibilities(&self) -> Vec<Bloom> {
        let blooms = match self.address {
            Some(ref addresses) if !addresses.is_empty() => addresses
                .iter()
                .map(|address| Bloom::from(BloomInput::Raw(address.as_bytes())))
                .collect(),
            _ => vec![Bloom::default()],
        };

        self.topics.iter().fold(blooms, |bs, topic| match *topic {
            None => bs,
            Some(ref topics) => bs
                .into_iter()
                .flat_map(|bloom| {
                    topics
                        .iter()
                        .map(|topic| {
                            let mut b = bloom;
                            b.accrue(BloomInput::Raw(topic.as_bytes()));
                            b
                        })
                        .collect::<Vec<Bloom>>()
                })
                .collect(),
        })
    }

    /// Returns true if given log entry matches filter.
    pub fn matches(&self, log: &LogEntry) -> bool {
        let matches = match self.address {
            Some(ref addresses) if !addresses.is_empty() => {
                addresses.iter().any(|address| &log.address == address)
            }
            _ => true,
        };

        matches
            && self
                .topics
                .iter()
                .enumerate()
                .all(|(i, topic)| match *topic {
                    Some(ref topics) if !topics.is_empty() => {
                        topics.iter().any(|topic| log.topics.get(i) == Some(topic))
                    }
                    _ => true,
                })
    }
}

#[cfg(test)]
mod tests {
    use crate::{filter::Filter, ids::BlockId, log_entry::LogEntry};
    use ethereum_types::{Bloom, H160, H256};
    use std::str::FromStr;

    #[test]
    fn test_bloom_possibilities_none() {
        let none_filter = Filter {
            from_block: BlockId::Earliest,
            to_block: BlockId::Latest,
            address: None,
            topics: vec![None, None, None, None],
            limit: None,
        };

        let possibilities = none_filter.bloom_possibilities();
        assert_eq!(possibilities.len(), 1);
        assert!(possibilities[0].is_zero())
    }

    // block 399849
    #[test]
    fn test_bloom_possibilities_single_address_and_topic() {
        let filter = Filter {
            from_block: BlockId::Earliest,
            to_block: BlockId::Latest,
            address: Some(vec![H160::from_str(
                "b372018f3be9e171df0581136b59d2faf73a7d5d",
            )
            .unwrap()]),
            topics: vec![
                Some(vec![H256::from_str(
                    "ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9",
                )
                .unwrap()]),
                None,
                None,
                None,
            ],
            limit: None,
        };

        let possibilities = filter.bloom_possibilities();
        assert_eq!(possibilities, vec![Bloom::from_str("00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000000004000000004000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000000000000000").unwrap()] as Vec<Bloom>);
    }

    #[test]
    fn test_bloom_possibilities_single_address_and_many_topics() {
        let filter = Filter {
            from_block: BlockId::Earliest,
            to_block: BlockId::Latest,
            address: Some(vec![H160::from_str(
                "b372018f3be9e171df0581136b59d2faf73a7d5d",
            )
            .unwrap()]),
            topics: vec![
                Some(vec![H256::from_str(
                    "ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9",
                )
                .unwrap()]),
                Some(vec![H256::from_str(
                    "ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9",
                )
                .unwrap()]),
                None,
                None,
            ],
            limit: None,
        };

        let possibilities = filter.bloom_possibilities();
        assert_eq!(possibilities, vec![Bloom::from_str("00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000000004000000004000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000000000000000").unwrap()] as Vec<Bloom>);
    }

    #[test]
    fn test_bloom_possibilites_multiple_addresses_and_topics() {
        let filter = Filter {
            from_block: BlockId::Earliest,
            to_block: BlockId::Latest,
            address: Some(vec![
                H160::from_str("b372018f3be9e171df0581136b59d2faf73a7d5d").unwrap(),
                H160::from_str("b372018f3be9e171df0581136b59d2faf73a7d5d").unwrap(),
            ]),
            topics: vec![
                Some(vec![
                    H256::from_str(
                        "ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9",
                    )
                    .unwrap(),
                    H256::from_str(
                        "ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9",
                    )
                    .unwrap(),
                ]),
                Some(vec![
                    H256::from_str(
                        "ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9",
                    )
                    .unwrap(),
                    H256::from_str(
                        "ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9",
                    )
                    .unwrap(),
                ]),
                Some(vec![H256::from_str(
                    "ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9",
                )
                .unwrap()]),
                None,
            ],
            limit: None,
        };

        // number of possibilites should be equal 2 * 2 * 2 * 1 = 8
        let possibilities = filter.bloom_possibilities();
        assert_eq!(possibilities.len(), 8);
        assert_eq!(possibilities[0], Bloom::from_str("00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000000004000000004000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000000000000000").unwrap());
    }

    #[test]
    fn test_filter_matches() {
        let filter = Filter {
            from_block: BlockId::Earliest,
            to_block: BlockId::Latest,
            address: Some(vec![H160::from_str(
                "b372018f3be9e171df0581136b59d2faf73a7d5d",
            )
            .unwrap()]),
            topics: vec![
                Some(vec![H256::from_str(
                    "ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9",
                )
                .unwrap()]),
                Some(vec![H256::from_str(
                    "ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23fa",
                )
                .unwrap()]),
                None,
                None,
            ],
            limit: None,
        };

        let entry0 = LogEntry {
            address: H160::from_str("b372018f3be9e171df0581136b59d2faf73a7d5d").unwrap(),
            topics: vec![
                H256::from_str("ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9")
                    .unwrap(),
                H256::from_str("ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23fa")
                    .unwrap(),
                H256::from_str("ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9")
                    .unwrap(),
            ],
            data: vec![],
        };

        let entry1 = LogEntry {
            address: H160::from_str("b372018f3be9e171df0581136b59d2faf73a7d5e").unwrap(),
            topics: vec![
                H256::from_str("ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9")
                    .unwrap(),
                H256::from_str("ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23fa")
                    .unwrap(),
                H256::from_str("ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9")
                    .unwrap(),
            ],
            data: vec![],
        };

        let entry2 = LogEntry {
            address: H160::from_str("b372018f3be9e171df0581136b59d2faf73a7d5d").unwrap(),
            topics: vec![H256::from_str(
                "ff74e91598aed6ae5d2fdcf8b24cd2c7be49a0808112a305069355b7160f23f9",
            )
            .unwrap()],
            data: vec![],
        };

        assert!(filter.matches(&entry0));
        assert!(!filter.matches(&entry1));
        assert!(!filter.matches(&entry2));
    }
}
