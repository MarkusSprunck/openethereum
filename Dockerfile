FROM cimg/rust:1.66.1-node@sha256:9ee2da9180d87363cbcaa65f7a2ae441fbca5e882526b48a492fcfb126dbb273 AS builder

WORKDIR /build

COPY . /build

RUN cargo build --color=always --release --features final

RUN strip target/release/openethereum

FROM --platform=linux/amd64 ubuntu:24.10@sha256:25895062172a2f39ae36da530f3db244b507d7ffb1c4dd42a3a487b5b446e996

RUN apt-get update && apt-get install libc6

EXPOSE 8545

WORKDIR /home/openethereum

RUN mkdir -p /home/openethereum/.local/share/io.parity.ethereum/

COPY --chown=openethereum:openethereum --from=builder ./build/target/release/openethereum /home/openethereum/openethereum
COPY                                   --from=builder ./build/.github/workflows/README.md /home/openethereum/README.md

ENTRYPOINT ["/home/openethereum/openethereum"]
