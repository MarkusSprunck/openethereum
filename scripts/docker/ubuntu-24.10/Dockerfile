FROM --platform=linux/amd64 ubuntu:24.10

ENV RUST_BACKTRACE 1

RUN apt-get update && apt-get install libc6

USER root

EXPOSE 8545

WORKDIR /home/openethereum

RUN mkdir -p /home/openethereum/.local/share/io.parity.ethereum/

COPY --chown=openethereum:openethereum ./openethereum /home/openethereum/openethereum

ENTRYPOINT ["/home/openethereum/openethereum"]

