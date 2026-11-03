# Smart DAW — Components

**Status:** normative · v0.1
**Scope:** every component that does not exist in `smart-design/COMPONENTS.md`.
**Rule:** if a component is not described here, it is not ready to build.

This file is the build sheet for the port. Each entry gives anatomy, states, sizes, behaviour,
and the accessibility contract. DAW-only tokens it introduces are declared in §12 and are the
only additions permitted.

---

## 1. Pane

The dockable/floatable surface.

**Anatomy:** header (32px) + body. Header = drag handle + title + pane controls + overflow +
float toggle. Body has 12px padding and no elevation.

**States:**

| State | Visual |
|---|---|
| Docked | `e1` + `rim-1`; part of the stage |
| Floating | `e4` + `rim-4`; header only carries 12–16px backdrop blur |
| Focused (floating, front) | stronger `rim-2`; no extra highlight |
| Focused (docked) | 2px accent ring on the header |
| Drag | scale 0.985, `e3`, cursor `grabbing` |
| Drop target | slot tint `accent-tint`, 2px accent outline |
| Double-clicked header | maximised in slot |

**Sizes:** header 32px fixed. Body min-height 120px.

**Behaviour:** drag header → float; drag over slot → dock. Slot highlight on hover.

**A11y:** header is `role="toolbar"`-like with `aria-label`. Float toggle is a real button
with `aria-pressed`. Pane body gets `aria-label` and, when it is a list, proper list roles.
Focus ring on the header when the pane is reachable by keyboard.

**Token additions:** none — reuses `raised`, `floating`, `accent-tint`.

---

## 2. Knob

The rotary continuous control. The most important component in the DAW.

**Anatomy:**

```
         ┌─── top highlight (rim, light from above)
       ┌───────────────────────┐
      │    ·  pointer dot        │   ← 30px visual
      │      value ring arc      │   ← accent tint from start to value
      └───────────────────────┘
         └─── bottom shade (rim) + cast shadow
```

- Body: `control` + `rim-1` + a top→bottom reflection gradient (§12 tokens).
- Pointer: 2px dot at the current angle, `ink-2`.
- Value ring: an arc from the start angle to the value angle, `accent-glow`, drawn in a
  track underneath the body so it never competes with the rim.
- Track: 3px circle behind the body, `hair`.

**States:**

| State | Visual |
|---|---|
| Rest | `control` + `rim-1` + gradient |
| Hover | `control-hover` + `e2` + pointer visible |
| Focus-visible | 2px accent ring + 3px halo, 2px offset |
| Pressed | `control-press` + `rim-inset` + scale 0.98 |
| Dragging | value ring brightens to `accent-tint-3`; tooltip shows name + value |
| Disabled | 45% opacity, value ring dims to `hair` |
| Modulated / automated | ring gains a subtle dashes pattern (§12) |

**Geometry:** 30px body, 36px ring, 44px invisible hit circle (the target, not the visual).

**Behaviour:**
- Vertical drag: up = increase, down = decrease. Fine: Shift.
- Mouse wheel over knob: turn in fixed increments.
- Double-click: reset to the parameter's default.
- Keyboard: Tab to focus, arrows ±1 step, PageUp/Down ±10, Home/End min/max.
- Right-click: parameter menu (fine control, reset, show automation).

**Value display:** tooltip on hover/drag shows `name  value`. For beats-linked params the
value is shown in musical terms (1/8, 1/4T), not raw numbers.

**A11y:** `role="slider"` with `aria-valuenow`, `aria-valuemin`, `aria-valuemax`,
`aria-valuetext`, `aria-label`, `aria-orientation="vertical"` (because drag is vertical).

---

## 3. Fader

The linear continuous control. Mixer only.

**Anatomy:**

```
   ┌────┐
   │thumb│ ← raised pill, control + rim-1 + top reflection gradient
   │    │
   │████│ ← fill from unity to current, accent-tint
   │    │
   │ ▃▃ │ ← unity notch, hair
   │    │
   └────┘   ← track groove, rim-well
```

- Track: `rim-well` groove, 4px wide, 140px travel.
- Thumb: 22×14 pill, `control` + `rim-1` + top reflection gradient.
- Fill: from the unity notch to the current value, `accent-tint`; above unity stays tinted
  (never turns danger unless clipping).
- Unity notch: 2px `hair` mark at 0 dB.
- dB readout: 11.5px mono, tabular, `ink-2`, sits beside the track.

**States:**

| State | Visual |
|---|---|
| Rest | as above |
| Hover | thumb `control-hover` + `e2` |
| Focus-visible | 2px accent ring on the thumb, 3px halo |
| Dragging | readout becomes `ink-1`; fill brightens to `accent-tint-3` |
| Clipping | readout + a 3px cap in `danger` on the strip's meter |
| Disabled | 45% opacity |

**Behaviour:** vertical drag, Shift = fine, double-click = 0 dB, wheel = ±1 dB,
arrows / PageUp / Home / End. Range −60 dB → +6 dB, 0 dB default.

**A11y:** `role="slider"`, `aria-valuetext` in dB, `aria-label` with the channel name.

---

## 4. Piano key (virtual)

The playable keyboard, also the pitch stripe of the piano roll.

**Anatomy:** 52 keys, white = 23px × 140px, black = 14px × 88px.

- White key: `panel-top` + `rim-1` + a top highlight + a bottom shade, with a faint
  reflection gradient. Raised — it is a struck object.
- Black key: `ink-1`-derived dark surface, shorter, sitting above the white row.
- Root key: `accent-tint` fill + 2px accent ring (when a scale is active).
- In-scale key: `accent-tint-2` fill.
- Out-of-scale key: `ink-4` at low weight.
- Pressed key: `control-press` + `rim-inset` + 4px travel downward.

**States:** rest, hover, pressed (mouse + QWERTY), out-of-scale, root, disabled.

**Behaviour:** click to strike and hold; QWERTY mapping (two rows, octave shift with Z/X);
note velocity follows the vertical position of the click within the key.

**A11y:** the keyboard is `role="group"` with each key a `button` carrying
`aria-label` (note name) and `aria-pressed`. Keyboard focus moves between keys with arrows,
not Tab.

---

## 5. Step cell

Channel rack cell.

**Anatomy:** 22×26 rounded square.

| State | Visual |
|---|---|
| Off | `control` + `rim-1` (raised, material — it is a trigger) |
| On | channel colour chip fill + `rim-1` |
| On, accented | channel colour + 2px `hair` inner ring |
| Hover | `control-hover` + `e2` |
| Selected | 2px accent ring, 2px offset, `accent-tint` overlay |
| Beat column | 1px `accent-tint` left edge |
| Playing now | a 2px accent bottom edge, one frame |

**Velocity:** shown as the cell's fill opacity (0.45 → 1.0) and, for steps above 90%, a
small dot in the corner. Velocity is a *tone* signal, not colour — colour is channel identity.

**Behaviour:** left-click toggles; drag across cells paints with the same state;
right-click / long-press opens the velocity editor (§6).

**A11y:** cells are `button`s with `aria-pressed` and `aria-label` ("step 5, velocity 80").
Arrow keys move a virtual cursor; Space toggles.

---

## 6. Velocity editor

Inline overlay on a cell or note.

**Anatomy:** a 90px-wide popover with a vertical 140px travel fader (reuses §3 geometry at
reduced scale) and a numeric field.

- Surface: `panel-top` + `e3` + `rim-2` — a floating surface, near-opaque.
- Value: tabular numerals, 11.5px.

**Behaviour:** drag or type; Enter commits; Escape cancels and restores the previous value.
Right-click on a note in the roll opens the same surface at the note.

**A11y:** focus moves into the field on open and returns to the cell on close.

---

## 7. Mixer strip

**Anatomy (top → bottom):**

```
┌──────────┐
│ FX slots │ ← 3 small slots, tonal, drop-target
│  gain    │ ← knob (30px)
│   pan    │ ← knob (30px)
│  M   S  ●│ ← mute/solo/arm, tonal pills, 44px hit
│ ┌─┐      │
│ │meter   │ ← meter bar (§8) beside the fader
│ │fader   │ ← fader (§3), 140px
│ │ dB     │ ← readout
│ └─┘      │
│ sends    │ ← 2 send knobs + return name
└──────────┘
```

- Strip width 96px (narrow mode 76px), gap 12px between strips.
- Strip background: transparent — the pane is the surface. No card-in-pane.
- Selected strip: 2px accent ring around the strip + `accent-tint` wash on the name.

**States:** normal, solo (name + `good-ink`), muted (name + `ink-4`, strip dims to 90%),
armed (record dot in `danger-tint`), master (accent ring by default), overloaded (meter cap).

**A11y:** strip is `role="group"` named by channel. Mute/solo/arm are `aria-pressed`
buttons. Fader and knobs expose their roles.

---

## 8. Meter

**Anatomy:** a 4px wide vertical bar with a peak-hold cap.

- Level: RMS, `good`.
- Warn zone (top 15%): `warn`.
- Peak zone (top 4%): `danger` fill.
- Hold cap: 3px bar that holds the peak for 1.4s then falls.
- Track: `hair`.

**Behaviour:** values come from an `AnalyserNode` read every animation frame. The bar is
**not** animated with a spring — it collapses exponentially and instantly on stop, because a
meter that decays gracefully is lying.

**A11y:** `aria-hidden` on the visual; the strip exposes a text peak readout updated twice a
second for screen readers (`aria-live="off"` — polite updates would spam).

---

## 9. Transport strip

**Anatomy:** one 52px row: `◀ | ■ | ● | · | 112.0 BPM | 3.2.1 | 1.1.1—5.1.1 | -6.2 dB | undo 12`.

- Surface: `raised` (`e1` + `rim-1`), sitting above the dock grid, spanning the window.
- Buttons: tonal pills (§10) — **not** skeuomorphic. Transport actions are momentary verbs.
- BPM: editable field, tabular; drag vertically to change ±1, ±2 to scrub.
- Position: bars.beats.ticks, tabular mono.
- Loop region: start—end readout, editable.

**States per button:** rest, hover, focus, pressed, active (filled icon + `accent-tint`),
armed (`danger-tint`, persistent), disabled (with reason in the tooltip).

**A11y:** every button is a real `button` with `aria-label`; play/pause use `aria-pressed`;
record uses `aria-pressed` and announces "arming channel 3" on change.

---

## 10. Control (tonal)

The parent's control primitive, reused verbatim. Muted here only to note the DAW uses it for
every momentary action: transport, mute/solo/arm, add-channel, zoom, preset switch, FX slot.
**No skeuomorphism, no glow, one rim.**

---

## 11. Note (piano roll)

**Anatomy:** a rounded rectangle on the pitch/time grid.

| State | Visual |
|---|---|
| Rest | channel colour fill, 85% opacity, `rim-1` |
| Hover | 4px resize affordance on the right edge, cursor `ew-resize` |
| Selected | 2px accent ring + `accent-tint` wash |
| Dragging | follows pointer, snaps to grid; scale snap when on |
| Velocity handle | a 4px bar at the note's bottom, drag vertical |

**Behaviour:** click empty = select; draw with the pencil tool or double-click; drag right =
resize; drag vertical = transpose by scale step; Shift+drag = free pitch; Delete removes;
`⌘A` selects all in view; arrows nudge by one grid step, Shift+arrows by one bar.

**A11y:** notes are focusable in the canvas order; arrow keys move the selected note.
Selection count is announced.

---

## 12. DAW token additions

These are the **only** tokens a DAW component may use that do not exist in
`smart-design/smart.toml`. They are declared here so the port can generate them and the
verifier can allowlist them.

| Token | Light | Dark | Use |
|---|---|---|---|
| `--daw-knob-shine` | `linear-gradient(180deg, rgba(255,255,255,.42), rgba(255,255,255,0) 46%, rgba(255,255,255,0) 54%, rgba(16,26,44,.10))` | same, rim inverted by `--rim-1` | knob body reflection |
| `--daw-fader-shine` | `linear-gradient(180deg, rgba(255,255,255,.5), rgba(255,255,255,.06) 60%, rgba(16,26,44,.12))` | same | fader thumb reflection |
| `--daw-key-shine` | `linear-gradient(180deg, rgba(255,255,255,.6), rgba(255,255,255,.05) 70%, rgba(16,26,44,.16))` | same, on lighter key | piano key reflection |
| `--daw-track` | `rgba(20,29,48,.075)` | `rgba(255,255,255,.085)` | groove / inactive track |
| `--daw-knob-ring-track` | `rgba(20,29,48,.10)` | `rgba(255,255,255,.12)` | knob value-ring track |
| `--daw-meter-warn` | `var(--warn)` | `var(--warn)` | meter warn zone |
| `--daw-meter-peak` | `var(--danger)` | `var(--danger)` | meter peak zone |
| `--daw-automation-dash` | `rgba(47,111,235,.35)` | `rgba(111,155,255,.35)` | automated-parameter ring dashes |

**Gradient usage is restricted.** These three gradients appear inside a knob, a fader thumb,
or a piano key. They never appear on a pane, a toolbar, a dialog, or any text surface. That is
the whole boundary between "premium skeuomorphism" and "Linux DAW with drop shadows".

---

## 13. Definition of done

A DAW component is complete only when:

- [ ] all states in its state table exist and are visible
- [ ] it uses no colour outside `tokens.css` and the §12 allowlist
- [ ] it has a 44px hit floor and a focus ring
- [ ] it works in light and dark with zero special-casing
- [ ] it is reachable and operable by keyboard alone
- [ ] it survives reduced motion (springs collapse to fades)
- [ ] its geometry matches §1–11 exactly, or the doc was amended first
