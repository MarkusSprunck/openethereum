//! Compatibility shim for the unmaintained `atty 0.2` crate.
//!
//! Re-implements the full public API using `std::io::IsTerminal` (stable since
//! Rust 1.70.0), which has no CVE.  Adding this crate as a `[patch.crates-io]`
//! override in the workspace root `Cargo.toml` replaces every consumer of the
//! original `atty` crate — including transitive dependencies such as
//! `clap 2.34.0` and `env_logger 0.5` — with this safe implementation.

use std::io::IsTerminal as _;

/// The stream to test.
#[derive(Clone, Copy, Debug)]
pub enum Stream {
    Stdout,
    Stderr,
    Stdin,
}

/// Returns `true` if the given stream is a TTY.
#[inline]
pub fn is(stream: Stream) -> bool {
    match stream {
        Stream::Stdout => std::io::stdout().is_terminal(),
        Stream::Stderr => std::io::stderr().is_terminal(),
        Stream::Stdin => std::io::stdin().is_terminal(),
    }
}

/// Returns `true` if the given stream is **not** a TTY.
#[inline]
pub fn isnt(stream: Stream) -> bool {
    !is(stream)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_and_isnt_are_inverse() {
        for stream in [Stream::Stdout, Stream::Stderr, Stream::Stdin] {
            assert_ne!(is(stream), isnt(stream));
        }
    }
}

