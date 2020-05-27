
# ------------------------------------------------------------------------------
# Build Stage
# ------------------------------------------------------------------------------


FROM rust:latest as cargo-build
RUN rustup component add rustfmt

RUN apt-get update

RUN apt-get install musl-tools  openssl libssl-dev -y

RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /usr/src/dhall-mock

COPY Cargo.toml Cargo.toml

COPY . .

RUN PKG_CONFIG_ALLOW_CROSS=1 RUSTFLAGS=-Clinker=musl-gcc cargo build --release --target=x86_64-unknown-linux-musl --features vendored

# ------------------------------------------------------------------------------
# Release Stage
# ------------------------------------------------------------------------------

FROM alpine:latest

RUN addgroup -g 1000 dhall-mock

RUN adduser -D -s /bin/sh -u 1000 -G dhall-mock dhall-mock

WORKDIR /home/dhall-mock/bin/

COPY --from=cargo-build /usr/src/dhall-mock/target/x86_64-unknown-linux-musl/release/dhall_mock_server .

RUN chown dhall-mock:dhall-mock dhall_mock_server

ENTRYPOINT ["./dhall_mock_server"]
EXPOSE 8088/tcp

