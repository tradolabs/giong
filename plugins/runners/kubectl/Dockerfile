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
release=$1
namespace=$2
values_file=$3
shift 3
kubectl config set-context --current --namespace="$namespace" > /dev/null
: > /tmp/manifests.yaml
for chart in "$@"; do
    helm dependency build "$chart"
    printf '%s\n' '---' >> /tmp/manifests.yaml
    if [ -n "$values_file" ]; then
        helm template "$release" "$chart" --namespace "$namespace" --no-hooks --values "$chart/$values_file" >> /tmp/manifests.yaml
    else
        helm template "$release" "$chart" --namespace "$namespace" --no-hooks >> /tmp/manifests.yaml
    fi
done
exec kubectl apply -f /tmp/manifests.yaml
SCRIPT
ENTRYPOINT ["/usr/local/bin/giong-kubectl"]
