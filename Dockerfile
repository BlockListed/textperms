FROM rust:bullseye as builder
WORKDIR /usr/src/textperms
COPY . .

# Install nightly rust version
RUN rustup install nightly
RUN rustup override set nightly

RUN cargo install --path .

FROM debian:bullseye-slim

RUN apt-get update && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/textperms /usr/local/bin

ENTRYPOINT [ "textperms" ]