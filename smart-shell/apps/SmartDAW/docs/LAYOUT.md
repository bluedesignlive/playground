# Smart DAW — Layout

**Status:** normative · v0.1
**Scope:** the pane model, the dock grid, the presets, and the drag gestures that move panes
between them.

---

## 1. The problem with FL's windows

FL Studio's window management is powerful and hostile. You can undock anything, float it
anywhere, resize it to nothing, and lose it forever behind another window. New producers
solve this by never undocking anything. Power users solve it by spending an afternoon
building a layout and saving it as a project.

Smart DAW takes a different position: **the layout is a preset you choose, not a puzzle you
solve.** Docking and floating still exist, and they still work — but the first-class answer
to "I want to work on drums now" is switching to a preset, not rearranging panes.

This is the single biggest behavioural difference from FL and the reason the app reads as
modern.

---

## 2. The pane model

A pane is a surface with:

- a 32px header containing the title, that pane's controls, and an overflow menu
- a body that holds that pane's content
- a dock state: `docked` or `floating`
- a slot assignment when docked
- remembered size, scroll, and zoom

### Pane anatomy

```
┌───────────────────────────────────────────┐
│ ⣿ drag handle   Pane title   [c][c] ⋯   │ ← 32px header, raised, e1 + rim-1
├───────────────────────────────────────────┤
│                                           │
│  body — flat, no elevation, no borders    │
│                                           │
└───────────────────────────────────────────┘
```

- The whole header is the drag handle except its buttons.
- Header buttons: pane-specific controls + a `⋯` overflow + a float/dock toggle.
- Dragging the header out of the grid floats the pane.
- Dragging a floating pane's header over a dock slot highlights the slot; releasing docks it.
- Clicking a floating pane's header raises it (z-order).
- Docked panes never overlap. Floating panes always float above the stage.

### Dock vs floating lighting

| State | Shadow | Rim | Rationale |
|---|---|---|---|
| Docked | `e1` | `rim-1` | Part of the stage, not in front of it |
| Floating | `e4` | `rim-4` | Temporarily in front — earns the stronger elevation |

A floating pane carries a faint backdrop blur behind its *header only* (12–16px), never
behind text. Text surfaces are near-opaque.

### Remembered state

Every pane remembers: dock slot, size, float position, zoom, scroll. Restored on reopen and
persisted with the project. Losing your scroll position when you re-open the piano roll is a
small betrayal; the prototype does not do it.

---

## 3. The default dock grid

Five dock slots plus a transport spine that never floats.

```
┌──────────────────────────────────────────────────────────────────────────┐
│  TOPBAR  project · undo/redo · presets · theme · master          ⌘K    │
├────────────┬──────────────┬───────────────────────────────┬──────────────┤
│            │              │                               │              │
│  Browser   │ Channel rack │       Piano roll              │    Mixer     │
│  220px     │  300px       │       flexible                │   260px      │
│  (SLOT_L)  │  (SLOT_CL)   │       (SLOT_CR)               │  (SLOT_R)    │
│            │              │                               │              │
│            │              │                               │              │
│            ├──────────────┴───────────────────────────────┤              │
│            │           Arrangement                       │              │
│            │           flexible height                   │              │
├────────────┴──────────────┴───────────────────────────────┴──────────────┤
│  TRANSPORT  ◀ ■ ● · 112.0 BPM  3.2.1  1.1.1—5.1.1  -6.2 dB   [undo 12]  │
└──────────────────────────────────────────────────────────────────────────┘
```

### Slot rules

| Slot | Min width | Resizable | Default content |
|---|---|---|---|
| `TOP` | none | height 44px | topbar — never a pane |
| `LEFT` | 180px | horizontal | Browser |
| `CENTER_LEFT` | 240px | horizontal | Channel rack |
| `CENTER_RIGHT` | 320px | horizontal | Piano roll |
| `RIGHT` | 220px | horizontal | Mixer |
| `BOTTOM` | none | vertical | Arrangement |
| `TRANSPORT` | none | height 52px | transport — never a pane |

- The `CENTER_RIGHT` slot absorbs all leftover width; every other slot is fixed or
  user-resized. This keeps the piano roll and arrangement — the surfaces that need room —
  from being squeezed by the mixer.
- Resizing one slot's edge shrinks `CENTER_RIGHT`. You cannot starve the primary surface.
- The transport bar spans the full window and is never floatable. Losing the transport is
  the failure mode that makes a DAW unusable.

---

## 4. The four presets

Each preset is a stored arrangement of panes across the slots. Switching is one click or one
key; the change animates with a spring over `dur-3`.

### Compose — the default

`Browser ← | Channel rack ← | **Piano roll** ↑ | Mixer →`, arrangement small at the bottom.

For writing chords, stabs, and log-drum melodies. The piano roll gets the maximum
share because it is where amapiano's melodic content is written. Arrangement shrinks to a
stub so the timeline is still visible without stealing space.

### Program

`**Channel rack** ← | Piano roll → | Browser | Mixer`, arrangement collapsed.

For drum programming and one-shot audition. The rack gets the wide share; the browser is
narrowed to a column of items you can audition; the roll is small but present so a stab can
be checked without a preset switch.

### Arrange

`**Arrangement** ↑ (full width) | Channel rack | Browser`, transport prominent.

For building the song — placing patterns, drawing automation, setting the loop region. The
arrangement takes the whole middle and the rack drops to a strip. Mixer floats by default in
this preset so gain staging happens beside the timeline on a second screen.

### Mix

`**Mixer** → (wide) | Arrangement ← | Channel rack ←`, browser closed by default.

For mixing. The mixer takes the right half at full width with skeuomorphic faders, the
arrangement sits beside it so automation is visible while you ride the fader, and the
browser is closed outright — it is noise while mixing.

### Preset switching behaviour

1. The current layout is saved as a "last" snapshot.
2. Panes animate to their new slots over 360ms with a spring.
3. Panes that are floating stay floating across a preset switch, unless the preset
   explicitly floats that pane — presets never force-close your work.
4. Preset choice persists with the project.

**Why four and not a slider:** a DAW user changes task roughly every few minutes. The
preset is the unit of that change. Anything finer is a power-user action.

---

## 5. Gestures

| Gesture | Result |
|---|---|
| Drag pane header | Move. Out of grid → float. Over a slot → highlight; release → dock |
| Drag slot divider | Resize slot; `CENTER_RIGHT` absorbs or returns space |
| Drag browser item → rack | Add channel / trigger instrument |
| Drag browser item → piano roll | Place audio or instrument channel |
| Drag rack channel row | Reorder channel (spring, `dur-3`) |
| Double-click pane header | Maximise pane within its slot (toggle) |
| Drag floating pane near another | Cascade or tile by proximity |
| `⌘T` | Tile floating panes |
| `⌘⇧F` | Float / dock the focused pane |
| Scroll over piano roll | Zoom vertical (pitch) / horizontal (time) |
| Scroll over a knob | Turn the knob |
| Scroll over mixer | Scroll strips horizontally (never change a value) |

**Scroll discipline:** every scrollable surface has one meaning. Mouse wheel over the mixer
never rescales a fader; wheel over a knob never scrolls the page. A scroll that moves two
things at once is a bug.

---

## 6. Compact and small surfaces

At 1024px width the pane set cannot hold. The order of sacrifice is decided, not improvised:

1. Browser collapses to icons, then closes.
2. Channel rack shrinks to 240px.
3. Mixer narrows to 220px and loses its send row (sends move to overflow).
4. Arrangement keeps its full width — it is the last thing to give.
5. Below 900px the DAW switches to a single-pane mode with a pane switcher in the transport.

At 390px the prototype is not usable as a DAW and says so rather than pretending — a phone
is not where you write amapiano. It becomes an audition remote: transport + channel rack.

---

## 7. First-run

On first load the user picks a theme (light or dark) and the choice persists. No
config file, no settings modal. After the choice the app opens directly in the Compose
preset.

---

## 8. Decisions frozen

1. Transport never floats and never closes.
2. `CENTER_RIGHT` absorbs resize; other slots resize against it.
3. Four presets; switching animates with a spring; presets never close floating panes.
4. Docked pane = `e1`/`rim-1`. Floating pane = `e4`/`rim-4`.
5. Zero hairlines between panes; tone and one seam separate them.
6. Pane header 32px, gap 12px, padding 12px.
7. Scroll has one meaning per surface.
8. Preset + theme persist with the project.
