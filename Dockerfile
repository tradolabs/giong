# syntax=docker/dockerfile:1
FROM docker:29-cli AS docker-cli

FROM alpine:3.23
RUN apk add --no-cache ca-certificates libgcc libssl3 nginx zlib \
    && mkdir -p /data/.giong /run/nginx
COPY --from=docker-cli /usr/local/bin/docker /usr/local/bin/docker
COPY --from=docker-cli /usr/local/libexec/docker/cli-plugins/docker-buildx /usr/local/libexec/docker/cli-plugins/docker-buildx
COPY --from=cli-builder /out/giong /usr/local/bin/giong
COPY --from=controller-builder /out/giong-controller /usr/local/bin/giong-controller
COPY --from=reconciler-builder /out/giong-reconciler /usr/local/bin/giong-reconciler
COPY --from=console-builder /out /usr/share/nginx/html
COPY nginx.conf /etc/nginx/nginx.conf
ENV HOME=/data
ENV GIONG_PROVIDERS_DIR=/usr/local/lib/giong/providers
WORKDIR /data
VOLUME ["/data/.giong"]
EXPOSE 80
CMD ["sh", "-c", "nginx && exec giong-controller"]
