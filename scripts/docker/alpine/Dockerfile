ARG RUST_VERSION="1.79"

# ------------------------------------------------------------------------------
# Build Stage
# ------------------------------------------------------------------------------
FROM rust:${RUST_VERSION}-slim AS BUILDER

RUN apt update -qq && apt install -y -qq --no-install-recommends build-essential libc6-dev cmake

RUN rustup set profile minimal && rustup target add x86_64-unknown-linux-gnu

RUN apt install -y -qq llvm-dev libclang-dev clang

WORKDIR /build

COPY . /build

ENV RUST_BACKTRACE=1

ENV RUSTFLAGS="-Ctarget-feature=+crt-static -Clink-args=-lm -Clink-args=-lpthread"

RUN cargo build --release --features final --target x86_64-unknown-linux-gnu

RUN strip target/x86_64-unknown-linux-gnu/release/openethereum

# ------------------------------------------------------------------------------
# Runtime Stage
# ------------------------------------------------------------------------------
FROM alpine:3.20

# show backtraces
ENV RUST_BACKTRACE 1

# curl and jq are installed to help create health and readiness checks on Kubernetes
RUN apk add --no-cache libstdc++ eudev-libs libgcc curl jq

RUN addgroup -g 1000 openethereum && adduser -u 1000 -G openethereum -s /bin/sh -D openethereum

USER openethereum

EXPOSE 8545

WORKDIR /home/openethereum

RUN mkdir -p /home/openethereum/.local/share/io.parity.ethereum/

COPY --chown=openethereum:openethereum --from=builder /build/target/x86_64-unknown-linux-gnu/release/openethereum ./

ENTRYPOINT ["/home/openethereum/openethereum"]
