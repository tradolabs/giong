# syntax=docker/dockerfile:1
FROM alpine/helm:3.19.0
RUN apk add --no-cache kubectl
WORKDIR /workspace
COPY . .
COPY --chmod=755 <<'SCRIPT' /usr/local/bin/giong-kubectl
#!/bin/sh
set -eu
umask 077
kubeconfig_file=$(mktemp)
printf '%s' "$KUBECONFIG" > "$kubeconfig_file"
export KUBECONFIG="$kubeconfig_file"
helm template "$@" > /tmp/manifests.yaml
exec kubectl apply -f /tmp/manifests.yaml
SCRIPT
ENTRYPOINT ["/usr/local/bin/giong-kubectl"]
