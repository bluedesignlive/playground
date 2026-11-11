# Smart Shell — GPUI on WebAssembly

A clean re-implementation of the egui `rust-wasm-ui-refinements2` Smart Shell, built on
**GPUI** (Zed's UI framework) through **gpui-kit 0.9** and compiled to **wasm32-unknown-unknown**.
No JavaScript or TypeScript is hand-written anywhere: the only script in the page is the
wasm-bindgen bootstrap that `xtask` embeds into `../index.html` (what `npm run build` serves).

## What's inside
- **Desktop shell** — wallpaper, menu bar with live clock, draggable/resizable floating windows
  with traffic lights, a 5-tile dock (Files, Photos, Terminal, Appearance, Fill Screen) and toasts.
- **Files** — the Smart design contract: workspace sidebar with counts and locations, storage card,
  sectioned Folders/Images grid, list view, sort menu, Ask Smart panel, breadcrumbs, context menus,
  Quick Look, details panel, clipboard copy/cut/paste, trash. Shortcuts: `D` theme, `G` grid,
  `⌘K` Ask Smart, `⌘C/X/V/D/A`, `Space` quick look, `/` search.
- **Terminal** — a real shell over the same in-memory VFS: `ls -l`, `cd`, `cat`, `tree`, `mkdir`,
  `touch`, `neofetch`, `theme dark|light`, `open files|photos|kit`, history with ↑/↓.
- **Photos** — library grid, favourites, viewer with prev/next and a live exposure slider.
- **Smart Kit** — a live gallery of the design system: elevation ladder e1–e5, buttons, segmented
  controls, switches, checkboxes, radios, draggable sliders (track bounds measured with a canvas),
  text field, color tokens, type ramp, toasts and dialogs.

Light and dark themes both follow `assets/tokens.toml` of the original build; depth comes from
tone + a 5-step shadow ladder with an inset top reflection, never from strokes.

## Build for the web
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
rustup target add wasm32-unknown-unknown
rustup toolchain install nightly-2026-10-31 --profile minimal -t wasm32-unknown-unknown  # pinned
cargo install wasm-bindgen-cli --version 0.2.130 --locked   # must match Cargo.lock
# optional: apt install binaryen (wasm-opt -Oz, needs --enable-threads)
bash tools/build_web.sh      # builds wasm, runs wasm-bindgen, bundles ../index.html
cd .. && npm run build       # vite-plugin-singlefile -> dist/index.html
```
`rust-toolchain.toml` pins the nightly that gpui-kit's web backend was built against.
`CARGO_BUILD_JOBS=1` is set inside the script because the GPUI codegen is memory-hungry on
small machines; raise it if you have RAM to spare.

Rendering uses WebGPU when the browser offers an adapter and falls back to WebGL2 (wgpu)
otherwise — verified headless with SwiftShader.
