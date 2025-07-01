# 1. Setup Development Environment

## 1.. Ubuntu 24.04 LTS

This fork of OpenEthereum project is developed and tested under Ubuntu.

### Check OS version (ubuntu)

```bash
lsb_release -a
```

```result
Distributor ID: Ubuntu
Description:    Ubuntu 24.04.2 LTS
Release:        24.04
Codename:       noble
```

### Install Tools

```bash
curl https://sh.rustup.rs -sSf | sh
export PATH=$PATH:$HOME/.cargo/bin
```

```bash
sudo apt update
sudo apt upgrade
sudo apt install yasm
sudo apt install tree
sudo apt install gcc-12 g++-12
````

## Select Rust Version

```bash
./scripts/setup-rust-1.79.sh
```



## 1.2. MacOS

The current state will not work under macOS and Windows is not tested.

### Install GCC-12 and G++-12

```bash
 brew install gcc
```

### Install CMake 3.28.3

```bash
curl --silent --location --retry 3 "https://gith`b.com/Kitware/CMake/releases/download/v3.28.3/cmake-3.28.3-macos-universal.dmg" --output cmake-macos.dmg
```

Then open dmg file and manually install.

Verify:

```bash
cmake --version
```

```result
cmake version 3.28.3

CMake suite maintained and supported by Kitware (kitware.com/cmake).
```
