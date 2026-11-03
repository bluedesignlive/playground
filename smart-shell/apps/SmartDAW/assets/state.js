/* ==========================================================================
   Smart DAW — project state
   The musical data model. Everything the UI renders comes from here.
   ========================================================================== */

export const SCALES = {
  Major: [0, 2, 4, 5, 7, 9, 11],
  Minor: [0, 2, 3, 5, 7, 8, 10],
  Dorian: [0, 2, 3, 5, 7, 9, 10],
  Phrygian: [0, 1, 3, 5, 7, 8, 10],
  Lydian: [0, 2, 4, 6, 7, 9, 11],
  Mixolydian: [0, 2, 4, 5, 7, 9, 10],
  Locrian: [0, 1, 3, 5, 6, 8, 10],
  'Harmonic minor': [0, 2, 3, 5, 7, 8, 11],
  'Melodic minor': [0, 2, 3, 5, 7, 9, 11],
  'Minor pentatonic': [0, 3, 5, 7, 10],
  'Major pentatonic': [0, 2, 4, 7, 9],
  Blues: [0, 3, 5, 6, 7, 10],
  'Whole tone': [0, 2, 4, 6, 8, 10],
};

export const ROOTS = ['C', 'C♯', 'D', 'D♯', 'E', 'F', 'F♯', 'G', 'G♯', 'A', 'A♯', 'B'];
export const ROOT_ALIAS = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'];

/* channel identity colours — never the accent. Cobalt stays selection. */
export const CH_COLORS = ['#c8573f', '#c9861a', '#2b9b6d', '#2f6feb', '#7a5cd0', '#c2557a', '#3f8fa8', '#8a6f3d'];

export const STEPS_PER_BAR = 64;    /* at the default grid of 16 (1/16 notes) */
export const NOTE_MIN = 24;         /* C-1 */
export const NOTE_MAX = 108;        /* C8+ */

let uid = 1;
export const nid = p => `${p}${uid++}`;

export function scaleNotes(root, mode) {
  const s = SCALES[mode] || SCALES.Minor;
  const set = new Set();
  for (let oct = 0; oct < 12; oct++) for (const i of s) set.add((root + i + oct * 12) % 12);
  return set;
}

export function inScale(pitch, root, mode) {
  return scaleNotes(root, mode).has(((pitch % 12) + 12) % 12);
}

/* snap a pitch to the nearest scale degree, searching outward */
export function snapToScale(pitch, root, mode) {
  if (inScale(pitch, root, mode)) return pitch;
  for (let d = 1; d <= 6; d++) {
    if (inScale(pitch + d, root, mode)) return pitch + d;
    if (inScale(pitch - d, root, mode)) return pitch - d;
  }
  return pitch;
}

/* ============================ channel factory ============================ */

export function makeChannel(spec) {
  return {
    id: spec.id || nid('ch'),
    name: spec.name,
    kind: spec.kind,
    color: spec.color,
    note: spec.note != null ? spec.note : 60,
    gain: 0, pan: 0, mute: false, solo: false, armed: false,
    params: Object.assign({}, spec.params || {}),
    pattern: new Array(64).fill(0),
    notes: [],
    inserts: [],
    sends: [],
  };
}

export function defaultChannels() {
  return [
    makeChannel({ name: 'Log Drum', kind: 'logdrum', color: CH_COLORS[0], note: 36, params: { decay: 0.30 } }),
    makeChannel({ name: 'Bass', kind: 'saw', color: CH_COLORS[1], note: 28, params: { decay: 0.35 } }),
    makeChannel({ name: 'Shaker', kind: 'shaker', color: CH_COLORS[2] }),
    makeChannel({ name: 'Clap', kind: 'clap', color: CH_COLORS[3] }),
    makeChannel({ name: 'Rim', kind: 'rim', color: CH_COLORS[4] }),
    makeChannel({ name: 'Kick', kind: 'kick', color: CH_COLORS[5] }),
    makeChannel({ name: 'Open Hat', kind: 'ohat', color: CH_COLORS[6] }),
    makeChannel({ name: 'Piano Stab', kind: 'piano', color: CH_COLORS[7], note: 60, params: { decay: 0.55 } }),
  ];
}

export function defaultReturns() {
  return [
    {
      id: 'reverbA', name: 'Reverb A', gain: 0, inserts: [
        { id: 'reverb', bypass: false, params: [
          { id: 'size', value: 1.8 }, { id: 'decay', value: 2.6 },
          { id: 'predelay', value: 0.012 }, { id: 'lowcut', value: 280 }, { id: 'mix', value: 1 },
        ] },
      ],
    },
    {
      id: 'delayA', name: 'Delay A', gain: 0, inserts: [
        { id: 'delay', bypass: false, params: [
          { id: 'time', value: 0.5595 }, { id: 'feedback', value: 0.42 },
          { id: 'lowpass', value: 4200 }, { id: 'ping', value: 0.6 }, { id: 'mix', value: 1 },
        ] },
      ],
    },
  ];
}

/* ============================ project ============================ */

export const project = {
  name: 'Untitled — Amapiano Sketch',
  tempo: 112,
  timeNum: 4,
  timeDen: 4,
  bars: 16,
  stepsPerBeat: 16,
  scale: { root: 9, mode: 'Minor', on: true },
  loop: { on: false, start: 0, end: 8 },
  playhead: 0,          /* in steps from song start */
  playing: false,
  recording: false,
  metronome: false,
  preset: 'compose',
  channels: defaultChannels(),
  returns: defaultReturns(),
  master: { gain: -1.5 },
  clips: [],
  selChannel: 0,
  selNotes: [],
  selectedSteps: [],
};

export const totalSteps = () => project.bars * project.timeNum * project.stepsPerBeat;
export const barSteps = () => project.timeNum * project.stepsPerBeat;
export const stepsPerBar = STEPS_PER_BAR;

export function beatDur() { return 60 / project.tempo; }
export function stepDur() { return beatDur() / project.stepsPerBeat; }
export function barDur() { return beatDur() * project.timeNum; }

export function stepToLabel(step) {
  const spb = project.stepsPerBeat;
  const bar = Math.floor(step / (spb * project.timeNum)) + 1;
  const beat = Math.floor((step % (spb * project.timeNum)) / spb) + 1;
  const tick = (step % spb) + 1;
  return `${bar}.${beat}.${tick}`;
}

export function stepToTime(step) { return step * stepDur(); }

/* ============================ undo ============================ */

const UNDO_LIMIT = 120;
export const history = { stack: [], idx: -1 };

function musical() {
  return JSON.stringify({
    channels: project.channels,
    returns: project.returns,
    clips: project.clips,
    scale: project.scale,
    master: project.master,
  });
}

export function snapshot(force) {
  const s = musical();
  if (!force && history.stack[history.idx] === s) return;
  history.stack = history.stack.slice(0, history.idx + 1);
  history.stack.push(s);
  if (history.stack.length > UNDO_LIMIT) history.stack.shift();
  history.idx = history.stack.length - 1;
  dirty();
}

export function undo() {
  if (history.idx <= 0) return false;
  history.idx--;
  restore(history.stack[history.idx]);
  return true;
}
export function redo() {
  if (history.idx >= history.stack.length - 1) return false;
  history.idx++;
  restore(history.stack[history.idx]);
  return true;
}
export function undoDepth() { return history.idx < 0 ? 0 : history.idx; }

function restore(json) {
  const d = JSON.parse(json);
  project.channels = d.channels;
  project.returns = d.returns;
  project.clips = d.clips;
  project.scale = d.scale;
  project.master = d.master;
  project.selNotes = [];
  project.selectedSteps = [];
  window.dispatchEvent(new CustomEvent('daw:state'));
  dirty();
}

/* ============================ dirty / save ============================ */

export let isDirty = false;
let saveCb = () => { };
export function onDirty(fn) { saveCb = fn; }
export function dirty() {
  isDirty = true;
  saveCb();
}
export function markClean() {
  isDirty = false;
  saveCb();
}

/* ============================ project file ============================ */

export function saveProject() {
  return {
    format: 'smart-daw-project',
    version: 1,
    name: project.name,
    tempo: project.tempo,
    timeNum: project.timeNum,
    timeDen: project.timeDen,
    bars: project.bars,
    stepsPerBeat: project.stepsPerBeat,
    scale: project.scale,
    loop: project.loop,
    metronome: project.metronome,
    preset: project.preset,
    channels: project.channels,
    returns: project.returns,
    clips: project.clips,
    master: project.master,
  };
}

export function loadProject(d) {
  if (!d || d.format !== 'smart-daw-project') return false;
  project.name = d.name || project.name;
  project.tempo = d.tempo || project.tempo;
  project.timeNum = d.timeNum || project.timeNum;
  project.timeDen = d.timeDen || project.timeDen;
  project.bars = d.bars || project.bars;
  project.stepsPerBeat = d.stepsPerBeat || project.stepsPerBeat;
  project.scale = d.scale || project.scale;
  project.loop = d.loop || project.loop;
  project.metronome = !!d.metronome;
  project.preset = d.preset || project.preset;
  project.channels = d.channels || defaultChannels();
  project.returns = d.returns || defaultReturns();
  project.clips = d.clips || [];
  project.master = d.master || { gain: -1.5 };
  project.selChannel = 0;
  window.dispatchEvent(new CustomEvent('daw:state'));
  return true;
}

/* history length — the undo depth the topbar shows */
history.len = () => history.stack.length - 1;
