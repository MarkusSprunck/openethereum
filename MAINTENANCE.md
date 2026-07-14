# OpenEthereum – Maintenance & Development Setup

## 1.0 Visual Studio Code Plugins

### 1.1 Required Plugins

- **CodeLLDB**: Native debugger for C++, Rust, and other compiled languages.
- **rust-analyzer**: Modern Rust language support for VS Code.
- **Rust Syntax**: Improved syntax highlighting for Rust.
- **Rust**: Additional Rust tools and integration.

### 1.2 Optional Plugins

- **Dependi**: Dependency management and security checks.
- **GitHub Copilot**: AI-powered programming assistant.
- **Even Better TOML**: TOML syntax support.
- **GitHub Actions**: Integration and management of GitHub Actions workflows.
- **Error Lens**: Enhanced display of errors and warnings in the editor.

---

## 2.0 Setting Up the Development Environment (Ubuntu 24.04 LTS)

> **Note:** Development and testing are performed on Ubuntu. macOS and Windows are not officially supported.

### 2.1 Installing Tools

1. **Install Rust & Cargo:**

    ```bash
    curl https://sh.rustup.rs -sSf | sh
    export PATH=$PATH:$HOME/.cargo/bin
    ```

2. **Install build tools:**

    ```bash
    sudo apt update
    sudo apt upgrade
    sudo apt install yasm gcc-12 g++-12
    ```

### 2.2 Select Rust Version

```bash
./scripts/setup-rust-1.97.sh
```

---

## 3.0 Setting Up the Development Environment (macOS)

> **Warning:** macOS is currently not fully supported.

### 3.1 Install Compiler and Libraries

```bash
brew install lz4 zstd snappy rocksdb
```

### 3.2 Select Rust Version

```bash
./scripts/setup-rust-1.97.sh
```

### 3.3 Install CMake (Version 3.28.3)

See: [CMake on macOS](https://gist.github.com/fscm/29fd23093221cf4d96ccfaac5a1a5c90)

---

## 4.0 Add Test Dependencies

For full test support:

```bash
git submodule update --init --recursive
```

---

## 5.0 Test Support for the Leopold Blockchain

OpenEthereum is maintained for the Cert4Trust Leopold Blockchain. Testing and debugging are primarily performed with this blockchain.

Further instructions for test configuration can be found in [OpenEthereum Test Client for Leopold Blockchain](.testing/README.md).

---

## 6.0 Security Vulnerabilities

During the maintenance update in July 2025 (v3.5.0), all vulnerabilities stemming from direct project dependencies (that have a fixed version available) or from transitive dependencies without complex cross-dependencies were fixed through dependency updates.

Resolving all vulnerabilities caused by transitive dependencies will likely require a larger codebase rework.

### 6.1 Fixed Vulnerabilities

#### jsonrpc-\*

The `jsonrpc` dependencies have been **upgraded from version `15.x.x` to `18.0.0` (2026-07-14, Phase 3 complete)**. This migration removed the entire vulnerable dependency chain:

```
jsonrpc-* v15 → v18  ✅ DONE
└───hyper v0.12.36 → 0.14.32  ✅ REMOVED
│   └───tokio v0.1.22 → gone   ✅ REMOVED (CVE chain)
│   └───h2 v0.1.26 → 0.3.27   ✅ REMOVED (CVE-2023-44487)
│   └───crossbeam-utils v0.7.2 → 0.8.22  ✅ REMOVED
│   └───time v0.1.45 → 0.3.53  ✅ REMOVED (RUSTSEC-2020-0071)
└───lock_api v0.3.4 (now only from kvdb-memorydb, still shim'd)
```

All RPC code was migrated from futures 0.1 to futures 0.3 + async/await as part of this upgrade.

#### parity-util-mem (lru + wee_alloc)

`parity-util-mem 0.7.0` is used at version 0.7.0 (a full upgrade to 0.11.0 is a Phase 4 blocker). Two transitive vulnerabilities were fixed via a local compatibility shim (`crates/util/parity-util-mem-compat`):

- `lru 0.5.3` → `0.7.8`: upgraded inside the shim (Dependabot #12 / #18) — **FIXED (2026-07-14)**
- `wee_alloc 0.4.5`: removed entirely (the `weealloc-global` feature was never activated) — **FIXED (2026-07-14)**

The shim is registered via `[patch.crates-io]` and is a workspace member.

-   https://github.com/MarkusSprunck/openethereum/security/dependabot/12 — **FIXED (2026-07-14)**
-   https://github.com/MarkusSprunck/openethereum/security/dependabot/18 — **FIXED (2026-07-14)**

#### atty (RUSTSEC-2021-0017)

Replaced by the `crates/util/atty-compat` shim backed by `std::io::IsTerminal` — **FIXED (2026-07-13)**

#### tempdir / remove_dir_all (RUSTSEC-2021-0126)

Replaced by `crates/util/tempdir-compat` (tempdir 0.3.7 API backed by tempfile 3.27.0) — **FIXED (2026-07-13)**

#### lock_api (CVE-2020-35910..35914)

Replaced by `crates/util/lock-api-compat` shim (backported Send/Sync bounds from 0.4.2) — **FIXED (2026-07-13)**

#### rpassword (GHSA-2p6r-x3vv-xqm2)

`rpassword` was upgraded from `1.0.2` to `7.5.0` (resolved to `7.5.4`). The partial-password-reveal vulnerability on interrupted input is fixed. API change: `prompt_password_stdout()` → `prompt_password()` in `cli-signer/src/lib.rs` — **FIXED (2026-07-14)**

---

### 6.2 Vulnerable Dependencies — Phase 4 Blockers

The following vulnerabilities cannot be fixed without a coordinated Phase 4 upgrade of `parity-crypto` → `ethereum-types` → `parity-util-mem`. Both are confirmed **not exploitable** in this codebase.

#### secp256k1 (GHSA-969w-q74q-9j8v, MEDIUM)

- **Current:** `0.17.2` (transitive via `parity-crypto 0.6.2`)
- **Fix requires:** `≥ 0.22.2`
- **Blocker:** `parity-crypto 0.6.2` → latest `0.9.0` still depends on `secp256k1 0.20`; Phase 4
- **Exploitability:** ✅ **Not exploitable** — the vulnerable method `Secp256k1::preallocated_gen_new` is never called anywhere in the codebase (confirmed 2026-07-14 via full grep)

#### rand (GHSA-cq8v-f236-94qc, LOW)

- **Current:** `0.7.3` (direct dependency in 6 crates)
- **Fix requires:** `≥ 0.10.1`
- **Blocker:** `ethereum-types 0.9.2` implements `Standard: Distribution<H256>` (rand 0.7 API); rand 0.9 renamed `Standard` → `StandardUniform`, breaking the impl. Upgrading `ethereum-types` is Phase 4.
- **Exploitability:** ✅ **Not exploitable** — two independent conditions both fail:
  1. rand's `log` feature is **not enabled** (not in `default = ["std"]`, never explicitly activated in any `Cargo.toml`)
  2. The project logger (`bin/oe/logger/`) does **not call** `rand::thread_rng()` — zero grep matches in all logger source files (confirmed 2026-07-14)
  Both conditions must be true simultaneously for the CVE to trigger.
