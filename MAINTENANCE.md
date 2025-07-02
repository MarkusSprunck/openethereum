# Setup Development Environment

## 1.0 Install Visual Studio Code Plugins

## 1.1 Mandatory

**CodeLLDB**: A native debugger powered by LLDB. Debug C++, Rust and other compiled languages.

**rust**: Extensions for rust

**rust-analyzer**: Rust language support for Visual Studio Code

**Rust Syntax**: Improved Rust syntax highlighting

## 1.2 Optional

**Dependi**: Empowers developers to efficiently manage dependencies and address vulnerabilities in Rust, etc

**GitHub Copilot**: Your AI pair programmer

**Even Better TOML**: Fully-featured TOML support

**GitHub Actions**: GitHub Actions workflows and runs for github.com hosted repositories in VS Code

**Error Lens**: Improve highlighting of errors, warnings and other language

## 2.0 Setup Development Machine (Ubuntu 24.04 LTS)

This fork of OpenEthereum project is developed and tested under Ubuntu.

### 2.1 Check OS version

```bash
lsb_release -a
```

```result
Distributor ID: Ubuntu
Description:    Ubuntu 24.04.2 LTS
Release:        24.04
Codename:       noble
```

### 2.2 Install Tools

First ensure that **cargo** has been installed.

```bash
curl https://sh.rustup.rs -sSf | sh
export PATH=$PATH:$HOME/.cargo/bin
```

Then install build tools.

```bash
sudo apt update
sudo apt upgrade
sudo apt install yasm
sudo apt install gcc-12 g++-12
```

### 2.3 Select Rust Version

```bash
./scripts/setup-rust-1.79.sh
```

## 3.0 Setup Development Machine (MacOS)

### 3.1 Check OS version

```bash
sw_vers
```

```result
ProductName:      macOS
ProductVersion:   15.5
BuildVersion:     24F74
```

The current state will not work under macOS and Windows is not tested.

### 3.2 Install GCC-12 and G++-12

```bash
 brew install gcc
```

### 3.3 Install CMake 3.28.3

```bash
curl --silent --location --retry 3 "https://gith`b.com/Kitware/CMake/releases/download/v3.28.3/cmake-3.28.3-macos-universal.dmg" --output cmake-macos.dmg
```

Then open cmake-macos.dmg file and manually install and verify version number.

```bash
cmake --version
```

Expected result

```text
cmake version 3.28.3
```

## 4.0 Add Test Dependencies to Project

For full testing support some addional stuff will be needed.

```bash
git submodule update --init --recursive
```

## 5.0 Add Test Support for Leopold Blockchain

The openethereum client is maintained for the Cert4Trust Leopold blockchain, so testing and debugging will mainly happen with this blockchain.

Find in the subfolder **.testing** a README.md file how to setup Leopold configuration for testing.
