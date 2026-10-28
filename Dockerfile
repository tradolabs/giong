FROM docker:29-cli AS docker-cli

FROM node:26-bookworm-slim AS console-builder
WORKDIR /console
COPY apps/console/package.json apps/console/pnpm-lock.yaml apps/console/pnpm-workspace.yaml ./
RUN npm install --global pnpm@11.0.5 \
    && pnpm install --frozen-lockfile
COPY apps/console ./
RUN pnpm build

FROM rust:1.99-bookworm AS builder
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY apps ./apps
COPY packages ./packages
RUN cargo build --locked --release -p giong-controller -p giong-cli -p giong-reconciler

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates git libssl3 zlib1g nginx-light \
    && rm -rf /var/lib/apt/lists/*
COPY --from=docker-cli /usr/local/bin/docker /usr/local/bin/docker
COPY --from=docker-cli /usr/local/libexec/docker/cli-plugins/docker-buildx /usr/local/libexec/docker/cli-plugins/docker-buildx
COPY --from=builder /src/target/release/giong /usr/local/bin/giong
COPY --from=builder /src/target/release/giong-controller /usr/local/bin/giong-controller
COPY --from=builder /src/target/release/giong-reconciler /usr/local/bin/giong-reconciler
COPY --from=console-builder /console/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/nginx.conf
ENV HOME=/data
ENV GIONG_PROVIDERS_DIR=/usr/local/lib/giong/providers
RUN mkdir -p /data/.giong
WORKDIR /data
VOLUME ["/data/.giong"]
EXPOSE 80
CMD ["sh", "-c", "nginx && exec giong-controller"]
