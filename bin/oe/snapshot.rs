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

//! Snapshot and restoration commands.

use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use crate::{hash::keccak, types::ids::BlockId};
use ethcore::{
    client::{DatabaseCompactionProfile, Mode, VMType},
    miner::Miner,
    snapshot::{
        io::{PackedReader, PackedWriter, SnapshotReader},
        service::Service as SnapshotService,
        Progress, RestorationStatus, SnapshotConfiguration, SnapshotService as SS,
    },
};
use ethcore_service::ClientService;

use crate::{
    cache::CacheConfig,
    db,
    helpers::{execute_upgrades, to_client_config},
    params::{fatdb_switch_to_bool, tracing_switch_to_bool, Pruning, SpecType, Switch},
    user_defaults::UserDefaults,
};
use dir::Directories;

/// Kinds of snapshot commands.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Kind {
    /// Take a snapshot.
    Take,
    /// Restore a snapshot.
    Restore,
}

/// Command for snapshot creation or restoration.
#[derive(Debug, PartialEq)]
pub struct SnapshotCommand {
    pub cache_config: CacheConfig,
    pub dirs: Directories,
    pub spec: SpecType,
    pub pruning: Pruning,
    pub pruning_history: u64,
    pub pruning_memory: usize,
    pub tracing: Switch,
    pub fat_db: Switch,
    pub compaction: DatabaseCompactionProfile,
    pub file_path: Option<String>,
    pub kind: Kind,
    pub block_at: BlockId,
    pub max_round_blocks_to_import: usize,
    pub snapshot_conf: SnapshotConfiguration,
}

// helper for reading chunks from arbitrary reader and feeding them into the
// service.
fn restore_using<R: SnapshotReader>(
    snapshot: Arc<SnapshotService>,
    reader: &R,
    recover: bool,
) -> Result<(), String> {
    let manifest = reader.manifest();

    info!(
        "Restoring to block #{} (0x{:?})",
        manifest.block_number, manifest.block_hash
    );

    snapshot
        .init_restore(manifest.clone(), recover)
        .map_err(|e| format!("Failed to begin restoration: {e}"))?;

    let (num_state, num_blocks) = (manifest.state_hashes.len(), manifest.block_hashes.len());

    let informant_handle = snapshot.clone();
    ::std::thread::spawn(move || {
        while let RestorationStatus::Ongoing {
            state_chunks_done,
            block_chunks_done,
            ..
        } = informant_handle.restoration_status()
        {
            info!(
                "Processed {state_chunks_done}/{num_state} state chunks and {block_chunks_done}/{num_blocks} block chunks."
            );
            ::std::thread::sleep(Duration::from_secs(5));
        }
    });

    info!("Restoring state");
    for &state_hash in &manifest.state_hashes {
        if snapshot.restoration_status() == RestorationStatus::Failed {
            return Err("Restoration failed".into());
        }

        let chunk = reader
            .chunk(state_hash)
            .map_err(|e| format!("Encountered error while reading chunk {state_hash:?}: {e}"))?;

        let hash = keccak(&chunk);
        if hash != state_hash {
            return Err(format!(
                "Mismatched chunk hash. Expected {state_hash:?}, got {hash:?}"
            ));
        }

        snapshot.feed_state_chunk(state_hash, &chunk);
    }

    info!("Restoring blocks");
    for &block_hash in &manifest.block_hashes {
        if snapshot.restoration_status() == RestorationStatus::Failed {
            return Err("Restoration failed".into());
        }

        let chunk = reader
            .chunk(block_hash)
            .map_err(|e| format!("Encountered error while reading chunk {block_hash:?}: {e}"))?;

        let hash = keccak(&chunk);
        if hash != block_hash {
            return Err(format!(
                "Mismatched chunk hash. Expected {block_hash:?}, got {hash:?}"
            ));
        }
        snapshot.feed_block_chunk(block_hash, &chunk);
    }

    match snapshot.restoration_status() {
        RestorationStatus::Ongoing { .. } => {
            Err("Snapshot file is incomplete and missing chunks.".into())
        }
        RestorationStatus::Initializing { .. } => {
            Err("Snapshot restoration is still initializing.".into())
        }
        RestorationStatus::Failed => Err("Snapshot restoration failed.".into()),
        RestorationStatus::Inactive => {
            info!("Restoration complete.");
            Ok(())
        }
    }
}

impl SnapshotCommand {
    // shared portion of snapshot commands: start the client service
    fn start_service(self) -> Result<ClientService, String> {
        // load spec file
        let spec = self.spec.spec(&self.dirs.cache)?;

        // load genesis hash
        let genesis_hash = spec.genesis_header().hash();

        // database paths
        let db_dirs = self
            .dirs
            .database(genesis_hash, None, spec.data_dir.clone());

        // user defaults path
        let user_defaults_path = db_dirs.user_defaults_path();

        // load user defaults
        let user_defaults = UserDefaults::load(&user_defaults_path)?;

        // select pruning algorithm
        let algorithm = self.pruning.to_algorithm(&user_defaults);

        // check if tracing is on
        let tracing = tracing_switch_to_bool(self.tracing, &user_defaults)?;

        // check if fatdb is on
        let fat_db = fatdb_switch_to_bool(self.fat_db, &user_defaults, algorithm)?;

        // prepare client and snapshot paths.
        let client_path = db_dirs.client_path(algorithm);
        let snapshot_path = db_dirs.snapshot_path();

        // execute upgrades
        execute_upgrades(&self.dirs.base, &db_dirs, algorithm, &self.compaction)?;

        // prepare client config
        let mut client_config = to_client_config(
            &self.cache_config,
            spec.name.to_lowercase(),
            Mode::Active,
            tracing,
            fat_db,
            self.compaction,
            VMType::default(),
            "".into(),
            algorithm,
            self.pruning_history,
            self.pruning_memory,
            true,
            self.max_round_blocks_to_import,
        );

        client_config.snapshot = self.snapshot_conf;

        let restoration_db_handler = db::restoration_db_handler(&client_path, &client_config);
        let client_db = restoration_db_handler
            .open(&client_path)
            .map_err(|e| format!("Failed to open database {e:?}"))?;

        let service = ClientService::start(
            client_config,
            &spec,
            client_db,
            &snapshot_path,
            restoration_db_handler,
            &self.dirs.ipc_path(),
            // TODO [ToDr] don't use test miner here
            // (actually don't require miner at all)
            Arc::new(Miner::new_for_tests(&spec, None)),
        )
        .map_err(|e| format!("Client service error: {e:?}"))?;

        Ok(service)
    }
    /// restore from a snapshot
    pub fn restore(self) -> Result<(), String> {
        let file = self.file_path.clone();
        let service = self.start_service()?;

        warn!("Snapshot restoration is experimental and the format may be subject to change.");
        warn!(
            "On encountering an unexpected error, please ensure that you have a recent snapshot."
        );

        let snapshot = service.snapshot_service();

        if let Some(file) = file {
            info!("Attempting to restore from snapshot at '{file}'");

            let reader = PackedReader::new(Path::new(&file))
                .map_err(|e| format!("Couldn't open snapshot file: {e}"))
                .and_then(|x| x.ok_or("Snapshot file has invalid format.".into()));

            let reader = reader?;
            restore_using(snapshot, &reader, true)?;
        } else {
            info!("Attempting to restore from local snapshot.");

            // attempting restoration with recovery will lead to deadlock
            // as we currently hold a read lock on the service's reader.
            match *snapshot.reader() {
                Some(ref reader) => restore_using(snapshot.clone(), reader, false)?,
                None => return Err("No local snapshot found.".into()),
            }
        }

        Ok(())
    }

    /// Take a snapshot from the head of the chain.
    pub fn take_snapshot(self) -> Result<(), String> {
        let file_path = self
            .file_path
            .clone()
            .ok_or("No file path provided.".to_owned())?;
        let file_path: PathBuf = file_path.into();
        let block_at = self.block_at;
        let service = self.start_service()?;

        warn!("Snapshots are currently experimental. File formats may be subject to change.");

        let writer = PackedWriter::new(&file_path)
            .map_err(|e| format!("Failed to open snapshot writer: {e}"))?;

        let progress = Arc::new(Progress::default());
        let p = progress.clone();
        let informant_handle = ::std::thread::spawn(move || {
            ::std::thread::sleep(Duration::from_secs(5));

            let mut last_size = 0;
            while !p.done() {
                let cur_size = p.size();
                if cur_size != last_size {
                    last_size = cur_size;
                    let bytes = crate::informant::format_bytes(cur_size as usize);
                    info!(
                        "Snapshot: {} accounts {} blocks {}",
                        p.accounts(),
                        p.blocks(),
                        bytes
                    );
                }

                ::std::thread::sleep(Duration::from_secs(5));
            }
        });

        if let Err(e) = service.client().take_snapshot(writer, block_at, &progress) {
            let _ = ::std::fs::remove_file(&file_path);
            return Err(format!(
                "Encountered fatal error while creating snapshot: {e}"
            ));
        }

        info!("snapshot creation complete");

        assert!(progress.done());
        informant_handle
            .join()
            .map_err(|_| "failed to join logger thread")?;

        Ok(())
    }
}

/// Execute this snapshot command.
pub fn execute(cmd: SnapshotCommand) -> Result<String, String> {
    match cmd.kind {
        Kind::Take => cmd.take_snapshot()?,
        Kind::Restore => cmd.restore()?,
    }

    Ok(String::new())
}
