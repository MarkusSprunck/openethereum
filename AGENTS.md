# GitHub Copilot Agent Instructions

**Version:** 1.0
**Last Updated:** 2026-07-10
**Project:** OpenEthereum v3.5.0 (Fast, Feature-rich Ethereum Client in Rust)

---

## 🤖 Purpose

This file provides AI coding agents with the essential context to be immediately productive in the OpenEthereum codebase. It covers architecture, build workflows, conventions, known technical debt, and reference points — adapted from `.github/templates/agents.md` and routed via `.github/copilot-instructions.md`.

---

## 📂 Project Overview

### Technology Stack

- **Language:** Rust (edition 2021, toolchain pinned to 1.88)
- **Build tool:** Cargo (workspace layout with standalone members)
- **Blockchain protocol:** Ethereum (GPL-3.0)
- **Database:** RocksDB via `kvdb-rocksdb`
- **Networking:** devp2p (`ethcore-network-devp2p`)
- **RPC:** `jsonrpc-core` v15 (HTTP `:8545`, WebSocket `:8546`)
- **Async runtime:** Tokio 1.x (`parity-runtime`)
- **Primary deployment target:** Cert4Trust Leopold Blockchain

### Key Components

- **Dual binary/library layout:** `[lib]` in `Cargo.toml` points to `bin/oe/lib.rs`, not `src/lib.rs`
- **Command dispatch:** `configuration.rs` parses CLI into `Cmd` enum; `lib.rs::execute()` dispatches
- **Full-node wiring:** `run.rs` connects client, sync, RPC, and miner subsystems
- **Feature-gated subsystems:** `accounts` (default), `secretstore`, `json-tests`, `deadlock_detection`, `memory_profiling`
- **Local crypto forks:** `aes`, `aesni`, `aes-soft`, `block-cipher-trait`, `stream-cipher` patched via `[patch.crates-io]`
- **Standalone workspace members:** `bin/ethkey`, `bin/ethstore`, `bin/evmbin`, `bin/chainspec` — NOT in main dependency tree

### Project Structure

```
openethereum/
├── bin/                                ← Executable entry points
│   ├── oe/                             ← Main client (lib.rs = library root, main.rs = binary entry)
│   │   ├── cli/                        ← CLI argument definitions (docopt + clap)
│   │   ├── db/                         ← RocksDB wrappers, bloom filters, migrations
│   │   ├── logger/                     ← Rotating file logger setup
│   │   ├── configuration.rs            ← CLI → Cmd enum mapping (2000+ lines, central dispatch)
│   │   ├── run.rs                      ← Full-node startup: client, sync, RPC, miner wiring
│   │   ├── lib.rs                      ← Library root; all mod declarations, start() public API
│   │   ├── main.rs                     ← Binary entry; arg parse, logger init, signal handling
│   │   ├── params.rs                   ← Node parameter structs (AccountsConfig, GasPricerConfig…)
│   │   ├── rpc.rs / rpc_apis.rs        ← RPC server setup and API registry
│   │   ├── account.rs / account_utils.rs ← Account CLI subcommands
│   │   ├── blockchain.rs               ← Blockchain import/export/reset CLI subcommands
│   │   ├── snapshot.rs                 ← Snapshot create/restore CLI subcommands
│   │   ├── signer.rs / secretstore.rs  ← Signing and secret store integration
│   │   ├── informant.rs                ← Sync progress display
│   │   ├── metrics.rs                  ← Prometheus metrics configuration
│   │   ├── modules.rs                  ← Subsystem module wiring
│   │   └── user_defaults.rs            ← Persistent user default settings
│   ├── ethkey/                         ← Key generation CLI (standalone workspace member)
│   │   └── src/
│   ├── ethstore/                       ← Key management CLI (standalone workspace member)
│   │   └── src/
│   ├── evmbin/                         ← EVM standalone runner (standalone workspace member)
│   │   ├── benches/
│   │   ├── res/
│   │   └── src/
│   └── chainspec/                      ← Chain specification tool (standalone workspace member)
│       └── src/
├── crates/                             ← Library crates (all in main dependency tree)
│   ├── accounts/                       ← Account management umbrella crate
│   │   ├── ethkey/                     ← Key pair generation, signing, verification
│   │   ├── ethstore/                   ← Keystore file management (UTC/JSON)
│   │   └── src/
│   ├── concensus/                      ← Consensus & mining
│   │   ├── ethash/                     ← Ethash PoW & ProgPoW implementation
│   │   └── miner/                      ← Miner, transaction pool, stratum
│   ├── db/                             ← Database layer
│   │   ├── bloom/                      ← Bloom filter primitives
│   │   ├── blooms-db/                  ← Bloom index database
│   │   ├── db/                         ← Generic DB traits (ethcore-db)
│   │   ├── journaldb/                  ← Journaling overlay for RocksDB
│   │   ├── memory-db/                  ← In-memory DB for tests
│   │   ├── migration-rocksdb/          ← RocksDB schema migration helpers
│   │   └── patricia-trie-ethereum/     ← Ethereum-specific patricia trie
│   ├── ethcore/                        ← Core blockchain engine
│   │   ├── blockchain/                 ← Block & transaction storage, chain metadata
│   │   ├── ethereum-forkid/            ← EIP-2124 fork identifier
│   │   ├── res/                        ← Built-in chain spec JSON files
│   │   │   └── json_tests/             ← Git submodule: official Ethereum test vectors
│   │   ├── service/                    ← ClientService: I/O loop, client lifecycle
│   │   ├── src/                        ← Core logic: EVM, consensus engine, miner, verification
│   │   ├── sync/                       ← devp2p block/tx synchronisation protocol
│   │   └── types/                      ← Shared types (common-types): block, tx, receipt…
│   ├── ethjson/                        ← JSON deserialization for chain specs and test fixtures
│   │   └── src/
│   ├── net/                            ← Networking stack
│   │   ├── fake-fetch/                 ← Test stub for HTTP fetch
│   │   ├── fetch/                      ← Async HTTP client
│   │   ├── network/                    ← devp2p network traits (ethcore-network)
│   │   ├── network-devp2p/             ← devp2p protocol implementation
│   │   └── node-filter/                ← Smart-contract-based peer permission filter
│   ├── rpc/                            ← JSON-RPC API
│   │   └── src/v1/                     ← All method implementations (eth_, net_, parity_…)
│   ├── rpc-common/                     ← Shared RPC types (Bytes, etc.)
│   ├── rpc-servers/                    ← HTTP (:8545) and WebSocket (:8546) server setup
│   ├── runtime/                        ← Async runtime
│   │   ├── io/                         ← ethcore-io: I/O handler and service loop
│   │   └── runtime/                    ← parity-runtime: tokio executor wrapper
│   ├── transaction-pool/               ← Pending transaction pool logic
│   ├── util/                           ← Shared utilities
│   │   ├── EIP-152/                    ← Blake2 compression (EIP-152)
│   │   ├── EIP-712/                    ← Structured data hashing (EIP-712)
│   │   ├── aes/ aes-soft/             ← Local AES fork (patched via [patch.crates-io])
│   │   ├── block-cipher-trait/         ← Local block-cipher-trait fork
│   │   ├── stream-cipher/              ← Local stream-cipher fork
│   │   ├── cli-signer/                 ← IPC signer client helpers
│   │   ├── dir/                        ← Default data/config path resolution
│   │   ├── keccak-hasher/              ← Keccak256 hasher for trie
│   │   ├── stats/                      ← Moving average & histogram stats
│   │   ├── version/                    ← parity-version: build version string
│   │   └── …                           ← fastmap, len-caching-lock, macros, memzero, …
│   └── vm/                             ← Virtual machine layer
│       ├── builtin/                    ← Precompiled contracts
│       ├── call-contract/              ← On-chain contract call helper
│       ├── evm/                        ← EVM interpreter implementation
│       ├── vm/                         ← VM traits and types
│       └── wasm/                       ← WASM interpreter
├── docs/                               ← Historical changelogs (v0.9 – v3.1)
├── scripts/                            ← Developer helper scripts
│   ├── setup-rust-1.88.sh              ← Pins exact Rust toolchain (run first)
│   ├── build-release.sh                ← cargo build --release --features final
│   ├── test-all-macos-arm64.sh         ← macOS test runner with Clang override
│   └── test-all-linux-gcc.sh           ← Linux test runner
├── Cargo.toml                          ← Root manifest; workspace, features, [patch] overrides
├── Cargo.lock                          ← Locked dependency versions (committed)
├── AGENTS.md                           ← AI agent instructions (this file)
├── MAINTENANCE.md                      ← Dev setup, CVE status, upgrade blockers
└── CHANGELOG.md                        ← Release history
```

---

## 🎯 Critical Instructions for Copilot

### 1. Dependency Management & Updates

#### ⚠️ MANDATORY Process

Read `.github/copilot-instructions.md` before making any dependency changes.

- **DO NOT** upgrade `jsonrpc-*` (v15 → v18) or `parity-util-mem` (0.7.0 → 0.11.0) without a full migration plan — both require coordinated changes across many crates and will introduce breaking `ethereum-types` conflicts
- **DO NOT** upgrade `secp256k1` independently — constrained by `parity-crypto v0.6.2` chain
- For `atty` (Windows-only CVE): safe to replace, used only in a few lines
- Always run `cargo build` after any `Cargo.toml` change to catch breakage early
- Check `MAINTENANCE.md` § 6.0 for the current CVE status before touching any vulnerable dependency

### 2. Documentation Standards

- Use Rust doc comments (`///`) for all public API items in `bin/oe/lib.rs` and crate roots
- Reference specific file paths and line ranges when describing changes (e.g., `configuration.rs` 2000+ lines)
- Document all non-obvious feature flag interactions (e.g., `accounts` feature gates `ethcore-accounts`)
- Include dates when documenting build/test verification results
- Avoid generic advice — always reference specific files or commands from this project

### 3. Modular Coding Rules

- Use `extern crate` style even in Rust 2021 crates — this codebase keeps old-style declarations for compatibility with pre-2018 upstream crates
- New subsystems must be feature-gated in `Cargo.toml` and declared conditionally in `bin/oe/lib.rs`
- Adding a new workspace member requires updating `[workspace] members` in root `Cargo.toml` only if it is truly standalone (not in main dep tree)
- `[patch.crates-io]` overrides must be mirrored for all affected crates to avoid version conflicts

---

## 📚 Project-Specific Guidelines

### Cargo Version Management

Versions are declared directly in `Cargo.toml` (no Maven-style property substitution). Coordinated upgrades follow this pattern:

1. Update version in root `Cargo.toml` dependency entry
2. Check all `crates/*/Cargo.toml` for the same dependency
3. Run `cargo build` to surface version conflicts
4. Run `cargo test --all` to confirm no regressions
5. Update `MAINTENANCE.md` CVE table and `AGENTS.md`

### Configuration Files

| File | Purpose |
|---|---|
| `Cargo.toml` | Root manifest; feature flags, workspace, `[patch]` overrides |
| `bin/oe/configuration.rs` | `Configuration → Cmd` mapping (central dispatch, 2000+ lines) |
| `bin/oe/params.rs` | Node parameter structs (`AccountsConfig`, `GasPricerConfig`, etc.) |
| `bin/oe/run.rs` | Full-node wiring: client, sync, RPC, miner |
| `crates/ethcore/res/` | Chain spec JSON files and official test vectors (submodule) |
| `MAINTENANCE.md` | Dev environment setup, CVE status, known upgrade blockers |

---

## 🔧 Development Workflow

### Running the Application

```bash
# Pin Rust toolchain (required once per environment)
./scripts/setup-rust-1.88.sh

# Start node (default: mainnet, RPC on :8545/:8546)
./target/release/openethereum

# Start with a specific chain (e.g., Leopold)
./target/release/openethereum --chain /path/to/leopold.json
```

### Build & Test

**1. Pin Rust version**
```bash
./scripts/setup-rust-1.88.sh
```

**2. Fetch Ethereum JSON test vectors** (required before first test run)
```bash
git submodule update --init --recursive
```

**3. Build**
```bash
cargo build                                   # debug (panic=abort, incremental)
cargo build --release --features final        # production binary
```

**4. Test**
```bash
cargo test --all                              # all crates
cargo test --package ethcore                  # single crate
cargo test --package evmbin -- --nocapture    # with stdout

# macOS arm64 (requires Clang override)
brew install lz4 zstd snappy rocksdb
export CC=/usr/bin/clang && export CXX=/usr/bin/clang++
time cargo test --all
```

> **Note:** `[profile.test]` uses `opt-level = 3` — compilation is slow, test execution is fast.

**5. Docker image** (equivalent to CI workflow)
```bash
docker buildx build \
  --platform linux/amd64 \
  -f .github/docker/ubuntu-rust-1.88/Dockerfile \
  -t ihkmunich/openethereum:latest-rust-1.88 \
  .
```

---

## 🛡️ Security Considerations

### Always Check

- [ ] No upgrade to `jsonrpc-*` or `parity-util-mem` without migration plan
- [ ] CVE status in `MAINTENANCE.md` § 6.0 reviewed before touching dependencies
- [ ] `secp256k1` version remains constrained by `parity-crypto v0.6.2`
- [ ] `atty` replacement is safe but only relevant for Windows builds
- [ ] New RPC endpoints require auth/CORS review in `crates/rpc-servers/src/`

### Known Vulnerable Dependencies ⚠️

| Dependency | Current | Fix Available | Blocker |
|---|---|---|---|
| `jsonrpc-*` | v15 | v18 | Requires hyper/tokio migration |
| `parity-util-mem` | 0.7.0 | 0.11.0 | `ethereum-types` breaking changes |
| `secp256k1` | 0.17.2 | 0.22.2 | `parity-crypto` chain constraint |
| `atty` | 0.2.14 | Replace crate | Windows-only CVE, low priority |

### RPC Security ⭐ IF APPLICABLE

- HTTP JSON-RPC on `:8545` — restrict with `--jsonrpc-hosts` and `--jsonrpc-cors` in production
- WebSocket on `:8546` — restrict with `--ws-origins` and `--ws-hosts`
- IPC socket enabled by default; disable with `--no-ipc` if not needed

---

## 📖 Reference Documentation

### Internal Docs

- `.github/copilot-instructions.md` — AI task router (read first)
- `.github/templates/agents.md` — AGENTS.md structure template
- `MAINTENANCE.md` — Dev setup (Ubuntu primary, macOS notes, CVE status)
- `CHANGELOG.md` — Release history
- `bin/oe/lib.rs` — Public API: `start()`, `ExecutionAction`, `Configuration`
- `bin/oe/configuration.rs` — Complete `Cmd` enum and CLI→config mapping
- `crates/ethcore/src/` — Core blockchain, EVM execution, consensus engine
- `crates/rpc/src/v1/` — All JSON-RPC method implementations

### External Resources

- [OpenEthereum Wiki](https://openethereum.github.io/)
- [Ethereum JSON Tests](https://github.com/ethereum/tests) (submodule at `crates/ethcore/res/json_tests/`)
- [jsonrpc-core v15 docs](https://docs.rs/jsonrpc-core/15.0.0)
- [Rust rustup toolchain management](https://rust-lang.github.io/rustup/)

---

## 🎓 Example Interactions

### Good Prompt (Dependency Update)

> "Check if `toml` can be safely updated to the latest version. Review `MAINTENANCE.md` for blockers, update `Cargo.toml`, run `cargo build`, and document the result."

**Expected actions:** Read `MAINTENANCE.md`, search for `toml` across all `Cargo.toml` files, update version, build, confirm no breakage.

### Bad Prompt (Dependency Update)

> "Update all dependencies to latest versions."

**What Copilot should do instead:** Refuse blanket upgrades. Check each dependency against the blockers in `MAINTENANCE.md` § 6.0 and the table in `AGENTS.md` before touching anything.

---

## 🚨 Emergency Procedures

### If Build Fails

1. Check Rust toolchain: `rustup show` — must be `1.88`; fix with `./scripts/setup-rust-1.88.sh`
2. Clean and rebuild: `cargo clean && cargo build`
3. On macOS: confirm `CC=/usr/bin/clang CXX=/usr/bin/clang++` are exported
4. Submodule missing: `git submodule update --init --recursive`
5. Version conflict: check `[patch.crates-io]` in root `Cargo.toml` — local crypto forks must match

---

## 🔄 Regular Maintenance

### Quarterly Tasks

- [ ] Review CVE alerts in `MAINTENANCE.md` § 6.0 and GitHub Dependabot
- [ ] Update Rust toolchain pin in `scripts/setup-rust-1.88.sh` if a new stable is required
- [ ] Run full test suite: `git submodule update --init --recursive && cargo test --all`
- [ ] Review `atty` replacement opportunity (Windows CVE, low effort)
- [ ] Sync `AGENTS.md` with any structural changes to `bin/oe/` or `crates/`

### Before Each Release

- [ ] Set version in root `Cargo.toml` and `crates/util/version/Cargo.toml` (both must match)
- [ ] Build with `cargo build --release --features final`
- [ ] Run `cargo test --all` with submodules initialized
- [ ] Update `CHANGELOG.md` with all changes
- [ ] Verify RPC endpoint security settings in release configuration

---

## 💡 Tips for Copilot

- Always read `MAINTENANCE.md` before modifying any dependency — it documents upgrade blockers
- The `[lib]` path pointing to `bin/oe/lib.rs` is intentional — do not create a `src/lib.rs`
- `extern crate` declarations in `bin/oe/lib.rs` are the authoritative list of available crates
- Standalone workspace members (`ethkey`, `ethstore`, `evmbin`, `chainspec`) have their own `Cargo.toml` and are built/tested independently
- `configuration.rs` is the single source of truth for all CLI flags — add new parameters there first
- When in doubt about a data flow, trace: `main.rs` → `start()` → `execute()` → `Cmd::Run` → `run::execute()`

---

## 📞 Support & Questions

1. Check `MAINTENANCE.md` for environment setup and known issues
2. Search `crates/ethcore/src/` for core blockchain behaviour questions
3. Review `bin/oe/configuration.rs` for CLI and configuration questions
4. File an issue at [github.com/openethereum/openethereum](https://github.com/openethereum/openethereum/issues)

---

**Last Reviewed:** 2026-07-10
**Next Review:** Q4 2026
**Maintained by:** Markus Sprunck
