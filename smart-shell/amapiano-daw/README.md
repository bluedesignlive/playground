# Smart DAW — Amapiano Production Studio

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
`smart-design/smart.toml`. Nothing raw.

---

## Run it

No build step, no dependencies, no network. Open the file:

```bash
cd prototypes/smart-shell/amapiano-daw
python3 -m http.server 8080
# → http://localhost:8080/index.html
```

A file server is required (not `file://`) because the modules and fonts are fetched.
Audio needs one click on the page — browsers gate `AudioContext` behind a gesture.

---

## Structure

```
amapiano-daw/
├── index.html              # the whole app surface, one file
├── README.md               # this file
├── css/
│   └── daw.css             # DAW-only components (pane, knob, fader, roll, mixer…)
├── js/
│   ├── app.js              # dock system, panes, piano roll, mixer, transport
│   └── audio.js            # Web Audio engine, scheduler, metering
├── assets/
│   ├── tokens.css          # generated from smart-design/smart.toml — never edited
│   ├── tokens.toml         # the contract snapshot this build is pinned to
│   ├── icons.js            # Solar icon sprite (shared with the shell prototype)
│   ├── fonts/              # Inter Variable
│   └── wallpaper.jpg       # the canvas behind the window
├── docs/
│   ├── UX_GUIDE.md         # DAW-specific interaction laws, derived from Smart Design
│   ├── FEATURE_PARITY.md   # parity matrix vs FL Studio / Ableton / Logic / Reaper
│   ├── LAYOUT.md           # dock-and-float model, slots, presets (the UX audit)
│   ├── COMPONENTS.md       # anatomy + states for every DAW-specific component
│   ├── KEYS.md             # complete keyboard map
│   └── AUDIO.md            # engine architecture and scheduling model
└── mocks/
    ├── audio/              # generated WAV files
    ├── images/             # sample thumbnails (SVG)
    └── samples/            # sample library documentation
```

---

## The design contract in one page

Read `docs/UX_GUIDE.md` for the reasoning. The short version:

| Law | How it lands here |
|-----|------------------|
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
|------|-------|
| Docs | complete |
| Design tokens | pinned to `smart-design/smart.toml` v0.1 |
| Implementation | in progress |
| Premium skeuomorphic knobs/faders | complete |
| Piano roll with scale highlighting | complete |
| Channel rack with per-step velocity | complete |
| Mixer with skeuomorphic controls | complete |
| Transport with loop markers | complete |
| AI command palette | complete |
| Project save/load | complete |
| Undo/redo | complete |

This prototype is the authority for how Smart DAW looks and behaves. The native port follows it.
