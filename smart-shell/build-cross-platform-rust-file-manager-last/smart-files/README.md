# Smart Files (Rust)

Cross-platform file manager rebuilt from the `smart-shell` HTML prototype. Native (Linux/macOS/Windows) and WebAssembly from one code base.

```
crates/smart-core        backend — no UI types. Vfs trait (local disk, memory, ADB phone), search (nucleo + cached index + content scan,
                         natural-language queries), copy/move/trash jobs with progress, real mount/unmount/eject, Open With / default apps,
                         thumbnails, tags/recents/settings, audio (rodio / HTMLAudio). Frontends call `Backend` and `poll()` events.
crates/smart-ui-egui     egui frontend: design tokens, depth primitives (elevation ladder, top-light rims), all views and dialogs.
assets/                  Inter, Solar icons (rasterised), wallpapers, demo content, tokens.toml
tools/build_web.sh       cargo build → wasm-bindgen → ../src/smart (Vite inlines it into dist/index.html)
```

* Desktop: `cargo run -p smart-ui-egui --release` (`--demo` for the seeded in-memory world).
* Web: `./tools/build_web.sh && npm run build` → single `dist/index.html`.
* Switching UI framework: write a new crate depending on `smart-core`; only drawing/input needs rewriting.
* Cloud: implement `Vfs` (+ a `DeviceProvider`) — e.g. WebDAV/S3 via OpenDAL — and register it; jobs, search and thumbnails work unchanged.
