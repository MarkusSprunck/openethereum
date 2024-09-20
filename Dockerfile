FROM cimg/rust:1.63-node@sha256:a091d3aba7a6d919f3e67d128039698851a2fb583441a7b6dfc7be4921aa8cdc AS builder

WORKDIR /build

COPY . /build

RUN cargo test --color=always --release --features final

RUN cargo build --color=always --release --features final

RUN strip target/release/openethereum

FROM --platform=linux/amd64 docker.io/library/ubuntu:24.04

RUN groupadd -g 10000 openethereum && useradd -m -u 10000 -g openethereum -s /bin/sh openethereum

USER openethereum

EXPOSE 8545

WORKDIR /home/openethereum

RUN mkdir -p /home/openethereum/.local/share/io.parity.ethereum/

COPY --chown=openethereum:openethereum --from=builder ./build/target/release/openethereum /home/openethereum/openethereum

ENTRYPOINT ["/home/openethereum/openethereum"]
