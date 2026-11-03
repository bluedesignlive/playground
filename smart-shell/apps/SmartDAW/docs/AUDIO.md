# Smart DAW — Audio Engine

**Status:** normative · v0.1
**Scope:** how sound is produced, scheduled, routed, and metered.

The audio engine is not the point of this prototype — the design is. But the design claims to
be a DAW, and a DAW that cannot make sound is a picture of one. This engine is real, small,
and dependency-free.

---

## 1. Graph

```
per channel:
  source → [insert chain] → stripGain → panner ─┐
                                                ├→ masterGain → limiter → destination
  source ─→ sendA ─────────────────────────────→ returnA (reverb) ─┐
  source ─→ sendB ─────────────────────────────→ returnB (delay) ──┤
                                                ↑                  │
                                                └──────────────────┘
per channel:
  stripGain → analyser → (metering only, not in the audible path)
```

- Inserts run per channel in order; each is a pair of `BiquadFilter`/`DynamicsCompressor`/
  `ConvolverNode`-free algorithmic nodes.
- Sends are pre- or post-fader. Returns are ordinary channels whose destination is the master.
- Master bus carries the last insert (a `DynamicsCompressor` used as a limiter) and the master
  level knob.

---

## 2. Scheduling model

The standard **lookahead scheduler**, because `setInterval` drifts and `requestAnimationFrame`
is throttled when the tab is hidden.

| Constant | Value | Reason |
|---|---|---|
| `LOOKAHEAD` | 0.10 s | tolerance for late timer wakeups |
| `SCHEDULE_AHEAD` | 0.20 s | how far ahead notes are committed |
| `TICK` | 25 ms | timer granularity |

```
tick every 25ms:
  while (nextNoteTime < audioCtx.currentTime + SCHEDULE_AHEAD):
    schedule all events whose time falls in (nextNoteTime, nextNoteTime + step]
    nextNoteTime += stepDuration
  draw the playhead from audioCtx.currentTime (linearly, never with a spring)
```

The playhead is drawn from `audioContext.currentTime` every animation frame, so it is accurate
even when the scheduler timer is throttled. **The playhead never derives from the scheduler's
own clock** — that would make tempo lie.

A note scheduled at `t` is created at `t` and stopped at `t + duration`. Because the whole step
is created ahead of time, a long sustain is held by `setValueAtTime` + `linearRampToValueAtTime`
on the gain, not by JS state.

---

## 3. Voices

All synthesis is done in JS — no sample assets ship, so the prototype stays self-contained and
the browser makes no network requests.

### 3.1 Log drum (the amapiano workhorse)

A pitched percussion voice:

```
osc (triangle) → pitchEnv (exponential, 40ms, −6 semitones down)
               → gainEnv (percussive: 4ms attack, 120ms decay)
               → [body filter, lowpass sweeping 3.2kHz → 400Hz over 90ms]
               → output
```

`freq` is MIDI-derived, so a log-drum channel doubles as a bass and a tom. That is exactly how
it is used in amapiano: a melodic percussion line. It is the reason log-drum gets its own voice
rather than being a "kick" sample.

### 3.2 Piano / stab

A subtractive poly voice with a fast attack:

```
2 osc (triangle + sine, detuned ±3 cents) → lowpass 1.8kHz
              → gainEnv (6ms attack, exponential decay per note length)
              → a touch of stereo spread via a slight per-voice detune
              → output
```

Good enough for the deep-house chords and log-drum melodic runs that the piano roll exists to
write. Not a sampled grand, and it does not pretend to be.

### 3.3 Drums

All noise- or click-based, one-shots:

| Drum | Construction |
|---|---|
| Kick | sine, 120→55 Hz over 90ms, 200ms decay, click transient |
| Snare | noise burst (bandpass 1.8kHz) + a 180 Hz body, 180ms decay |
| Clap | 3 noise bursts at 0, 9, 18 ms, bandpass 1.1 kHz, 140ms tail |
| Shaker | white noise → highpass 6 kHz, 45ms decay |
| Rim | noise + a 2 kHz resonant band, 25 ms |
| Hat (open/closed) | noise → highpass 8 kHz, 80 / 220 ms decay |

Noise comes from a shared `AudioBuffer` of white noise created once at init.

---

## 4. Effects

Every effect is a chain of native `AudioNode`s. All parameters are exposed as knobs.

| FX | Nodes | Params |
|---|---|---|
| EQ (3-band) | 3 `BiquadFilter` (low shelf / peaking / high shelf) | low gain, mid gain+Q, high gain |
| Compressor | `DynamicsCompressor` | threshold, ratio, attack, release, makeup |
| Limiter | `DynamicsCompressor` (ratio 20, fast attack) | ceiling, release |
| Gate | `DynamicsCompressor` with a low threshold | threshold, attack, hold |
| Reverb | a programmatically generated impulse response → `ConvolverNode` | size, decay, mix, predelay |
| Delay | `DelayNode` + feedback gain + lowpass in the loop | time (synced), feedback, mix, ping-pong |
| Filter | `BiquadFilter` (LP/BP/HP selectable) | cutoff, resonance, drive |
| Saturation | a `WaveShaperNode` with a computed tanh curve | drive, mix |
| Chorus | two `DelayNode`s with LFO-modulated delay time | rate, depth, mix |
| Phaser | a chain of 4 allpass `BiquadFilter`s + LFO on frequency | rate, depth, feedback |

**Reverb IRs are generated**, not loaded: exponentially decaying noise with a shape chosen by
"size". It costs a few milliseconds at init and keeps the prototype asset-free.

---

## 5. Metering

Each channel strip owns an `AnalyserNode` fed from its strip gain (a parallel tap, not in the
audible path).

- `fftSize = 1024`, read as a time-domain buffer.
- Peak = max absolute sample in the window.
- RMS = `sqrt(mean(x²))` over the window.
- dB = `20 * log10(value)` clamped to −60…+6.
- Peak hold: hold for 1400 ms, then fall at 20 dB/s.

**Meter visual ≠ spring.** The bar follows the audio frame. It collapses instantly when
transport stops — a meter that keeps decaying after the signal is gone is a lie.

The read is done in `requestAnimationFrame`, one loop for all strips, sharing the analysis
buffer. This is the only per-frame work in the app.

---

## 6. Channel routing

```
channel
  ├─ source (synth voice / drum)
  ├─ inserts: [eq?, comp?, sat?, delay?, reverb?]
  ├─ gain (fader, dB)
  ├─ panner (StereoPanner)
  ├─ sends: [ [returnA, amount], [returnB, amount] ]
  └─ analyser
```

Return channels are ordinary channels with no source; they hold the reverb and delay inserts
and feed the master.

The master channel has no fader below −∞ (it stops at 0 dB and above only limits) — the master
goes into the limiter, so clipping there is a mix problem, not a level problem.

---

## 7. Export

`OfflineAudioContext` renders the same graph over the song length, then `encodeWAV` writes a
16-bit PCM WAV and triggers a download. Stems are the same render with one channel soloed.

**Export never touches the live graph** — it builds a parallel one, so the user can keep
working while a bounce runs.

---

## 8. Performance budget

| Target | Value |
|---|---|
| Voices in flight | ~24 polyphony per channel; 6 channels = 144 oscillator nodes worst case |
| Nodes per channel | ≈ 12 |
| Analysers | one per strip, one per return, one on master |
| Audio thread work per tick | O(events in the current step) — typically 0–8 |

Voice nodes are re-used from a free list rather than created and dropped, because allocation
in the audio callback path is the classic Web Audio glitch source. A voice returns to the pool
on its `gainEnv` `setValueAtTime` end via `onended`.

---

## 9. Failure modes and honesty

| Situation | Behaviour |
|---|---|
| AudioContext suspended | a single persistent banner: "Click anywhere to enable audio" — never a silent failure |
| Tab hidden | scheduler keeps running (timers throttle but lookahead absorbs it); playhead is drawn from `currentTime` on return |
| Analyser unavailable | meters fall back to a synthesised guess from scheduled events (marked in the UI) |
| Device with no audio output | everything still schedules; only the banner shows |

No spinner, no fake progress, no "loading…". Either the engine is running or the app says why.
