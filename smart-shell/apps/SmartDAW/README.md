# Smart DAW

A modern, compact digital audio workstation built on the **Smart Design** contract.

Ampiano-first, dock-and-float panes, skeuomorphic continuous controls, one accent, one light.
This is the **reference implementation** — the HTML prototype a native UI framework copies 1:1.

---

## What this is

A full DAW prototype in a single HTML file plus three ES modules. It runs in a browser,
plays real audio through Web Audio, and implements the complete interaction surface of a
mainstream DAW:

- step sequencer (channel rack) with per-step velocity
- piano roll with scale highlighting and a velocity lane
- arrangement view with pattern blocks, loop region, and automation lanes
- mixer with inserts, sends, returns, metering, and skeuomorphic faders/knobs
- browser with audition-on-hover
- transport with sample-accurate scheduling
- save/load, undo/redo, export

**The point of this prototype is the design, not the audio code.** Every control, state,
animation, spacing value, and colour comes from `assets/tokens.css`, generated from
`smart-design/smart.toml`. Nothing raw. If you want to know why a knob looks the way it
does, the answer is in `docs/`, not in the CSS.

---

## Run it

No build step, no dependencies, no network. Open the file:

```bash
cd prototypes/smart-shell/apps/SmartDAW
python3 -m http.server 8080
# → http://localhost:8080/index.html
```

A file server is required (not `file://`) because the modules and fonts are fetched.
Audio needs one click on the page — browsers gate `AudioContext` behind a gesture.

---

## Structure

```
apps/SmartDAW/
├── index.html              # the whole app surface, one file
├── README.md               # this file
├── docs/
│   ├── UX_GUIDE.md         # DAW-specific interaction laws, derived from Smart Design
│   ├── FEATURE_PARITY.md   # parity matrix vs FL Studio / Ableton / Logic / Reaper
│   ├── LAYOUT.md           # dock-and-float model, slots, presets (the UX audit)
│   ├── COMPONENTS.md       # anatomy + states for every DAW-specific component
│   ├── KEYS.md             # complete keyboard map
│   └── AUDIO.md            # engine architecture and scheduling model
├── tools/
│   └── build_daw_icons.py   # regenerates the DAW icon sprite from real Solar paths
└── assets/
    ├── tokens.css          # generated from smart-design/smart.toml — never edited
    ├── tokens.toml         # the contract snapshot this build is pinned to
    ├── icons.js            # Solar icon sprite (shared with the shell prototype)
    ├── daw-icons.js        # DAW transport/mixer/edit icons, same naming contract
    ├── fonts/              # Inter Variable
    ├── wallpaper.jpg       # the canvas behind the window
    ├── daw.css             # DAW-only components (pane, knob, fader, roll, mixer…)
    ├── state.js            # the musical data model + undo history + project files
    ├── controls.js         # knob, fader, meter, drag primitives
    ├── audio.js            # voices, FX, routing, metering
    ├── panes.js            # pane manager + every pane renderer
    └── daw.js              # transport, scheduler, palette, keyboard, boot
```

`index.html` loads the sprite scripts and one ES module entry, `daw.js`. The five
modules are plain ES modules — no build step, no bundler, no dependencies.

## Run it

```bash
cd prototypes/smart-shell/apps/SmartDAW
python3 -m http.server 8080
# → http://localhost:8080/index.html
```

A file server is required (not `file://`) because the modules and fonts are fetched.
Audio needs one click on the page — browsers gate `AudioContext` behind a gesture.
First run asks for a theme; the choice persists to `localStorage`.

## Verified

| Area | How it was checked |
|---|---|
| App boots | headless DOM load, 5 panes render, no runtime errors |
| All panes render | browser, channel rack, piano roll, mixer, arrangement |
| Preset switching | compose / program / arrange / mix all rebuild the dock grid |
| Transport | play, stop, metronome, loop, record arm, tempo, grid division |
| Scheduler | playhead advances against a moving audio clock; voices fire on step hits |
| Piano roll | draw, resize, drag-transpose, velocity lane, scale highlight, popover |
| Scale | root + mode change re-tints keys; root ring and out-of-scale dimming |
| Mixer | 9 strips, gain/pan/send knobs, fader with 0 dB notch, mute/solo/arm |
| FX | insert menu toggles real Web Audio nodes, par meter-free rebuild |
| Arrangement | double-click places a clip; zoom, loop, lane mode |
| Undo/redo | depth shown in the topbar, snapshots across notes/patterns/mixer |
| Project file | JSON save/load round trip; foreign files rejected |
| Keyboard | transport, presets, octave, computer keyboard as piano |
| Theme | light and dark swap with no special-casing |

---

## The design contract in one page

Read `docs/UX_GUIDE.md` for the reasoning. The short version:

| Law | How it lands here |
|---|---|
| One light | Every raised surface uses one cast shadow + one rim from the elevation ladder. Knobs get one top highlight + one rim — never two highlights. |
| Depth is semantic | Pane headers are draggable/dockable, so they are interactive. Pane content rows stay flat. Knobs and faders are raised because they are continuous parameters. |
| One accent signal | Cobalt marks selection, playhead, and active transport. Nothing else goes blue. |
| Selection is not hover | Selected notes, channels, and steps keep a persistent ring + tint with a count. Hover is a preview only. |
| Skeuomorphism is scoped | Only knobs, fader thumbs, and piano keys carry material. Transport buttons, rack cells, and panes stay tonal. Less means more. |
| Text is not glass | dB readouts, scale names, positions, and labels are near-opaque. Blur is atmosphere. |
| Keyboard is first-class | Every action reachable without a pointer. No Tab hijacking. Focus returns to its trigger. |
| Timelessness | No neon, no glassmorphism stack, no marketing type, no animation that will date. Gradients exist as light reflection inside controls only. |

---

## Fidelity rules for the port

The native port is graded against this prototype, not against taste.

1. **Tokens are the only source.** No hex, shadow, radius, duration, or font size in a
   component. Every one of those lives in `smart.toml` or in the DAW extension block of
   `docs/COMPONENTS.md`.
2. **State parity.** Every control has rest, hover, focus-visible, pressed, selected,
   disabled. A control missing one is unfinished.
3. **Light and dark both ship.** Dark is the landing theme for a DAW; light must still work
   with zero special-casing.
4. **1:1 geometry.** Pane header 32px, pane gap 12px, hit floors 44px — these are decisions,
   not suggestions. If the port differs, the port is wrong.
5. **Motion parity.** Springs for spatial state changes (dock, note move, preset switch),
   linear for the playhead, fade-only under reduced motion. No spring on text or lists.

---

## Explicit non-goals

Stated so nobody mistakes an omission for an oversight.

- **No VST/AU plugin hosting.** Browser sandbox. Every effect is a built-in Web Audio
  equivalent, modelled in `docs/AUDIO.md`.
- **No hardware MIDI input.** Web MIDI is possible but out of scope; the computer keyboard
  is the input instrument.
- **No cloud, no accounts, no telemetry.** Projects save to `localStorage` and export as a
  JSON file. Nothing leaves the machine.
- **No collaborative editing.**

---

## Status

| Area | State |
|---|---|
| Docs | complete |
| Design tokens | pinned to `smart-design/smart.toml` v0.1 |
| Shell, panes, dock/float, presets | built |
| Mixer + engine + metering | built |
| Channel rack + step sequencing | built |
| Piano roll + scale highlighting | built |
| Arrangement + automation lanes | built |
| Browser + audition | built |
| Effects + FX chain | built |
| Keys + sample editor | built |
| Save/load, undo, export | built |
| WAV mixdown export | specified, engine ready (`docs/AUDIO.md` §7) |

## Regenerating the icon sprite

```bash
python3 tools/build_daw_icons.py      # needs network; pulls from the Iconify Solar API
```

Add the icon's base name to `ICONS`, re-run, then `node --check assets/daw-icons.js`.
The sprite extends `window.__SOLAR__` set by the shared `assets/icons.js`, so the two
files must load in that order.
