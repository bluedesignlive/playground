# Smart DAW — Layout Specification

## Dock-and-Float Model

### Overview
Panes are window-like shells that are either **docked** (fixed slot in main grid) or **floating** (detached, draggable, restorable).

### Dock Slots
The main grid has 5 dock slots:
1. **Browser** (left, 280px wide)
2. **Channel Rack** (center-left)
3. **Piano Roll** (center-right, stretch)
4. **Transport** (bottom, full width)
5. **Mixer** (right, floatable)

### Floating Panes
- Drag pane header out of dock grid → floats
- Drag back into slot → docks
- Floating panes: position fixed, z-index 60, e4 shadow
- Docked panes: part of grid, e1 shadow

### Pane State Memory
Each pane remembers:
- Dock slot (if docked)
- Float position (if floating)
- Size (width/height)
- Scroll position
- Zoom level (piano roll)

### Keyboard Shortcuts
- Ctrl/Cmd+Shift+F: Toggle float/dock
- Ctrl/Cmd+T: Tile floating panes
- Ctrl/Cmd+P: Toggle playlist

### Layout Presets
- **Default**: Browser | Channel Rack | Piano Roll (transport bottom, mixer right)
- **Mixer Focus**: Mixer expanded, piano roll narrowed
- **Piano Focus**: Piano roll full width, channel rack collapsed
- **Arrangement**: Playlist visible below piano roll

### Responsive Behavior
- < 1024px: Browser hidden, 2-column grid
- < 768px: Channel rack hidden, 1-column grid

### Grid Template
```css
.daw-grid {
  display: grid;
  grid-template-columns: 280px 1fr 1fr;
  grid-template-rows: 1fr auto 1fr;
  grid-template-areas:
    "browser channel piano"
    "browser transport piano"
    "browser mixer piano";
  gap: 12px;
  padding: 12px;
}
```
