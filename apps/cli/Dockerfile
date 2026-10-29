# syntax=docker/dockerfile:1
FROM rust:1.99-alpine3.23 AS builder
RUN apk add --no-cache build-base cmake git openssl-dev perl pkgconf zlib-dev
ENV CARGO_PROFILE_RELEASE_STRIP=symbols
ENV RUSTFLAGS="-C target-feature=-crt-static"
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY apps ./apps
COPY packages ./packages
ARG TARGETARCH
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,id=giong-musl-${TARGETARCH},target=/src/target,sharing=locked \
    cargo build --locked --release -p giong-cli \
    && mkdir -p /out \
    && cp target/release/giong /out/giong

FROM alpine:3.23 AS runtime
RUN apk add --no-cache ca-certificates libgcc libssl3 zlib && mkdir -p /data/.giong
COPY --from=builder /out/giong /usr/local/bin/giong
ENV HOME=/data
WORKDIR /data
VOLUME ["/data/.giong"]
ENTRYPOINT ["giong"]
