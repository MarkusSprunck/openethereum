FROM alpine:edge AS builder

RUN apk add --no-cache build-base cargo cmake eudev-dev linux-headers perl rust git

WORKDIR /build

COPY . /build

# RUN cargo test --all --features final

RUN cargo build --release --features final --target x86_64-alpine-linux-musl --verbose

RUN strip target/x86_64-alpine-linux-musl/release/openethereum

FROM alpine:edge

RUN apk add --no-cache libstdc++ eudev-libs libgcc curl jq

RUN addgroup -g 10000 openethereum && adduser -u 10000 -G openethereum -s /bin/sh -D openethereum

USER openethereum

EXPOSE 8545

WORKDIR /home/openethereum

RUN mkdir -p /home/openethereum/.local/share/io.parity.ethereum/

COPY --chown=openethereum:openethereum --from=builder /build/target/x86_64-alpine-linux-musl/release/openethereum /home/openethereum/openethereum

ENTRYPOINT ["/home/openethereum/openethereum"]
