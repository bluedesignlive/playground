# Smart DAW — Feature Parity Matrix

## vs FL Studio / Ableton / Logic / Reaper

| Feature | FL Studio | Ableton | Logic | Reaper | Smart DAW |
|---------|-----------|---------|-------|--------|-----------|
| Step sequencer | ✓ | — | — | — | ✓ |
| Piano roll | ✓ | ✓ | ✓ | ✓ | ✓ (scale highlight) |
| Arrangement view | ✓ | ✓ | ✓ | ✓ | ✓ |
| Mixer with faders | ✓ | ✓ | ✓ | ✓ | ✓ (skeuomorphic) |
| Transport controls | ✓ | ✓ | ✓ | ✓ | ✓ |
| Browser/library | ✓ | ✓ | ✓ | ✓ | ✓ (AI search) |
| Floating panes | ✓ | — | — | — | ✓ |
| Save/load projects | ✓ | ✓ | ✓ | ✓ | ✓ (JSON) |
| Undo/redo | ✓ | ✓ | ✓ | ✓ | ✓ |
| Keyboard shortcuts | ✓ | ✓ | ✓ | ✓ | ✓ (full parity) |
| Scale highlighting | — | — | — | — | ✓ (unique) |
| AI assistant | — | — | — | — | ✓ (unique) |
| Skeuomorphic knobs | — | — | — | — | ✓ (unique) |
| Dark/light mode | — | — | — | — | ✓ (unique) |
| 44px hit targets | — | — | — | — | ✓ (unique) |

## DAW-Specific Features

### Audio Engine
- Web Audio API for real-time audio processing
- Per-channel routing with gain and pan
- Drum synthesis (kick, snare, hi-hat)
- Note synthesis (sine wave with envelope)
- Metronome with accent

### MIDI/Note Editing
- Piano roll with full CRUD
- Scale highlighting (in-scale/out-of-scale/root)
- Snap-to-scale toggle
- Velocity editing (drag + double-click)
- Note resize (right edge drag)
- Note move (drag)
- Copy/paste notes
- Delete notes

### Step Sequencing
- 16-step grid per channel
- Per-step velocity (right-click to set)
- Mute/solo per channel
- Color-coded channels

### Mixing
- Skeuomorphic faders (light reflection)
- Gain/pan knobs (skeuomorphic)
- Per-channel meters
- Mute/solo/arm buttons
- FX insert slots
- Master strip (accent-ringed)

### Arrangement
- Pattern clips with colors
- Drag to reposition
- Timeline view with bars/beats

### AI Integration
- Command palette (Ctrl+K)
- Natural language commands
- AI history (auditable, undoable)
- Contextual AI pills
- AI mix suggestions

### Project Management
- JSON export/import
- Undo/redo (50 levels)
- Auto-save to localStorage (planned)

## Non-Goals (Explicit)
- No VST/AU plugin hosting (browser sandbox)
- No hardware MIDI input (Web MIDI possible but out of scope)
- No cloud/accounts/telemetry
- No collaborative editing
