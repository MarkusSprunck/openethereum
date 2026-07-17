# GitHub Copilot Agent Instructions

**Version:** 2.6
**Last Updated:** 2026-07-17
**Project:** OpenEthereum v3.5.1 (Fast, Feature-rich Ethereum Client in Rust)
---

## 🤖 Purpose

This file provides AI coding agents with the essential context to be immediately productive in the OpenEthereum codebase. It covers architecture, build workflows, conventions, known technical debt, and reference points — adapted from `.github/templates/agents.md` and routed via `.github/copilot-instructions.md`.

---

## 📂 Project Overview

### Technology Stack

- **Language:** Rust (edition 2021, toolchain pinned to 1.97.1)
- **Build tool:** Cargo (workspace layout with standalone members)
- **Rust upgrade:** 1.88 → 1.97.1 (2026-07-10); see `scripts/setup-rust-1.97.1.sh`
- **Blockchain protocol:** Ethereum (GPL-3.0)
- **Database:** RocksDB via `kvdb-rocksdb`
- **Networking:** devp2p (`ethcore-network-devp2p`)
- **RPC:** `jsonrpc-core` v18 (HTTP `:8545`, WebSocket `:8546`)
- **Async runtime:** Tokio 1.x (`parity-runtime`)
- **Primary deployment target:** Cert4Trust Leopold Blockchain

### Key Components

- **Dual binary/library layout:** `[lib]` in `Cargo.toml` points to `bin/oe/lib.rs`, not `src/lib.rs`
- **Command dispatch:** `configuration.rs` parses CLI into `Cmd` enum; `lib.rs::execute()` dispatches
- **Full-node wiring:** `run.rs` connects client, sync, RPC, and miner subsystems
- **Feature-gated subsystems:** `accounts` (default), `secretstore`, `json-tests`, `deadlock_detection`, `memory_profiling`
- **Local crypto forks:** `aes`, `aesni`, `aes-soft`, `block-cipher-trait`, `stream-cipher` patched via `[patch.crates-io]`
- **CVE patch shims:** `atty-compat` (RUSTSEC-2021-0017), `tempdir-compat` (RUSTSEC-2021-0126), `lock-api-compat` (CVE-2020-35910..35914), and `parity-util-mem-compat` (lru RUSTSEC Dependabot #12/#18) are local shims registered via `[patch.crates-io]`; all four must be workspace members so Cargo resolves them
- **Standalone workspace members:** `bin/ethkey`, `bin/ethstore`, `bin/evmbin`, `bin/chainspec` — NOT in main dependency tree

### Project Structure

```
openethereum/
├── bin/                                ← Executable entry points
│   ├── oe/                             ← Main client (lib.rs = library root, main.rs = binary entry)
│   │   ├── account.rs / account_utils.rs ← Account CLI subcommands
│   │   ├── blockchain.rs               ← Blockchain import/export/reset CLI subcommands
│   │   ├── cli/                        ← CLI argument definitions (docopt + clap)
│   │   ├── configuration.rs            ← CLI → Cmd enum mapping (2000+ lines, central dispatch)
│   │   ├── db/                         ← RocksDB wrappers, bloom filters, migrations
│   │   ├── informant.rs                ← Sync progress display
│   │   ├── lib.rs                      ← Library root; all mod declarations, start() public API
│   │   ├── logger/                     ← Rotating file logger setup
│   │   ├── main.rs                     ← Binary entry; arg parse, logger init, signal handling
│   │   ├── metrics.rs                  ← Prometheus metrics configuration
│   │   ├── modules.rs                  ← Subsystem module wiring
│   │   ├── params.rs                   ← Node parameter structs (AccountsConfig, GasPricerConfig…)
│   │   ├── rpc.rs / rpc_apis.rs        ← RPC server setup and API registry
│   │   ├── run.rs                      ← Full-node startup: client, sync, RPC, miner wiring
│   │   ├── signer.rs / secretstore.rs  ← Signing and secret store integration
│   │   ├── snapshot.rs                 ← Snapshot create/restore CLI subcommands
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
│   ├── aes/ aes-soft/             ← Local AES fork (patched via [patch.crates-io])
│   ├── atty-compat/               ← CVE shim replacing atty 0.2.14 (RUSTSEC-2021-0017); **FIXED (2026-07-13)**
│   ├── block-cipher-trait/        ← Local block-cipher-trait fork
│   ├── cli-signer/                ← IPC signer client helpers
│   ├── dir/                       ← Default data/config path resolution
│   ├── stats/                     ← Moving average & histogram stats
│   ├── keccak-hasher/             ← Keccak256 hasher for trie
│   ├── lock-api-compat/           ← CVE shim replacing lock_api 0.3.4 (CVE-2020-35910..35914); **FIXED (2026-07-13)**
│   ├── parity-util-mem-compat/    ← CVE shim: fork of parity-util-mem 0.7.0 with lru 0.5.3→0.7.8 (Dependabot #12/#18); **FIXED (2026-07-14)**
│   ├── stream-cipher/             ← Local stream-cipher fork
│   ├── tempdir-compat/            ← CVE shim replacing tempdir 0.3.7 (RUSTSEC-2021-0126); **FIXED (2026-07-13)**
│   ├── version/                   ← parity-version: build version string
│   └── …                          ← fastmap, len-caching-lock, macros, memzero, …
│   └── vm/                             ← Virtual machine layer
│       ├── builtin/                    ← Precompiled contracts
│       ├── call-contract/              ← On-chain contract call helper
│       ├── evm/                        ← EVM interpreter implementation
│       ├── vm/                         ← VM traits and types
│       └── wasm/                       ← WASM interpreter
├── docs/                               ← Historical changelogs (v0.9 – v3.1)
├── scripts/                            ← Developer helper scripts
│   ├── build-artifacts-cli-tools-linux-gcc.sh   ← Build CLI tool artifacts (Linux GCC)
│   ├── build-artifacts-cli-tools-macos-arm64.sh ← Build CLI tool artifacts (macOS arm64)
│   ├── build-release.sh                ← cargo build --release --features final
│   ├── find-native-libraries-required.sh ← Discover native .so/.dylib deps of release binary
│   ├── setup-rust-1.97.1.sh            ← Pins exact Rust toolchain (run first)
│   ├── test-all-linux-gcc.sh           ← Linux test runner
│   ├── test-all-macos-arm64.sh         ← macOS test runner with Clang override
│   └── generate-code-coverage-html.sh  ← Generate HTML coverage report (llvm-cov)
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

- **DO NOT** perform further major upgrades of `jsonrpc-*` (currently v18) or upgrade `parity-util-mem` (0.7.0 → 0.11.0) without a full migration plan — both require coordinated changes across many crates and can introduce breaking `ethereum-types` conflicts
- **DO NOT** upgrade `secp256k1` independently — constrained by `parity-crypto v0.6.2` chain
- For `term_size` (unmaintained): replace with `terminal_size = "0.3"`
- **Do NOT upgrade `rayon`** beyond 1.1 without re-testing on macOS — 1.12 introduced EMFILE failures; pinned at 1.1 intentionally
- **Do NOT upgrade `number_prefix`** beyond 0.2.8 — 0.4.0 changed `binary_prefix()` to `NumberPrefix::binary()` and required qualified variant names
- Follow the phased dependency upgrade sequence: Phase 2 (term_size→terminal_size), Phase 3 (`jsonrpc-*` v18) is complete, Phase 4 (`secp256k1`/`rand`/`ethereum-types`/`parity-util-mem` chain — blocked by `parity-crypto` and type compatibility)
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
- `[patch.crates-io]` shims (`atty-compat`, `tempdir-compat`, `lock-api-compat`, `parity-util-mem-compat`) also require a `[workspace] members` entry so Cargo resolves them — see existing entries as the pattern
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
./scripts/setup-rust-1.97.1.sh

# Start node (default: mainnet, RPC on :8545/:8546)
./target/release/openethereum

# Start with a specific chain (e.g., Leopold)
./target/release/openethereum --chain /path/to/leopold.json
```

### Build & Test

**1. Pin Rust version**
```bash
./scripts/setup-rust-1.97.1.sh
```

**2. Fetch Ethereum JSON test vectors** (required before first test run)
```bash
git submodule update --init --recursive
```

**3. Build**
```bash
cargo build                                   # debug (panic=abort, incremental)
cargo build --release --features final        # production binary
./scripts/build-release.sh                    # equivalent convenience script
```

**4. Test**
```bash
cargo test --all                              # all crates
cargo test --package ethcore                  # single crate
cargo test --package evmbin -- --nocapture    # with stdout

# macOS arm64 (requires Clang override AND raised FD limit — use the script)
./scripts/test-all-macos-arm64.sh
# Equivalent manual steps:
#   ulimit -n 65536
#   brew install lz4 zstd snappy rocksdb
#   export CC=/usr/bin/clang && export CXX=/usr/bin/clang++
#   cargo test --all

# ⚠️ macOS EMFILE note: rayon-core 1.13.0 opens extra OS FDs per thread via
# kqueue, which combined with RocksDB exhausts the default macOS FD limit (256).
# rayon is intentionally pinned at 1.1 in Cargo.toml (not 1.12). Always use
# ./scripts/test-all-macos-arm64.sh (sets ulimit -n 65536) instead of bare
# `cargo test --all` on macOS.
```

> **Note:** `[profile.test]` uses `opt-level = 3` — compilation is slow, test execution is fast.

**5. Docker image** (CI-equivalent build)
```bash
docker buildx build \
  --platform linux/amd64 \
  -f .github/docker/ubuntu-rust-1.97.1/Dockerfile.ci \
  -t ihkmunich/openethereum:latest-local \
  .
```

> **CI workflows:**
> - `docker-ubuntu-latest.yml` — triggered on push to `main`; pushes tag `latest-rust-1.97`; steps: Test Execution → Release Build → Docker build & push
> - `docker-ubuntu-release.yml` — triggered on tag `v*`; pushes versioned tags; steps: Test Execution → Release Build → Docker build & push
> - Legacy image base `ubuntu-rust-1.88` remains in `.github/docker/ubuntu-rust-1.88/` for reference

---

## 🛡️ Security Considerations

### Always Check

- [ ] No upgrade to `parity-util-mem` without migration plan
- [ ] CVE status in `MAINTENANCE.md` § 6.0 reviewed before touching dependencies
- [ ] `secp256k1` version remains constrained by `parity-crypto v0.6.2`
- [ ] `atty` replacement is safe (already FIXED 2026-07-13)
- [ ] `lock_api` CVE backport-fix is in place for kvdb-memorydb chain; jsonrpc chain eliminated (Phase 3 DONE)
- [ ] `lru` vulnerability in `parity-util-mem` fixed via `parity-util-mem-compat` shim (FIXED 2026-07-14)
- [ ] `rpassword` upgraded from `1.0.2` to `7.5.0` (GHSA-2p6r-x3vv-xqm2, FIXED 2026-07-14)
- [ ] New RPC endpoints require auth/CORS review in `crates/rpc-servers/src/`

### Known Vulnerable Dependencies ⚠️

| Dependency | Current | Fix Available | Blocker |
|---|---|---|---|
| `secp256k1` | 0.17.2 | 0.22.2 (GHSA-969w-q74q-9j8v, MEDIUM) | `parity-crypto` chain constraint (Phase 4 blocked); not exploitable (`preallocated_gen_new` is never called) |
| `rand` | 0.7.3 | 0.10.1 (GHSA-cq8v-f236-94qc, LOW) | Blocked by `ethereum-types 0.9.2` — Phase 4; not exploitable (no custom logger calling `thread_rng()`) |

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
- `.testing/README.md` — Leopold Blockchain test client configuration (referenced in `MAINTENANCE.md` §5.0)
- `CHANGELOG.md` — Release history
- `bin/oe/lib.rs` — Public API: `start()`, `ExecutionAction`, `Configuration`
- `bin/oe/configuration.rs` — Complete `Cmd` enum and CLI→config mapping
- `crates/ethcore/src/` — Core blockchain, EVM execution, consensus engine
- `crates/rpc/src/v1/` — All JSON-RPC method implementations

### External Resources

- [OpenEthereum Wiki](https://openethereum.github.io/)
- [Ethereum JSON Tests](https://github.com/ethereum/tests) (submodule at `crates/ethcore/res/json_tests/`)
- [jsonrpc-core v18 docs](https://docs.rs/jsonrpc-core/18.0.0)
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

1. Check Rust toolchain: `rustup show` — must be `1.97.1`; fix with `./scripts/setup-rust-1.97.1.sh`
2. Clean and rebuild: `cargo clean && cargo build`
3. On macOS: confirm `CC=/usr/bin/clang CXX=/usr/bin/clang++` are exported
4. Submodule missing: `git submodule update --init --recursive`
5. Version conflict: check `[patch.crates-io]` in root `Cargo.toml` — local crypto forks must match

---

## 🔄 Regular Maintenance

### Quarterly Tasks

- [ ] Review CVE alerts in `MAINTENANCE.md` § 6.0 and GitHub Dependabot
- [ ] Update Rust toolchain pin in `scripts/setup-rust-1.97.1.sh` if a new stable is required
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

**Last Reviewed:** 2026-07-17
**Next Review:** Q4 2026
**Maintained by:** Markus Sprunck

**Changelog:**
- v2.6 (2026-07-17): Fixed script filename: `setup-rust-1.97.sh` → `setup-rust-1.97.1.sh` in Technology Stack, Build & Test, and Emergency Procedures sections (actual file on disk is `scripts/setup-rust-1.97.1.sh` as confirmed by CHANGELOG v3.5.1)
- v2.5 (2026-07-17): Refreshed dependency guidance after Phase 3 completion: clarified `jsonrpc-*` is already on v18 and updated phased sequence to emphasize remaining Phase 4 blockers (`secp256k1`/`rand`/`ethereum-types`/`parity-util-mem` chain); updated Modular Coding Rules to include `parity-util-mem-compat` in the required `[patch.crates-io]` workspace-shim list; corrected CI-equivalent Docker build command to use `.github/docker/ubuntu-rust-1.97.1/Dockerfile.ci`
- v2.4 (2026-07-14): Fixed rpassword vulnerability (GHSA-2p6r-x3vv-xqm2): upgraded `rpassword` from `1.0.2` to `7.5.0` (resolved to `7.5.4`); API change `prompt_password_stdout()` → `prompt_password()` in `cli-signer/src/lib.rs`; corrected version header from 2.2 → 2.3 (changelog was ahead of header); updated Technology Stack to reference `jsonrpc-core` v18 (not v15); updated External Resources link to v18 docs; added rpassword to Security checklist; 0 errors
- v2.3 (2026-07-14): Phase 3 complete — migrated `jsonrpc-*` from v15 to v18; all RPC code migrated from futures 0.1 to futures 0.3 + async/await; `parity-rpc` edition updated to 2021; removed `tokio 0.1.22`, `hyper 0.12.36`, `h2 0.1.26` (CVE-2023-44487), `crossbeam-utils 0.7.2`, `time 0.1.45` (RUSTSEC-2020-0071), `net2 0.2.39`, `parity-tokio-ipc 0.4`, `parity-ws 0.10.1`, `futures-cpupool` from Cargo.lock; `lock-api-compat` shim no longer needed for jsonrpc chain (still needed for kvdb-memorydb); `cli-signer` migrated to futures 0.3; `ethcore-stratum` updated for v18 API; 0 errors, 0 test regressions
- v2.2 (2026-07-14): Removed unmaintained `wee_alloc 0.4.5` from `parity-util-mem-compat`: deleted optional dep, removed `weealloc-global` feature, stripped dead cfg-branch from `allocators.rs` and `lib.rs`; `wee_alloc` fully absent from Cargo.lock; 0 warnings 0 errors
- v2.1 (2026-07-14): Fixed lru RUSTSEC vulnerabilities (Dependabot #12/#18): created `crates/util/parity-util-mem-compat` local fork of `parity-util-mem 0.7.0` with `lru` upgraded from `0.5.3` to `0.7.8`; the `LruCache<K,V,S>` API used (`.iter()`, `.len()`) is identical in both versions so no source changes were required; registered via `[patch.crates-io]` and added to `[workspace] members`; `lru 0.5.3` fully removed from Cargo.lock; 0 warnings 0 errors; updated CVE table, Key Components, project structure tree, Modular Coding Rules, and Security checklist; updated MAINTENANCE.md § parity-util-mem to mark both Dependabot alerts as FIXED
- v2.0 (2026-07-13): Removed CodeQL entirely from both CI workflows (unstable autobuild, non-deterministic results); deleted `.github/codeql/codeql-config.yml` and `.github/codeql/` directory; removed `security-events: write` permission from both workflow files; restored `Test Execution` to its original position (before Release Build) in `docker-ubuntu-latest.yml`; cleaned up all CodeQL references in AGENTS.md; fixed flaky test `should_not_return_pending_external_transactions_with_too_low_priority_fee_if_priority_fees_are_enforced` by replacing `new_queue()` (max_mem_usage=100, enough for 3 txs only) with an inline queue using `max_mem_usage: usize::MAX` to prevent allocator-dependent eviction of tx2 on Linux CI
- v1.9 (2026-07-13): Replaced `lru-cache = "0.1"` with `lru = "0.7.8"` across all 4 dependent crates (`memory-cache`, `ethcore`, `network-devp2p`, `node-filter`); migrated all call sites: `.insert()→.put()`, `.remove()→.pop()`, `.remove_lru()→.pop_lru()`, `.capacity()→.cap()`, `.set_capacity()→.resize()`; rewrote `clone_all()` in `state/account.rs` to manually copy LruCache entries since lru 0.7.x does not implement Clone; updated CVE table, Dep Management bullet, Phase sequence
- v1.8 (2026-07-13): Fixed lock_api CVEs (CVE-2020-35910..35914): created `crates/util/lock-api-compat` shim (fork of lock_api 0.3.4 with backported Send/Sync bounds from 0.4.2); registered via `[patch.crates-io]`; fixes transitive chain kvdb-memorydb→parking_lot 0.9.0 and jsonrpc-*→parking_lot 0.10.2; added `.github/dependabot.yml` to prevent Dependabot from breaking the `parity-crypto`/yanked-aes dependency chain; updated CVE table, Key Components, project structure tree, Modular Coding Rules, and Security checklist
- v1.7 (2026-07-13): Fixed atty CVE (RUSTSEC-2021-0017): `crates/util/atty-compat` shim (backed by `std::io::IsTerminal`) already registered via `[patch.crates-io]` — AGENTS.md was still showing it as pending Phase 2; updated CVE table, Dep Management atty bullet, Phase 2 sequence, Key Components (CVE patch shims note), project structure tree (added `atty-compat/` and `tempdir-compat/` entries), and Modular Coding Rules (`[patch.crates-io]` shims require workspace member entry)
- v1.6 (2026-07-13): Fixed remove_dir_all CVE (RUSTSEC-2021-0126): created `crates/util/tempdir-compat` local compat shim (tempdir 0.3.7 API backed by tempfile 3.27.0); registered via `[patch.crates-io]` in root Cargo.toml; removes tempdir 0.3.7 and remove_dir_all 0.5.3 entirely from Cargo.lock; added workspace member entry; all 4 shim unit tests pass; updated MAINTENANCE.md § Vulnerable Dependencies; updated AGENTS.md CVE table
- v1.5 (2026-07-13): Removed references to non-existent `UPDATE_PLAN.md`; fixed version header (1.3→1.4); added `.testing/README.md` reference; inlined Phase 2–4 upgrade sequence
- v1.4 (2026-07-10): Fixed 44 Rust 1.97 compiler warnings: mismatched_lifetime_syntaxes (added explicit `'_` to 38 return types across 23 crates/files), unused_parens (5 sites in vm/access_list.rs and db/db.rs), dead_code (is_global_s annotated with #[allow(dead_code)] in network-devp2p/ip_utils.rs, useless self-assignment and unused mut removed in rpc/transaction.rs)
- v1.3 (2026-07-10): Corrected version to 3.5.1; fixed Rust upgrade note (1.88→1.97); added release Docker workflow; documented macOS EMFILE/rayon pin; expanded Known Vulnerable Dependencies table with lru-cache, tempdir, remove_dir_all, term_size; added rayon and number_prefix pin warnings
- v1.2 (2026-07-10): Upgraded Rust toolchain from 1.97 to 1.97; added setup-rust-1.97.sh, .github/docker/ubuntu-rust-1.97.1/Dockerfile, and .github/workflows/docker-ubuntu-latest.yml
- v1.1 (2026-07-10): Added missing scripts, UPDATE_PLAN.md references, and Phase 1 completion status
