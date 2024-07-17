FROM cimg/rust:1.61-node AS builder
WORKDIR /build
COPY . /build
RUN cargo build --color=always --release --features final
RUN strip target/release/openethereum

FROM --platform=linux/amd64 ubuntu:24.10
ENV RUST_BACKTRACE 1
RUN apt-get update && apt-get install libc6
USER root
EXPOSE 8545
WORKDIR /home/openethereum
RUN mkdir -p /home/openethereum/.local/share/io.parity.ethereum/
COPY --chown=openethereum:openethereum --from=builder ./build/target/release/openethereum /home/openethereum/openethereum
ENTRYPOINT ["/home/openethereum/openethereum"]
