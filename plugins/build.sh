#!/bin/sh
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
destination="${GIONG_PLUGINS_DIR:-$HOME/.giong/plugins}"
case "$destination" in
    "~") destination="$HOME" ;;
    "~/"*) destination="$HOME/${destination#'~/'}" ;;
    /*) ;;
    *) destination="$PWD/$destination" ;;
esac
cd "$root"
for package in giong-repository-source giong-kubectl-runner; do
    cargo build --locked --release --target wasm32-wasip2 -p "$package"
done
output="${CARGO_TARGET_DIR:-target}/wasm32-wasip2/release"
install_plugin() {
    target="$destination/$1.wasm"
    mkdir -p "$(dirname -- "$target")"
    staged=$(mktemp "$target.XXXXXX")
    trap 'rm -f "$staged"' EXIT HUP INT TERM
    cp "$output/$2.wasm" "$staged"
    mv "$staged" "$target"
    printf 'Installed plugin: %s\n' "$target"
}
install_plugin sources/repository giong_repository_source
install_plugin runners/kubectl giong_kubectl_runner
