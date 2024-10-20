ARG RUST_VERSION="1.79"

# ------------------------------------------------------------------------------
# Build Stage
# ------------------------------------------------------------------------------
FROM rust:${RUST_VERSION}-slim AS BUILDER

RUN apt update -qq && apt install -y -qq --no-install-recommends build-essential libc6-dev cmake

RUN rustup set profile minimal && rustup target add x86_64-unknown-linux-gnu

RUN apt install -y -qq llvm-dev libclang-dev clang

WORKDIR /build

COPY . /build

ENV RUST_BACKTRACE=1

ENV RUSTFLAGS="-Ctarget-feature=+crt-static -Clink-args=-lm -Clink-args=-lpthread"

RUN cargo build --release --target x86_64-unknown-linux-gnu

RUN strip target/x86_64-unknown-linux-gnu/release/openethereum

# ------------------------------------------------------------------------------
# Runtime Stage
# ------------------------------------------------------------------------------
FROM scratch

EXPOSE 8545

COPY --from=builder /build/target/x86_64-unknown-linux-gnu/release/openethereum /openethereum

ENTRYPOINT ["/openethereum"]
