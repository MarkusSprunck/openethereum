ARG RUST_VERSION="1.97.1"

# ------------------------------------------------------------------------------
# Build Stage - builds with testing
# ------------------------------------------------------------------------------

FROM cimg/rust:${RUST_VERSION} AS builder

WORKDIR /build

COPY . /build

ENV RUST_BACKTRACE=1

RUN cargo test --all

RUN cargo build --release --features final

RUN strip target/release/openethereum

# ------------------------------------------------------------------------------
# Runtime Stage
# ------------------------------------------------------------------------------

FROM --platform=$TARGETPLATFORM docker.io/library/ubuntu:25.10

RUN apt-get -y update && apt-get -y install curl jq && apt-get clean

ENV RUST_BACKTRACE=1

RUN groupadd -g 5093 openethereum && useradd -m -u 5093 -g openethereum -s /bin/sh openethereum

USER openethereum

#      rpc  ws   listener  discovery
#      ↓    ↓    ↓         ↓
EXPOSE 8545 8546 30303/tcp 30303/udp

WORKDIR /home/openethereum

COPY --chown=openethereum:openethereum --from=builder /build/target/release/openethereum ./

ENTRYPOINT ["/home/openethereum/openethereum"]

