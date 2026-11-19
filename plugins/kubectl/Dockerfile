# syntax=docker/dockerfile:1
FROM alpine/helm:3.19.0 AS helm
RUN cp "$(command -v helm)" /helm

FROM rust:1.99-alpine3.23 AS builder
RUN apk add --no-cache build-base cmake git openssl-dev pkgconf zlib-dev
ENV CARGO_PROFILE_RELEASE_STRIP=symbols
ENV RUSTFLAGS="-C target-feature=-crt-static"
WORKDIR /src
ARG TARGETARCH
RUN --mount=type=bind,source=.,target=/src \
    --mount=type=cache,id=giong-plugin-registry,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,id=giong-plugin-git,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,id=giong-plugin-kubectl-target-${TARGETARCH},target=/build-target,sharing=locked \
    CARGO_TARGET_DIR=/build-target cargo build --locked --release -p giong-plugins-kubectl \
    && install -D /build-target/release/giong-plugins-kubectl /out/execute

FROM alpine:3.23
RUN apk add --no-cache ca-certificates coreutils kubectl libgcc libssl3 zlib \
    && install -d -o 65532 -g 65532 /workspace \
    && install -o 65532 -g 65532 -m 644 /dev/null /workspace/.keep
COPY --from=helm /helm /usr/local/bin/helm
COPY --from=builder /out/execute /usr/local/bin/giong-plugin
ENV HOME=/workspace
WORKDIR /workspace
USER 65532:65532
ENTRYPOINT ["/usr/local/bin/giong-plugin"]
