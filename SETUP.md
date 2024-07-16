
```shell
brew uninstall rust
```

```shell
brew install rustup
```

```shell
rustup toolchain add 1.61 --profile minimal
```

```shell
rustup install 1.61
```

```shell
rustup update
```

```shell
rustup override set 1.61
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

Use cimg/rust:1.61-node build image in IntelliJ

```shell
docker pull cimg/rust:1.61-node
```

### Create docker image

This creates an Ubuntu 24.10 image

```shell
cp target/release/openethereum scripts/docker/ubuntu-24.10/openethereum
cd scripts/docker/ubuntu-24.10
./build.sh
docker push ihkmuenchen/openethereum:3.3.6-rc0
rm -f openethereum
```
