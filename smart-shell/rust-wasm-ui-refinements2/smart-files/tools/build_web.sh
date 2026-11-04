#!/usr/bin/env bash
# Build the egui frontend for the browser and drop the wasm-bindgen output next to the Vite entry.
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build -p smart-ui-egui --lib --target wasm32-unknown-unknown --profile wasm
wasm-bindgen --target web --no-typescript --out-dir ../src/smart target/wasm32-unknown-unknown/wasm/smart_ui_egui.wasm
if command -v wasm-opt >/dev/null; then wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext --enable-mutable-globals -o ../src/smart/smart_ui_egui_bg.wasm ../src/smart/smart_ui_egui_bg.wasm || true; fi
ls -la ../src/smart
sed -i "s#module_or_path = new URL('smart_ui_egui_bg.wasm', import.meta.url);#throw new Error('wasm url required');#" ../src/smart/smart_ui_egui.js
