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

//! Disk-backed, ref-counted `JournalDB` implementation.

use std::{
    collections::{BTreeMap, HashMap},
    io,
    sync::Arc,
};

use super::{traits::JournalDB, LATEST_ERA_KEY};
use bytes::Bytes;
use ethcore_db::{DBTransaction, DBValue, KeyValueDB};
use ethereum_types::H256;
use hash_db::HashDB;
use keccak_hasher::KeccakHasher;
use memory_db::MemoryDB;
use overlaydb::OverlayDB;
use parity_util_mem::{allocators::new_malloc_size_ops, MallocSizeOf};
use rlp::{decode, encode};
use util::{DatabaseKey, DatabaseValueRef, DatabaseValueView};
use DB_PREFIX_LEN;

/// Implementation of the `HashDB` trait for a disk-backed database with a memory overlay
/// and latent-removal semantics.
///
/// Like `OverlayDB`, there is a memory overlay; `commit()` must be called in order to
/// write operations out to disk. Unlike `OverlayDB`, `remove()` operations do not take effect
/// immediately. Rather some age (based on a linear but arbitrary metric) must pass before
/// the removals actually take effect.
///
/// journal format:
/// ```text
/// [era, 0] => [ id, [insert_0, ...], [remove_0, ...] ]
/// [era, 1] => [ id, [insert_0, ...], [remove_0, ...] ]
/// [era, n] => [ ... ]
/// ```
///
/// when we make a new commit, we journal the inserts and removes.
/// for each `end_era` that we journaled that we are no passing by,
/// we remove all of its removes assuming it is canonical and all
/// of its inserts otherwise.
// TODO: store last_era, reclaim_period.
pub struct RefCountedDB {
    forward: OverlayDB,
    backing: Arc<dyn KeyValueDB>,
    latest_era: Option<u64>,
    inserts: Vec<H256>,
    removes: Vec<H256>,
    column: Option<u32>,
}

impl RefCountedDB {
    /// Create a new instance given a `backing` database.
    pub fn new(backing: Arc<dyn KeyValueDB>, column: Option<u32>) -> RefCountedDB {
        let latest_era = backing
            .get(column, &LATEST_ERA_KEY)
            .expect("Low-level database error.")
            .map(|v| decode::<u64>(&v).expect("decoding db value failed"));

        RefCountedDB {
            forward: OverlayDB::new(backing.clone(), column),
            backing,
            inserts: vec![],
            removes: vec![],
            latest_era,
            column,
        }
    }
}

impl HashDB<KeccakHasher, DBValue> for RefCountedDB {
    fn get(&self, key: &H256) -> Option<DBValue> {
        self.forward.get(key)
    }
    fn contains(&self, key: &H256) -> bool {
        self.forward.contains(key)
    }
    fn insert(&mut self, value: &[u8]) -> H256 {
        let r = self.forward.insert(value);
        self.inserts.push(r);
        r
    }
    fn emplace(&mut self, key: H256, value: DBValue) {
        self.inserts.push(key);
        self.forward.emplace(key, value);
    }
    fn remove(&mut self, key: &H256) {
        self.removes.push(*key);
    }
}

impl ::traits::KeyedHashDB for RefCountedDB {
    fn keys(&self) -> HashMap<H256, i32> {
        self.forward.keys()
    }
}

impl JournalDB for RefCountedDB {
    fn boxed_clone(&self) -> Box<dyn JournalDB> {
        Box::new(RefCountedDB {
            forward: self.forward.clone(),
            backing: self.backing.clone(),
            latest_era: self.latest_era,
            inserts: self.inserts.clone(),
            removes: self.removes.clone(),
            column: self.column,
        })
    }

    fn get_sizes(&self, sizes: &mut BTreeMap<String, usize>) {
        let mut ops = new_malloc_size_ops();
        sizes.insert(
            String::from("db_ref_counted_inserts"),
            self.inserts.size_of(&mut ops),
        );
        sizes.insert(
            String::from("db_ref_counted_removes"),
            self.removes.size_of(&mut ops),
        );
    }

    fn is_empty(&self) -> bool {
        self.latest_era.is_none()
    }

    fn backing(&self) -> &Arc<dyn KeyValueDB> {
        &self.backing
    }

    fn latest_era(&self) -> Option<u64> {
        self.latest_era
    }

    fn journal_under(&mut self, batch: &mut DBTransaction, now: u64, id: &H256) -> io::Result<u32> {
        // record new commit's details.
        let mut db_key = DatabaseKey {
            era: now,
            index: 0usize,
        };
        let mut last;

        while self
            .backing
            .get(self.column, {
                last = encode(&db_key);
                &last
            })?
            .is_some()
        {
            db_key.index += 1;
        }

        {
            let value_ref = DatabaseValueRef {
                id,
                inserts: &self.inserts,
                deletes: &self.removes,
            };

            batch.put(self.column, &last, &encode(&value_ref));
        }

        let ops = self.inserts.len() + self.removes.len();

        trace!(target: "rcdb", "new journal for time #{}.{} => {}: inserts={:?}, removes={:?}", now, db_key.index, id, self.inserts, self.removes);

        self.inserts.clear();
        self.removes.clear();

        if self.latest_era.is_none_or(|e| now > e) {
            batch.put(self.column, &LATEST_ERA_KEY, &encode(&now));
            self.latest_era = Some(now);
        }

        Ok(ops as u32)
    }

    fn mark_canonical(
        &mut self,
        batch: &mut DBTransaction,
        end_era: u64,
        canon_id: &H256,
    ) -> io::Result<u32> {
        // apply old commits' details
        let mut db_key = DatabaseKey {
            era: end_era,
            index: 0usize,
        };
        let mut last;
        while let Some(rlp_data) = {
            self.backing.get(self.column, {
                last = encode(&db_key);
                &last
            })?
        } {
            let view = DatabaseValueView::from_rlp(&rlp_data);
            let our_id = view.id().expect("rlp read from db; qed");
            let to_remove = if canon_id == &our_id {
                view.deletes()
            } else {
                view.inserts()
            }
            .expect("rlp read from db; qed");
            trace!(target: "rcdb", "delete journal for time #{}.{}=>{}, (canon was {}): deleting {:?}", end_era, db_key.index, our_id, canon_id, to_remove);
            for i in &to_remove {
                self.forward.remove(i);
            }
            batch.delete(self.column, &last);
            db_key.index += 1;
        }

        let r = self.forward.commit_to_batch(batch)?;
        Ok(r)
    }

    fn inject(&mut self, batch: &mut DBTransaction) -> io::Result<u32> {
        self.inserts.clear();
        for remove in self.removes.drain(..) {
            self.forward.remove(&remove);
        }
        self.forward.commit_to_batch(batch)
    }

    fn consolidate(&mut self, mut with: MemoryDB<KeccakHasher, DBValue>) {
        for (key, (value, rc)) in with.drain() {
            for _ in 0..rc {
                self.emplace(key, value.clone());
            }

            for _ in rc..0 {
                self.remove(&key);
            }
        }
    }

    fn state(&self, id: &H256) -> Option<Bytes> {
        self.backing
            .get_by_prefix(self.column, &id[0..DB_PREFIX_LEN])
            .map(|b| b.into_vec())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use hash_db::HashDB;
    use keccak::keccak;
    use JournalDB;

    fn new_db() -> RefCountedDB {
        let backing = Arc::new(ethcore_db::InMemoryWithMetrics::create(0));
        RefCountedDB::new(backing, None)
    }

    #[test]
    fn long_history() {
        // history is 3
        let mut jdb = new_db();
        let h = jdb.insert(b"foo");
        jdb.commit_batch(0, &keccak(b"0"), None).unwrap();
        assert!(jdb.contains(&h));
        jdb.remove(&h);
        jdb.commit_batch(1, &keccak(b"1"), None).unwrap();
        assert!(jdb.contains(&h));
        jdb.commit_batch(2, &keccak(b"2"), None).unwrap();
        assert!(jdb.contains(&h));
        jdb.commit_batch(3, &keccak(b"3"), Some((0, keccak(b"0"))))
            .unwrap();
        assert!(jdb.contains(&h));
        jdb.commit_batch(4, &keccak(b"4"), Some((1, keccak(b"1"))))
            .unwrap();
        assert!(!jdb.contains(&h));
    }

    #[test]
    fn latest_era_should_work() {
        // history is 3
        let mut jdb = new_db();
        assert_eq!(jdb.latest_era(), None);
        let h = jdb.insert(b"foo");
        jdb.commit_batch(0, &keccak(b"0"), None).unwrap();
        assert_eq!(jdb.latest_era(), Some(0));
        jdb.remove(&h);
        jdb.commit_batch(1, &keccak(b"1"), None).unwrap();
        assert_eq!(jdb.latest_era(), Some(1));
        jdb.commit_batch(2, &keccak(b"2"), None).unwrap();
        assert_eq!(jdb.latest_era(), Some(2));
        jdb.commit_batch(3, &keccak(b"3"), Some((0, keccak(b"0"))))
            .unwrap();
        assert_eq!(jdb.latest_era(), Some(3));
        jdb.commit_batch(4, &keccak(b"4"), Some((1, keccak(b"1"))))
            .unwrap();
        assert_eq!(jdb.latest_era(), Some(4));
    }

    #[test]
    fn complex() {
        // history is 1
        let mut jdb = new_db();

        let foo = jdb.insert(b"foo");
        let bar = jdb.insert(b"bar");
        jdb.commit_batch(0, &keccak(b"0"), None).unwrap();
        assert!(jdb.contains(&foo));
        assert!(jdb.contains(&bar));

        jdb.remove(&foo);
        jdb.remove(&bar);
        let baz = jdb.insert(b"baz");
        jdb.commit_batch(1, &keccak(b"1"), Some((0, keccak(b"0"))))
            .unwrap();
        assert!(jdb.contains(&foo));
        assert!(jdb.contains(&bar));
        assert!(jdb.contains(&baz));

        let foo = jdb.insert(b"foo");
        jdb.remove(&baz);
        jdb.commit_batch(2, &keccak(b"2"), Some((1, keccak(b"1"))))
            .unwrap();
        assert!(jdb.contains(&foo));
        assert!(!jdb.contains(&bar));
        assert!(jdb.contains(&baz));

        jdb.remove(&foo);
        jdb.commit_batch(3, &keccak(b"3"), Some((2, keccak(b"2"))))
            .unwrap();
        assert!(jdb.contains(&foo));
        assert!(!jdb.contains(&bar));
        assert!(!jdb.contains(&baz));

        jdb.commit_batch(4, &keccak(b"4"), Some((3, keccak(b"3"))))
            .unwrap();
        assert!(!jdb.contains(&foo));
        assert!(!jdb.contains(&bar));
        assert!(!jdb.contains(&baz));
    }

    #[test]
    fn fork() {
        // history is 1
        let mut jdb = new_db();

        let foo = jdb.insert(b"foo");
        let bar = jdb.insert(b"bar");
        jdb.commit_batch(0, &keccak(b"0"), None).unwrap();
        assert!(jdb.contains(&foo));
        assert!(jdb.contains(&bar));

        jdb.remove(&foo);
        let baz = jdb.insert(b"baz");
        jdb.commit_batch(1, &keccak(b"1a"), Some((0, keccak(b"0"))))
            .unwrap();

        jdb.remove(&bar);
        jdb.commit_batch(1, &keccak(b"1b"), Some((0, keccak(b"0"))))
            .unwrap();

        assert!(jdb.contains(&foo));
        assert!(jdb.contains(&bar));
        assert!(jdb.contains(&baz));

        jdb.commit_batch(2, &keccak(b"2b"), Some((1, keccak(b"1b"))))
            .unwrap();
        assert!(jdb.contains(&foo));
        assert!(!jdb.contains(&baz));
        assert!(!jdb.contains(&bar));
    }

    #[test]
    fn inject() {
        let mut jdb = new_db();
        let key = jdb.insert(b"dog");
        jdb.inject_batch().unwrap();

        assert_eq!(jdb.get(&key).unwrap(), DBValue::from_slice(b"dog"));
        jdb.remove(&key);
        jdb.inject_batch().unwrap();

        assert!(jdb.get(&key).is_none());
    }
}
