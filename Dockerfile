FROM cimg/rust:1.63-node@sha256:a091d3aba7a6d919f3e67d128039698851a2fb583441a7b6dfc7be4921aa8cdc AS builder

WORKDIR /build

COPY . /build

# RUN cargo test --locked --all --release --features "json-tests"

RUN cargo build --color=always --release --features final

RUN strip target/release/openethereum

FROM --platform=linux/amd64 ubuntu:24.10@sha256:25895062172a2f39ae36da530f3db244b507d7ffb1c4dd42a3a487b5b446e996

RUN apt-get update && apt-get install libc6

EXPOSE 8545

WORKDIR /home/openethereum

RUN mkdir -p /home/openethereum/.local/share/io.parity.ethereum/

COPY --chown=openethereum:openethereum --from=builder ./build/target/release/openethereum /home/openethereum/openethereum

ENTRYPOINT ["/home/openethereum/openethereum"]
