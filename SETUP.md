## Build Docker

### Create OpenEthereum

Use cimg/rust:1.66.1-node build image in IntelliJ

```shell
docker pull cimg/rust:1.61-node
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
rustup toolchain add 1.66.1 --profile minimal
```

```shell
rustup install 1.66.1
```

```shell
rustup update
```

```shell
rustup override set 1.66.1
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
