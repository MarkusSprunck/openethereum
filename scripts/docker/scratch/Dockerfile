ARG RUST_VERSION="1.79"

# ------------------------------------------------------------------------------
# Build Stage
# ------------------------------------------------------------------------------
FROM rust:${RUST_VERSION}-slim AS builder

RUN apt update -qq && apt install -y -qq --no-install-recommends build-essential libc6-dev cmake

RUN rustup set profile minimal && rustup target add x86_64-unknown-linux-gnu

RUN apt install -y -qq llvm-dev libclang-dev clang

WORKDIR /build

COPY . /build

ENV RUST_BACKTRACE=1

ENV RUSTFLAGS="-Ctarget-feature=+crt-static -Clink-args=-lm -Clink-args=-lpthread"

RUN cargo build --release --features final --target x86_64-unknown-linux-gnu

RUN strip target/x86_64-unknown-linux-gnu/release/openethereum

# Add a non-priveliged user account. I'm using a specific ID just so
# I can prove I'm running as that user inside the application.
RUN addgroup openethereum &&  \
    adduser --ingroup openethereum --uid 1000 --shell /bin/false openethereum &&  \
    cat /etc/passwd | grep openethereum > /etc/passwd_openethereum \


# ------------------------------------------------------------------------------
# Runtime Stage
# ------------------------------------------------------------------------------
FROM scratch

EXPOSE 8545

# Copy the /etc/passwd file with the user I want to run as defined in it.
COPY --from=builder /etc/passwd_openethereum /etc/passwd

COPY --from=builder /build/target/x86_64-unknown-linux-gnu/release/openethereum /openethereum

USER openethereum

ENTRYPOINT ["/openethereum"]
