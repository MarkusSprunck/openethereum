# Setup Development Environment [Ubuntu 20.04.6 LTS]

This fork of OpenEthereum project is developed and tested under Ubuntu. 
The current state will not work under macOS and Windows is not tested.

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
