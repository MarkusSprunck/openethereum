ARG RUST_VERSION="1.79"

# ------------------------------------------------------------------------------
# Cargo Build Stage
# ------------------------------------------------------------------------------
FROM rust:${RUST_VERSION}-slim AS BUILDER

RUN apt update -qq && apt install -y -qq --no-install-recommends musl-tools build-essential libc6-dev cmake musl-dev

RUN rustup set profile minimal && rustup target add x86_64-alpine-linux-musl

RUN apt install -y -qq llvm-dev libclang-dev clang

WORKDIR /build

COPY . /build

ENV RUST_BACKTRACE=1

ENV RUSTFLAGS="-Ctarget-feature=-crt-static -Clink-args=-lm"

RUN ln -s /bin/g++ /bin/musl-g++

RUN cargo build --release --target x86_64-alpine-linux-musl
RUN strip target/x86_64-alpine-linux-musl/release/openethereum

FROM alpine:edge

# show backtraces
ENV RUST_BACKTRACE 1

# curl and jq are installed to help create health and readiness checks on Kubernetes
RUN apk add --no-cache libstdc++ eudev-libs libgcc curl jq

RUN addgroup -g 1000 openethereum \
  && adduser -u 1000 -G openethereum -s /bin/sh -D openethereum

USER openethereum

EXPOSE 8080 8545 8180

WORKDIR /home/openethereum

RUN mkdir -p /home/openethereum/.local/share/io.parity.ethereum/
COPY --chown=openethereum:openethereum --from=builder /openethereum/target/x86_64-alpine-linux-musl/release/openethereum ./

ENTRYPOINT ["/home/openethereum/openethereum"]
