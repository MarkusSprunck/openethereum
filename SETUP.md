# Setup Tools [macOS]

## Install Tools (once)

```shell
brew install rustup
brew install perl
brew install yasm
xcode-select --install
```

# Setup Tools [Ubuntu 24.04]

## Install Tools (once)

```shell
sudo apt install rustup
sudo apt install perl
sudo apt install yasm
sudo apt install cmake
```

## Select Rust Version

```shell
rustup toolchain add 1.63 --profile minimal
rustup install 1.63
rustup override set 1.63
```

# Create Docker Image

```shell
./docker-build.sh
```

# Build

```shell
./scripts/actions/clean-target.sh 
./scripts/actions/build-linux.sh
```
