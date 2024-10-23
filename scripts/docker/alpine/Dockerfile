ARG RUST_VERSION="1.79"
ARG RUST_FEATURES=""

# ------------------------------------------------------------------------------
# Build Stage - builds without testing
# ------------------------------------------------------------------------------

FROM rust:${RUST_VERSION}-slim AS builder

RUN apt update -qq && apt install -y -qq --no-install-recommends build-essential libc6-dev cmake

RUN rustup set profile minimal && rustup target add x86_64-unknown-linux-gnu

RUN apt install -y -qq llvm-dev libclang-dev clang

WORKDIR /build

COPY . /build

ENV RUST_BACKTRACE=1

ENV RUSTFLAGS="-Ctarget-feature=+crt-static"

RUN cargo build --release ${RUST_FEATURES} --target x86_64-unknown-linux-gnu

# ------------------------------------------------------------------------------
# Runtime Stage
# ------------------------------------------------------------------------------

FROM alpine:3.20

RUN apk add --no-cache curl jq

ENV RUST_BACKTRACE=1

RUN addgroup -g 5093 openethereum && adduser -u 5093 -G openethereum -s /bin/sh -D openethereum

USER openethereum

#      rpc  ws   listener  discovery
#      ↓    ↓    ↓         ↓
EXPOSE 8545 8546 30303/tcp 30303/udp

WORKDIR /home/openethereum

COPY --chown=openethereum:openethereum --from=builder /build/target/x86_64-unknown-linux-gnu/release/openethereum ./

ENTRYPOINT ["/home/openethereum/openethereum"]
