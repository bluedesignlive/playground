# Smart DAW — UX Guide

**Status:** normative · v0.1
**Scope:** every pane, control, and gesture in Smart DAW.
**Parent:** `smart-design/UI_GUIDE.md`. Where the parent is silent, this document decides.
Where it speaks, it wins for the shell.

---

## 1. What a DAW has to answer

The file manager answers four questions. A DAW answers six, and three of them are new:

| # | Question | Answered by |
|---|---|---|
| 1 | Where is the playhead? | Transport position, playhead line, loop region |
| 2 | What is playing? | Armed tracks, active steps, the one accent signal |
| 3 | What is selected? | Note/step/channel selection with a count |
| 4 | What happens if I act? | Destination preview, audition-on-hover before commit |
| 5 | **What does it sound like?** | Meters, gain staging, solo, live value readouts |
| 6 | **Can I get out of this?** | Undo/redo depth, always-visible transport, layout presets |

Questions 5 and 6 are the ones DAs historically fail. This design treats metering and
escape as primary surfaces, not accessories.

---

## 2. The DAW laws

Ten laws. They are the only new physics; everything else is the parent system.

**1. Continuous parameters get material; momentary actions get tone.**
A knob turns. A fader travels. A piano key is struck. These earn skeuomorphism — light
reflection and gradient, because the user needs to feel the object under the pointer.
Play, stop, record, mute, solo are momentary verbs. They stay tonal. This single rule is
what separates a premium DAW from a Linux DAW with drop shadows, and it is why the mixer
reads as an instrument rather than as a settings panel.

**2. One light, one rim, everywhere.**
Light comes from top-centre for every surface including every knob. A knob has exactly one
top highlight and one bottom shade. Never two highlights, never a coloured glow on chrome.

**3. One accent at a time.**
Cobalt marks selection, playhead, and the active transport control. A selected note and a
selected channel both take accent — but never in the same row, and never alongside a second
accent (no accent icon tile plus accent ring plus accent keycap). The strongest signal
carries the state.

**4. Undo depth is a visible affordance.**
The transport always shows undo/redo depth as a count. A user who cannot see that history
exists will not trust destructive edits — which kills experimentation, which is the whole
point of a DAW.

**5. Audition before commit.**
Anything you can drag into the project — a sample, a preset, a channel — previews on hover
before it lands. The user should never have to commit to hear it.

**6. The playhead is exempt from easing.**
It tracks time linearly. Springs are for spatial state changes: dock/undock, note move,
preset switch, pane reorder. Applying a spring to the playhead would make the tempo lie.

**7. Zero borders between panes.**
Docked panes are separated by tone and one seam, never by hairlines. Borders between panes
turn a layout into a grid of boxes and defeat calm density. Internal structure (rows,
columns, scale marks) is where hairlines are allowed.

**8. Numbers are tabular, always.**
Tempo, position, dB, time, and counts use tabular numerals so a changing readout does not
jitter its neighbours. A jittering BPM field is a bug, not a style.

**9. Every control survives the keyboard.**
Including knobs and faders. If it can be dragged, arrows can move it; if it can be reset,
there is a key for it. No control is pointer-only.

**10. Layout is presets, not puzzles.**
The default arrangement is one of four audited presets. Users change task, then change
preset. Docking and floating are power-user tools; presets are the first-class answer.
See `docs/LAYOUT.md`.

---

## 3. Surfaces — the DAW ladder

The parent has six surface roles. A DAW adds two.

| Level | Token | Use | Shadow | Rim |
|---|---|---|---|---|
| 0 | `canvas` | Wallpaper behind the window | none | none |
| 1 | `window` | The DAW shell | `e5` | `rim-5` |
| 2 | `stage` | Everything inside the window | none | none |
| 3 | `raised` | Pane headers, toolbar controls, knob bodies | `e1` | `rim-1` |
| 4 | `floating` | Floating panes, menus, palette | `e4` | `rim-4` |
| 5 | `modal` | Task dialogs | `e5` | `rim-5` |
| **3b** | **`pane`** | docked pane surface | `e1` | `rim-1` |
| **3c** | **`control-material`** | knob body, fader thumb, piano key | `e1` | `rim-1` + reflection gradient |

`pane` and `control-material` reuse level-3 tokens; they exist as named roles so no
component invents its own. A docked pane is *part of the stage* and therefore earns the
smallest elevation; a floating pane is *temporarily in front* and earns `e4`.

Forbidden: a raised card inside a raised pane; two top highlights; a coloured shadow on
transport buttons; glass as a legibility strategy.

---

## 4. Density and compactness

The brief is "less means more". That is a density law, not a stain of minimalism.

| Element | Value | Reason |
|---|---|---|
| Pane header | **32px** | The shell's 60px appbar is wrong for a pane title, not a window title |
| Pane gap | **12px** | Pane separation is tonal, so the gap can shrink without losing legibility |
| Pane padding | **12px** | Matches gap; content sits on the same rhythm |
| Hit floor | **44px** hit area | Even on a 20px knob, track, or step cell |
| Row height in rack | **28px** | Dense enough for 64+ steps visible; text floors still hold |
| Step cell | **22px wide × 26px** | 64 steps at 22px = 1408px, so the rack scrolls — that is fine |
| Fader travel | **140px** | Enough for fine gain control; a 200px fader wastes vertical room |
| Knob diameter | **30px** visual / 44px hit | Knob size is material, not target size |

**The floor rule:** 10.5px labels, 11.5px meta, 13px body. A DAW has a lot to say, and
shrinking type below the floor to fit density is the failure mode this rule exists to stop.
Reduce labels, move actions to overflow, or widen the pane — never reduce type.

---

## 5. The attention budget

One primary anchor, one primary action per pane.

| Zone | Budget |
|---|---|
| Transport | Transport controls, BPM, position, master level. Nothing else. |
| Pane header | Title, the 2–4 controls that pane needs, one overflow. |
| Browser | Tree, search, one preview affordance. |
| Channel rack | Add-channel, and per-row mute/solo/arm. No toolbars. |
| Piano roll | Scale, grid, snap, fold. Four controls, one row. |
| Mixer | Insert slots, gain, pan, fader. The strip is the layout. |
| Arrangement | Zoom, loop, automation mode, snap. |

**Removal test:** hide one element. If comprehension does not drop, leave it hidden.

---

## 6. Interaction states

Every control defines six states. The DAW adds a seventh that audio requires.

| State | Visual | Behaviour |
|---|---|---|
| Rest | Tonal or material; no accent | Ready |
| Hover | One tonal step; one elevation step only if raised | Preview, never a commit |
| Focus-visible | 2px accent ring + 3px halo, 2px offset | Keyboard location |
| Pressed | Scale 0.98 + inner shade + lift removed | Immediate feedback |
| Selected | Accent ring + tint, persistent, announced | State with a count |
| Disabled | 45% opacity, `aria-disabled`, reason on demand | Cannot act |
| **Live** | Meter, value ring, position moving | Current signal — exempt from hover rules |

Hover-only actions are forbidden unless reachable via keyboard and coarse pointer. On a
touch surface, audition-on-hover becomes audition-on-press.

---

## 7. Motion

Durations and easings come from the parent. Durations come from the parent system:

| Transition | Duration | Easing |
|---|---|---|
| Pane dock / undock / preset switch | `dur-3` 360ms | `spring` |
| Note move, resize, nudge | `dur-1` 140ms | `ease-out` |
| Scale re-highlight | `dur-2` 220ms | `ease-in-out` |
| Pane reorder in rack | `dur-3` 360ms | `spring` |
| Menu / popover open | `dur-menu` 220ms | `ease-out` |
| Playhead | linear, per tick | none — exempt |
| Meter decay | exponential, ~80ms/20dB | none — exempt |

**Never** spring: text, list scrolling, the playhead, meters, fader tracks while dragging
under their own parameter, or any hover.

Reduced motion keeps every state change; only the springs and travel collapse to fades.

---

## 8. Colour, for audio specifically

| Use | Token | Rule |
|---|---|---|
| Selection / playhead / active transport | `accent` | The one accent |
| In-scale key fill | `accent-tint-2` | Subtle |
| Out-of-scale key dim | `ink-4` | Decorative only, never meaningful text |
| Root key | `accent-tint` + ring | Strongest scale signal |
| Recording armed | `danger-tint` | Persistent state, never hover |
| Meter safe / warn / peak | `good` / `warn` / `danger` | Semantic level, colour always paired with height |
| Grid lines (in roll) | `hair` | Subtle |
| Beat marks (in roll) | `accent-tint` | Slightly stronger than `hair`, never `accent` |
| Step cells, off | `control` | Raised, material |
| Step cells, on | channel colour chip | Per-channel colour is a signal, not decoration |

Channel colour exists — a DAW is full of channels and they need identity — but channel
colour is a *channel identity* signal, and it must never compete with cobalt for
selection. A selected channel shows a cobalt ring; its body keeps its channel colour.

---

## 9. Microcopy

| Context | Prefer | Avoid |
|---|---|---|
| Transport | Play, Stop, Record | "Start", "Run" |
| Empty roll | "Nothing here yet — draw a note." | "No data." |
| Unsaved | "3 changes not saved" | "Warning!" |
| Undo depth | "Undo (12)" | "Undo" |
| Export | "Export WAV" | "Render" |
| Automation mode | Write, Touch, Latch, Read | "Armed" |
| Overload | "Clipping — lower the master" | "Error" |

---

## 10. Anti-patterns, DAW edition

Reject on sight.

1. **Neon glow on a knob.** The value ring is tint, not light emission.
2. **Two highlights on one knob.** One top highlight, one bottom shade.
3. **Accent everywhere.** Every step lit, every channel blue, every meter accent.
4. **A spring on the playhead.** It makes tempo a lie.
5. **27 hairlines in the mixer.** Pane separation is tonal; internal rows use hairlines.
6. **A 9px readout.** dB is 11.5px mono, tabular. The floor is the floor.
7. **Hover-only audition.** On touch it must be press.
8. **Flat buttons for faders.** A fader is a continuous parameter and gets material.
9. **Faking progress.** A meter that shows level before the signal exists.
10. **Play only on click.** Keyboard transport is parity, not a bonus.

---

## 11. Verification gates

Automated:

- No CSS custom property in `daw.css` that is not in `tokens.css` or the documented DAW
  extension block in `docs/COMPONENTS.md`.
- No duplicate HTML IDs. `node --check` passes on both modules.
- Contrast pairs meet AA in light and dark.

Manual matrix:

| Surface | States |
|---|---|
| Transport | rest, playing, recording, loop on, overload, coarse pointer |
| Knob | rest, hover, focus, drag, fine-drag, reset, disabled, reduced motion |
| Fader | rest, hover, focus, drag, keyboard, dB clip, unity notch |
| Piano roll | empty, note, selected note, velocity edit, scale on/off, folded |
| Mixer | solo, mute, armed, metering, master selected, send exists |
| Channel rack | step on/off, velocity edit, add, reorder, mute/solo |
| Arrangement | clip, loop region, automation lane, zoom, pattern clone |
| Browser | preview, search, drag, empty |
| Panes | each pane docked and each floating; each preset |
| System | light, dark, first-run prompt, reduced motion, 1280/1440/1024/390 |
