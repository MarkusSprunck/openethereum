FROM cimg/rust:1.63-node@sha256:a091d3aba7a6d919f3e67d128039698851a2fb583441a7b6dfc7be4921aa8cdc AS builder

WORKDIR /build

COPY . /build

RUN cargo test --color=always --release --features final

RUN cargo build --color=always --release --features final

RUN strip target/release/openethereum

FROM --platform=linux/amd64 ubuntu:24.10@sha256:67541378af7d535606e684a8234d56ca0725b6a4d8b0bbf19cebefed98e06f42

RUN groupadd -g 1000 openethereum; \
	useradd -m -u 1000 -g openethereum -s /bin/sh openethereum

USER openethereum

EXPOSE 8545

WORKDIR /home/openethereum

RUN mkdir -p /home/openethereum/.local/share/io.parity.ethereum/

COPY --chown=openethereum:openethereum --from=builder ./build/target/release/openethereum /home/openethereum/openethereum

ENTRYPOINT ["/home/openethereum/openethereum"]
