FROM cimg/rust:1.79-node AS builder

RUN echo "nothing to do here"

WORKDIR /build

COPY . /build

# RUN cargo test --all --features final

RUN cargo build --release --features final

RUN strip target/release/openethereum

FROM alpine:edge

RUN apk add --no-cache libstdc++ eudev-libs libgcc curl jq

RUN addgroup -g 10000 openethereum && adduser -u 10000 -G openethereum -s /bin/sh -D openethereum

USER openethereum

EXPOSE 8545

WORKDIR /home/openethereum

RUN mkdir -p /home/openethereum/.local/share/io.parity.ethereum/

COPY --chown=openethereum:openethereum --from=builder /build/target/release/openethereum /home/openethereum/openethereum

ENTRYPOINT ["/home/openethereum/openethereum"]
