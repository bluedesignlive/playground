# Smart DAW — Component Anatomy

## 1. Pane (Floating/Docked)

### Anatomy
- Header (32px): drag handle + title + actions (float/dock + close)
- Content area: scrollable, padding 8px

### States
- Default: e1 shadow + rim-1
- Hover: e2 shadow + rim-2
- Focus: visible focus ring
- Floating: e4 shadow + rim-4, position fixed
- Docked: e1 shadow + rim-1, part of grid

### Sizes
- Header height: 32px (row_height)
- Min width: 200px
- Min height: 150px

### A11y
- `role="region"`, `aria-label` with pane name
- Header is drag handle
- Close button has `aria-label="Close"`
- Float/dock button has `aria-label="Float/dock"`

## 2. Transport Strip

### Anatomy
- Play/stop/record/loop/metronome buttons (40×40px)
- BPM field (pill shape)
- Song position readout
- Loop range display
- Loop in/out buttons
- Export/import buttons
- AI pill

### States
- Play: active (accent background)
- Record: active (danger background)
- Loop: active (accent background)
- Metro: active (accent background)

### A11y
- All buttons have `title` attribute
- BPM input has `aria-label="BPM"`
- Playhead has `aria-hidden="true"`

## 3. Channel Rack

### Anatomy
- Channel row: color chip + name + mute/solo
- Step grid: 16 columns × N rows
- Each step: 14×36px, toggle on/off
- Velocity shown as opacity

### States
- Step on: accent background, opacity = velocity/100
- Step off: control background, opacity 0.3
- Hover: control-hover background
- Mute active: ink-1 background
- Solo active: warn background

### A11y
- Steps are buttons with `aria-pressed`
- Mute/solo are buttons with `aria-pressed`

## 4. Piano Roll

### Anatomy
- Keyboard: 48px wide, C-1 to G9
- Grid: 48 rows × 16 bars
- Notes: draggable, resizable, velocity-editable
- Ruler: bar numbers

### States
- Key (white): #fff background
- Key (black): ink-1 background
- Key (scale-on): accent-tint-2
- Key (scale-root): accent-tint-3 + accent ring
- Key (selected): accent background
- Note (default): channel color + accent glow
- Note (selected): accent ring + 2px accent border
- Note (hover): lift + stronger shadow

### A11y
- Keyboard keys are clickable
- Notes are draggable with keyboard support
- Scale toggle has `aria-pressed`

## 5. Knob (Skeuomorphic)

### Anatomy
- Base: control surface + rim-1
- Pointer: 3px wide, 16px tall, ink-1
- Value ring: accent tint arc (opacity 0→0.7 on active)
- Top highlight: radial gradient at 30% 30%
- Bottom shade: radial gradient at 70% 70%

### States
- Default: rim-1 + e1
- Hover: rim-1 + e2
- Active: scale 0.96 + rim-inset
- Focus: value ring visible (opacity 0.7)

### Sizes
- Diameter: 44px (touch target)
- Pointer height: 16px

### A11y
- `aria-label` with knob name
- `aria-valuenow` with current value
- Keyboard: up/down to turn, Enter to reset

## 6. Fader (Skeuomorphic)

### Anatomy
- Track: 34px wide, 160px tall, rim-well
- Thumb: 30×34px, raised pill, top highlight
- Value readout: 11.5px mono

### States
- Thumb (default): rim-1 + e1
- Thumb (hover): rim-1 + e2
- Thumb (active): rim-inset + scale 0.95
- Double-click: reset to 0 dB

### A11y
- `aria-label` with fader name
- `aria-valuenow` with dB value
- Keyboard: up/down to adjust

## 7. Mixer Strip

### Anatomy
- Channel name (rotated -90°)
- Meter (100px tall)
- Fader (160px tall)
- Gain/pan knobs (2× 44px)
- Mute/solo/arm buttons
- FX insert slot

### States
- Master: accent border + rim-accent
- Mute active: ink-1 background
- Solo active: warn background
- Arm active: danger background

## 8. AI Pill

### Anatomy
- Icon (20px circle, accent background)
- Label (meta size, ink-2)
- Hover: border accent + e3 shadow

### States
- Default: rim-2 + e2
- Hover: rim-accent + e3
- Ring: accent border + accent-tint-3 glow (1.2s pulse)

## 9. Command Palette

### Anatomy
- Search input (body size)
- Results list (scrollable)
- Each item: icon + label + shortcut

### States
- Item (default): ink-1 text
- Item (hover): row-hover background
- Selected: accent-tint background

## 10. Playlist Clip

### Anatomy
- Background: channel color
- Label: channel name + "pattern"
- Loop handle: right edge, 6px wide

### States
- Default: e1 shadow + rim-1
- Selected: accent ring + accent-glow
- Hover: lift + e2 shadow
- Dragging: cursor grabbing

### A11y
- `role="button"`, `aria-label` with clip name
- Keyboard: arrow keys to move, Delete to remove
