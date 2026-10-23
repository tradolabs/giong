FROM docker:29-cli AS docker-cli

FROM rust:1.99-bookworm AS builder
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY apps ./apps
COPY packages ./packages
RUN cargo build --locked --release -p giong-controller -p giong-cli

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates git libssl3 zlib1g \
    && rm -rf /var/lib/apt/lists/*
COPY --from=docker-cli /usr/local/bin/docker /usr/local/bin/docker
COPY --from=docker-cli /usr/local/libexec/docker/cli-plugins/docker-buildx /usr/local/libexec/docker/cli-plugins/docker-buildx
COPY --from=builder /src/target/release/giong /usr/local/bin/giong
COPY --from=builder /src/target/release/giong-controller /usr/local/bin/giong-controller
ENV HOME=/data
RUN mkdir -p /data/.giong
WORKDIR /data
VOLUME ["/data/.giong"]
EXPOSE 3000
CMD ["giong-controller"]
