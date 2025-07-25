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

//! State of all accounts in the system expressed in Plain Old Data.

use ethereum_types::{Address, H256};
use ethjson;
use itertools::Itertools;
use pod_account::{self, PodAccount};
use std::{collections::BTreeMap, fmt};
use triehash::sec_trie_root;
use types::state_diff::StateDiff;

/// State of all accounts in the system expressed in Plain Old Data.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct PodState(BTreeMap<Address, PodAccount>);

impl PodState {
    /// Contruct a new object from the `m`.
    pub fn new() -> PodState {
        Default::default()
    }

    /// Contruct a new object from the `m`.
    pub fn from(m: BTreeMap<Address, PodAccount>) -> PodState {
        PodState(m)
    }

    /// Get the underlying map.
    pub fn get(&self) -> &BTreeMap<Address, PodAccount> {
        &self.0
    }

    /// Get the root hash of the trie of the RLP of this.
    pub fn root(&self) -> H256 {
        sec_trie_root(self.0.iter().map(|(k, v)| (k, v.rlp())))
    }

    /// Drain object to get the underlying map.
    pub fn drain(self) -> BTreeMap<Address, PodAccount> {
        self.0
    }
}

impl From<ethjson::blockchain::State> for PodState {
    fn from(s: ethjson::blockchain::State) -> PodState {
        let state = s
            .into_iter()
            .map(|(addr, acc)| (addr.into(), PodAccount::from(acc)))
            .collect();
        PodState(state)
    }
}

impl From<ethjson::spec::State> for PodState {
    fn from(s: ethjson::spec::State) -> PodState {
        let state: BTreeMap<_, _> = s
            .into_iter()
            .filter(|pair| !pair.1.is_empty())
            .map(|(addr, acc)| (addr.into(), PodAccount::from(acc)))
            .collect();
        PodState(state)
    }
}

impl fmt::Display for PodState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (add, acc) in &self.0 {
            writeln!(f, "{add} => {acc}")?;
        }
        Ok(())
    }
}

/// Calculate and return diff between `pre` state and `post` state.
pub fn diff_pod(pre: &PodState, post: &PodState) -> StateDiff {
    StateDiff {
        raw: pre
            .get()
            .keys()
            .merge(post.get().keys())
            .filter_map(|acc| {
                pod_account::diff_pod(pre.get().get(acc), post.get().get(acc)).map(|d| (*acc, d))
            })
            .collect(),
    }
}

#[cfg(test)]
mod test {
    use super::PodState;
    use ethereum_types::H160;
    use pod_account::PodAccount;
    use std::collections::BTreeMap;
    use types::{account_diff::*, state_diff::*};

    #[test]
    fn create_delete() {
        let a = PodState::from(map![
            H160::from_low_u64_be(1) => PodAccount {
                balance: 69.into(),
                nonce: 0.into(),
                code: Some(Vec::new()),
                storage: map![],
            }
        ]);
        assert_eq!(
            super::diff_pod(&a, &PodState::new()),
            StateDiff {
                raw: map![
                    H160::from_low_u64_be(1) => AccountDiff{
                        balance: Diff::Died(69.into()),
                        nonce: Diff::Died(0.into()),
                        code: Diff::Died(vec![]),
                        storage: map![],
                    }
                ]
            }
        );
        assert_eq!(
            super::diff_pod(&PodState::new(), &a),
            StateDiff {
                raw: map![
                    H160::from_low_u64_be(1) => AccountDiff{
                        balance: Diff::Born(69.into()),
                        nonce: Diff::Born(0.into()),
                        code: Diff::Born(vec![]),
                        storage: map![],
                    }
                ]
            }
        );
    }

    #[test]
    fn create_delete_with_unchanged() {
        let a = PodState::from(map![
            H160::from_low_u64_be(1) => PodAccount {
                balance: 69.into(),
                nonce: 0.into(),
                code: Some(Vec::new()),
                storage: map![],
            }
        ]);
        let b = PodState::from(map![
            H160::from_low_u64_be(1) => PodAccount {
                balance: 69.into(),
                nonce: 0.into(),
                code: Some(Vec::new()),
                storage: map![],
            },
            H160::from_low_u64_be(2) => PodAccount {
                balance: 69.into(),
                nonce: 0.into(),
                code: Some(Vec::new()),
                storage: map![],
            }
        ]);
        assert_eq!(
            super::diff_pod(&a, &b),
            StateDiff {
                raw: map![
                    H160::from_low_u64_be(2) => AccountDiff{
                        balance: Diff::Born(69.into()),
                        nonce: Diff::Born(0.into()),
                        code: Diff::Born(vec![]),
                        storage: map![],
                    }
                ]
            }
        );
        assert_eq!(
            super::diff_pod(&b, &a),
            StateDiff {
                raw: map![
                    H160::from_low_u64_be(2) => AccountDiff{
                        balance: Diff::Died(69.into()),
                        nonce: Diff::Died(0.into()),
                        code: Diff::Died(vec![]),
                        storage: map![],
                    }
                ]
            }
        );
    }

    #[test]
    fn change_with_unchanged() {
        let a = PodState::from(map![
            H160::from_low_u64_be(1) => PodAccount {
                balance: 69.into(),
                nonce: 0.into(),
                code: Some(Vec::new()),
                storage: map![],
            },
            H160::from_low_u64_be(2) => PodAccount {
                balance: 69.into(),
                nonce: 0.into(),
                code: Some(Vec::new()),
                storage: map![],
            }
        ]);
        let b = PodState::from(map![
            H160::from_low_u64_be(1) => PodAccount {
                balance: 69.into(),
                nonce: 1.into(),
                code: Some(Vec::new()),
                storage: map![],
            },
            H160::from_low_u64_be(2) => PodAccount {
                balance: 69.into(),
                nonce: 0.into(),
                code: Some(Vec::new()),
                storage: map![],
            }
        ]);
        assert_eq!(
            super::diff_pod(&a, &b),
            StateDiff {
                raw: map![
                    H160::from_low_u64_be(1) => AccountDiff{
                        balance: Diff::Same,
                        nonce: Diff::Changed(0.into(), 1.into()),
                        code: Diff::Same,
                        storage: map![],
                    }
                ]
            }
        );
    }
}
