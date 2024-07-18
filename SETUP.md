# Setup Tools [macOS]

## Install Tools (once)

```shell
brew install rustup
brew install perl
brew install yasm
xcode-select --install
```

## Select Rust Version

```shell
rustup toolchain add 1.63 --profile minimal
rustup install 1.63
rustup override set 1.63
```

# Create local Docker Image

```shell
./docker-build.sh
```
