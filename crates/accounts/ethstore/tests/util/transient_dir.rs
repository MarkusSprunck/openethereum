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

use ethstore::{
    accounts_dir::{KeyDirectory, RootDiskDirectory},
    Error, SafeAccount,
};
use rand::{rngs::OsRng, RngCore};
use std::{env, fs, path::PathBuf};

pub fn random_dir() -> PathBuf {
    let mut rng = OsRng;
    let mut dir = env::temp_dir();
    dir.push(format!("{:x}-{:x}", rng.next_u64(), rng.next_u64()));
    dir
}

pub struct TransientDir {
    dir: RootDiskDirectory,
    path: PathBuf,
}

impl TransientDir {
    pub fn create() -> Result<Self, Error> {
        let path = random_dir();
        let result = TransientDir {
            dir: RootDiskDirectory::create(&path)?,
            path,
        };

        Ok(result)
    }

    pub fn open() -> Self {
        let path = random_dir();
        TransientDir {
            dir: RootDiskDirectory::at(&path),
            path,
        }
    }
}

impl Drop for TransientDir {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.path).expect("Expected to remove temp dir");
    }
}

impl KeyDirectory for TransientDir {
    fn load(&self) -> Result<Vec<SafeAccount>, Error> {
        self.dir.load()
    }

    fn update(&self, account: SafeAccount) -> Result<SafeAccount, Error> {
        self.dir.update(account)
    }

    fn insert(&self, account: SafeAccount) -> Result<SafeAccount, Error> {
        self.dir.insert(account)
    }

    fn remove(&self, account: &SafeAccount) -> Result<(), Error> {
        self.dir.remove(account)
    }

    fn unique_repr(&self) -> Result<u64, Error> {
        self.dir.unique_repr()
    }
}
