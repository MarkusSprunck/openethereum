# Setup Tools [Ubuntu 20.04.6 LTS] (Focal Fossa)

## Install Tools (once)

```shell
curl https://sh.rustup.rs -sSf | sh
export PATH=$PATH:$HOME/.cargo/bin
sudo apt update
sudo apt upgrade
sudo apt install yasm
````

## Select Rust Version

```shell
./scripts/setup-rust-1.79.sh
```

## Build Artifacts

Build all artifacts for testing.

```shell
./scripts/build-artifacts.sh
```

## Setup for Leopold (Staging) Tests

### Two Secret Files will be needed

Change your secrets to get a unique identity.

```shell
cd .testing/environment/staging/secrets
echo "123" > AccountMnemonic
echo "456" > NetworkMnemonic
```

### Create secrets based on mnemonics

```shell
cd .testing
./secrets_generation.sh
./setup_folders.sh
```

### Start Leopold Node

```shell
./scripts/test-leopold.sh
```
