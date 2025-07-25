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

//! State database abstraction. For more info, see the doc for `StateDB`

use std::{
    collections::{BTreeMap, HashSet, VecDeque},
    io,
    sync::Arc,
};

use ethereum_types::{Address, H256};
use hash_db::HashDB;
use journaldb::JournalDB;
use keccak_hasher::KeccakHasher;
use kvdb::{DBTransaction, DBValue};
use lru_cache::LruCache;
use memory_cache::MemoryLruCache;
use parking_lot::Mutex;
use types::BlockNumber;

use state::{self, Account};

const STATE_CACHE_BLOCKS: usize = 12;

// The percentage of supplied cache size to go to accounts.
const ACCOUNT_CACHE_RATIO: usize = 90;

/// Shared canonical state cache.
struct AccountCache {
    /// DB Account cache. `None` indicates that account is known to be missing.
    // When changing the type of the values here, be sure to update `mem_used` and
    // `new`.
    accounts: LruCache<Address, Option<Account>>,
    /// Information on the modifications in recently committed blocks; specifically which addresses
    /// changed in which block. Ordered by block number.
    modifications: VecDeque<BlockChanges>,
}

/// Buffered account cache item.
struct CacheQueueItem {
    /// Account address.
    address: Address,
    /// Acccount data or `None` if account does not exist.
    account: SyncAccount,
    /// Indicates that the account was modified before being
    /// added to the cache.
    modified: bool,
}

#[derive(Debug)]
/// Accumulates a list of accounts changed in a block.
struct BlockChanges {
    /// Block number.
    number: BlockNumber,
    /// Block hash.
    hash: H256,
    /// Parent block hash.
    parent: H256,
    /// A set of modified account addresses.
    accounts: HashSet<Address>,
    /// Block is part of the canonical chain.
    is_canon: bool,
}

/// State database abstraction.
/// Manages shared global state cache which reflects the canonical
/// state as it is on the disk. All the entries in the cache are clean.
/// A clone of `StateDB` may be created as canonical or not.
/// For canonical clones local cache is accumulated and applied
/// in `sync_cache`
/// For non-canonical clones local cache is dropped.
///
/// Global cache propagation.
/// After a `State` object has been committed to the trie it
/// propagates its local cache into the `StateDB` local cache
/// using `add_to_account_cache` function.
/// Then, after the block has been added to the chain the local cache in the
/// `StateDB` is propagated into the global cache.
pub struct StateDB {
    /// Backing database.
    db: Box<dyn JournalDB>,
    /// Shared canonical state cache.
    account_cache: Arc<Mutex<AccountCache>>,
    /// DB Code cache. Maps code hashes to shared bytes.
    code_cache: Arc<Mutex<MemoryLruCache<H256, Arc<Vec<u8>>>>>,
    /// Local dirty cache.
    local_cache: Vec<CacheQueueItem>,
    cache_size: usize,
    /// Hash of the block on top of which this instance was created or
    /// `None` if cache is disabled
    parent_hash: Option<H256>,
    /// Hash of the committing block or `None` if not committed yet.
    commit_hash: Option<H256>,
    /// Number of the committing block or `None` if not committed yet.
    commit_number: Option<BlockNumber>,
}

impl StateDB {
    /// Create a new instance wrapping `JournalDB` and the maximum allowed size
    /// of the LRU cache in bytes. Actual used memory may (read: will) be higher due to bookkeeping.
    // TODO: make the cache size actually accurate by moving the account storage cache
    // into the `AccountCache` structure as its own `LruCache<(Address, H256), H256>`.
    pub fn new(db: Box<dyn JournalDB>, cache_size: usize) -> StateDB {
        let acc_cache_size = cache_size * ACCOUNT_CACHE_RATIO / 100;
        let code_cache_size = cache_size - acc_cache_size;
        let cache_items = acc_cache_size / ::std::mem::size_of::<Option<Account>>();

        StateDB {
            db,
            account_cache: Arc::new(Mutex::new(AccountCache {
                accounts: LruCache::new(cache_items),
                modifications: VecDeque::new(),
            })),
            code_cache: Arc::new(Mutex::new(MemoryLruCache::new(code_cache_size))),
            local_cache: Vec::new(),
            cache_size,
            parent_hash: None,
            commit_hash: None,
            commit_number: None,
        }
    }

    /// Journal all recent operations under the given era and ID.
    pub fn journal_under(
        &mut self,
        batch: &mut DBTransaction,
        now: u64,
        id: &H256,
    ) -> io::Result<u32> {
        let records = self.db.journal_under(batch, now, id)?;
        self.commit_hash = Some(*id);
        self.commit_number = Some(now);
        Ok(records)
    }

    // t_nb 9.15
    /// Mark a given candidate from an ancient era as canonical, enacting its removals from the
    /// backing database and reverting any non-canonical historical commit's insertions.
    pub fn mark_canonical(
        &mut self,
        batch: &mut DBTransaction,
        end_era: u64,
        canon_id: &H256,
    ) -> io::Result<u32> {
        self.db.mark_canonical(batch, end_era, canon_id)
    }

    // t_nb 9.10 Propagate local cache into the global cache and synchonize
    /// the global cache with the best block state.
    /// This function updates the global cache by removing entries
    /// that are invalidated by chain reorganization. `sync_cache`
    /// should be called after the block has been committed and the
    /// blockchain route has ben calculated.
    pub fn sync_cache(&mut self, enacted: &[H256], retracted: &[H256], is_best: bool) {
        trace!(
            "sync_cache id = (#{:?}, {:?}), parent={:?}, best={}",
            self.commit_number,
            self.commit_hash,
            self.parent_hash,
            is_best
        );
        let mut cache = self.account_cache.lock();
        let cache = &mut *cache;

        // Purge changes from re-enacted and retracted blocks.
        // Filter out commiting block if any.
        let mut clear = false;
        for block in enacted
            .iter()
            .filter(|h| self.commit_hash.as_ref() != Some(*h))
        {
            clear = clear || {
                if let Some(ref mut m) = cache.modifications.iter_mut().find(|m| &m.hash == block) {
                    trace!("Reverting enacted block {block:?}");
                    m.is_canon = true;
                    for a in &m.accounts {
                        trace!("Reverting enacted address {a:?}");
                        cache.accounts.remove(a);
                    }
                    false
                } else {
                    true
                }
            };
        }

        for block in retracted {
            clear = clear || {
                if let Some(ref mut m) = cache.modifications.iter_mut().find(|m| &m.hash == block) {
                    trace!("Retracting block {block:?}");
                    m.is_canon = false;
                    for a in &m.accounts {
                        trace!("Retracted address {a:?}");
                        cache.accounts.remove(a);
                    }
                    false
                } else {
                    true
                }
            };
        }
        if clear {
            // We don't know anything about the block; clear everything
            trace!("Wiping cache");
            cache.accounts.clear();
            cache.modifications.clear();
        }

        // Propagate cache only if committing on top of the latest canonical state
        // blocks are ordered by number and only one block with a given number is marked as canonical
        // (contributed to canonical state cache)
        if let (Some(ref number), Some(ref hash), Some(ref parent)) =
            (self.commit_number, self.commit_hash, self.parent_hash)
        {
            if cache.modifications.len() == STATE_CACHE_BLOCKS {
                cache.modifications.pop_back();
            }
            let mut modifications = HashSet::new();
            trace!("committing {} cache entries", self.local_cache.len());
            for account in self.local_cache.drain(..) {
                if account.modified {
                    modifications.insert(account.address);
                }
                if is_best {
                    let acc = account.account.0;
                    if let Some(&mut Some(ref mut existing)) =
                        cache.accounts.get_mut(&account.address)
                    {
                        if let Some(new) = acc {
                            if account.modified {
                                existing.overwrite_with(new);
                            }
                            continue;
                        }
                    }
                    cache.accounts.insert(account.address, acc);
                }
            }

            // Save modified accounts. These are ordered by the block number.
            let block_changes = BlockChanges {
                accounts: modifications,
                number: *number,
                hash: *hash,
                is_canon: is_best,
                parent: *parent,
            };
            let insert_at = cache
                .modifications
                .iter()
                .enumerate()
                .find(|&(_, m)| m.number < *number)
                .map(|(i, _)| i);
            trace!("inserting modifications at {insert_at:?}");
            if let Some(insert_at) = insert_at {
                cache.modifications.insert(insert_at, block_changes);
            } else {
                cache.modifications.push_back(block_changes);
            }
        }
    }

    /// Conversion method to interpret self as `HashDB` reference
    pub fn as_hash_db(&self) -> &dyn HashDB<KeccakHasher, DBValue> {
        self.db.as_hash_db()
    }

    /// Conversion method to interpret self as mutable `HashDB` reference
    pub fn as_hash_db_mut(&mut self) -> &mut dyn HashDB<KeccakHasher, DBValue> {
        self.db.as_hash_db_mut()
    }

    /// Clone the database.
    pub fn boxed_clone(&self) -> StateDB {
        StateDB {
            db: self.db.boxed_clone(),
            account_cache: self.account_cache.clone(),
            code_cache: self.code_cache.clone(),
            local_cache: Vec::new(),
            cache_size: self.cache_size,
            parent_hash: None,
            commit_hash: None,
            commit_number: None,
        }
    }

    /// Clone the database for a canonical state.
    pub fn boxed_clone_canon(&self, parent: &H256) -> StateDB {
        StateDB {
            db: self.db.boxed_clone(),
            account_cache: self.account_cache.clone(),
            code_cache: self.code_cache.clone(),
            local_cache: Vec::new(),
            cache_size: self.cache_size,
            parent_hash: Some(*parent),
            commit_hash: None,
            commit_number: None,
        }
    }

    /// Check if pruning is enabled on the database.
    pub fn is_pruned(&self) -> bool {
        self.db.is_pruned()
    }

    /// Heap size used.
    pub fn get_sizes(&self, sizes: &mut BTreeMap<String, usize>) {
        self.db.get_sizes(sizes);

        sizes.insert(
            String::from("account_cache_len"),
            self.account_cache.lock().accounts.len(),
        );
        sizes.insert(
            String::from("code_cache_size"),
            self.code_cache.lock().current_size(),
        );
    }

    /// Returns underlying `JournalDB`.
    pub fn journal_db(&self) -> &dyn JournalDB {
        &*self.db
    }

    /// Query how much memory is set aside for the accounts cache (in bytes).
    pub fn cache_size(&self) -> usize {
        self.cache_size
    }

    /// Check if the account can be returned from cache by matching current block parent hash against canonical
    /// state and filtering out account modified in later blocks.
    fn is_allowed(
        addr: &Address,
        parent_hash: &H256,
        modifications: &VecDeque<BlockChanges>,
    ) -> bool {
        if modifications.is_empty() {
            return true;
        }
        // Ignore all accounts modified in later blocks
        // Modifications contains block ordered by the number
        // We search for our parent in that list first and then for
        // all its parent until we hit the canonical block,
        // checking against all the intermediate modifications.
        let mut parent = parent_hash;
        for m in modifications {
            if &m.hash == parent {
                if m.is_canon {
                    return true;
                }
                parent = &m.parent;
            }
            if m.accounts.contains(addr) {
                trace!("Cache lookup skipped for {addr:?}: modified in a later block");
                return false;
            }
        }
        trace!("Cache lookup skipped for {addr:?}: parent hash is unknown");
        false
    }
}

impl state::Backend for StateDB {
    fn as_hash_db(&self) -> &dyn HashDB<KeccakHasher, DBValue> {
        self.db.as_hash_db()
    }

    fn as_hash_db_mut(&mut self) -> &mut dyn HashDB<KeccakHasher, DBValue> {
        self.db.as_hash_db_mut()
    }

    fn add_to_account_cache(&mut self, addr: Address, data: Option<Account>, modified: bool) {
        self.local_cache.push(CacheQueueItem {
            address: addr,
            account: SyncAccount(data),
            modified,
        })
    }

    fn cache_code(&self, hash: H256, code: Arc<Vec<u8>>) {
        let mut cache = self.code_cache.lock();

        cache.insert(hash, code);
    }

    fn get_cached_account(&self, addr: &Address) -> Option<Option<Account>> {
        self.parent_hash.as_ref().and_then(|parent_hash| {
            let mut cache = self.account_cache.lock();
            if !Self::is_allowed(addr, parent_hash, &cache.modifications) {
                return None;
            }
            cache
                .accounts
                .get_mut(addr)
                .map(|a| a.as_ref().map(|a| a.clone_basic()))
        })
    }

    fn get_cached<F, U>(&self, a: &Address, f: F) -> Option<U>
    where
        F: FnOnce(Option<&mut Account>) -> U,
    {
        self.parent_hash.as_ref().and_then(|parent_hash| {
            let mut cache = self.account_cache.lock();
            if !Self::is_allowed(a, parent_hash, &cache.modifications) {
                return None;
            }
            cache.accounts.get_mut(a).map(|c| f(c.as_mut()))
        })
    }

    fn get_cached_code(&self, hash: &H256) -> Option<Arc<Vec<u8>>> {
        let mut cache = self.code_cache.lock();

        cache.get_mut(hash).map(|code| code.clone())
    }
}

/// Sync wrapper for the account.
struct SyncAccount(Option<Account>);
/// That implementation is safe because account is never modified or accessed in any way.
/// We only need `Sync` here to allow `StateDb` to be kept in a `RwLock`.
/// `Account` is `!Sync` by default because of `RefCell`s inside it.
unsafe impl Sync for SyncAccount {}

#[cfg(test)]
mod tests {
    use ethereum_types::{Address, H256, U256};
    use kvdb::DBTransaction;
    use state::{Account, Backend};
    use test_helpers::get_temp_state_db;

    #[test]
    fn state_db_smoke() {
        let _ = ::env_logger::try_init();

        let state_db = get_temp_state_db();
        let root_parent = H256::random();
        let address = Address::random();
        let h0 = H256::random();
        let h1a = H256::random();
        let h1b = H256::random();
        let h2a = H256::random();
        let h2b = H256::random();
        let h3a = H256::random();
        let h3b = H256::random();
        let mut batch = DBTransaction::new();

        // blocks  [ 3a(c) 2a(c) 2b 1b 1a(c) 0 ]
        // balance [ 5     5     4  3  2     2 ]
        let mut s = state_db.boxed_clone_canon(&root_parent);
        s.add_to_account_cache(address, Some(Account::new_basic(2.into(), 0.into())), false);
        s.journal_under(&mut batch, 0, &h0).unwrap();
        s.sync_cache(&[], &[], true);

        let mut s = state_db.boxed_clone_canon(&h0);
        s.journal_under(&mut batch, 1, &h1a).unwrap();
        s.sync_cache(&[], &[], true);

        let mut s = state_db.boxed_clone_canon(&h0);
        s.add_to_account_cache(address, Some(Account::new_basic(3.into(), 0.into())), true);
        s.journal_under(&mut batch, 1, &h1b).unwrap();
        s.sync_cache(&[], &[], false);

        let mut s = state_db.boxed_clone_canon(&h1b);
        s.add_to_account_cache(address, Some(Account::new_basic(4.into(), 0.into())), true);
        s.journal_under(&mut batch, 2, &h2b).unwrap();
        s.sync_cache(&[], &[], false);

        let mut s = state_db.boxed_clone_canon(&h1a);
        s.add_to_account_cache(address, Some(Account::new_basic(5.into(), 0.into())), true);
        s.journal_under(&mut batch, 2, &h2a).unwrap();
        s.sync_cache(&[], &[], true);

        let mut s = state_db.boxed_clone_canon(&h2a);
        s.journal_under(&mut batch, 3, &h3a).unwrap();
        s.sync_cache(&[], &[], true);

        let s = state_db.boxed_clone_canon(&h3a);
        assert_eq!(
            s.get_cached_account(&address).unwrap().unwrap().balance(),
            &U256::from(5)
        );

        let s = state_db.boxed_clone_canon(&h1a);
        assert!(s.get_cached_account(&address).is_none());

        let s = state_db.boxed_clone_canon(&h2b);
        assert!(s.get_cached_account(&address).is_none());

        let s = state_db.boxed_clone_canon(&h1b);
        assert!(s.get_cached_account(&address).is_none());

        // reorg to 3b
        // blocks  [ 3b(c) 3a 2a 2b(c) 1b 1a 0 ]
        let mut s = state_db.boxed_clone_canon(&h2b);
        s.journal_under(&mut batch, 3, &h3b).unwrap();
        s.sync_cache(&[h1b, h2b, h3b], &[h1a, h2a, h3a], true);
        let s = state_db.boxed_clone_canon(&h3a);
        assert!(s.get_cached_account(&address).is_none());
    }
}
