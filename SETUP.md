## Build Docker

### Create OpenEthereum

Use cimg/rust:1.69.0-node build image in IntelliJ

```shell
docker pull cimg/rust:1.69.0-node
```

### Create docker image

This creates an Ubuntu 24.10 image

```shell
./build.sh
```

## Setup Tools (macOS)


```shell
brew uninstall rust
```

```shell
brew install rustup
```

```shell
rustup toolchain add 1.62.1 --profile minimal
rustup install 1.62.1
rustup override set 1.62.1
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
