# Setup Development Environment [Ubuntu 24.04 LTS]

This fork of OpenEthereum project is developed and tested under Ubuntu. 
The current state will not work under macOS and Windows is not tested.

## Check OS version

```shell
lsb_release -a
```

Expected:
```
No LSB modules are available.
Distributor ID: Ubuntu
Description:    Ubuntu 24.04.2 LTS
Release:        24.04
Codename:       noble
```


## Install Tools

```shell
curl https://sh.rustup.rs -sSf | sh
export PATH=$PATH:$HOME/.cargo/bin
```

```shell
sudo apt update
```

```shell
sudo apt upgrade
```

```shell
sudo apt install yasm
```

```shell
sudo apt install tree
````

## Select Rust Version

```shell
./scripts/setup-rust-1.79.sh
```

## Install GCC-12 and G++-12 and set environment

```shell
sudo apt install gcc-12 g++-12
export CC=/usr/bin/gcc-12
export CXX=/usr/bin/g++-12
```