/* ==========================================================================
   Smart DAW — application shell
   Transport, scheduler, keyboard, palette, menus, boot. The entry module.
   See docs/KEYS.md for the full key map.
   ========================================================================== */

import { $, $$, el, esc, hydrateIcons, cssv, Knob, Fader, setMeter, invalidateCssCache } from './controls.js';
import {
  project, snapshot, undo, redo, undoDepth, markClean, onDirty, dirty,
  totalSteps, barSteps, stepToLabel, stepToTime, noteName, SCALES, ROOTS, ROOT_ALIAS,
  saveProject, loadProject,
} from './state.js';
import { engine, noteToFreq, FX_DEFS, FX_ORDER, SCHEDULE_AHEAD } from './audio.js';
import {
  initPanes, applyPreset, buildPane, floatPane, dockPane, maximizePane, tileFloating,
  openPane, closePane, isOpen, paneRenderers, notify, octave,
} from './panes.js';
import { setOctave, KEYBOARD } from './panes.js';

/* ============================================================
   boot
   ============================================================ */
function boot() {
  const b = $('#boot');
  const saved = localStorage.getItem('smart-daw-theme');
  if (saved) { b.hidden = true; document.documentElement.dataset.theme = saved; }
  else b.hidden = false;
  $$('[data-theme-pick]').forEach(btn => btn.addEventListener('click', () => {
    document.documentElement.dataset.theme = btn.dataset.themePick;
    localStorage.setItem('smart-daw-theme', btn.dataset.themePick);
    b.hidden = true;
    hydrateAll();
    onEnter();
  }));
  const autoload = localStorage.getItem('smart-daw-project');
  if (autoload) { try { loadProject(JSON.parse(autoload)); } catch (e) { /* ignore */ } }
}

/* ============================================================
   icons + initial render
   ============================================================ */
function hydrateAll() {
  hydrateIcons(document);
  window.dispatchEvent(new CustomEvent('daw:redraw'));
}

function onEnter() {
  engine.enable().then(ok => {
    if (!ok) return;
    if (pendingPlay) { pendingPlay = false; transportPlay(); }
    engine.channels = project.channels;
    engine.returns = project.returns;
    engine.rebuildAll();
    stopGate();
    markClean();
    hydrateAll();
  });
  $('#audiogate').classList.remove('show');
}

function startGate() {
  const g = $('#audiogate');
  if (!engine.ready) { g.hidden = false; requestAnimationFrame(() => g.classList.add('show')); }
}
function stopGate() {
  const g = $('#audiogate');
  g.classList.remove('show');
  setTimeout(() => { g.hidden = true; }, 240);
}
let pendingPlay = false;
$('#audioEnable').addEventListener('click', onEnter);
document.addEventListener('pointerdown', () => {
  if (!engine.ctx) onEnter();
  else engine.resume();
}, { once: true });

/* ============================================================
   theme
   ============================================================ */
$('#themeBtn').addEventListener('click', toggleTheme);
function toggleTheme() {
  const cur = document.documentElement.dataset.theme === 'dark' ? 'light' : 'dark';
  document.documentElement.dataset.theme = cur;
  localStorage.setItem('smart-daw-theme', cur);
  invalidateCssCache();
  hydrateAll();
}

/* ============================================================
   transport
   ============================================================ */
let schedTimer = null;
let nextStepTime = 0;
let startTime = 0;

const playBtn = $('#playBtn'), stopBtn = $('#stopBtn'), recBtn = $('#recBtn');
const loopBtn = $('#loopBtn'), metroBtn = $('#metroBtn'), posMain = $('#posMain'), posTime = $('#posTime');

playBtn.addEventListener('click', () => transportPlay());
stopBtn.addEventListener('click', () => transportStop());
recBtn.addEventListener('click', () => {
  project.recording = !project.recording;
  recBtn.setAttribute('aria-pressed', String(project.recording));
  const anyArmed = project.channels.some(c => c.armed);
  if (project.recording && !anyArmed) {
    project.channels[project.selChannel].armed = true;
    refreshPanes();
  }
  toast(project.recording ? 'Recording armed — play to record' : 'Recording off', project.recording ? 'record' : 'stop');
});
loopBtn.addEventListener('click', () => {
  project.loop.on = !project.loop.on;
  syncTransport();
});
metroBtn.addEventListener('click', () => {
  project.metronome = !project.metronome;
  syncTransport();
});
$('#rewindBtn').addEventListener('click', () => { project.playhead = 0; syncTransport(); refreshPanes(); });
$('#bpmField').addEventListener('change', e => setTempo(parseFloat(e.target.value) || project.tempo));
$('#gridBtn').addEventListener('click', ev => gridMenu(ev.clientX, ev.clientY));
$('#loopSetBtn').addEventListener('click', () => {
  project.loop = { on: true, start: Math.floor(project.playhead / barSteps()), end: Math.ceil((project.playhead + barSteps()) / barSteps()) };
  syncTransport();
});

export function setTempo(v) {
  project.tempo = Math.max(20, Math.min(300, v));
  $('#bpmField').value = project.tempo.toFixed(1);
  window.dispatchEvent(new CustomEvent('daw:redraw'));
}

function transportPlay() {
  if (project.playing) { transportStop(); return; }
  if (!engine.ctx) { pendingPlay = true; onEnter(); return; }
  engine.resume();
  /* play from the loop start when looping */
  if (project.loop.on) project.playhead = project.loop.start * barSteps();
  project.playing = true;
  nextStepTime = engine.ctx.currentTime + 0.08;
  startTime = nextStepTime - project.playhead * stepDurMs() / 1000;
  schedTimer = setInterval(scheduler, 25);
  syncTransport();
}
function transportStop() {
  project.playing = false;
  clearInterval(schedTimer);
  schedTimer = null;
  syncTransport();
  refreshPanes();
}
function stepDurMs() { return (60 / project.tempo / project.stepsPerBeat) * 1000; }

/* the lookahead scheduler — see docs/AUDIO.md §2 */
function scheduler() {
  if (!engine.ctx) return;
  const dur = stepDurMs() / 1000;
  while (nextStepTime < engine.ctx.currentTime + SCHEDULE_AHEAD) {
    scheduleStep(project.playhead, nextStepTime);
    nextStepTime += dur;
    project.playhead++;
  }
}

function scheduleStep(step, when) {
  const total = totalSteps();
  /* loop wrapping */
  if (project.loop.on) {
    const ls = project.loop.start * barSteps(), le = project.loop.end * barSteps();
    if (step >= le) {
      const over = step - le;
      project.playhead = ls + over;
      step = project.playhead;
    }
  } else if (step >= total) {
    project.playhead = 0;
    step = 0;
  }

  const beatInBar = Math.floor((step % barSteps()) / project.stepsPerBeat);
  const isDownbeat = step % project.stepsPerBeat === 0;

  /* metronome */
  if (project.metronome && engine.ready) {
    try {
      const o = engine.ctx.createOscillator(), g = engine.ctx.createGain();
      o.frequency.value = beatInBar === 0 ? 1600 : 1000;
      g.gain.setValueAtTime(0.0001, when);
      g.gain.exponentialRampToValueAtTime(0.2, when + 0.002);
      g.gain.exponentialRampToValueAtTime(0.0002, when + 0.05);
      o.connect(g); g.connect(engine.masterBus);
      o.start(when); o.stop(when + 0.08);
    } catch (e) { /* ignore */ }
  }

  /* clips first — they are the arrangement's voice */
  for (const clip of project.clips) {
    const clipStart = clip.bar * barSteps();
    const clipEnd = clipStart + clip.length * barSteps();
    if (step < clipStart || step >= clipEnd) continue;
    const ch = project.channels.find(c => c.id === clip.channelId);
    if (!ch) continue;
    const local = step - clipStart;
    const vel = ch.pattern[local];
    if (vel > 0) hit(ch, vel);
  }
  /* patterns that have no clips play straight from the rack */
  for (const ch of project.channels) {
    if (project.clips.some(c => c.channelId === ch.id)) continue;
    const local = step % barSteps();
    const vel = ch.pattern[local];
    if (vel > 0) hit(ch, vel);
  }
}

function hit(ch, vel) {
  const soloed = project.channels.some(c => c.solo);
  if (ch.mute || (soloed && !ch.solo)) return;
  engine.trigger(ch.id, { vel });
}

function syncTransport() {
  playBtn.setAttribute('aria-pressed', String(project.playing));
  playBtn.querySelector('.ic').dataset.ic = project.playing ? 'pause' : 'play';
  hydrateIcons(playBtn);
  recBtn.setAttribute('aria-pressed', String(project.recording));
  loopBtn.setAttribute('aria-pressed', String(project.loop.on));
  loopBtn.classList.toggle('on', project.loop.on);
  metroBtn.setAttribute('aria-pressed', String(project.metronome));
  metroBtn.classList.toggle('on', project.metronome);
  posMain.textContent = stepToLabel(project.playhead);
  const loopRead = $('#loopRead');
  if (loopRead) {
    loopRead.textContent = `${project.loop.start + 1}.1.1 — ${project.loop.end + 1}.1.1`;
    loopRead.classList.toggle('on', project.loop.on);
  }
}

/* the playhead is drawn linearly, from the audio clock, never with a spring */
function frame() {
  if (engine.ctx) {
    const meters = engine.readMeters();
    $$('[data-meter]').forEach((m, i) => {
      const v = meters.channels[i];
      if (v) setMeterSafe(m, v.peak, v.hold);
    });
    const mm = $('[data-meter="master"]');
    if (mm) setMeterSafe(mm, meters.master.peak, meters.master.hold);
    const vs = $('#voiceStat');
    if (vs) vs.innerHTML = `<span class="ic" data-ic="soundwave"></span>${engine.activeVoices} voices`;
  }
  const ps = project.playing ? posMain : null;
  if (ps) posMain.textContent = stepToLabel(project.playhead);
  if (posTime) posTime.textContent = stepToTime ? fmtTime(stepToTime(project.playhead)) : '';
  requestAnimationFrame(frame);
}
function setMeterSafe(node, peak, hold) {
  setMeter(node, peak, peak, hold);
}
function fmtTime(v) {
  const m = Math.floor(v / 60), s = Math.floor(v % 60), ms = Math.floor((v % 1) * 1000);
  return `${m}:${String(s).padStart(2, '0')}.${String(ms).padStart(3, '0')}`;
}

/* ============================================================
   topbar: undo depth, master knob, project
   ============================================================ */
$('#undoBtn').addEventListener('click', () => { if (undo()) { refreshPanes(); syncUndo(); } });
$('#redoBtn').addEventListener('click', () => { if (redo()) { refreshPanes(); syncUndo(); } });
function syncUndo() {
  const d = undoDepth();
  $('#undoDepth').textContent = String(d);
  $('#undoBtn').disabled = d <= 0;
  $('#redoBtn').disabled = d >= history.len();
}

$('#projName').addEventListener('input', e => { project.name = e.target.value; dirty(); });
$('#projName').addEventListener('change', () => snapshot(false));

$('#cmdBtn').addEventListener('click', () => togglePalette());
$('#keysBtn').addEventListener('click', () => toggleKeys());

$$('[data-preset]').forEach(b => b.addEventListener('click', () => applyPreset(b.dataset.preset)));

/* master knob in the topbar */
function initMasterKnob() {
  const node = $('#masterKnob');
  {
    new Knob(node, {
      min: -60, max: 6, step: 0.5, value: project.master.gain, label: 'Master level',
      format: v => (v <= -59.5 ? '−∞' : (v > 0 ? '+' : '') + v.toFixed(1)) + ' dB',
      onChange: v => { project.master.gain = v; engine.setMasterDb(v); },
    });
  }
}

/* ============================================================
   toast
   ============================================================ */
let toastTimer = null;
export function toast(msg, icon = 'info-circle') {
  const t = $('#toast');
  t.innerHTML = `<span class="ic" data-ic="${icon}"></span><span>${msg}</span>`;
  hydrateIcons(t);
  t.classList.add('show');
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => t.classList.remove('show'), 3200);
}

/* ============================================================
   overlays: context menu, popovers, palette, velocity editor
   ============================================================ */
const overlay = $('#overlay');

export function ctxMenu(x, y, items) {
  closeOverlay();
  const m = el('div', 'menu');
  let html = '';
  for (const it of items) {
    if (it.sep) { html += '<div class="msep"></div>'; continue; }
    if (it.label && it.head) { html += `<div class="mlabel">${esc(it.label)}</div>`; continue; }
    html += `<button class="mi${it.danger ? ' danger' : ''}${it.on ? ' on' : ''}" ${it.disabled ? 'aria-disabled="true"' : ''}>
      <span class="ic" data-ic="${it.icon || 'widget'}"></span><span class="ml">${esc(it.label)}</span>
      ${it.key ? `<span class="mk">${esc(it.key)}</span>` : ''}
      ${it.on ? '<span class="ic check" data-ic="check-circle"></span>' : ''}
    </button>`;
  }
  m.innerHTML = html;
  overlay.appendChild(m);
  const w = m.offsetWidth, h = m.offsetHeight;
  m.style.left = Math.min(x, window.innerWidth - w - 12) + 'px';
  m.style.top = Math.min(y, window.innerHeight - h - 12) + 'px';
  hydrateIcons(m);
  const btns = $$('.mi', m);
  const rows = items.filter(i => !i.sep && !i.head);
  btns.forEach((b, i) => {
    b.addEventListener('click', () => {
      const it = rows[i];
      if (it && it.fn) it.fn();
      closeOverlay();
    });
  });
  overlay.classList.add('show');
  requestAnimationFrame(() => btns[0] && btns[0].focus());
  return m;
}

export function closeOverlay() {
  overlay.innerHTML = '';
  overlay.classList.remove('show', 'scrimless');
  const p = $('#paletteLayer');
  if (p) p.classList.remove('show');
}
overlay.addEventListener('pointerdown', e => { if (e.target === overlay || e.target.classList.contains('overlay-scrim')) closeOverlay(); });

/* ---- scale selector ---- */
export function openScalePopover(x, y) {
  closeOverlay();
  const p = el('div', 'popover');
  p.style.left = Math.min(x, window.innerWidth - 260) + 'px';
  p.style.top = Math.min(y, window.innerHeight - 330) + 'px';
  p.innerHTML = `
    <div class="popover-head"><span class="ic" data-ic="scale"></span>Scale</div>
    <div class="roots" role="group" aria-label="Root note"></div>
    <div class="popover-grid" role="listbox" aria-label="Mode"></div>
    <div style="display:flex;gap:6px;margin-top:10px">
      <label class="pill" style="gap:5px;height:30px;padding:0 10px;flex:1;justify-content:center">
        <input type="checkbox" id="scaleShowAll" ${project.scale.on ? '' : 'checked'} hidden><span class="ic" data-ic="eye"></span>Show all keys
      </label>
    </div>`;
  overlay.appendChild(p);
  hydrateIcons(p);
  const roots = $('.roots', p), grid = $('.popover-grid', p);
  ROOT_ALIAS.forEach((r, i) => {
    const b = el('button', 'root-opt' + (project.scale.root === i ? ' on' : ''), r);
    b.setAttribute('aria-pressed', String(project.scale.root === i));
    b.addEventListener('click', () => {
      project.scale.root = i;
      snapshot(false);
      $$('.root-opt', roots).forEach((n, j) => { n.classList.toggle('on', j === i); n.setAttribute('aria-pressed', String(j === i)); });
      redrawRoll();
      updateScaleLabel();
    });
    roots.appendChild(b);
  });
  for (const mode of Object.keys(SCALES)) {
    const b = el('button', 'scale-opt' + (project.scale.mode === mode ? ' on' : ''), `<span class="ic" data-ic="check-circle"></span>${esc(mode)}`);
    b.setAttribute('role', 'option');
    b.setAttribute('aria-selected', String(project.scale.mode === mode));
    b.addEventListener('click', () => {
      project.scale.mode = mode;
      snapshot(false);
      $$('.scale-opt', grid).forEach((n, j) => { n.classList.toggle('on', Object.keys(SCALES)[j] === mode); n.setAttribute('aria-selected', String(Object.keys(SCALES)[j] === mode)); });
      redrawRoll();
      updateScaleLabel();
    });
    grid.appendChild(b);
  }
  $('#scaleShowAll', p).addEventListener('change', e => {
    project.scale.on = !e.target.checked;
    snapshot(false);
    redrawRoll();
    updateScaleLabel();
  });
  overlay.classList.add('show');
  const escH = e => { if (e.key === 'Escape') { closeOverlay(); document.removeEventListener('keydown', escH); } };
  document.addEventListener('keydown', escH);
  return p;
}

function updateScaleLabel() {
  const lab = $('#rollScaleLabel');
  if (lab) lab.textContent = project.scale.on ? ROOTS[project.scale.root] + ' ' + project.scale.mode : 'All keys';
}

/* ---- grid division menu ---- */
const GRID_DIVS = [
  { label: '1/4', v: 4 }, { label: '1/8', v: 8 }, { label: '1/16', v: 16 },
  { label: '1/32', v: 32 }, { label: '1/64', v: 64 },
];
export function gridMenu(x, y) {
  closeOverlay();
  const items = GRID_DIVS.map(d => ({
    icon: 'widget', label: d.label, key: String(GRID_DIVS.indexOf(d) + 1),
    on: project.stepsPerBeat === d.v,
    fn: () => { project.stepsPerBeat = d.v; $('#gridLabel').textContent = d.label; refreshPanes(); },
  }));
  return ctxMenu(x, y, items);
}
export function openGridMenu(x, y) { return gridMenu(x, y); }

/* ---- command palette ---- */
const COMMANDS = [
  { icon: 'play', label: 'Play / Stop', key: 'Space', fn: transportPlay },
  { icon: 'stop', label: 'Stop', fn: transportStop },
  { icon: 'record', label: 'Toggle record arm', key: 'R', fn: () => $('#recBtn').click() },
  { icon: 'repeat', label: 'Toggle loop', key: 'L', fn: () => $('#loopBtn').click() },
  { icon: 'tuning-2', label: 'Toggle metronome', key: 'N', fn: () => $('#metroBtn').click() },
  { icon: 'music-note', label: 'Piano roll', fn: () => applyPreset('compose') },
  { icon: 'widget', label: 'Channel rack', fn: () => applyPreset('program') },
  { icon: 'clapperboard', label: 'Arrangement', fn: () => applyPreset('arrange') },
  { icon: 'slider-vertical', label: 'Mixer', fn: () => applyPreset('mix') },
  { icon: 'scale', label: 'Scale highlighting…', fn: () => openScalePopover(window.innerWidth / 2, 120) },
  { icon: 'quit-full-screen', label: 'Float the focused pane', key: '⌘⇧F', fn: () => openPane('roll') },
  { icon: 'full-screen', label: 'Dock the focused pane', fn: () => closePane('roll') },
  { icon: 'maximize', label: 'Maximise the focused pane', fn: () => maximizePane(project.preset === 'compose' ? 'roll' : 'arrange') },
  { icon: 'reorder', label: 'Tile floating panes', key: '⌘T', fn: tileFloating },
  { icon: 'alt-arrow-left', label: 'Undo', key: '⌘Z', fn: () => $('#undoBtn').click() },
  { icon: 'alt-arrow-right', label: 'Redo', key: '⌘⇧Z', fn: () => $('#redoBtn').click() },
  { icon: 'diskette', label: 'Save project', key: '⌘S', fn: () => saveNow() },
  { icon: 'upload-track', label: 'Export project file', key: '⌘⇧S', fn: () => exportJson() },
  { icon: 'folder-with-files', label: 'Open project file…', key: '⌘O', fn: () => importJson() },
  { icon: 'vinyl-record', label: 'Audition the selected channel', fn: () => { const ch = project.channels[project.selChannel]; engine.trigger(ch.id, { vel: 100 }); } },
  { icon: 'magic-wand', label: 'Humanise the selected channel', fn: () => humanise(project.channels[project.selChannel]) },
  { icon: 'wind', label: 'Randomise the selected pattern', fn: () => randomise(project.channels[project.selChannel]) },
  { icon: 'stars-minimalistic', label: 'Switch theme', key: 'D', fn: toggleTheme },
  { icon: 'soundwave', label: 'Toggle Keys', fn: () => toggleKeys() },
];

function toggleKeys() { isOpen('keys') ? closePane('keys') : openPane('keys'); }

let palIdx = 0;
function togglePalette() {
  let layer = $('#paletteLayer');
  if (!layer || !layer.querySelector('#cmdInput')) buildPalette();
  layer = $('#paletteLayer');
  if (layer.classList.contains('show')) { closePalette(); return; }
  layer.hidden = false;
  layer.classList.add('show');
  const input = $('#cmdInput');
  input.value = '';
  palIdx = 0;
  drawPalette('');
  requestAnimationFrame(() => input.focus());
}
function closePalette() {
  const layer = $('#paletteLayer');
  if (layer) { layer.classList.remove('show'); layer.hidden = true; }
}
function buildPalette() {
  const layer = el('div', 'palette-layer');
  layer.id = 'paletteLayer';
  layer.innerHTML = `
    <div class="palette-scrim" id="paletteScrim"></div>
    <section class="palette" role="dialog" aria-modal="true" aria-label="Command palette">
      <div class="palette-search">
        <span class="ic" data-ic="magnifer"></span>
        <input id="cmdInput" type="text" placeholder="Search commands…" autocomplete="off" spellcheck="false"
          role="combobox" aria-expanded="true" aria-controls="paletteList" aria-label="Search commands">
        <button class="cmd-esc" id="cmdClose" type="button">esc</button>
      </div>
      <div class="palette-list" id="paletteList" role="listbox" aria-label="Commands"></div>
      <div class="palette-foot">
        <span><span class="kbd">↑</span><span class="kbd">↓</span>move</span>
        <span><span class="kbd">↵</span>run</span>
        <span><span class="kbd">esc</span>close</span>
      </div>
    </section>`;
  $('#win').appendChild(layer);
  hydrateIcons(layer);
  const input = $('#cmdInput');
  input.addEventListener('input', () => drawPalette(input.value));
  input.addEventListener('keydown', e => {
    const rows = $$('.cmd-row', layer);
    if (e.key === 'ArrowDown') { e.preventDefault(); palIdx = Math.min(rows.length - 1, palIdx + 1); markPalette(rows); }
    if (e.key === 'ArrowUp') { e.preventDefault(); palIdx = Math.max(0, palIdx - 1); markPalette(rows); }
    if (e.key === 'Enter') { e.preventDefault(); const r = rows[palIdx]; if (r) r.click(); closePalette(); }
    if (e.key === 'Escape') { closePalette(); }
  });
  $('#cmdClose').addEventListener('click', closePalette);
  $('#paletteScrim').addEventListener('click', closePalette);
}
function drawPalette(q) {
  const layer = $('#paletteLayer');
  if (!layer) { buildPalette(); return; }
  const list = $('#paletteList', layer);
  const query = q.trim().toLowerCase();
  const rows = COMMANDS.filter(c => !query || c.label.toLowerCase().includes(query));
  list.innerHTML = rows.length ? rows.map((c, i) => `
    <button class="cmd-row${i === 0 ? ' sel' : ''}" data-ci="${i}" role="option" aria-selected="${i === 0}">
      <span class="cmd-ic"><span class="ic" data-ic="${c.icon}"></span></span>
      <b>${esc(c.label)}</b>
      ${c.key ? `<span class="cmd-k">${esc(c.key)}</span>` : ''}
    </button>`).join('')
    : `<div class="pane-empty"><span class="ic" data-ic="magnifer"></span><b>No command matches “${esc(q)}”</b><small>Try a shorter word.</small></div>`;
  palIdx = 0;
  hydrateIcons(list);
  $$('.cmd-row', list).forEach((b, i) => {
    b.addEventListener('click', () => { rows[i].fn(); closePalette(); });
    b.addEventListener('pointerenter', () => { palIdx = i; markPalette($$('.cmd-row', list)); });
  });
}
function markPalette(rows) {
  rows.forEach((r, i) => {
    r.classList.toggle('sel', i === palIdx);
    r.setAttribute('aria-selected', String(i === palIdx));
  });
}

/* ---- humanise / randomise ---- */
export function humanise(ch) {
  for (let i = 0; i < ch.pattern.length; i++) {
    if (ch.pattern[i] > 0) ch.pattern[i] = Math.max(24, Math.min(127, ch.pattern[i] + (Math.random() * 40 - 20)));
  }
  snapshot(false);
  refreshPanes();
  toast('Velocity humanised', 'magic-wand');
}
export function randomise(ch) {
  ch.pattern = ch.pattern.map((v, i) => (i % 2 === 0 && Math.random() < 0.32 ? 60 + Math.random() * 67 : v));
  snapshot(false);
  refreshPanes();
  toast('Pattern varied', 'wind');
}
function redrawRoll() { window.dispatchEvent(new CustomEvent('daw:redraw')); }

/* ---- project save / export ---- */
function saveNow() {
  localStorage.setItem('smart-daw-project', JSON.stringify(saveProject()));
  markClean();
  toast(`“${esc(project.name)}” saved to this browser`, 'check-circle');
}
function exportJson() {
  const blob = new Blob([JSON.stringify(saveProject(), null, 2)], { type: 'application/json' });
  const a = document.createElement('a');
  a.href = URL.createObjectURL(blob);
  a.download = project.name.replace(/[^\w\s-]/g, '').trim().replace(/\s+/g, '-').toLowerCase() + '.smartdaw.json';
  a.click();
  URL.revokeObjectURL(a.href);
  toast('Project exported', 'upload-track');
}
function importJson() {
  const input = document.createElement('input');
  input.type = 'file'; input.accept = '.json,application/json';
  input.addEventListener('change', () => {
    const f = input.files[0];
    if (!f) return;
    const r = new FileReader();
    r.onload = () => {
      try {
        loadProject(JSON.parse(r.result));
        engine.channels = project.channels;
        engine.returns = project.returns;
        engine.rebuildAll();
        refreshPanes();
        syncTransport();
        toast('Project opened', 'folder-with-files');
      } catch (e) { toast('That file is not a Smart DAW project', 'info-circle'); }
    };
    r.readAsText(f);
  });
  input.click();
}

/* ============================================================
   keyboard — docs/KEYS.md
   ============================================================ */
document.addEventListener('keydown', e => {
  const mod = e.metaKey || e.ctrlKey;
  const tag = (e.target.tagName || '').toLowerCase();
  const typing = tag === 'input' || tag === 'textarea' || tag === 'select';

  /* mod combos first */
  if (mod) {
    const k = e.key.toLowerCase();
    if (k === 'k') { e.preventDefault(); togglePalette(); return; }
    if (k === 's') { e.preventDefault(); e.shiftKey ? exportJson() : saveNow(); return; }
    if (k === 'o') { e.preventDefault(); importJson(); return; }
    if (k === 'z') { e.preventDefault(); if (undo()) { refreshPanes(); syncUndo(); } return; }
    if (k === 'y') { e.preventDefault(); if (redo()) { refreshPanes(); syncUndo(); } return; }
    if (e.shiftKey && k === 'f') { e.preventDefault(); toggleKeys(); return; }
    if (e.shiftKey && k === 't') { e.preventDefault(); tileFloating(); return; }
    if (k === '1' || k === '2' || k === '3' || k === '4') {
      e.preventDefault();
      applyPreset(['compose', 'program', 'arrange', 'mix'][+k - 1]);
      return;
    }
    return;
  }
  if (typing) {
    if (e.key === 'Escape') e.target.blur();
    return;
  }

  /* computer keyboard as piano */
  if (KEYBOARD[e.code] != null && !e.repeat) {
    const p = KEYBOARD[e.code] + (octave.base + 1) * 12;
    const ch = project.channels[project.selChannel];
    renderKeyDown(p);
    engine.trigger(ch.id, { vel: 96, dur: 0.5, note: p });
    recordIfPlaying(ch, p, 96);
    return;
  }

  switch (e.key) {
    case ' ': e.preventDefault(); transportPlay(); break;
    case 'Escape':
      if ($('#paletteLayer') && $('#paletteLayer').classList.contains('show')) closePalette();
      else closeOverlay();
      break;
    case 'Home': project.playhead = 0; syncTransport(); refreshPanes(); break;
    case 'End': project.playhead = totalSteps(); syncTransport(); refreshPanes(); break;
    case 'ArrowLeft': project.playhead = Math.max(0, project.playhead - (e.shiftKey ? barSteps() : 1)); syncTransport(); refreshPanes(); break;
    case 'ArrowRight': project.playhead = Math.min(totalSteps(), project.playhead + (e.shiftKey ? barSteps() : 1)); syncTransport(); refreshPanes(); break;
    case 'r': case 'R': $('#recBtn').click(); break;
    case 'l': case 'L': $('#loopBtn').click(); break;
    case 'n': case 'N': $('#metroBtn').click(); break;
    case 'd': case 'D': toggleTheme(); break;
    case '?': openShortcuts(); break;
    case ',': setTempo(project.tempo - 1); break;
    case '.': setTempo(project.tempo + 1); break;
    case 'z': setOctave(-1); break;
    case 'x': setOctave(1); break;
    default:
      if (e.key >= '1' && e.key <= '5') {
        const d = GRID_DIVS[+e.key - 1];
        if (d) { project.stepsPerBeat = d.v; $('#gridLabel').textContent = d.label; refreshPanes(); }
      }
  }
});
document.addEventListener('keyup', e => {
  if (KEYBOARD[e.code] != null) renderKeyDown(null);
});
function octaveBase() { return octave.base; }
function renderKeyDown(pitch) {
  const n = pitch == null ? null : $(`.vk[data-pitch="${pitch}"]`);
  $$('.vk.is-kbd').forEach(k => k.classList.remove('is-kbd'));
  if (n) n.classList.add('is-kbd');
}

function recordIfPlaying(ch, pitch, vel) {
  if (project.playing && ch.armed) {
    ch.notes.push({ step: Math.round(project.playhead), len: Math.max(1, project.stepsPerBeat / 4), pitch, vel });
    ch.notes.sort((a, b) => a.step - b.step);
    snapshot(false);
    refreshPanes();
  }
}

/* ---- shortcuts sheet ---- */
const SHORTCUTS = [
  ['Space', 'Play / stop'], ['R', 'Record arm'], ['L', 'Loop'], ['N', 'Metronome'],
  ['⌘1–4', 'Presets'], ['⌘T', 'Tile floating panes'], ['⌘⇧F', 'Float / dock'],
  ['⌘Z', 'Undo'], ['⌘⇧Z', 'Redo'], ['⌘S', 'Save'], ['⌘⇧S', 'Export'], ['⌘O', 'Open'],
  ['⌘K', 'Command palette'], ['D', 'Theme'], ['Z / X', 'Octave'], ['Q2W3ER…', 'Play notes'],
  ['1–5', 'Grid division'], [', .', 'Tempo −1 / +1'], ['← →', 'Playhead one step'],
  ['⇧← →', 'Playhead one bar'], ['Home / End', 'Song start / end'], ['?', 'This sheet'],
];
function openShortcuts() {
  closeOverlay();
  const p = el('div', 'popover');
  p.style.cssText = 'left:50%;top:12vh;transform:translateX(-50%);max-height:70vh;overflow:auto;min-width:360px';
  p.innerHTML = `<div class="popover-head"><span class="ic" data-ic="widget"></span>Keyboard</div>
    <div class="sc-list">${SHORTCUTS.map(([k, v]) => `
      <div class="sc-row"><span>${esc(v)}</span><kbd>${esc(k)}</kbd></div>`).join('')}</div>`;
  overlay.appendChild(p);
  hydrateIcons(p);
  overlay.classList.add('show');
}

/* ============================================================
   redraw + state listeners
   ============================================================ */
window.addEventListener('daw:redraw', () => refreshPanes());
window.addEventListener('daw:state', () => { refreshPanes(); syncTransport(); });
window.addEventListener('daw:transport', syncTransport);
window.addEventListener('daw:toast', e => toast(e.detail.msg, e.detail.icon));
window.addEventListener('daw:fxadd', e => openFxMenu({ getBoundingClientRect: () => ({ left: 200, top: 200 }) }, e.detail.ch.id, e.detail.ch));
window.addEventListener('daw:panemenu', e => {
  const { id, btn } = e.detail;
  const r = btn.getBoundingClientRect();
  ctxMenu(r.left, r.bottom + 6, [
    { icon: 'maximize', label: 'Maximise', fn: () => maximizePane(id) },
    { icon: 'quit-full-screen', label: panesMode(id) === 'float' ? 'Dock pane' : 'Float pane', key: '⌘⇧F', fn: () => panesMode(id) === 'float' ? dockPane(id) : floatPane(id) },
    { icon: 'close-circle', label: 'Close pane', danger: true, fn: () => closePane(id) },
  ]);
});
function panesMode(id) {
  const p = window.__panes && window.__panes[id];
  return p && p.mode ? p.mode : 'docked';
}

/* ---- fx menu (opened from a mixer slot) ---- */
export function openFxMenu(slotEl, channelId) {
  const ch = channelId === 'master' ? null : project.channels.find(c => c.id === channelId);
  const target = ch || { inserts: [] };
  const r = slotEl.getBoundingClientRect();
  const items = FX_ORDER.map(id => ({
    icon: (FX_DEFS[id] || {}).icon || 'widget',
    label: FX_DEFS[id] ? FX_DEFS[id].name : id,
    on: target.inserts.some(f => f.id === id),
    fn: () => {
      const existing = target.inserts.find(f => f.id === id);
      if (existing) { existing.bypass = !existing.bypass; }
      else target.inserts.push({ id, bypass: false, params: (FX_DEFS[id].params || []).map(p => ({ id: p.id, value: p.value })) });
      snapshot(false);
      if (ch) engine.rebuildChannel(ch.id);
      refreshPanes();
      if (isOpen('fx')) buildPane('fx');
    },
  }));
  ctxMenu(r.left, r.bottom + 6, items);
}

/* ---- preserve selection across redraws ---- */
let selChannelOnRedraw = -1;
function refreshPanes() {
  selChannelOnRedraw = project.selChannel;
  for (const id of Object.keys(paneRenderers)) {
    const body = $('.pane[data-pane="' + id + '"] .pane-body');
    if (body) { paneRenderers[id](body, {}); }
  }
  syncUndo();
  updateScaleLabel();
}

/* ============================================================
   go
   ============================================================ */
function init() {
  boot();
  hydrateIcons(document);
  initPanes();
  initMasterKnob();
  syncTransport();
  syncUndo();
  onDirty(() => {
    const st = $('#projState');
    if (st) st.innerHTML = `<i class="pdot dirty"></i>Unsaved`;
    syncUndo();
  });
  markClean();
  refreshPanes();
  frame();
  startGate();
  window.__panes = {};
  /* expose the live pane map for the header menu */
  snapshot(true);
}
init();
