
```shell
brew uninstall rust
```

```shell
brew install rustup
```

```shell
rustup toolchain add 1.59 --profile minimal
```

```shell
rustup install 1.59
```

```shell
rustup update
```

```shell
rustup override set 1.59
```

```shell
xcode-select --install
```

```shell
brew install perl
```

```shell
brew install yasm
```

## Build

### Create OpenEthereum

Use cimg/rust:1.59-node build image in IntelliJ

```shell
docker pull cimg/rust:1.59-node
```

### Create docker image

This creates an Ubuntu 24.10 image

```shell
cp target/release/openethereum scripts/docker/ubuntu-latest/openethereum
cd scripts/docker/ubuntu-latest
./build.sh
rm -f openethereum
```
