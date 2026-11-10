#!/bin/sh
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
destination="${GIONG_RUNNERS_DIR:-$HOME/.giong/plugins/runners}"
case "$destination" in
    "~") destination="$HOME" ;;
    "~/"*) destination="$HOME/${destination#'~/'}" ;;
    /*) ;;
    *) destination="$PWD/$destination" ;;
esac
cd "$root"
cargo build --locked --release --target wasm32-wasip2 -p giong-kubectl-runner
output="${CARGO_TARGET_DIR:-target}/wasm32-wasip2/release"
mkdir -p "$destination"
# Rename into place so a running host never reads a partially written module.
staged=$(mktemp "$destination/.kubectl.wasm.XXXXXX")
trap 'rm -f "$staged"' EXIT HUP INT TERM
cp "$output/giong_kubectl_runner.wasm" "$staged"
mv "$staged" "$destination/kubectl.wasm"
printf 'Installed kubectl runner: %s/kubectl.wasm\n' "$destination"
