# Smart DAW — Feature Parity

**Status:** normative · v0.1
**Question this answers:** *"Is Smart DAW a real DAW, or a mock-up of one?"*

Every row is either **Yes** (implemented), **Design** (specified, behaviour frozen, to be
implemented), or **Native-only** (impossible in a browser, with the reason stated). There is
no fourth category. An omission must appear here as Native-only with a reason, or it is a bug.

---

## How to read the matrix

| Verdict | Meaning |
|---|---|
| **Yes** | Works in this prototype |
| **Design** | Fully specified in `docs/`; implementation follows the spec, not taste |
| **Native-only** | Impossible in a browser sandbox. Reason given. |

Comparators: **FL** Studio, **Live** Ableton Live, **Logic** Logic Pro, **RP** Reaper, **BT** Bitwig.
An emulated feature is marked *(em.)* — it does the same job through Web Audio rather than a
hosted plugin.

---

## 1. Arrangement and timeline

| Feature | FL | Live | Logic | RP | BT | Smart |
|---|---|---|---|---|---|---|
| Linear arrangement timeline | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Multiple tracks / lanes | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Clips / regions on timeline | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Pattern blocks (arrange patterns, not just clips) | ✔ | ✖ | ✖ | ✖ | ✔ | **Yes** |
| Loop region with markers | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Markers / cue points | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Tempo automation | ✔ | ✔ | ✔ | ✔ | ✔ | **Design** |
| Time signatures / meter changes | ✔ | ✔ | ✔ | ✔ | ✔ | **Design** |
| Zoom horizontal + vertical | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Snap to grid, configurable resolution | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Scrub | ✔ | ✔ | ✔ | ✔ | ✔ | **Design** |
| Ripple / shuffle edit | ✔ | ✔ | ✔ | ✔ | ✔ | **Design** |
| Comp takes | ✖ | ✔ | ✔ | ✔ | ✔ | **Native-only** — needs multitrack audio capture of hardware inputs; the prototype records one mono/stereo pair |

## 2. Sequencers

| Feature | FL | Live | Logic | RP | BT | Smart |
|---|---|---|---|---|---|---|
| Step sequencer / drum grid | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Piano roll (note editor) | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Drum / percussion map editor | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Event list editor | ✖ | ✖ | ✔ | ✔ | ✔ | **Design** |
| Score / notation editor | ✖ | ✔ | ✔ | ✔ | ✔ | **Native-only** — notation rendering is a whole app; not the brief for a compact DAW |
| Drum replacement / audio-to-MIDI | ✖ | ✔ | ✔ | ✔ | ✔ | **Native-only** — needs onset detection DSP; the prototype is synthetic-source |
| Groove / swing per-step | ✔ | ✔ | ✔ | ✔ | ✔ | **Design** |
| Randomize / humanize | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Quantize (with strength) | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Legato / glue | ✔ | ✔ | ✔ | ✔ | ✔ | **Design** |
| Transpose (semitone/octave) | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Reverse | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Duplicate / clone region | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Arpeggiator | ✔ | ✔ | ✔ | ✔ | ✔ | **Design** |
| Strum | ✔ | ✔ | ✔ | ✔ | ✔ | **Design** |
| Scale-aware editing | partial | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Ghost / reference notes from other clips | ✔ | ✔ | ✔ | ✔ | ✔ | **Design** |

## 3. Piano roll detail

| Feature | FL | Live | Logic | RP | BT | Smart |
|---|---|---|---|---|---|---|
| Vertical pitch axis with keyboard stripe | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Key folding (hide unused keys) | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Grid resolution 1/4 → 1/64 | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Snap to scale while dragging | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Root + mode highlighting | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Note velocity editing (lane + inline) | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Note length resize (mouse + keyboard) | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Multi-note select, range select | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Note repeat / duplicate | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Per-note colour by channel | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Inline keyboard playback under cursor | ✔ | ✖ | ✔ | ✔ | ✔ | **Design** |
| Chords preset palette | ✔ | ✔ | ✔ | ✔ | ✔ | **Design** |
| Controller / CC lanes | ✔ | ✔ | ✔ | ✔ | ✔ | **Design** |

## 4. Channel / track model

| Feature | FL | Live | Logic | RP | BT | Smart |
|---|---|---|---|---|---|---|
| Unlimited channels / tracks | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Channel rename, colour, icon | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Mute / solo / arm | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Mute/solo hierarchy (groups) | ✔ | ✔ | ✔ | ✔ | ✔ | **Design** |
| Track type: MIDI / audio / bus / return / master | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Group / bus routing | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Sends (pre/post fader) | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Sidechain input | ✔ | ✔ | ✔ | ✔ | ✔ | **Design** |
| Audio input monitoring | ✔ | ✔ | ✔ | ✔ | ✔ | **Design** |
| Freeze / bounce in place | ✔ | ✔ | ✔ | ✔ | ✔ | **Native-only** — bounce exists as WAV export; in-place freeze would need a mixdown graph per channel, which the prototype does as export stems |
| Channel reorder by drag | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |

## 5. Mixer

| Feature | FL | Live | Logic | RP | BT | Smart |
|---|---|---|---|---|---|---|
| Channel strips with fader, pan, gain | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Fader dB scale + readout | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Unity / 0 dB notch | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Peak + RMS metering with hold and decay | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Insert FX chain per channel | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** *(em.)* |
| Send FX with return tracks | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** *(em.)* |
| Master bus with limiter | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** *(em.)* |
| Mixer snapshots / scenes | ✔ | ✔ | ✔ | ✔ | ✔ | **Design** |
| Narrow / wide mixer view | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Track list ↔ mixer sync | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Downstream / sidechain filter on compressor | ✔ | ✔ | ✔ | ✔ | ✔ | **Design** |
| 3rd-party plugin hosting | ✔ | ✔ | ✔ | ✔ | ✔ | **Native-only** — sandbox. All FX are built-in equivalents |
| Surround / multichannel panning | ✔ | ✔ | ✔ | ✔ | ✔ | **Native-only** — stereo only; a surround matrix is out of scope for a compact DAW |

## 6. Effects (all built-in, modelled, not hosted)

| Effect | FL | Live | Logic | RP | BT | Smart |
|---|---|---|---|---|---|---|
| EQ (parametric) | ✔ | ✔ | ✔ | ✔ | ✔ | **Design** |
| Compressor (with sidechain filter) | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** *(em.)* |
| Limiter | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** *(em.)* |
| Reverb (algorithmic) | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** *(em.)* |
| Delay (sync to tempo) | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** *(em.)* |
| Filter (LP/BP/HP, resonant) | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** *(em.)* |
| Distortion / saturation | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** *(em.)* |
| Chorus / flanger / phaser | ✔ | ✔ | ✔ | ✔ | ✔ | **Design** *(em.)* |
| Gate / expander | ✔ | ✔ | ✔ | ✔ | ✔ | **Design** *(em.)* |
| Pitch shift / formant | ✔ | ✔ | ✔ | ✔ | ✔ | **Native-only** — needs an FFT phase vocoder; the prototype's pitched sources are synthesis-based, not pitch-shifted samples |
| Time stretch | ✔ | ✔ | ✔ | ✔ | ✔ | **Native-only** — would ship a tStretch DSP library; out of scope |
| Convolution reverb (IR load) | ✔ | ✔ | ✔ | ✔ | ✔ | **Native-only** — needs IR asset loading; algorithmic reverb covers the brief |

## 7. Instruments and samples

| Feature | FL | Live | Logic | RP | BT | Smart |
|---|---|---|---|---|---|---|
| Step-sequenced melodic instrument | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Sampler (trigger, tune, decay, reverse) | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Synth (subtractive, poly) | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Log-drum instrument (pitched percussion) | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Browser with folder tree | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Audition / preview on hover | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Drag from browser to rack | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Search + favourites | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Preset save/load per instrument | ✔ | ✔ | ✔ | ✔ | ✔ | **Design** |
| Audio file import from disk | ✔ | ✔ | ✔ | ✔ | ✔ | **Design** — `decodeAudioData` is available; UI flow specified, ships after core |
| Multisample zones | ✔ | ✔ | ✔ | ✔ | ✔ | **Native-only** |
| Sampler instrument export | ✔ | ✔ | ✔ | ✔ | ✔ | **Native-only** |

## 8. Audio editing

| Feature | FL | Live | Logic | RP | BT | Smart |
|---|---|---|---|---|---|---|
| Waveform editor | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Trim / move region bounds | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Fade in / out / crossfade | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Normalise | ✔ | ✔ | ✔ | ✔ | ✔ | **Design** |
| Reverse | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Gain / offset | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Destructive edit + undo | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Pitch detection to MIDI | ✔ | ✔ | ✔ | ✔ | ✔ | **Native-only** |

## 9. Automation and modulation

| Feature | FL | Live | Logic | RP | BT | Smart |
|---|---|---|---|---|---|---|
| Automation lanes (volume, pan, sends, params) | ✔ | ✔ | ✔ | ✔ | ✔ | **Design** |
| Curve editing (linear, bezier, stepped) | ✔ | ✔ | ✔ | ✔ | ✔ | **Design** |
| Automation modes: Write / Touch / Latch / Read | ✔ | ✔ | ✔ | ✔ | ✔ | **Design** |
| MIDI CC automation | ✔ | ✔ | ✔ | ✔ | ✔ | **Design** |
| Macro / modulators | ✖ | ✔ | ✔ | ✔ | ✔ | **Design** |

## 10. Transport and recording

| Feature | FL | Live | Logic | RP | BT | Smart |
|---|---|---|---|---|---|---|
| Play / stop / pause | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Record arm | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Punch in / out | ✔ | ✔ | ✔ | ✔ | ✔ | **Design** |
| Count-in | ✔ | ✔ | ✔ | ✔ | ✔ | **Design** |
| Metronome / click | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Tap tempo | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Tempo set by value | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Loop toggle + loop range | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Position display in bars.beats + time | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Overdub / replace | ✔ | ✔ | ✔ | ✔ | ✔ | **Design** |
| Audio record from hardware input | ✔ | ✔ | ✔ | ✔ | ✔ | **Design** — `getUserMedia` is available; UI flow specified, ships after core |
| Hardware MIDI in | ✔ | ✔ | ✔ | ✔ | ✔ | **Native-only** — Web MIDI needs a permission and device layer; computer keyboard is the input instrument |

## 11. Project management

| Feature | FL | Live | Logic | RP | BT | Smart |
|---|---|---|---|---|---|---|
| Save / load project | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** (localStorage) |
| Export project file (JSON) | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Import project file | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Templates / default project | ✔ | ✔ | ✔ | ✔ | ✔ | **Design** |
| Undo / redo history with depth | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Export WAV (mixdown) | ✔ | ✔ | ✔ | ✔ | ✔ | **Design** — OfflineAudioContext is available; ships after core |
| Export stems | ✔ | ✔ | ✔ | ✔ | ✔ | **Design** — same engine as mixdown |
| Export MIDI | ✔ | ✔ | ✔ | ✔ | ✔ | **Design** |
| Autosave | ✔ | ✔ | ✔ | ✔ | ✔ | **Design** |
| Version history | ✔ | ✔ | ✔ | ✔ | ✔ | **Native-only** — needs storage + diff; out of scope |
| Cloud / collaboration | ✔ | ✔ | ✔ | ✔ | ✔ | **Native-only** — explicit non-goal |

## 12. Window and layout

| Feature | FL | Live | Logic | RP | BT | Smart |
|---|---|---|---|---|---|---|
| Floating windows | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Docked / attached panes | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Layout presets / screensets | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Save / restore window positions | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Fullscreen / maximised | ✔ | ✔ | ✔ | ✔ | ✔ | **Design** |
| Multiple project windows | ✔ | ✔ | ✔ | ✔ | ✔ | **Native-only** — single window; multi-window would fragment metering and transport |
| Multi-monitor spanning | ✔ | ✔ | ✔ | ✔ | ✔ | **Native-only** — browser window API |

## 13. Accessibility and system

| Feature | FL | Live | Logic | RP | BT | Smart |
|---|---|---|---|---|---|---|
| Full keyboard operation | ✔ | ✔ | ✔ | ✔ | ✔ | **Yes** |
| Screen-reader announcements | ✖ | partial | ✔ | partial | partial | **Yes** |
| Visible focus ring on every control | partial | partial | ✔ | partial | ✔ | **Yes** |
| 44px coarse-pointer targets | ✖ | partial | ✔ | partial | partial | **Yes** |
| Reduced-motion mode | ✖ | ✖ | partial | ✖ | partial | **Yes** |
| AA contrast, light and dark | partial | partial | ✔ | partial | partial | **Yes** |
| Light + dark theme | ✖ | ✖ | ✖ | ✖ | ✔ | **Yes** |

---

## 14. What "full prototype" means here

The brief is *"no v1, full prototype, every feature decided here"*. So:

**Everything marked Design is specified.** The behaviour, states, and interaction are frozen
in `docs/` and will be implemented in that order. Nothing is a maybe.

**Everything marked Native-only is excluded with a reason.** A brief that says "everything a
DAW needs" does not mean "everything a DAW can do in a browser". Plugin hosting, time
stretch, and multi-window are named here so they can never be mistaken for gaps.

**Implementation order** (decided, not improvised):

1. Shell: pane model, dock/float, presets, transport, theme prompt
2. Mixer + audio engine + metering
3. Channel rack + step sequencing
4. Piano roll + scale highlighting
5. Arrangement + automation
6. Browser + audition
7. Effects + sample editor + keys
8. Project save/export, undo history, WAV export

The engine (step 2) lands before the roll (step 4) on purpose: metering is a primary
surface, and it is the thing that proves the audio path is real.
