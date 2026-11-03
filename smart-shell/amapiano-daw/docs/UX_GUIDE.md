# Smart DAW — UX Guide

## Design Contract

This document describes the DAW-specific interaction laws, derived from Smart Design principles.

### 1. Product Premise

**Smart DAW** makes making amapiano easy. Amapiano is log-drum driven, percussive, melodic (deep house chords + piano rolls), and arrangement-heavy with long builds and drops.

### 2. Design Philosophy

| Smart Law | DAW Interpretation |
|-----------|-------------------|
| One light | Every raised pane uses one cast shadow + one rim. Knobs use one top highlight + one rim — never stack highlights. |
| Depth is semantic | Pane headers are interactive (draggable/dockable); content rows stay flat. Knobs/faders are raised controls. |
| One accent signal | Selected channel, selected piano-roll note, and active transport use ONE accent (cobalt). |
| Selection is not hover | A selected note/channel persists with a ring + tint; hover is only a preview. |
| Text is not glass | Transport labels, mixer fader dB readouts, scale names are near-opaque; blur is atmosphere only. |
| No invisible controls | Every knob has a name, focus ring, 44px hit area, and tooltip. |
| Motion explains | Pane dock/undock, note nudge, scale change use springs; transport playhead moves linearly. |
| Timelessness | No neon, no glassmorphism stack, no oversized type. Gradients exist only inside knobs/faders as material reflection. |

### 3. Layout Behavior

#### Pane Model
Panes are window-like shells that are either **docked** (fixed slot in main grid) or **floating** (detached, draggable, restorable).

- One glass plane per pane, one seam, one outer rim
- Floating panes cast e4; docked panes cast e1
- Pane header = drag handle + close + float/dock toggle
- Panes remember: dock slot, size, float position, scroll, zoom

#### Default Dock Layout
```
┌─────────────────────────────────────────────────────────────┐
│ Browser  │  Channel Rack  │         Piano Roll              │
│ (dock L) │  (dock C-L)    │         (dock C-R, stretch)     │
│          │                │                                 │
│          │  Transport     │                                 │
│          │  (dock bottom) │                                 │
│          │                │  Mixer                          │
│          │                │  (dock right, floatable)        │
└─────────────────────────────────────────────────────────────┘
```

#### Compactness Rules
- Pane header height: 32px (row_height)
- Pane content gap: 12px (gap_md)
- No borders between docked panes — separation by tone (hair) and one seam
- All controls minimum 44px hit area
- Typography floors: labels 10.5px, meta 11.5px, body 13px
- Fader dB readouts = 11.5px mono

### 4. Skeuomorphic Layer

Reserved exclusively for knobs and faders. Never applied to panes or text.

#### Knob Anatomy
- Base: control surface + rim_1 (top white highlight, bottom dark line)
- Pointer: small dot/line at current angle
- Value ring: accent tint arc (one accent signal)
- Pressed: control_press + scale 0.98 + inner shade
- Hover: control_hover + slight lift (e2)
- Turn: click-drag vertical OR scroll wheel
- Double-click = reset to default

#### Fader Anatomy
- Vertical travel with top highlight + bottom rim
- Thumb is raised pill with its own top highlight
- Track groove = rim_well
- Double-click = unity gain (0 dB)

### 5. Interaction Model

#### Pointer
- Left-click: activate/select
- Right-click: context menu
- Scroll: zoom piano roll; over knob = turn
- Drag: pane headers, piano-roll notes, fader thumb, knob turn

#### Keyboard
- Space: Play/stop
- R: Record arm/toggle
- L: Loop toggle
- I/O: Set loop in/out
- T: Tap tempo
- Tab: Cycle focus between panes
- Arrow keys: Move selected note
- Shift+arrow: Resize selected note
- S: Toggle scale highlight
- K: Open scale selector
- Ctrl/Cmd+K: Command palette (AI)
- Ctrl/Cmd+P: Toggle playlist
- Ctrl/Cmd+Shift+F: Float/dock active pane
- Esc: Close menus, return focus
- Delete/Backspace: Remove selection
- Ctrl/Cmd+Z/Y: Undo/redo

#### Touch
- 44px minimum hit targets everywhere
- No hover-only controls
- Long-press = context menu
- Pinch to zoom piano roll

### 6. Motion

| Transition | Duration | Easing | Use |
|------------|----------|--------|-----|
| Pane dock/undock | 360ms | Spring | Spatial |
| Note nudge/resize | 140ms | Ease-out | Spatial |
| Scale change re-highlight | 220ms | Ease-in-out | State |
| Transport playhead | Linear per beat | — | Continuous |
| Menu open | 220ms | Ease-out | Overlay |
| AI result landing | 220ms | Ease-out | State |
| Reduced motion | Fade only | — | Fallback |
