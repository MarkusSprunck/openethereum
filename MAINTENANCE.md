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
brew install gcc@12 lz4 zstd snappy rocksdb
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
