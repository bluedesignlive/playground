#!/usr/bin/env bash
# Build the GPUI app for the browser and bundle it into ../index.html (what `npm run build` serves).
#   needs: rustup (nightly pinned in rust-toolchain.toml + wasm32-unknown-unknown), wasm-bindgen-cli 0.2.130, binaryen (optional)
set -euo pipefail
cd "$(dirname "$0")/.."
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-1}"   # gpui is memory hungry; keep rustc to one job on small machines
cargo build --release --target wasm32-unknown-unknown
OUT=target/web
rm -rf "$OUT" && mkdir -p "$OUT"
wasm-bindgen --target no-modules --no-typescript --out-dir "$OUT" target/wasm32-unknown-unknown/release/smart_gpui.wasm
ls -la "$OUT"
if [ -z "${SKIP_OPT:-}" ] && command -v wasm-opt >/dev/null; then
  wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext --enable-mutable-globals --enable-reference-types --enable-multivalue --enable-threads \
    -o "$OUT/opt.wasm" "$OUT/smart_gpui_bg.wasm" && mv "$OUT/opt.wasm" "$OUT/smart_gpui_bg.wasm" || echo "wasm-opt skipped"
fi
(cd xtask && cargo +stable run --release -q -- bundle "../$OUT" ../../index.html)
