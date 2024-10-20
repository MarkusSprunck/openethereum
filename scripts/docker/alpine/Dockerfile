ARG RUST_VERSION="1.79"

# ------------------------------------------------------------------------------
# Cargo Build Stage
# ------------------------------------------------------------------------------
FROM rust:${RUST_VERSION}-slim AS BUILDER

RUN apt update -qq && apt install -y -qq --no-install-recommends musl-tools build-essential libc6-dev cmake musl-dev

RUN rustup set profile minimal && rustup target add x86_64-unknown-linux-musl

RUN apt install -y -qq llvm-dev libclang-dev clang

WORKDIR /build

COPY . /build

ENV RUST_BACKTRACE=1

ENV RUSTFLAGS="-Ctarget-feature=-crt-static -Clink-args=-lm"

RUN ln -s /bin/g++ /bin/musl-g++

RUN cargo build --release --target x86_64-unknown-linux-musl

RUN strip target/x86_64-unknown-linux-musl/release/openethereum

# ------------------------------------------------------------------------------
# Runtime Stage
# ------------------------------------------------------------------------------
FROM alpine:3.20.3@sha256:beefdbd8a1da6d2915566fde36db9db0b524eb737fc57cd1367effd16dc0d06d

# show backtraces
ENV RUST_BACKTRACE 1

# curl and jq are installed to help create health and readiness checks on Kubernetes
# RUN apk add --no-cache libstdc++ eudev-libs libgcc curl jq

RUN addgroup -g 1000 openethereum

RUN adduser -u 1000 -G openethereum -s /bin/sh -D openethereum

USER openethereum

EXPOSE 8545

WORKDIR /home/openethereum

COPY --chown=openethereum:openethereum --from=builder /build/target/x86_64-unknown-linux-musl/release/openethereum ./

ENTRYPOINT ["/home/openethereum/openethereum"]
