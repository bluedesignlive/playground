# Smart DAW — Keyboard Map

**Status:** normative · v0.1
**Rule:** every action possible with a pointer is possible from the keyboard. If it is not
listed here, it is not finished.

Modifier notation: `⌘` = Cmd on macOS, Ctrl elsewhere.

---

## 1. Global

| Key | Action |
|---|---|
| `Space` | Play / Stop |
| `Enter` | Play / Stop (alternate, respects focus) |
| `R` | Record arm toggle on the focused channel |
| `L` | Loop toggle |
| `M` | Metronome toggle |
| `Z` / `X` | Octave down / up for the computer keyboard |
| `,` / `.` | Nudge tempo −1 / +1 |
| `Shift+,` / `Shift+.` | Nudge tempo −5 / +5 |
| `D` | Toggle light / dark theme |
| `⌘K` | Command palette |
| `⌘Z` | Undo |
| `⌘⇧Z` / `⌘Y` | Redo |
| `⌘S` | Save project (localStorage) |
| `⌘⇧S` | Export project as JSON file |
| `⌘O` | Open project from file |
| `⌘E` | Export WAV mixdown |
| `⌘A` | Select all in the focused pane's context |
| `Esc` | Close the top-most overlay; returns focus to its trigger |
| `?` | Keyboard shortcut sheet |
| `Tab` | Move focus to the next control — **never hijacked** |
| `⇧Tab` | Previous control |
| `⌘1` `⌘2` `⌘3` `⌘4` | Switch layout preset: Compose / Program / Arrange / Mix |
| `⌘T` | Tile floating panes |
| `⌘⇧F` | Float / dock the focused pane |
| `⌘\` | Toggle the browser |

---

## 2. Transport

| Key | Action |
|---|---|
| `Space` | Play / Stop |
| `⇧Space` | Play from the loop start |
| `Home` | Go to song start |
| `End` | Go to song end |
| `←` / `→` | Move playhead by one grid division |
| `⇧←` / `⇧→` | Move playhead by one bar |
| `1` `2` `3` `4` `5` | Grid division 1/4, 1/8, 1/16, 1/32, 1/64 |
| `T` | Tap tempo |
| `N` | Toggle metronome |

---

## 3. Piano roll

| Key | Action |
|---|---|
| `←` `→` `↑` `↓` | Move selected note by one grid step |
| `⇧←` / `⇧→` | Resize note by one grid step |
| `⇧↑` / `⇧↓` | Transpose selected note by one scale step |
| `Alt ↑` / `Alt ↓` | Transpose selected note by one semitone |
| `Delete` / `Backspace` | Delete selected notes |
| `⌘D` | Duplicate selection |
| `⌘A` | Select all notes in view |
| `⇧click` | Range select from the anchor |
| `+` / `-` | Zoom horizontal |
| `⇧+` / `⇧-` | Zoom vertical (pitch) |
| `F` | Toggle key folding |
| `S` | Toggle scale highlighting |
| `K` | Open the scale selector |
| `V` | Cycle pencil / select / velocity tools |
| `Q` | Quantize selection to grid |
| `H` | Humanize selection |
| `G` | Toggle ghost notes |
| `C` | Transpose down one octave (selection) |
| `⇧C` | Transpose up one octave (selection) |

---

## 4. Channel rack

| Key | Action |
|---|---|
| `↑` `↓` | Move between channels |
| `Home` / `End` | First / last channel |
| `←` `→` | Move between steps |
| `Space` | Toggle step under the cursor |
| `Shift` + drag over cells | Paint steps on |
| `Alt` + drag | Paint steps off |
| `Enter` | Rename the focused channel |
| `⌘↑` / `⌘↓` | Move channel up / down (reorder) |
| `⌘D` | Duplicate channel |
| `Backspace` | Delete channel |
| `M` / `S` / `R` | Mute / solo / arm the focused channel |
| `0` | Add a new channel |

---

## 5. Mixer

| Key | Action |
|---|---|
| `←` `→` | Previous / next strip |
| `Home` / `End` | First / last strip (master) |
| `↑` / `↓` | Focus fader, then knob below/above |
| `Tab` | Cycle focus within a strip (slots → gain → pan → mute/solo/arm → fader → sends) |
| `←` / `→` on a fader | ±1 dB |
| `PageUp` / `PageDown` on a fader | ±6 dB |
| `Home` / `End` on a fader | −∞ dB / +6 dB |
| `↑` / `↓` on a knob | ±1 step |
| `PageUp` / `PageDown` on a knob | ±10 steps |
| `Home` / `End` on a knob | Min / max |
| `Backspace` on a fader | Reset to 0 dB |
| `M` / `S` / `R` | Mute / solo / arm the focused strip |

---

## 6. Arrangement

| Key | Action |
|---|---|
| `←` `→` | Move playhead by one grid division |
| `⇧←` / `⇧→` | Move playhead by one bar |
| `⌘←` / `⌘→` | Jump to previous / next marker |
| `↑` / `↓` | Move between tracks |
| `Space` | Play / Stop (same as global) |
| `B` | Toggle the pencil tool |
| `Delete` | Delete selected clip |
| `⌘D` | Duplicate clip |
| `⌘R` | Repeat loop the selection |
| `+` / `-` | Zoom horizontal |
| `⇧+` / `⇧-` | Zoom vertical (track height) |
| `A` | Toggle the automation lane for the focused track |
| `⇧A` | Cycle automation mode: Write → Touch → Latch → Read |
| `N` | New marker at the playhead |

---

## 7. Browser

| Key | Action |
|---|---|
| `↑` `↓` | Move through items |
| `→` | Expand a folder |
| `←` | Collapse, or go to the parent |
| `Home` / `End` | First / last item |
| `Enter` | Preview the item (and add if it is a sample) |
| `/` | Focus search |
| `Esc` | Clear search |

---

## 8. Computer keyboard as a piano

Two rows map to one octave; `Z` / `X` shift the octave.

| Row | Notes |
|---|---|
| `Z S X D C V G B H N J M` | C, C#, D, D#, E, F, F#, G, G#, A, A#, B |
| `Q 2 W 3 E R 5 T 6 Y 7 U` | C(+1), C#(+1), D(+1) … one octave above |
| `I 9 O 0 P` | D(+1), D#(+1), E(+1), F(+1) — upper accents |

Velocity follows the vertical position of the click when a key is struck with a pointer, and
a fixed 96 for keyboard strikes.

---

## 9. Focus discipline

- `Tab` moves between controls in DOM order. It never jumps columns or panes.
- Arrow keys move *within* a widget (a list, a strip, a knob).
- `Escape` closes the top overlay and returns focus to the control that opened it.
- `?` opens the shortcut sheet from anywhere; closing it returns focus.
- No control is reachable only by hovering.

---

## 10. Reduced motion

Every key still works. Springs collapse to fades. The playhead, meters, and level readouts
are exempt from reduced-motion suppression because they are truthful signal, not decoration.
