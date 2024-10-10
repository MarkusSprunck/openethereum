FROM cimg/rust:1.79-node AS builder

WORKDIR /build

COPY . /build

RUN cargo test --color=always --release --features final

RUN cargo build --color=always --release --features final

RUN strip target/release/openethereum

FROM --platform=linux/amd64 docker.io/library/ubuntu:24.10

RUN apt-get -y update; apt-get -y install curl

RUN groupadd -g 10000 openethereum && useradd -m -u 10000 -g openethereum -s /bin/sh openethereum

USER openethereum

EXPOSE 8545

WORKDIR /home/openethereum

RUN mkdir -p /home/openethereum/.local/share/io.parity.ethereum/

COPY --chown=openethereum:openethereum --from=builder ./build/target/release/openethereum /home/openethereum/openethereum

ENTRYPOINT ["/home/openethereum/openethereum"]
