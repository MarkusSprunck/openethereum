ARG RUST_VERSION="1.79"

# ------------------------------------------------------------------------------
# Build Stage
# ------------------------------------------------------------------------------

FROM cimg/rust:${RUST_VERSION} AS builder

RUN echo "nothing to do here"

WORKDIR /build

COPY . /build

RUN cargo build --release --features final

RUN strip target/release/openethereum


# ------------------------------------------------------------------------------
# Runtime Stage
# ------------------------------------------------------------------------------
FROM --platform=linux/amd64 docker.io/library/ubuntu:24.10

RUN apt-get -y update; apt-get -y install curl jq

RUN groupadd -g 1000 openethereum && useradd -m -u 1000 -g openethereum -s /bin/sh openethereum

USER openethereum

EXPOSE 8545

WORKDIR /home/openethereum

RUN mkdir -p /home/openethereum/.local/share/io.parity.ethereum/

COPY --chown=openethereum:openethereum --from=builder /build/target/release/openethereum /home/openethereum/openethereum

ENTRYPOINT ["/home/openethereum/openethereum"]
