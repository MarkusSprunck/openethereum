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

//! Account system expressed in Plain Old Data.

use bytes::Bytes;
use ethereum_types::{BigEndianHash, H256, U256};
use ethjson;
use ethtrie::RlpCodec;
use hash::keccak;
use hash_db::HashDB;
use itertools::Itertools;
use keccak_hasher::KeccakHasher;
use kvdb::DBValue;
use rlp::{self, RlpStream};
use rustc_hex::ToHex;
use serde::Serializer;
use state::Account;
use std::{collections::BTreeMap, fmt};
use trie::TrieFactory;
use triehash::sec_trie_root;
use types::account_diff::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
/// An account, expressed as Plain-Old-Data (hence the name).
/// Does not have a DB overlay cache, code hash or anything like that.
pub struct PodAccount {
    /// The balance of the account.
    pub balance: U256,
    /// The nonce of the account.
    pub nonce: U256,
    #[serde(serialize_with = "opt_bytes_to_hex")]
    /// The code of the account or `None` in the special case that it is unknown.
    pub code: Option<Bytes>,
    /// The storage of the account.
    pub storage: BTreeMap<H256, H256>,
}

fn opt_bytes_to_hex<S>(opt_bytes: &Option<Bytes>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.collect_str(&format_args!(
        "0x{}",
        opt_bytes.as_ref().map_or("".to_string(), |b| b.to_hex())
    ))
}

impl PodAccount {
    /// Convert Account to a PodAccount.
    /// NOTE: This will silently fail unless the account is fully cached.
    pub fn from_account(acc: &Account) -> PodAccount {
        PodAccount {
            balance: *acc.balance(),
            nonce: *acc.nonce(),
            storage: acc
                .storage_changes()
                .iter()
                .fold(BTreeMap::new(), |mut m, (k, v)| {
                    m.insert(*k, *v);
                    m
                }),
            code: acc.code().map(|x| x.to_vec()),
        }
    }

    /// Returns the RLP for this account.
    pub fn rlp(&self) -> Bytes {
        let mut stream = RlpStream::new_list(4);
        stream.append(&self.nonce);
        stream.append(&self.balance);
        stream.append(&sec_trie_root(
            self.storage
                .iter()
                .map(|(k, v)| (k, rlp::encode(&v.into_uint()))),
        ));
        stream.append(&keccak(self.code.as_ref().unwrap_or(&vec![])));
        stream.out()
    }

    /// Place additional data into given hash DB.
    pub fn insert_additional(
        &self,
        db: &mut dyn HashDB<KeccakHasher, DBValue>,
        factory: &TrieFactory<KeccakHasher, RlpCodec>,
    ) {
        match self.code {
            Some(ref c) if !c.is_empty() => {
                db.insert(c);
            }
            _ => {}
        }
        let mut r = H256::default();
        let mut t = factory.create(db, &mut r);
        for (k, v) in &self.storage {
            if let Err(e) = t.insert(k.as_bytes(), &rlp::encode(&v.into_uint())) {
                warn!("Encountered potential DB corruption: {e}");
            }
        }
    }
}

impl From<ethjson::blockchain::Account> for PodAccount {
    fn from(a: ethjson::blockchain::Account) -> Self {
        PodAccount {
            balance: a.balance.into(),
            nonce: a.nonce.into(),
            code: Some(a.code.into()),
            storage: a
                .storage
                .into_iter()
                .map(|(key, value)| {
                    let key: U256 = key.into();
                    let value: U256 = value.into();
                    (
                        BigEndianHash::from_uint(&key),
                        BigEndianHash::from_uint(&value),
                    )
                })
                .collect(),
        }
    }
}

impl From<ethjson::spec::Account> for PodAccount {
    fn from(a: ethjson::spec::Account) -> Self {
        PodAccount {
            balance: a.balance.map_or_else(U256::zero, Into::into),
            nonce: a.nonce.map_or_else(U256::zero, Into::into),
            code: Some(a.code.map_or_else(Vec::new, Into::into)),
            storage: a.storage.map_or_else(BTreeMap::new, |s| {
                s.into_iter()
                    .map(|(key, value)| {
                        let key: U256 = key.into();
                        let value: U256 = value.into();
                        (
                            BigEndianHash::from_uint(&key),
                            BigEndianHash::from_uint(&value),
                        )
                    })
                    .collect()
            }),
        }
    }
}

impl fmt::Display for PodAccount {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "(bal={}; nonce={}; code={} bytes, #{}; storage={} items)",
            self.balance,
            self.nonce,
            self.code.as_ref().map_or(0, |c| c.len()),
            self.code.as_ref().map_or_else(H256::default, keccak),
            self.storage.len(),
        )
    }
}

/// Determine difference between two optionally existant `Account`s. Returns None
/// if they are the same.
pub fn diff_pod(pre: Option<&PodAccount>, post: Option<&PodAccount>) -> Option<AccountDiff> {
    match (pre, post) {
		(None, Some(x)) => Some(AccountDiff {
			balance: Diff::Born(x.balance),
			nonce: Diff::Born(x.nonce),
			code: Diff::Born(x.code.as_ref().expect("account is newly created; newly created accounts must be given code; all caches should remain in place; qed").clone()),
			storage: x.storage.iter().map(|(k, v)| (*k, Diff::Born(*v))).collect(),
		}),
		(Some(x), None) => Some(AccountDiff {
			balance: Diff::Died(x.balance),
			nonce: Diff::Died(x.nonce),
			code: Diff::Died(x.code.as_ref().expect("account is deleted; only way to delete account is running SUICIDE; account must have had own code cached to make operation; all caches should remain in place; qed").clone()),
			storage: x.storage.iter().map(|(k, v)| (*k, Diff::Died(*v))).collect(),
		}),
		(Some(pre), Some(post)) => {
			let storage: Vec<_> = pre.storage.keys().merge(post.storage.keys())
				.filter(|k| pre.storage.get(k).unwrap_or(&H256::default()) != post.storage.get(k).unwrap_or(&H256::default()))
				.collect();
			let r = AccountDiff {
				balance: Diff::new(pre.balance, post.balance),
				nonce: Diff::new(pre.nonce, post.nonce),
				code: match (pre.code.clone(), post.code.clone()) {
					(Some(pre_code), Some(post_code)) => Diff::new(pre_code, post_code),
					_ => Diff::Same,
				},
				storage: storage.into_iter().map(|k|
					(*k, Diff::new(
						pre.storage.get(k).cloned().unwrap_or_else(H256::default),
						post.storage.get(k).cloned().unwrap_or_else(H256::default)
					))).collect(),
			};
			if r.balance.is_same() && r.nonce.is_same() && r.code.is_same() && r.storage.is_empty() {
				None
			} else {
				Some(r)
			}
		},
		_ => None,
	}
}

#[cfg(test)]
mod test {
    use super::{diff_pod, PodAccount};
    use ethereum_types::H256;
    use std::collections::BTreeMap;
    use types::account_diff::*;

    #[test]
    fn existence() {
        let a = PodAccount {
            balance: 69.into(),
            nonce: 0.into(),
            code: Some(vec![]),
            storage: map![],
        };
        assert_eq!(diff_pod(Some(&a), Some(&a)), None);
        assert_eq!(
            diff_pod(None, Some(&a)),
            Some(AccountDiff {
                balance: Diff::Born(69.into()),
                nonce: Diff::Born(0.into()),
                code: Diff::Born(vec![]),
                storage: map![],
            })
        );
    }

    #[test]
    fn basic() {
        let a = PodAccount {
            balance: 69.into(),
            nonce: 0.into(),
            code: Some(vec![]),
            storage: map![],
        };
        let b = PodAccount {
            balance: 42.into(),
            nonce: 1.into(),
            code: Some(vec![]),
            storage: map![],
        };
        assert_eq!(
            diff_pod(Some(&a), Some(&b)),
            Some(AccountDiff {
                balance: Diff::Changed(69.into(), 42.into()),
                nonce: Diff::Changed(0.into(), 1.into()),
                code: Diff::Same,
                storage: map![],
            })
        );
    }

    #[test]
    fn code() {
        let a = PodAccount {
            balance: 0.into(),
            nonce: 0.into(),
            code: Some(vec![]),
            storage: map![],
        };
        let b = PodAccount {
            balance: 0.into(),
            nonce: 1.into(),
            code: Some(vec![0]),
            storage: map![],
        };
        assert_eq!(
            diff_pod(Some(&a), Some(&b)),
            Some(AccountDiff {
                balance: Diff::Same,
                nonce: Diff::Changed(0.into(), 1.into()),
                code: Diff::Changed(vec![], vec![0]),
                storage: map![],
            })
        );
    }

    #[test]
    fn storage() {
        let a = PodAccount {
            balance: 0.into(),
            nonce: 0.into(),
            code: Some(vec![]),
            storage: map_into![
                H256::from_low_u64_be(1) => H256::from_low_u64_be(1),
                H256::from_low_u64_be(2) => H256::from_low_u64_be(2),
                H256::from_low_u64_be(3) => H256::from_low_u64_be(3),
                H256::from_low_u64_be(4) => H256::from_low_u64_be(4),
                H256::from_low_u64_be(5) => H256::from_low_u64_be(0),
                H256::from_low_u64_be(6) => H256::from_low_u64_be(0),
                H256::from_low_u64_be(7) => H256::from_low_u64_be(0)
            ],
        };
        let b = PodAccount {
            balance: 0.into(),
            nonce: 0.into(),
            code: Some(vec![]),
            storage: map_into![
                H256::from_low_u64_be(1) => H256::from_low_u64_be(1),
                H256::from_low_u64_be(2) => H256::from_low_u64_be(3),
                H256::from_low_u64_be(3) => H256::from_low_u64_be(0),
                H256::from_low_u64_be(5) => H256::from_low_u64_be(0),
                H256::from_low_u64_be(7) => H256::from_low_u64_be(7),
                H256::from_low_u64_be(8) => H256::from_low_u64_be(0),
                H256::from_low_u64_be(9) => H256::from_low_u64_be(9)
            ],
        };
        assert_eq!(
            diff_pod(Some(&a), Some(&b)),
            Some(AccountDiff {
                balance: Diff::Same,
                nonce: Diff::Same,
                code: Diff::Same,
                storage: map![
                    H256::from_low_u64_be(2) => Diff::new(H256::from_low_u64_be(2), H256::from_low_u64_be(3)),
                    H256::from_low_u64_be(3) => Diff::new(H256::from_low_u64_be(3), H256::from_low_u64_be(0)),
                    H256::from_low_u64_be(4) => Diff::new(H256::from_low_u64_be(4), H256::from_low_u64_be(0)),
                    H256::from_low_u64_be(7) => Diff::new(H256::from_low_u64_be(0), H256::from_low_u64_be(7)),
                    H256::from_low_u64_be(9) => Diff::new(H256::from_low_u64_be(0), H256::from_low_u64_be(9))
                ],
            })
        );
    }
}
