FROM rust:slim AS builder

RUN apt-get update 
RUN apt-get install -y musl-tools cmake build-essential libc6-dev
RUN rustup toolchain add 1.79 --profile minimal
RUN rustup install 1.79
RUN rustup override set 1.79
RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /build

COPY . /build

ENV RUST_BACKTRACE=1

ENV CXX=/usr/bin/g++

# RUN cargo test --all --features final

RUN RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --features final --target x86_64-unknown-linux-musl

RUN strip target/x86_64-unknown-linux-musl/release/openethereum

FROM scratch

# RUN apk add --no-cache libstdc++ eudev-libs libgcc curl jq

# RUN addgroup -g 10000 openethereum && adduser -u 10000 -G openethereum -s /bin/sh -D openethereum

USER openethereum

EXPOSE 8545

# WORKDIR /home/openethereum

# RUN mkdir -p /home/openethereum/.local/share/io.parity.ethereum/

COPY --chown=openethereum:openethereum --from=builder /openethereum/target/x86_64-unknown-linux-musl/release/openethereum /home/openethereum/openethereum

ENTRYPOINT ["/home/openethereum/openethereum"]
