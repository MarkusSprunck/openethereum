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
./scripts/setup-rust-1.88.sh
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
./scripts/setup-rust-1.88.sh
```

### 3.3 Install CMake (Version 3.28.3)

See: [CMake on MacOS](https://gist.github.com/fscm/29fd23093221cf4d96ccfaac5a1a5c90)

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

## 6.0 Security Vulnerabilities

During the maintenance update in July 2025 (v3.5.0), all vulnerabilities stemming from direct project dependencies (that have a fixed version available) or from transitive dependencies without complex cross-dependencies were fixed through dependency updates.

Resolving all vulnerabilities caused by transitive dependencies will likely require a larger codebase rework:

### jsonrpc-\*

The `jsonrpc` dependencies are currently used at version `15.x.x` and require an upgrade to `18.0.0`. This upgrade will in turn update several outdated, vulnerable dependencies (simplified view):

```
jsonrpc-* v15 -> v18
└───hyper v0.12.36 -> 0.14.12
│   └───tokio v0.1.22 -> 1.8.4
│   └───crossbeam-utils v0.7.2 -> 0.8.7
│   └───h2 v0.1.26 -> 0.3.26
│   └───memoffset v0.5.6 -> 0.6.2
│   └───time v0.1.45 -> 0.2.23
└───lock_api v0.3.4 -> 0.4.2
```

### parity-util-mem

`parity-util-mem` is a direct dependency of several crates. It is currently used at version `0.7.0` and would need to be upgraded to `0.11.0`. As it is pre-1.0, this will likely introduce breaking changes. This will also require upgrading `ethereum-types` (`0.9.2` -> `0.13`), causing additional code changes and version conflicts that need to be resolved.

This upgrade will fix the `lru` vulnerabilities:

-   https://github.com/MarkusSprunck/openethereum/security/dependabot/12
-   https://github.com/MarkusSprunck/openethereum/security/dependabot/18

### Vulnerable Dependencies Without a Fixed Version

For some vulnerabilities, even after upgrading the depencies there is no compatible version that fixes the vulnerability. Since they are moderate and low severity, it is to be decided if a fix/workaround outweighs the efforts for each vulnerability:

-   [secp256k1](https://github.com/MarkusSprunck/openethereum/security/dependabot/23): Used at `0.17.2`, would need `0.22.2`. We use `parity-crypto v0.6.2`, which in its most recent version `v0.9.0` still depends on `secp256k1 v0.20`
-   [remove_dir_all](https://github.com/MarkusSprunck/openethereum/security/dependabot/24): Used at `0.5.3`, would need `0.8.0`. We use `tempdir` in its most recent version `v0.3.7`, which depends on `remove_dir_all v0.5`
-   [atty](https://github.com/MarkusSprunck/openethereum/security/dependabot/29): Used as a direct dependency at the most recent version `0.2.14`. Should be easy to replace as it is only used in a couple lines of code but the vulnerability seems to be only relevant for Windows
