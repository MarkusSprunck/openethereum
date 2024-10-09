# Setup Tools [Ubuntu 20.04.6 LTS] (Focal Fossa)

## Install Tools (once)

```shell
curl https://sh.rustup.rs -sSf | sh
export PATH=$PATH:$HOME/.cargo/bin
sudo apt update
sudo apt upgrade
sudo apt install yasm
```

## Select Rust Version

```shell
rustup toolchain add 1.70 --profile minimal
rustup install 1.70
rustup override set 1.70
```

## Create Docker Image

```shell
./docker-build.sh
```

## Build

```shell
./scripts/actions/clean-target.sh
```

Build for development, creates a debug build and does not optimize, such that the build is faster.

```shell
./scripts/actions/build-dev.sh
```

```shell
./scripts/actions/build-linux.sh
```
