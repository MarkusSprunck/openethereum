//! Compatibility shim for the deprecated `tempdir 0.3.7` crate.
//!
//! Re-implements the full `tempdir` public API using `tempfile`, which relies on
//! `remove_dir_all >= 0.8` (fixed RUSTSEC-2021-0126 / CVE).  Adding this crate
//! as a `[patch.crates-io]` override in the workspace root `Cargo.toml` ensures
//! that every consumer of the original `tempdir` crate – including transitive
//! build-dependencies such as `skeptic` (via `local-encoding`) – uses the safe
//! implementation automatically.

use std::io;
use std::path::{Path, PathBuf};

/// A directory in the file-system that is automatically deleted when it goes
/// out of scope.  Drop-in replacement for `tempdir::TempDir`.
pub struct TempDir(tempfile::TempDir);

impl TempDir {
    /// Create a new temporary directory with the given prefix in the system's
    /// default temporary directory.
    pub fn new(prefix: &str) -> io::Result<Self> {
        tempfile::Builder::new()
            .prefix(prefix)
            .tempdir()
            .map(TempDir)
    }

    /// Create a new temporary directory with the given prefix inside `tmpdir`.
    pub fn new_in<P: AsRef<Path>>(tmpdir: P, prefix: &str) -> io::Result<Self> {
        tempfile::Builder::new()
            .prefix(prefix)
            .tempdir_in(tmpdir)
            .map(TempDir)
    }

    /// Returns the path to the temporary directory.
    pub fn path(&self) -> &Path {
        self.0.path()
    }

    /// Converts the object into a `PathBuf`, preventing automatic deletion.
    pub fn into_path(self) -> PathBuf {
        // tempfile 3.x deprecated into_path() in favour of keep() → PathBuf
        self.0.keep()
    }

    /// Closes and removes the temporary directory, returning an error if
    /// deletion fails.
    pub fn close(self) -> io::Result<()> {
        self.0.close()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creates_dir() {
        let dir = TempDir::new("oe_test_").unwrap();
        assert!(dir.path().exists());
        assert!(dir.path().is_dir());
    }

    #[test]
    fn test_new_in_creates_dir() {
        let outer = TempDir::new("oe_outer_").unwrap();
        let inner = TempDir::new_in(outer.path(), "oe_inner_").unwrap();
        assert!(inner.path().exists());
    }

    #[test]
    fn test_into_path_survives_drop() {
        let p = {
            let dir = TempDir::new("oe_persist_").unwrap();
            dir.into_path()
        };
        assert!(p.exists());
        std::fs::remove_dir_all(&p).unwrap();
    }

    #[test]
    fn test_close_removes_dir() {
        let dir = TempDir::new("oe_close_").unwrap();
        let p = dir.path().to_path_buf();
        dir.close().unwrap();
        assert!(!p.exists());
    }
}

