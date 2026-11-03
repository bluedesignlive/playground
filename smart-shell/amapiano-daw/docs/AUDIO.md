# Smart DAW — Audio Engine Architecture

## Overview

The audio engine uses the Web Audio API for real-time audio processing. All audio is synthesized (no samples loaded) for the prototype, with mock WAV files available for future integration.

## Signal Flow

```
Channel Source → Channel Gain → Channel Pan → Master Gain → Analyser → Destination
```

## Components

### AudioContext
- Created on first user interaction (browser requirement)
- Sample rate: system default (usually 44100 Hz)
- Latency hint: 'interactive'

### Channel Routing
Each channel has:
- Gain node (volume control)
- StereoPanner node (pan control)
- Connected to master gain

### Synth Voices

#### Drums
- **Kick**: Sine wave, 150→30 Hz exponential ramp, 400ms decay
- **Snare**: Square wave (180 Hz) + white noise, 150ms decay
- **Hi-Hat**: Square wave (8000 Hz), 60ms decay

#### Melodic
- **Bass/Chords/Lead/Pad**: Sine wave, ADSR envelope
- Frequency: 440 × 2^((midi-69)/12)
- Attack: 50ms, Decay: note duration

#### Metronome
- Accent: Sine wave (1000 Hz), 50ms decay
- Normal: Square wave (500 Hz), 50ms decay

### Scheduling

The transport uses `requestAnimationFrame` for timing:
- Position updated every frame
- Step triggers at 1/16 note boundaries
- Note triggers when playhead crosses note start
- Metronome triggers on beat boundaries

### Metering
- Analyser node on master output
- FFT size: 256
- Frequency data used for level display
- Updated every animation frame

## Future Enhancements
- Sample loading from mocks/audio/
- Offline rendering for export
- MIDI input support
- Plugin hosting (Web Audio effects)
- Automation lanes
- Sidechain compression
