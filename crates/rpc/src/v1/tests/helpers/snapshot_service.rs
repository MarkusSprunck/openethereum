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

use ethcore::snapshot::{CreationStatus, ManifestData, RestorationStatus, SnapshotService};

use bytes::Bytes;
use ethereum_types::H256;
use parking_lot::Mutex;

/// Mocked snapshot service (used for sync info extensions).
pub struct TestSnapshotService {
    status: Mutex<RestorationStatus>,
}

impl TestSnapshotService {
    /// Create a test snapshot service. Only the `status` function matters -- it'll
    /// return `Inactive` by default.
    pub fn new() -> Self {
        TestSnapshotService {
            status: Mutex::new(RestorationStatus::Inactive),
        }
    }

    /// Set the restoration status.
    pub fn set_status(&self, status: RestorationStatus) {
        *self.status.lock() = status;
    }
}

impl SnapshotService for TestSnapshotService {
    fn manifest(&self) -> Option<ManifestData> {
        None
    }
    fn manifest_block(&self) -> Option<(u64, H256)> {
        None
    }
    fn supported_versions(&self) -> Option<(u64, u64)> {
        None
    }
    fn completed_chunks(&self) -> Option<Vec<H256>> {
        Some(vec![])
    }
    fn chunk(&self, _hash: H256) -> Option<Bytes> {
        None
    }
    fn restoration_status(&self) -> RestorationStatus {
        *self.status.lock()
    }
    fn creation_status(&self) -> CreationStatus {
        CreationStatus::Inactive
    }
    fn begin_restore(&self, _manifest: ManifestData) {}
    fn abort_restore(&self) {}
    fn abort_snapshot(&self) {}
    fn restore_state_chunk(&self, _hash: H256, _chunk: Bytes) {}
    fn restore_block_chunk(&self, _hash: H256, _chunk: Bytes) {}
    fn shutdown(&self) {}
}
