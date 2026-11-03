import { $, $ as $_, $$, Fader, Knob, cssv, el, esc, hydrateIcons, setMeter } from './controls.js';
import { FX_DEFS, FX_LABEL, FX_ORDER, FX_PARAMS, dbToGain, engine, noteToFreq } from './audio.js';
import { ROOTS, SCALES, barSteps, inScale, noteName, project, snapToScale, snapshot, stepDur, stepToLabel, totalSteps } from './state.js';
import { ctxMenu, openGridMenu, openScalePopover } from './daw.js';


/* ==========================================================================
   Smart DAW — pane manager
   Dock / float, drag out and back, presets. One glass plane, one seam, one rim.
   See docs/LAYOUT.md.
   ========================================================================== */


const PRESETS = {
  compose:  { browser: 'left', rack: 'cl', roll: 'cr', mixer: 'right', keys: null,  arrange: 'bottom', editor: null },
  program:  { browser: 'cl',   rack: 'cr', roll: 'cr', mixer: 'right', keys: null,  arrange: 'bottom', editor: null },
  arrange:  { browser: 'left', rack: 'cl', roll: null,  mixer: null,   keys: null,  arrange: 'cr',    editor: null },
  mix:      { browser: null,  rack: 'cl', roll: null,  mixer: 'right', keys: null,  arrange: 'cr',    editor: null },
};

const PANE_TITLES = {
  browser: 'Browser', rack: 'Channel rack', roll: 'Piano roll',
  mixer: 'Mixer', arrange: 'Arrangement', keys: 'Keys', editor: 'Sample editor', fx: 'FX chain',
};

let panes = {};
let zTop = 100;
const onPaneChange = new Set();

export function notify() { onPaneChange.forEach(fn => fn()); }

export function slotEl(name) { return $(`.slot[data-slot="${name}"]`); }

export function buildPane(id, mode) {
  const host = mode === 'float' ? $('#floats') : slotEl(PRESETS[project.preset][id] || 'cr');
  let p = panes[id];
  if (!p) {
    p = panes[id] = {
      id, title: PANE_TITLES[id] || id,
      node: el('div', 'pane'),
      body: el('div', 'pane-body'),
      mode: 'docked', slot: PRESETS[project.preset][id] || 'cr',
      render: paneRenderers[id],
      memory: {},
    };
    p.node.dataset.pane = id;
    p.node.setAttribute('role', 'group');
    p.node.setAttribute('aria-label', p.title);
    p.header = el('div', 'pane-h');
    p.header.innerHTML =
      `<span class="ic grip" data-ic="grip-vertical"></span>` +
      `<span class="pane-title">${esc(p.title)}</span>` +
      `<span class="pane-actions">` +
      `<button class="iconbtn" data-pane-act="max" data-tip="Maximise" aria-label="Maximise ${esc(p.title)}"><span class="ic" data-ic="maximize"></span></button>` +
      `<button class="iconbtn" data-pane-act="float" data-tip="Float  ⌘⇧F" aria-label="Float ${esc(p.title)}" aria-pressed="false"><span class="ic" data-ic="quit-full-screen"></span></button>` +
      `<button class="iconbtn" data-pane-act="menu" data-tip="More" aria-label="More for ${esc(p.title)}"><span class="ic" data-ic="menu-dots"></span></button>` +
      `</span>`;
    p.node.appendChild(p.header);
    p.node.appendChild(p.body);
  }

  if (mode === 'float' && p.mode !== 'float') {
    const mem = p.floatMem || {};
    p.node.classList.add('is-floating', 'is-front');
    p.node.style.left = (mem.x ?? 200) + 'px';
    p.node.style.top = (mem.y ?? 140) + 'px';
    p.node.style.width = (mem.w ?? 460) + 'px';
    p.node.style.height = (mem.h ?? 340) + 'px';
    p.mode = 'float';
    $$('[data-pane-act="float"]', p.header).forEach(b => {
      b.setAttribute('aria-pressed', 'true');
      b.querySelector('.ic').dataset.ic = 'full-screen';
    });
    host.parentElement.appendChild(p.node);
  } else if (mode === 'docked' && p.mode !== 'docked') {
    p.node.classList.remove('is-floating', 'is-front');
    p.node.style.cssText = '';
    p.mode = 'docked';
    p.slot = PRESETS[project.preset][id] || p.slot;
    $$('[data-pane-act="float"]', p.header).forEach(b => {
      b.setAttribute('aria-pressed', 'false');
      b.querySelector('.ic').dataset.ic = 'quit-full-screen';
    });
    host.appendChild(p.node);
  } else if (mode === 'docked' && p.mode === 'docked') {
    host.appendChild(p.node);
  }

  p.render(p.body, p);
  hydrateIcons(p.node);
  notify();
  return p;
}

export const paneRenderers = {};
export function setRenderer(id, fn) { paneRenderers[id] = fn; }

/* ============================ presets ============================ */

export function applyPreset(name, silent) {
  if (!PRESETS[name]) return;
  project.preset = name;
  const dock = $('#dock');
  dock.dataset.preset = name;
  $$('[data-preset]').forEach(b => b.classList.toggle('on', b.dataset.preset === name));

  /* build the panes this preset needs; ones already floating stay floating */
  for (const id of Object.keys(PRESETS[name])) {
    if (!PRESETS[name][id]) continue;
    const existing = panes[id];
    buildPane(id, existing && existing.mode === 'float' ? 'float' : 'docked');
  }
  /* remove docked panes the preset does not use */
  for (const id of Object.keys(panes)) {
    if (!PRESETS[name][id] && panes[id].mode === 'docked') closePane(id);
  }

  dock.classList.remove('zoomed');
  if (!silent) notify();
}

export function floatPane(id) {
  const p = panes[id];
  if (!p) { buildPane(id, 'float'); return; }
  const slot = slotEl(p.slot);
  if (p.mode === 'docked' && slot) {
    const b = p.node.getBoundingClientRect(), sb = slot.getBoundingClientRect();
    p.floatMem = { x: Math.max(8, b.left - sb.left + 40), y: Math.max(8, b.top - sb.top + 40), w: b.width, h: b.height };
  }
  buildPane(id, 'float');
}
export function dockPane(id) { if (panes[id]) buildPane(id, 'docked'); }

export function focusPane(id) {
  const p = panes[id];
  if (!p || p.mode !== 'float') return;
  $$('.pane.is-floating').forEach(n => n.classList.remove('is-front'));
  p.node.classList.add('is-front');
  p.node.style.zIndex = ++zTop;
}

export function maximizePane(id) {
  const dock = $('#dock');
  const p = panes[id];
  if (!p) return;
  const was = dock.classList.contains('zoomed');
  $$('.slot').forEach(s => s.classList.remove('is-max'));
  if (was) { dock.classList.remove('zoomed'); }
  else {
    dock.classList.add('zoomed');
    slotEl(p.slot || PRESETS[project.preset][id]).classList.add('is-max');
  }
  notify();
}

export function tileFloating() {
  const floats = $('#floats');
  const nodes = $$('.pane.is-floating', floats);
  if (nodes.length < 2) return;
  const dockBox = $('#dock').getBoundingClientRect();
  const cols = Math.ceil(Math.sqrt(nodes.length));
  const rows = Math.ceil(nodes.length / cols);
  nodes.forEach((n, i) => {
    const cx = i % cols, cy = Math.floor(i / cols);
    n.style.left = 10 + cx * ((dockBox.width - 20) / cols) + 'px';
    n.style.top = 10 + cy * ((dockBox.height - 20) / rows) + 'px';
    n.style.width = (dockBox.width - 20) / cols - 10 + 'px';
    n.style.height = (dockBox.height - 20) / rows - 10 + 'px';
  });
}

/* ============================ drag: dock → float ============================ */

export function bindPaneDrag() {
  let drag = null, ghost = null, dropSlot = null;

  document.addEventListener('pointerdown', ev => {
    const head = ev.target.closest && ev.target.closest('.pane-h');
    if (!head) return;
    if (ev.target.closest('button,.iconbtn,input,.pill,.knob')) return;
    const pane = head.closest('.pane');
    if (!pane) return;
    drag = { pane, id: pane.dataset.pane, x: ev.clientX, y: ev.clientY, moved: false };
  });

  document.addEventListener('pointermove', ev => {
    if (!drag) return;
    const dx = ev.clientX - drag.x, dy = ev.clientY - drag.y;
    if (!drag.moved && Math.hypot(dx, dy) < 6) return;
    if (!drag.moved) {
      drag.moved = true;
      if (panes[drag.id].mode === 'docked') floatPane(drag.id);
      drag.pane.classList.add('is-dragging');
      const dockBox = $('#dock').getBoundingClientRect();
      ghost = { dx: drag.pane.offsetWidth, dy: drag.pane.offsetHeight, dockBox };
    }
    const p = panes[drag.id];
    const dockBox = $('#dock').getBoundingClientRect();
    let x = ev.clientX - dockBox.left - 80;
    let y = ev.clientY - dockBox.top - 16;
    x = Math.max(-40, Math.min(dockBox.width - 60, x));
    y = Math.max(-20, Math.min(dockBox.height - 60, y));
    drag.pane.style.left = x + 'px';
    drag.pane.style.top = y + 'px';

    /* slot highlight */
    $$('.slot').forEach(s => s.classList.remove('is-shadow'));
    dropSlot = null;
    for (const s of $$('.slot')) {
      const b = s.getBoundingClientRect();
      if (ev.clientX >= b.left && ev.clientX <= b.right && ev.clientY >= b.top && ev.clientY <= b.bottom) {
        s.classList.add('is-shadow');
        dropSlot = s;
        break;
      }
    }
  });

  document.addEventListener('pointerup', () => {
    if (!drag) return;
    const p = panes[drag.id];
    drag.pane.classList.remove('is-dragging');
    $$('.slot').forEach(s => s.classList.remove('is-shadow'));
    if (dropSlot && dropSlot.children.length === 0) {
      dockPane(drag.id);
    }
    drag = null; ghost = null; dropSlot = null;
    notify();
  });
}

/* ============================ pane header actions ============================ */

export function bindPaneHeader() {
  document.addEventListener('click', ev => {
    const btn = ev.target.closest('[data-pane-act]');
    if (!btn) return;
    const pane = btn.closest('.pane');
    if (!pane) return;
    const id = pane.dataset.pane;
    const act = btn.dataset.paneAct;
    if (act === 'float') { panes[id].mode === 'float' ? dockPane(id) : floatPane(id); }
    if (act === 'max') maximizePane(id);
    if (act === 'menu') window.dispatchEvent(new CustomEvent('daw:panemenu', { detail: { id, btn } }));
  });
  document.addEventListener('pointerdown', ev => {
    const pane = ev.target.closest && ev.target.closest('.pane.is-floating');
    if (pane) focusPane(pane.dataset.pane);
  }, true);
}

/* ============================ integration ============================ */


export function initPanes() {
  bindPaneDrag();
  bindPaneHeader();
  applyPreset(project.preset, true);
}

/* ==========================================================================
   pane renderers — each pane draws itself when asked
   ========================================================================== */


/* ---- browser ---- */
const BROWSER_TREE = [
  { id: 'samples', label: 'Samples', icon: 'library', kids: [
    'Log Drum 808.wav', 'Log Drum deep.wav', 'Shaker 16th.wav', 'Clap big.wav', 'Rim tight.wav', 'Kick sub.wav', 'Open hat 808.wav',
  ] },
  { id: 'instruments', label: 'Instruments', icon: 'music-note', kids: ['Log Drum MK-I', 'Deep House Piano', 'Amapiano Bass', 'Shaker Bank'] },
  { id: 'fx', label: 'FX', icon: 'widget', kids: ['Plate Reverb', 'Ping Pong Delay', 'Auto Filter', 'Tape Saturation'] },
  { id: 'presets', label: 'Presets', icon: 'star', kids: ['Amapiano Roll', 'Deep Stab', 'Log Bass', 'Shaker Loop'] },
  { id: 'user', label: 'User library', icon: 'folder', kids: [] },
];

export function renderBrowser(root, pane) {
  root.innerHTML = `
    <div class="browser">
      <div class="browser-top">
        <label class="search">
          <span class="ic" data-ic="magnifer"></span>
          <input type="text" placeholder="Search everything" aria-label="Search browser">
        </label>
        <div class="browser-cols" role="tablist">
          <button class="on" data-br="list" role="tab" aria-selected="true"><span class="ic" data-ic="hamburger-menu"></span>All</button>
          <button data-br="fav" role="tab" aria-selected="false"><span class="ic" data-ic="star"></span>Star</button>
        </div>
      </div>
      <div class="browser-body scroll"></div>
    </div>`;
  const q = $('input', root);
  const list = $('.browser-body', root);
  let open = new Set(['samples']);
  let mode = 'list';

  const draw = () => {
    const filter = q.value.trim().toLowerCase();
    let html = '';
    const items = [];
    for (const node of BROWSER_TREE) {
      const kids = node.kids.filter(k => !filter || k.toLowerCase().includes(filter));
      if (!filter && !open.has(node.id)) {
        html += `<button class="btree-row" data-tree="${node.id}"><span class="ic" data-ic="${node.icon}"></span><span class="btw">${esc(node.label)}</span><span class="bchev"></span></button>`;
        continue;
      }
      html += `<button class="btree-row" data-tree="${node.id}"><span class="ic" data-ic="${node.icon}"></span><span class="btw">${esc(node.label)}</span><span class="bchev open"></span></button>`;
      for (const k of kids) items.push([node.id, node.icon, k]);
    }
    if (items.length) {
      html += `<div class="mlabel" style="margin:8px 0 3px">${filter ? 'Matches' : 'Contents'}</div>`;
      for (const [pid, icon, k] of items) {
        const ext = k.split('.').pop().toUpperCase();
        const isAud = k.toLowerCase().includes('drum') || k.includes('wav');
        html += `<button class="bitem" data-preview="${esc(k)}" data-kind="${isAud ? 'drum' : 'inst'}">
          <span class="bth"><span class="ic" data-ic="${isAud ? 'vinyl-record' : icon.replace(/-outline$/, '')}"></span></span>
          <span class="bname">${esc(k.replace(/\.[a-z0-9]+$/i, ''))}</span>
          <span class="bmeta">${ext.length <= 4 ? ext : ''}</span>
        </button>`;
      }
    } else if (filter) {
      html += `<div class="pane-empty"><span class="ic" data-ic="magnifer"></span><b>Nothing matches “${esc(filter)}”</b><small>Try a shorter name.</small></div>`;
    }
    list.innerHTML = html;
    hydrateIcons(list);
  };

  list.addEventListener('click', ev => {
    const t = ev.target.closest('[data-tree]');
    if (t) {
      const id = t.dataset.tree;
      open.has(id) ? open.delete(id) : open.add(id);
      draw();
      return;
    }
    const b = ev.target.closest('[data-preview]');
    if (b) previewBrowserItem(b.dataset.preview, b.dataset.kind, b);
  });
  q.addEventListener('input', draw);
  $$('.browser-cols button', root).forEach(b => b.addEventListener('click', () => {
    $$('.browser-cols button', root).forEach(x => { x.classList.remove('on'); x.setAttribute('aria-selected', 'false'); });
    b.classList.add('on'); b.setAttribute('aria-selected', 'true');
    mode = b.dataset.br;
    draw();
  }));
  draw();
}

/* audition-on-hover: previews before commit — a UX law, not a bonus */
function previewBrowserItem(name, kind, node) {
  let pending = null;
  node.addEventListener('pointerenter', () => {
    if (!engine.ready) return;
    node.classList.add('auditioning');
    const delay = setTimeout(() => engine.auditionOne(name, kind), 90);
    pending = delay;
  });
  node.addEventListener('pointerleave', () => {
    node.classList.remove('auditioning');
    if (pending) clearTimeout(pending);
    engine.stopAudition();
  });
}

/* ---- channel rack ---- */
export function renderRack(root, pane) {
  root.innerHTML = `
    <div class="rack">
      <div class="rack-top">
        <button class="pill" id="rackAdd"><span class="ic" data-ic="add"></span>Channel</button>
        <select class="pill" id="rackSteps" aria-label="Step count"></select>
        <div class="rack-steps-cap">
          <span class="tb-label">Steps</span><b id="rackStepN" style="color:var(--ink-1);font-size:11px;font-weight:600">16</b>
        </div>
        <label class="pill" style="gap:5px;height:28px;padding:0 9px">
          <input type="checkbox" checked id="rackVel" hidden><span class="ic" data-ic="slider-vertical"></span>Velocity
        </label>
      </div>
      <div class="rack-body">
        <div class="rack-rows" id="rackRows"></div>
      </div>
    </div>`;
  const grid = $('#rackRows', root);
  if (grid) grid.__draw = drawRack;
  bindRack(root);
  drawRack();
}

function drawRack() {
  const root = $('#rackRows');
  if (!root) return;
  const N = project.timeNum * project.stepsPerBeat;   /* steps per bar */
  let html = '';
  for (let ci = 0; ci < project.channels.length; ci++) {
    const ch = project.channels[ci];
    html += `<div class="rack-row${ci === project.selChannel ? ' is-selected' : ''}${ch.mute ? ' is-muted' : ''}"
                 data-channel="${ci}" style="--stepc:${ch.color}">
      <div class="rack-head">
        <span class="rack-chip" style="background:${ch.color}"></span>
        <span class="rack-name">${esc(ch.name)}</span>
        <button class="tbtn${ch.mute ? ' on' : ''}" data-rk="mute" data-channel="${ci}" aria-label="Mute ${esc(ch.name)}" aria-pressed="${ch.mute}"><span class="ic" data-ic="volume-cross"></span></button>
        <button class="tbtn solo${ch.solo ? ' on' : ''}" data-rk="solo" data-channel="${ci}" aria-label="Solo ${esc(ch.name)}" aria-pressed="${ch.solo}"><span class="ic" data-ic="volume-loud"></span></button>
        <button class="tbtn arm${ch.armed ? ' on' : ''}" data-rk="arm" data-channel="${ci}" aria-label="Arm ${esc(ch.name)}" aria-pressed="${ch.armed}"><span class="ic" data-ic="record"></span></button>
      </div>
      <div class="rack-grid" role="grid" aria-label="${esc(ch.name)} steps">`;
    for (let s = 0; s < N; s++) {
      const v = ch.pattern[s] || 0;
      const on = v > 0;
      const playing = playStepChannel === ci && playStepStep === s;
      html += `<button class="step${on ? ' is-on' : ''}${on && v > 100 ? ' is-accent' : ''}${s % 4 === 0 ? ' is-beat' : ''}${playing ? ' is-playing' : ''}"
        data-channel="${ci}" data-step="${s}" aria-pressed="${on}"
        aria-label="${esc(ch.name)} step ${s + 1}, velocity ${on ? Math.round(v) : 0}"
        style="--stepc:${ch.color}"><span class="step-dot" style="opacity:${on ? 0.4 + 0.6 * (v / 127) : 0}"></span></button>`;
    }
    html += `</div></div>`;
  }
  root.innerHTML = html;
  hydrateIcons(root);
}

/* playing-state pointers so the rack can show a playhead */
export let playStepStep = -1;
export let playStepChannel = -1;
export function setPlayStep(step, channel) {
  playStepStep = step;
  playStepChannel = channel;
  const live = $('.rack .rack-row .step.is-playing');
  if (live) live.classList.remove('is-playing');
  const root = $('#rackRows');
  if (!root) return;
  const cells = $$('.step', root);
  const row = cells[0] ? cells[0].closest('.rack-row') : null;
  if (!row) return;
  const cell = row.querySelector(`.step[data-step="${step % (project.timeNum * project.stepsPerBeat)}"]`);
  if (cell) cell.classList.add('is-playing');
}

export function bindRack(root) {
  if (root.dataset.bound) return;
  root.dataset.bound = '1';
  root.addEventListener('click', ev => {
    const cell = ev.target.closest('.step');
    if (cell) {
      const ci = +cell.dataset.channel, s = +cell.dataset.step;
      const ch = project.channels[ci];
      const cur = ch.pattern[s] || 0;
      ch.pattern[s] = cur > 0 ? 0 : 96;
      snapshot(false);
      if (cur === 0) engine.trigger(ch.id, { vel: 96 });
      drawRack();
      return;
    }
    const b = ev.target.closest('[data-rk]');
    if (b) {
      const ci = +b.dataset.channel;
      const ch = project.channels[ci];
      const which = b.dataset.rk;
      if (which === 'mute') ch.mute = !ch.mute;
      if (which === 'solo') ch.solo = !ch.solo;
      if (which === 'arm') { project.channels.forEach(c => c.armed = false); ch.armed = true; }
      snapshot(false);
      engine.applyChannelParams(ch.id);
      window.dispatchEvent(new CustomEvent('daw:redraw'));
      drawRack();
    }
  });
}

/* ============================ mixer ============================ */

export function renderMixer(root, pane) {
  root.innerHTML = `
    <div class="mixer">
      <div class="mixer-body scroll-x" id="mixerBody"></div>
    </div>`;
  drawMixer();
  bindMixer(root);
}

function stripHtml(ch, isMaster, idx) {
  const fdb = isMaster ? project.master.gain : ch.gain;
  return `
    <div class="strip${isMaster ? ' is-master' : ''}${project.selChannel === idx ? ' is-selected' : ''}${ch.mute ? ' is-muted' : ''}"
         data-mix="${isMaster ? 'master' : idx}" data-channel="${ch.id}"
         role="group" aria-label="${esc(ch.name)} mixer strip">
      <div class="fxslots">
        <button class="fxslot${ch.inserts && ch.inserts.length ? ' filled' : ''}" data-fx="0" aria-label="${esc(ch.name)} insert slot 1">
          <span class="ic" data-ic="${ch.inserts && ch.inserts.length ? 'widget' : 'add'}"></span>
          <span>${ch.inserts && ch.inserts.length ? FX_DEFS[ch.inserts[0].id].name : 'Add FX'}</span>
        </button>
        <button class="fxslot" data-fx="1" aria-label="${esc(ch.name)} insert slot 2">
          <span class="ic" data-ic="add"></span><span>Add FX</span>
        </button>
      </div>
      <div class="strip-knobs">
        <div class="strip-knob-cell">
          <span class="cap">Gain</span>
          <span class="knob knob-gain" data-knob="gain" role="slider" tabindex="0"
            aria-label="${esc(ch.name)} gain" aria-valuenow="${fdb}" aria-valuetext="${fdb} dB"><canvas class="knob-cv"></canvas></span>
          <span class="val" data-read="gain">${fdb > 0 ? '+' : ''}${fdb.toFixed(1)}</span>
        </div>
        <div class="strip-knob-cell">
          <span class="cap">Pan</span>
          <span class="knob knob-pan" data-knob="pan" role="slider" tabindex="0"
            aria-label="${esc(ch.name)} pan" aria-valuenow="${ch.pan}" aria-valuetext="${ch.pan < 0 ? Math.round(-ch.pan * 50) + '% L' : ch.pan > 0 ? Math.round(ch.pan * 50) + '% R' : 'centre'}"><canvas class="knob-cv"></canvas></span>
          <span class="val" data-read="pan">${panLabel(ch.pan)}</span>
        </div>
      </div>
      <div class="strip-msr">
        <span class="fader" data-fader="level" role="slider" tabindex="0"
          aria-label="${esc(ch.name)} level" aria-valuenow="${fdb}" aria-valuetext="${fdb} dB">
          <i class="fader-track"></i><i class="fader-fill"></i><i class="fader-unity"></i><i class="fader-thumb"></i>
        </span>
        <span class="meter" data-meter aria-hidden="true"><i></i><i class="meter-cap"></i></span>
        <span class="strip-read" data-read="dB">${faderRead(fdb)}</span>
      </div>
      <div class="strip-btns">
        <button class="tbtn${ch.mute ? ' on' : ''}" data-mix-btn="mute" aria-label="Mute ${esc(ch.name)}" aria-pressed="${ch.mute}"><span class="ic" data-ic="volume-cross"></span></button>
        <button class="tbtn solo${ch.solo ? ' on' : ''}" data-mix-btn="solo" aria-label="Solo ${esc(ch.name)}" aria-pressed="${ch.solo}"><span class="ic" data-ic="volume-loud"></span></button>
        <button class="tbtn arm${ch.armed ? ' on' : ''}" data-mix-btn="arm" aria-label="Arm ${esc(ch.name)}" aria-pressed="${ch.armed}"><span class="ic" data-ic="record"></span></button>
      </div>
      <div class="strip-sends">
        <div class="strip-knob-cell">
          <span class="cap">Rev</span>
          <span class="knob knob-sm" data-send="reverbA" role="slider" tabindex="0" aria-label="${esc(ch.name)} reverb send"
            aria-valuenow="${sendAmount(ch, 'reverbA')}" aria-valuetext="${Math.round(sendAmount(ch, 'reverbA') * 100)} percent"><canvas class="knob-cv"></canvas></span>
        </div>
        <div class="strip-knob-cell">
          <span class="cap">Del</span>
          <span class="knob knob-sm" data-send="delayA" role="slider" tabindex="0" aria-label="${esc(ch.name)} delay send"
            aria-valuenow="${sendAmount(ch, 'delayA')}" aria-valuetext="${Math.round(sendAmount(ch, 'delayA') * 100)} percent"><canvas class="knob-cv"></canvas></span>
        </div>
      </div>
      <div class="strip-name">${esc(ch.name)}</div>
      <div class="strip-src">${esc(KIND_LABEL[ch.kind] || ch.kind)}</div>
    </div>`;
}

const KIND_LABEL = {
  logdrum: 'Log Drum', piano: 'Piano', sine: 'Sine', saw: 'Saw Bass', square: 'Square',
  kick: 'Kick', snare: 'Snare', clap: 'Clap', shaker: 'Shaker', rim: 'Rim', hat: 'Hat', ohat: 'Open Hat',
};
const panLabel = p => p === 0 ? 'C' : (p < 0 ? 'L' + Math.round(-p * 50) : 'R' + Math.round(p * 50));
const faderRead = db => db <= -59.5 ? '−∞' : (db > 0 ? '+' : '') + db.toFixed(1);
const sendAmount = (ch, id) => (ch.sends.find(s => s.returnId === id) || {}).amount || 0;

export function drawMixer() {
  const body = $('#mixerBody');
  if (!body) return;
  let html = '';
  project.channels.forEach((ch, i) => { html += stripHtml(ch, false, i); });
  html += stripHtml({ id: 'master', name: 'Master', kind: 'master', gain: project.master.gain, pan: 0, mute: false, solo: false, armed: false, inserts: [], sends: [] }, true, 'master');
  body.innerHTML = html;
  hydrateIcons(body);

  /* live controls */
  $$('.strip', body).forEach(strip => {
    const isMaster = strip.dataset.mix === 'master';
    const ch = isMaster ? { id: 'master', gain: project.master.gain, pan: 0, sends: [] } : project.channels[+strip.dataset.mix];
    const gainK = new Knob($('[data-knob="gain"]', strip), {
      min: -60, max: 6, step: 0.5, value: ch.gain, label: isMaster ? 'Master' : ch.name + ' gain',
      format: v => (v <= -59.5 ? '−∞' : (v > 0 ? '+' : '') + v.toFixed(1)) + ' dB',
      onChange: v => {
        if (isMaster) { project.master.gain = v; engine.setMasterDb(v); }
        else { ch.gain = v; engine.applyChannelParams(ch.id); }
        $('[data-read="gain"]', strip).textContent = (v > 0 ? '+' : '') + v.toFixed(1);
      },
    });
    const panK = new Knob($('[data-knob="pan"]', strip), {
      min: -1, max: 1, step: 0.02, value: ch.pan || 0, label: (ch.name || 'Master') + ' pan',
      format: v => panLabel(v),
      onChange: v => {
        ch.pan = v;
        if (!isMaster) engine.applyChannelParams(ch.id);
        $('[data-read="pan"]', strip).textContent = panLabel(v);
      },
    });
    const fad = new Fader($('[data-fader="level"]', strip), {
      value: ch.gain, label: (ch.name || 'Master') + ' level',
      onChange: v => {
        if (isMaster) { project.master.gain = v; engine.setMasterDb(v); }
        else { ch.gain = v; engine.applyChannelParams(ch.id); }
        gainK.set(v, true);
        $('[data-read="gain"]', strip).textContent = (v > 0 ? '+' : '') + v.toFixed(1);
        $('[data-read="dB"]', strip).textContent = faderRead(v);
      },
    });
    strip.__knobs = { gain: gainK, pan: panK, fader: fad };
    $$('[data-send]', strip).forEach(kn => {
      const rid = kn.dataset.send;
      const amt = sendAmount(ch, rid);
      new Knob(kn, {
        min: 0, max: 1, step: 0.01, value: amt, label: rid === 'reverbA' ? 'Reverb send' : 'Delay send',
        format: v => Math.round(v * 100) + '%',
        onChange: v => {
          const s = ch.sends.find(x => x.returnId === rid);
          if (s) s.amount = v; else ch.sends.push({ returnId: rid, amount: v });
          if (!isMaster) engine.rebuildChannel(ch.id);
        },
      });
    });
  });
}

function bindMixer(root) {
  if (root.dataset.bound) return;
  root.dataset.bound = '1';
  root.addEventListener('click', ev => {
    const sel = ev.target.closest('.strip');
    if (sel && !ev.target.closest('.knob,.fader,button')) {
      document.querySelectorAll('.strip').forEach(s => s.classList.remove('is-selected'));
      sel.classList.add('is-selected');
      return;
    }
    const fx = ev.target.closest('[data-fx]');
    if (fx) { openFxMenu(fx, fx.closest('.strip').dataset.channel); return; }
    const btn = ev.target.closest('[data-mix-btn]');
    if (btn) {
      const stripEl = btn.closest('.strip');
      const isMaster = stripEl.dataset.mix === 'master';
      const ch = isMaster ? project.master : project.channels[+stripEl.dataset.mix];
      const which = btn.dataset.mixBtn;
      if (which === 'mute') ch.mute = !ch.mute;
      if (which === 'solo') ch.solo = !ch.solo;
      if (which === 'arm') { project.channels.forEach(c => c.armed = false); ch.armed = true; }
      btn.classList.toggle('on', ch.mute || ch.solo || ch.armed);
      btn.setAttribute('aria-pressed', String(ch.mute || ch.solo || ch.armed));
      snapshot(false);
      if (!isMaster) { engine.applyChannelParams(ch.id); syncStripStates(); }
      return;
    }
  });
}

function syncStripStates() {
  $$('.strip').forEach(strip => {
    const isMaster = strip.dataset.mix === 'master';
    const ch = isMaster ? project.master : project.channels[+strip.dataset.mix];
    if (!ch) return;
    strip.classList.toggle('is-muted', !!ch.mute);
    $$('[data-mix-btn]', strip).forEach(b => {
      const w = b.dataset.mixBtn;
      const on = w === 'mute' ? ch.mute : w === 'solo' ? ch.solo : ch.armed;
      b.classList.toggle('on', !!on);
      b.setAttribute('aria-pressed', String(!!on));
    });
  });
}

/* ============================ piano roll ============================ */

/* one document-level drag listener for every piano roll instance */
let activeRoll = null;
let rollDrag = null;

const KEY_H = 14;          /* px per semitone */
const PITCH_MIN = 24;      /* C-1 */
const PITCH_MAX = 108;
const ROW_COUNT = PITCH_MAX - PITCH_MIN + 1;
const VEL_H = 42;

export const roll = {
  zoomX: 48,               /* px per step */
  scrollY: 0,
  scrollStep: 0,
  fold: false,
  snapScale: true,
  showAll: false,
  velocityLane: true,
  tool: 'pencil',
  sel: new Set(),
};

/* visible keys, honouring fold */
function visibleKeys() {
  if (!roll.fold) return Array.from({ length: ROW_COUNT }, (_, i) => PITCH_MIN + i);
  const used = new Set();
  for (const ch of project.channels) for (const n of ch.notes) used.add(n.pitch);
  const scale = project.scale.on && !roll.showAll;
  const set = new Set(used);
  if (scale) {
    for (let p = PITCH_MIN; p <= PITCH_MAX; p++) if (inScale(p, project.scale.root, project.scale.mode)) set.add(p);
  } else set.add(60);
  const keys = [...set].filter(p => p >= PITCH_MIN && p <= PITCH_MAX).sort((a, b) => a - b);
  return keys.length ? keys : [60];
}

function keyY(pitch) {
  const keys = visibleKeys();
  const i = keys.indexOf(pitch);
  return i < 0 ? null : (keys.length - 1 - i) * KEY_H;
}

export function renderRoll(root, pane) {
  root.innerHTML = `
    <div class="roll">
      <div class="roll-top">
        <button class="pill" id="rollScale" aria-haspopup="dialog" aria-expanded="false">
          <span class="ic" data-ic="scale"></span><span class="val" id="rollScaleLabel">Minor</span>
          <span class="chev"></span>
        </button>
        <button class="pill" id="rollGrid" aria-haspopup="listbox" aria-expanded="false">
          <span class="ic" data-ic="widget"></span><span class="val">1/16</span><span class="chev"></span>
        </button>
        <button class="tbtn" id="rollFold" aria-label="Fold unused keys" aria-pressed="false" data-tip="Fold keys"><span class="ic" data-ic="reorder"></span></button>
        <button class="tbtn" id="rollVelLane" aria-label="Velocity lane" aria-pressed="true" data-tip="Velocity lane"><span class="ic" data-ic="slider-vertical"></span></button>
        <label class="pill" style="gap:5px;height:28px;padding:0 9px;margin-left:2px">
          <input type="checkbox" id="rollSnap" checked hidden><span class="ic" data-ic="magnet"></span>Snap
        </label>
        <span class="roll-spacer"></span>
        <div class="roll-cols">
          <button class="tbtn" id="rollNotes" data-tip="Notes" aria-label="Notes"><span class="ic" data-ic="music-note"></span></button>
          <button class="tbtn" id="rollVelBtn" data-tip="Velocity" aria-label="Velocity tool"><span class="ic" data-ic="slider-horizontal"></span></button>
          <button class="tbtn" id="rollSelBtn" data-tip="Select" aria-label="Select tool"><span class="ic" data-ic="cursor"></span></button>
        </div>
      </div>
      <div class="roll-body">
        <div class="roll-keys" id="rollKeys">
          <div class="roll-keys-ruler">Pitch</div>
        </div>
        <div class="roll-canvas-wrap" id="rollWrap">
          <canvas class="roll-canvas" id="rollCanvas"></canvas>
          <div class="roll-stat" id="rollStat"></div>
        </div>
      </div>
      <div class="roll-vlane" id="rollVlane">
        <canvas class="roll-vlane-canvas" id="rollVcanv"></canvas>
      </div>
    </div>`;
  const canvas = $('#rollCanvas', root);
  const keys = $('#rollKeys', root);
  const wrap = $('#rollWrap', root);

  /* key stripe — the skeuomorphic playable surface */
  const drawKeys = () => {
    const list = visibleKeys();
    let html = '';
    list.forEach((p, i) => {
      const y = (list.length - 1 - i) * KEY_H;
      const pc = ((p % 12) + 12) % 12;
      const black = [1, 3, 6, 8, 10].includes(pc);
      const scaleOn = project.scale.on && !roll.showAll;
      const inS = inScale(p, project.scale.root, project.scale.mode);
      const isRoot = pc === project.scale.root;
      const cls = ['pkey', black ? 'pkey-black' : 'pkey-white',
        scaleOn ? (isRoot ? 'is-root' : inS ? 'is-in' : 'is-out') : ''].join(' ');
      html += `<button class="${cls}" style="top:${y}px;height:${KEY_H}px" data-pitch="${p}"
        aria-label="${noteName(p)}${scaleOn && !inS ? ' not in scale' : ''}" aria-pressed="false">
        ${!black ? `<span class="pk-lab">${noteName(p)}</span>` : ''}</button>`;
    });
    keys.innerHTML = `<div class="roll-keys-ruler">Pitch</div>` + html;
  };

  const drawRoll = () => {
    const dpr = Math.min(2, window.devicePixelRatio || 1);
    const w = wrap.clientWidth, h = wrap.clientHeight;
    if (!w || !h) return;
    canvas.width = Math.round(w * dpr); canvas.height = Math.round(h * dpr);
    canvas.style.width = w + 'px'; canvas.style.height = h + 'px';
    const c = canvas.getContext('2d');
    c.setTransform(dpr, 0, 0, dpr, 0, 0);
    c.clearRect(0, 0, w, h);
    const keys = visibleKeys();
    const totalW = totalSteps() * roll.zoomX;
    const stepPx = roll.zoomX;
    const spb = project.stepsPerBeat;
    const scaleOn = project.scale.on && !roll.showAll;

    /* pitch rows */
    keys.forEach((p, i) => {
      const y = (keys.length - 1 - i) * KEY_H;
      const pc = ((p % 12) + 12) % 12;
      const black = [1, 3, 6, 8, 10].includes(pc);
      const inS = inScale(p, project.scale.root, project.scale.mode);
      const isRoot = pc === project.scale.root;
      if (black) { c.fillStyle = cssv('--daw-track'); c.fillRect(0, y, Math.max(w, totalW), KEY_H); }
      if (scaleOn && isRoot) { c.fillStyle = 'rgba(47,111,235,0.10)'; c.fillRect(0, y, Math.max(w, totalW), KEY_H); }
      else if (scaleOn && inS) { c.fillStyle = 'rgba(47,111,235,0.045)'; c.fillRect(0, y, Math.max(w, totalW), KEY_H); }
      c.strokeStyle = cssv('--hair'); c.lineWidth = 1;
      c.strokeRect(0.5, y + 0.5, Math.max(w, totalW) - 1, KEY_H - 1);
    });

    /* vertical grid */
    const barSt = barSteps();
    const x0 = roll.scrollStep * stepPx;
    c.strokeStyle = 'rgba(47,111,235,0.10)'; c.lineWidth = 1;
    for (let s = 0; s * stepPx + x0 <= w; s++) {
      const abs = s + roll.scrollStep;
      const x = s * stepPx + (x0 % stepPx);
      if (abs % barSt === 0) { c.strokeStyle = 'rgba(47,111,235,0.24)'; c.lineWidth = 1; c.beginPath(); c.moveTo(x + .5, 0); c.lineTo(x + .5, h); c.stroke(); }
      else if (abs % spb === 0) { c.strokeStyle = 'rgba(47,111,235,0.10)'; c.lineWidth = 1; c.beginPath(); c.moveTo(x + .5, 0); c.lineTo(x + .5, h); c.stroke(); }
      else { c.strokeStyle = 'rgba(20,29,48,0.045)'; c.beginPath(); c.moveTo(x + .5, 0); c.lineTo(x + .5, h); c.stroke(); }
    }

    /* notes */
    for (const ch of project.channels) {
      if (VOICE_NOTES_KIND[ch.kind] === undefined) continue;
      if (!VOICE_NOTES_KIND[ch.kind]) continue;
      for (const n of ch.notes) {
        const y = keyY(n.pitch);
        if (y == null) continue;
        const x = n.step * stepPx - roll.scrollStep * stepPx;
        if (x + n.len * stepPx < 0 || x > w) continue;
        const selected = roll.sel.has(ch.id + ':' + n.step + ':' + n.pitch);
        c.fillStyle = ch.color;
        c.globalAlpha = selected ? 0.95 : 0.82;
        roundRect(c, x, y + 1, Math.max(3, n.len * stepPx - 1), KEY_H - 3, 3);
        c.fill();
        c.globalAlpha = 1;
        if (selected) {
          c.strokeStyle = '#2f6feb'; c.lineWidth = 2;
          roundRect(c, x, y + 1, Math.max(3, n.len * stepPx - 1), KEY_H - 3, 3);
          c.stroke();
        }
      }
    }

    /* playhead — linear, exempt from easing */
    if (project.playing || project.playhead > 0) {
      const px = (project.playhead - roll.scrollStep) * stepPx;
      if (px >= -2 && px <= w + 2) {
        c.strokeStyle = '#2f6feb'; c.lineWidth = 1.5;
        c.beginPath(); c.moveTo(px, 0); c.lineTo(px, h); c.stroke();
      }
    }
  };

  const drawVlane = () => {
    const lane = $('#rollVlane', root);
    const cv = $('#rollVcanv', root);
    if (!lane || !cv) return;
    lane.hidden = !roll.velocityLane;
    if (!roll.velocityLane) return;
    const dpr = Math.min(2, window.devicePixelRatio || 1);
    const w = lane.clientWidth, h = lane.clientHeight;
    if (!w || !h) return;
    cv.width = Math.round(w * dpr); cv.height = Math.round(h * dpr);
    cv.style.width = w + 'px'; cv.style.height = h + 'px';
    const c = cv.getContext('2d');
    c.setTransform(dpr, 0, 0, dpr, 0, 0);
    c.clearRect(0, 0, w, h);
    const barSt = barSteps(), stepPx = roll.zoomX;
    for (let s = 0; (s + roll.scrollStep) * stepPx <= w; s++) {
      const abs = s + roll.scrollStep;
      const x = s * stepPx;
      if (abs % barSt === 0) { c.strokeStyle = 'rgba(47,111,235,0.24)'; c.beginPath(); c.moveTo(x + .5, 0); c.lineTo(x + .5, h); c.stroke(); }
    }
    c.strokeStyle = cssv('--hair'); c.beginPath();
    for (const v of [32, 64, 96, 127]) {
      const y = h - (v / 127) * h;
      c.moveTo(0, y); c.lineTo(w, y);
    }
    c.stroke();
    for (const ch of project.channels) {
      if (!VOICE_NOTES_KIND[ch.kind]) continue;
      for (const n of ch.notes) {
        const x = n.step * stepPx - roll.scrollStep * stepPx;
        if (x < -20 || x > w) continue;
        const y = h - (n.vel / 127) * h;
        c.strokeStyle = ch.color; c.lineWidth = 2;
        const prev = ch.notes[ch.notes.indexOf(n) - 1];
        if (prev && y !== -1) {
          c.beginPath();
          const px = prev.step * stepPx - roll.scrollStep * stepPx;
          const py = h - (prev.vel / 127) * h;
          c.moveTo(px, py); c.lineTo(x, y); c.stroke();
        }
        c.fillStyle = ch.color;
        c.beginPath(); c.arc(x, y, 2.6, 0, 6.2832); c.fill();
        const selected = roll.sel.has(ch.id + ':' + n.step + ':' + n.pitch);
        if (selected) { c.strokeStyle = '#2f6feb'; c.lineWidth = 1.4; c.stroke(); }
      }
    }
  };

  const draw = () => { drawKeys(); drawRoll(); drawVlane(); updateStat(); };
  root.__draw = draw;
  activeRoll = root;
  root.__stat = () => updateStat();

  /* ---- interactions ---- */
  const pos = e => {
    const b = canvas.getBoundingClientRect();
    const x = e.clientX - b.left, y = e.clientY - b.top;
    const keys = visibleKeys();
    const idx = Math.floor(y / KEY_H);
    const pitch = keys[Math.min(keys.length - 1, Math.max(0, keys.length - 1 - idx))];
    const step = Math.max(0, Math.round((x + roll.scrollStep * roll.zoomX) / roll.zoomX));
    return { x, y, pitch, step };
  };
  const noteAt = (ch, step, pitch) => ch.notes.find(n => n.pitch === pitch && step >= n.step && step < n.step + n.len);

  canvas.addEventListener('pointerdown', e => {
    const p = pos(e);
    const ch = project.channels[project.selChannel];
    if (!ch) return;
    try { if (canvas.setPointerCapture) canvas.setPointerCapture(e.pointerId); } catch (e2) { /* ignore */ }

    /* existing note? */
    for (const c of project.channels) {
      for (const n of c.notes) {
        if (noteAt(c, p.step, p.pitch)) {
          const key = c.id + ':' + n.step + ':' + n.pitch;
          if (roll.tool === 'select') { roll.sel.clear(); roll.sel.add(key); notifyChange(); draw(); return; }
          const resizing = p.x >= (n.step * roll.zoomX - roll.scrollStep * roll.zoomX) + n.len * roll.zoomX - 5;
          rollDrag = { ch: c, note: n, key, mode: resizing ? 'resize' : 'move', x0: e.clientX, s0: n.step, p0: n.pitch, l0: n.len };
          engine.trigger(c.id, { vel: n.vel, dur: n.len * stepDur() });
          return;
        }
      }
    }
    if (roll.tool === 'select') { roll.sel.clear(); drag = { marquee: true, x0: p.x, y0: p.y }; draw(); return; }
    if (!VOICE_NOTES_KIND[ch.kind]) { toast(ch.name + ' is a percussion channel — draw it in the channel rack'); return; }
    let pitch = p.pitch;
    if (roll.snapScale && project.scale.on) pitch = snapToScale(pitch, project.scale.root, project.scale.mode);
    const len = 1;
    const note = { step: p.step, len, pitch, vel: 96 };
    ch.notes.push(note);
    ch.notes.sort((a, b) => a.step - b.step);
    roll.sel.clear();
    roll.sel.add(ch.id + ':' + note.step + ':' + note.pitch);
    engine.trigger(ch.id, { vel: 96, dur: len * stepDur() });
    rollDrag = { ch, note, mode: 'resize', x0: e.clientX, s0: note.step, p0: note.pitch, l0: note.len };
    snapshot(false);
    draw();
  });

  canvas.addEventListener('wheel', e => {
    e.preventDefault();
    if (e.ctrlKey || e.metaKey) { roll.zoomX = clamp(roll.zoomX * (e.deltaY < 0 ? 1.12 : 0.89), 14, 260); }
    else roll.scrollStep = Math.max(0, Math.min(totalSteps() - 4, roll.scrollStep + Math.round(e.deltaY / 8)));
    draw();
  }, { passive: false });

  keys.addEventListener('pointerdown', e => {
    const k = e.target.closest('[data-pitch]');
    if (!k) return;
    const p = +k.dataset.pitch;
    const ch = project.channels[project.selChannel];
    engine.trigger(ch.id, { vel: 100, dur: 0.5, note: p });
    k.classList.add('is-down');
    const up = () => { k.classList.remove('is-down'); k.removeEventListener('pointerup', up); k.removeEventListener('pointerleave', up); };
    k.addEventListener('pointerup', up); k.addEventListener('pointerleave', up);
  });

  canvas.addEventListener('contextmenu', e => {
    e.preventDefault();
    const p = pos(e);
    let hit = null;
    for (const ch of project.channels) for (const n of ch.notes) if (noteAt(ch, p.step, p.pitch)) hit = { ch, n };
    if (hit) {
      roll.sel.clear(); roll.sel.add(hit.ch.id + ':' + hit.n.step + ':' + hit.n.pitch);
      notifyChange(); draw();
      ctxMenu(e.clientX, e.clientY, [
        { icon: 'slider-vertical', label: 'Edit velocity…', fn: () => openVelocityEditor(hit, e.clientX, e.clientY) },
        { icon: 'copy', label: 'Duplicate', key: '⌘D', fn: () => duplicateSelection() },
        { icon: 'trash-bin-minimalistic', label: 'Delete', key: 'Del', danger: true, fn: () => deleteSelection() },
      ]);
    } else {
      ctxMenu(e.clientX, e.clientY, [
        { icon: 'scale', label: 'Set scale…', fn: () => openScalePopover(e.clientX, e.clientY) },
        { icon: 'widget', label: 'Grid division…', fn: () => openGridMenu(e.clientX, e.clientY) },
        { sep: true },
        { icon: 'magnet', label: roll.snapScale ? 'Free pitch' : 'Snap to scale', fn: () => { roll.snapScale = !roll.snapScale; $('#rollSnap', root).checked = roll.snapScale; } },
        { icon: 'reorder', label: roll.fold ? 'Unfold keys' : 'Fold unused keys', fn: () => { roll.fold = !roll.fold; draw(); } },
      ]);
    }
  });

  /* toolbar */
  const setTool = t => {
    roll.tool = t;
    canvas.className = 'roll-canvas' + (t === 'pencil' ? ' is-pencil' : t === 'select' ? ' is-select' : '');
    $$('.roll-cols .tbtn', root).forEach(b => b.classList.toggle('on-solid', b.id === (t === 'velocity' ? 'rollVelBtn' : t === 'select' ? 'rollSelBtn' : 'rollNotes')));
  };
  $('#rollScale', root).addEventListener('click', e => openScalePopover(e.clientX, e.clientY));
  $('#rollGrid', root).addEventListener('click', e => openGridMenu(e.clientX, e.clientY));
  $('#rollFold', root).addEventListener('click', e => { roll.fold = !roll.fold; e.currentTarget.classList.toggle('on-solid', roll.fold); e.currentTarget.setAttribute('aria-pressed', String(roll.fold)); draw(); });
  $('#rollVelLane', root).addEventListener('click', e => {
    roll.velocityLane = !roll.velocityLane;
    e.currentTarget.classList.toggle('on-solid', roll.velocityLane);
    e.currentTarget.setAttribute('aria-pressed', String(roll.velocityLane));
    draw();
  });
  $('#rollNotes', root).addEventListener('click', () => setTool('pencil'));
  $('#rollVelBtn', root).addEventListener('click', () => setTool('velocity'));
  $('#rollSelBtn', root).addEventListener('click', () => setTool('select'));
  $('#rollSnap', root).addEventListener('change', e => { roll.snapScale = e.target.checked; });

  /* velocity lane — click and drag to set note velocity */
  const vcanv = $('#rollVcanv', root);
  if (vcanv) {
    vcanv.style.cursor = 'ns-resize';
    if (!vcanv.dataset.vbound) { vcanv.dataset.vbound = '1'; bindVelocityLane(vcanv); }
  }

  const updateStat = () => {
    const st = $('#rollStat', root);
    if (!st) return;
    const ch = project.channels[project.selChannel];
    const n = ch ? ch.notes.length : 0;
    st.innerHTML = `<span class="ic" data-ic="scale"></span>${roll.sel.size ? roll.sel.size + ' selected' : n + ' notes'} · ${project.scale.on ? ROOTS[project.scale.root] + ' ' + project.scale.mode : 'all keys'}`;
    hydrateIcons(st);
  };
  root.__stat = updateStat;

  draw();
  setTool('pencil');
}

const VOICE_NOTES_KIND = {
  logdrum: true, piano: true, sine: true, saw: true, square: true,
  kick: false, snare: false, clap: false, shaker: false, rim: false, hat: false, ohat: false,
};
export { VOICE_NOTES_KIND };

function roundRect(c, x, y, w, h, r) {
  r = Math.min(r, w / 2, h / 2);
  c.beginPath();
  c.moveTo(x + r, y);
  c.arcTo(x + w, y, x + w, y + h, r);
  c.arcTo(x + w, y + h, x, y + h, r);
  c.arcTo(x, y + h, x, y, r);
  c.arcTo(x, y, x + w, y, r);
  c.closePath();
}
function clamp(v, a, b) { return v < a ? a : v > b ? b : v; }
function notifyChange() { window.dispatchEvent(new CustomEvent('daw:redraw')); }
function toast(msg, icon) { window.dispatchEvent(new CustomEvent('daw:toast', { detail: { msg, icon } })); }

export function deleteSelection() {
  for (const ch of project.channels) ch.notes = ch.notes.filter(n => !roll.sel.has(ch.id + ':' + n.step + ':' + n.pitch));
  roll.sel.clear();
  snapshot(false);
  redrawRoll();
  toast('Notes deleted', 'trash-bin-minimalistic');
}
export function duplicateSelection() {
  let n = 0;
  for (const ch of project.channels) {
    const add = ch.notes.filter(x => roll.sel.has(ch.id + ':' + x.step + ':' + x.pitch)).map(x => ({ step: x.step + barSteps(), len: x.len, pitch: x.pitch, vel: x.vel }));
    ch.notes.push(...add); n += add.length;
  }
  roll.sel.clear();
  snapshot(false);
  redrawRoll();
  toast(n ? `${n} note${n > 1 ? 's' : ''} duplicated` : 'Nothing selected', 'copy');
}
function redrawRoll() { window.dispatchEvent(new CustomEvent('daw:redraw')); }
export { redrawRoll };

/* one velocity-lane binding, reused across every render */
function bindVelocityLane(vcanv) {
  let vdrag = null;
  const vpos = e => {
    const b = vcanv.getBoundingClientRect();
    const h = b.height || 42;
    return {
      step: Math.max(0, Math.round((e.clientX - b.left + roll.scrollStep * roll.zoomX) / roll.zoomX)),
      vel: Math.max(1, Math.min(127, Math.round((1 - (e.clientY - b.top) / h) * 127))),
    };
  };
  const redraw = () => { if (activeRoll && activeRoll.__draw) activeRoll.__draw(); };
  vcanv.addEventListener('pointerdown', e => {
    const p = vpos(e);
    let hit = null;
    for (const ch of project.channels) {
      for (const n of ch.notes) if (n.step === p.step) { hit = { ch, n }; break; }
      if (hit) break;
    }
    if (!hit) return;
    try { if (vcanv.setPointerCapture) vcanv.setPointerCapture(e.pointerId); } catch (e2) { /* ignore */ }
    vdrag = hit;
    hit.n.vel = p.vel;
    engine.trigger(hit.ch.id, { vel: p.vel, dur: hit.n.len * stepDur() });
    redraw();
  });
  vcanv.addEventListener('pointermove', e => {
    if (!vdrag) return;
    vdrag.n.vel = vpos(e).vel;
    redraw();
  });
  vcanv.addEventListener('pointerup', () => {
    if (!vdrag) return;
    vdrag = null;
    snapshot(false);
  });
}


document.addEventListener('pointermove', e => {
  if (!rollDrag || !activeRoll || !activeRoll.isConnected) return;
  const root = activeRoll;
  const canvas = $('#rollCanvas', root);
  const b = canvas.getBoundingClientRect();
  const x = e.clientX - b.left, y = e.clientY - b.top;
  const keys = visibleKeys();
  const pitch = keys[Math.min(keys.length - 1, Math.max(0, keys.length - 1 - Math.floor(y / KEY_H)))];
  const step = Math.max(0, Math.round((x + roll.scrollStep * roll.zoomX) / roll.zoomX));
  if (rollDrag.marquee) {
    roll.sel.clear();
    for (const ch of project.channels) for (const n of ch.notes) {
      const nx = n.step * roll.zoomX - roll.scrollStep * roll.zoomX;
      const ny = keyY(n.pitch);
      if (nx >= Math.min(rollDrag.x0, x) && nx <= Math.max(rollDrag.x0, x) &&
          ny >= Math.min(rollDrag.y0, y) && ny + KEY_H <= Math.max(rollDrag.y0, y) + KEY_H) {
        roll.sel.add(ch.id + ':' + n.step + ':' + n.pitch);
      }
    }
    root.__draw();
    return;
  }
  const dStep = Math.round((e.clientX - rollDrag.x0) / roll.zoomX);
  if (rollDrag.mode === 'resize') {
    rollDrag.note.len = Math.max(1, Math.min(rollDrag.l0 + dStep, totalSteps() - rollDrag.note.step));
  } else {
    let np = rollDrag.p0 - Math.round((e.clientY - (rollDrag.y0 || e.clientY)) / KEY_H);
    np = Math.max(PITCH_MIN, Math.min(PITCH_MAX, np));
    if (roll.snapScale && project.scale.on) np = snapToScale(np, project.scale.root, project.scale.mode);
    rollDrag.note.step = Math.max(0, rollDrag.s0 + dStep);
    rollDrag.note.pitch = np;
    roll.sel.clear();
    roll.sel.add(rollDrag.ch.id + ':' + rollDrag.note.step + ':' + np);
    rollDrag.ch.notes.sort((a, b) => a.step - b.step);
  }
  root.__draw();
});
document.addEventListener('pointerup', () => {
  if (!rollDrag) return;
  if (rollDrag.note) engine.trigger(rollDrag.ch.id, { vel: rollDrag.note.vel, dur: rollDrag.note.len * stepDur() });
  rollDrag = null;
  snapshot(false);
});

function openVelocityEditor(hit, x, y) {
  const { ch, note } = hit;
  const p = el('div', 'popover');
  p.style.left = Math.min(x, window.innerWidth - 200) + 'px';
  p.style.top = Math.min(y, window.innerHeight - 140) + 'px';
  p.innerHTML = `
    <div class="popover-head"><span class="ic" data-ic="slider-vertical"></span>Velocity</div>
    <div style="display:flex;align-items:center;gap:10px">
      <span class="knob" style="--daw-knob-d:34px;--daw-knob-hit:44px" id="velKnob" role="slider" tabindex="0"
        aria-label="Velocity" aria-valuemin="1" aria-valuemax="127" aria-valuenow="${note.vel}"><canvas class="knob-cv"></canvas><span class="knob-face"></span></span>
      <div style="display:flex;flex-direction:column;gap:4px">
        <b style="font-size:15px;font-weight:640;letter-spacing:-.02em">${note.vel}</b>
        <small style="font-size:10px;color:var(--ink-3)">${noteName(note.pitch)} · step ${note.step}</small>
      </div>
    </div>
    <div style="display:flex;gap:6px;margin-top:10px">
      <button class="btn quiet" id="velCancel">Cancel</button>
      <button class="btn primary" id="velOk">Done</button>
    </div>`;
  document.getElementById('overlay').appendChild(p);
  hydrateIcons(p);
  new Knob($('#velKnob', p), { min: 1, max: 127, step: 1, value: note.vel, label: 'Velocity', format: v => String(Math.round(v)), onChange: v => { note.vel = Math.round(v); } });
  $('#velOk', p).addEventListener('click', () => { snapshot(false); redrawRoll(); closePopover(p); });
  $('#velCancel', p).addEventListener('click', () => closePopover(p));
  const k = e => { if (e.key === 'Escape') { closePopover(p); document.removeEventListener('keydown', k); } };
  document.addEventListener('keydown', k);
}
function closePopover(p) { if (p && p.parentNode) p.parentNode.removeChild(p); }

/* ============================ arrangement ============================ */


const proj = project;

const arr = { pxPerStep: 6, scrollBar: 0, lane: 'vol', mode: 'read' };

export function renderArrange(root, pane) {
  root.innerHTML = `
    <div class="arr">
      <div class="arr-top">
        <button class="pill" id="arrNew"><span class="ic" data-ic="add"></span>Pattern</button>
        <button class="pill" id="arrDup"><span class="ic" data-ic="copy"></span>Clone</button>
        <button class="pill" id="arrClear"><span class="ic" data-ic="trash-bin-minimalistic"></span>Clear</button>
        <select class="pill" id="arrLane" aria-label="Automation lane"></select>
        <button class="pill" id="arrMode"><span class="ic" data-ic="pen"></span><span id="arrModeLabel">Read</span><span class="chev"></span></button>
        <span class="roll-spacer" style="flex:1"></span>
        <button class="pill" id="arrZoomOut"><span class="ic" data-ic="minus-circle"></span></button>
        <button class="pill" id="arrZoomIn"><span class="ic" data-ic="add"></span></button>
      </div>
      <div class="arr-body">
        <div class="arr-ruler"><canvas class="arr-ruler-canvas" id="arrRuler"></canvas></div>
        <div class="arr-tracks" id="arrTracks"></div>
        <div class="arr-lanes" id="arrLanes"></div>
      </div>
    </div>`;
  drawArrange(root);
  bindArrange(root);
}

function laneOptions(name, cur) {
  return ['vol', 'pan', 'gain'].includes(cur) ? cur : 'vol';
}

function drawArrange(root) {
  const tracks = $('#arrTracks', root);
  const lanes = $('#arrLanes', root);
  if (!tracks) return;
  const barSt = barSteps();
  const pxStep = arr.pxPerStep;
  const width = totalSteps() * pxStep;

  /* ruler */
  const rc = $('#arrRuler', root);
  if (rc) {
    const dpr = Math.min(2, window.devicePixelRatio || 1);
    const w = rc.clientWidth, h = 18;
    rc.width = w * dpr; rc.height = h * dpr; rc.style.width = w + 'px'; rc.style.height = h + 'px';
    const c = rc.getContext('2d'); c.setTransform(dpr, 0, 0, dpr, 0, 0);
    c.clearRect(0, 0, w, h);
    c.font = '9px InterVariable, system-ui, sans-serif';
    for (let bar = 0; bar * barSt * pxStep <= w; bar++) {
      const x = bar * barSt * pxStep - arr.scrollBar * pxStep;
      if (x < -30) continue;
      c.fillStyle = cssv('--ink-3'); c.textAlign = 'left';
      c.fillText(String(bar + 1), x + 3, 12);
      c.strokeStyle = 'rgba(47,111,235,0.20)'; c.lineWidth = 1;
      c.beginPath(); c.moveTo(x + .5, 4); c.lineTo(x + .5, h); c.stroke();
    }
    /* loop region */
    if (project.loop.on) {
      const lx = project.loop.start * barSt * pxStep - arr.scrollBar * pxStep;
      const lw = (project.loop.end - project.loop.start) * barSt * pxStep;
      c.fillStyle = 'rgba(47,111,235,0.12)';
      c.fillRect(lx, 4, lw, h - 4);
    }
  }

  /* tracks */
  let html = '';
  project.channels.forEach((ch, i) => {
    html += `<div class="arr-track${project.selChannel === i ? ' is-selected' : ''}" data-tr="${i}">
      <span class="arr-track-name"><span class="rack-chip" style="background:${ch.color};width:7px;height:7px"></span>${esc(ch.name)}</span>
      <canvas class="arr-canvas" data-arc="${i}"></canvas>
    </div>`;
  });
  tracks.innerHTML = html;
  $$('.arr-canvas', tracks).forEach((cv, i) => {
    const ch = project.channels[i];
    const dpr = Math.min(2, window.devicePixelRatio || 1);
    const w = cv.clientWidth, h = cv.clientHeight;
    cv.width = w * dpr; cv.height = h * dpr; cv.style.width = w + 'px'; cv.style.height = h + 'px';
    const c = cv.getContext('2d'); c.setTransform(dpr, 0, 0, dpr, 0, 0);
    c.clearRect(0, 0, w, h);
    const barSt2 = barSteps(), px = arr.pxPerStep;
    for (let s = 0; s * px - arr.scrollBar * px <= w; s++) {
      if (s % barSt2 === 0) { c.strokeStyle = 'rgba(47,111,235,0.16)'; c.beginPath(); c.moveTo(s * px - arr.scrollBar * px + .5, 0); c.lineTo(s * px - arr.scrollBar * px + .5, h); c.stroke(); }
      else if (s % project.stepsPerBeat === 0) { c.strokeStyle = 'rgba(20,29,48,0.05)'; c.beginPath(); c.moveTo(s * px - arr.scrollBar * px + .5, 0); c.lineTo(s * px - arr.scrollBar * px + .5, h); c.stroke(); }
    }
    /* clips */
    for (const clip of proj.clips) {
      if (clip.channelId !== ch.id) continue;
      const x = clip.bar * barSt2 * px - arr.scrollBar * px;
      const wd = clip.length * barSt2 * px;
      if (x + wd < 0 || x > w) continue;
      c.fillStyle = ch.color; c.globalAlpha = 0.82;
      rr(c, x, 4, Math.max(6, wd - 2), h - 10, 4); c.fill();
      c.globalAlpha = 1;
      c.fillStyle = 'rgba(255,255,255,0.9)';
      c.font = '10px InterVariable, system-ui, sans-serif';
      c.fillText(clip.name, x + 6, h / 2 + 2);
      /* mini pattern preview inside the clip */
      c.fillStyle = 'rgba(0,0,0,0.22)';
      const pat = ch.pattern;
      for (let s = 0; s < wd - 4; s += 3) {
        const si = s % 64;
        if (pat[si]) { const bh = (pat[si] / 127) * (h - 22); c.fillRect(x + 2 + s, h - 8 - bh, 1.4, bh); }
      }
    }
    /* playhead */
    if (project.playing || project.playhead > 0) {
      const pxh = project.playhead * px - arr.scrollBar * px;
      c.strokeStyle = '#2f6feb'; c.lineWidth = 1.5;
      c.beginPath(); c.moveTo(pxh, 0); c.lineTo(pxh, h); c.stroke();
    }
  });

  /* automation lane */
  const laneName = { vol: 'Volume', pan: 'Pan', gain: 'Pre-gain' };
  html = `<div class="arr-lane${arr.mode !== 'read' ? ' mode' : ''}">
      <span class="arr-lane-name"><span class="ic" data-ic="${arr.mode === 'read' ? 'eye' : 'pen'}"></span>${laneName[arr.lane]} · ${arr.mode}</span>
      <canvas class="arr-lane-canvas" id="arrLaneCv"></canvas>
    </div>`;
  lanes.innerHTML = html;
  const lc = $('#arrLaneCv', root);
  if (lc) {
    const dpr = Math.min(2, window.devicePixelRatio || 1);
    const w = lc.clientWidth, h = lc.clientHeight;
    lc.width = w * dpr; lc.height = h * dpr; lc.style.width = w + 'px'; lc.style.height = h + 'px';
    const c = lc.getContext('2d'); c.setTransform(dpr, 0, 0, dpr, 0, 0);
    c.clearRect(0, 0, w, h);
    const barSt2 = barSteps(), px = arr.pxPerStep;
    for (let s = 0; s * px - arr.scrollBar * px <= w; s++) {
      if (s % barSt2 === 0) { c.strokeStyle = 'rgba(47,111,235,0.16)'; c.beginPath(); c.moveTo(s * px - arr.scrollBar * px + .5, 0); c.lineTo(s * px - arr.scrollBar * px + .5, h); c.stroke(); }
    }
    const ch = project.channels[project.selChannel];
    if (ch) {
      const pts = (ch.auto && ch.auto[arr.lane]) || [];
      if (pts.length > 1) {
        c.strokeStyle = cssv('--accent'); c.lineWidth = 1.6;
        c.beginPath();
        pts.forEach((pt, i) => {
          const x = pt.step * px - arr.scrollBar * px;
          const norm = arr.lane === 'pan' ? (pt.v + 1) / 2 : pt.v;
          const y = h - norm * h;
          i ? c.lineTo(x, y) : c.moveTo(x, y);
        });
        c.stroke();
        pts.forEach(pt => {
          const x = pt.step * px - arr.scrollBar * px;
          const norm = arr.lane === 'pan' ? (pt.v + 1) / 2 : pt.v;
          const y = h - norm * h;
          c.fillStyle = cssv('--accent');
          c.beginPath(); c.arc(x, y, 2.6, 0, 6.2832); c.fill();
        });
      }
      if (project.playing || project.playhead > 0) {
        const pxh = project.playhead * px - arr.scrollBar * px;
        c.strokeStyle = '#2f6feb'; c.lineWidth = 1.5;
        c.beginPath(); c.moveTo(pxh, 0); c.lineTo(pxh, h); c.stroke();
      }
    }
  }
  hydrateIcons(root);
  const laneSel = $('#arrLane', root);
  if (laneSel && !laneSel.dataset.done) {
    laneSel.dataset.done = '1';
    laneSel.innerHTML = `<option value="vol">Volume</option><option value="pan">Pan</option><option value="gain">Pre-gain</option>`;
    laneSel.addEventListener('change', () => { arr.lane = laneSel.value; drawArrange(root); });
  }
}

function rr(c, x, y, w, h, r) {
  r = Math.min(r, w / 2, h / 2);
  c.beginPath(); c.moveTo(x + r, y);
  c.arcTo(x + w, y, x + w, y + h, r); c.arcTo(x + w, y + h, x, y + h, r);
  c.arcTo(x, y + h, x, y, r); c.arcTo(x, y, x + w, y, r); c.closePath();
}

function bindArrange(root) {
  if (root.dataset.bound) return;
  root.dataset.bound = '1';


  const tracks = () => $('#arrTracks', root);
  root.addEventListener('click', ev => {
    const cv = ev.target.closest('.arr-canvas');
    if (!cv) return;
    const i = +cv.dataset.arc;
    project.selChannel = i;
    $$('.arr-track', tracks()).forEach((t, x) => t.classList.toggle('is-selected', x === i));
    window.dispatchEvent(new CustomEvent('daw:redraw'));
  });

  root.addEventListener('dblclick', ev => {
    const cv = ev.target.closest('.arr-canvas');
    if (!cv) return;
    const i = +cv.dataset.arc;
    const ch = project.channels[i];
    const b = cv.getBoundingClientRect();
    const bar = Math.max(0, Math.floor((ev.clientX - b.left + arr.scrollBar * barSteps() * arr.pxPerStep) / (barSteps() * arr.pxPerStep)));
    proj.clips.push({ id: 'c' + Math.random().toString(36).slice(2, 7), channelId: ch.id, bar, length: 4, name: ch.name + ' ' + (bar + 1) + '–' + (bar + 4) });
    snapshot(true);
    drawArrange(root);
  });

  root.addEventListener('contextmenu', ev => {
    const cv = ev.target.closest('.arr-canvas');
    if (!cv) return;
    ev.preventDefault();
    const i = +cv.dataset.arc;
    const ch = project.channels[i];
    const b = cv.getBoundingClientRect();
    const bar = Math.max(0, Math.floor((ev.clientX - b.left + arr.scrollBar * barSteps() * arr.pxPerStep) / (barSteps() * arr.pxPerStep)));
    const clip = proj.clips.find(c => c.channelId === ch.id && bar >= c.bar && bar < c.bar + c.length);
    if (clip) {
      window.dispatchEvent(new CustomEvent('daw:redraw'));
      ctxMenu(ev.clientX, ev.clientY, [
        { icon: 'copy', label: 'Clone pattern', key: '⌘D', fn: () => { proj.clips.push({ ...clip, id: 'c' + Math.random().toString(36).slice(2, 7), bar: clip.bar + clip.length, name: clip.name }); snapshot(true); drawArrange(root); } },
        { icon: 'refresh', label: 'Vary (humanise)', fn: () => { humaniseClip(ch); snapshot(true); drawArrange(root); } },
        { icon: 'scale', label: 'Set loop here', fn: () => { project.loop = { on: true, start: clip.bar, end: clip.bar + clip.length }; window.dispatchEvent(new CustomEvent('daw:transport')); } },
        { sep: true },
        { icon: 'trash-bin-minimalistic', label: 'Delete clip', danger: true, fn: () => { proj.clips = proj.clips.filter(c => c !== clip); snapshot(true); drawArrange(root); } },
      ]);
    } else {
      ctxMenu(ev.clientX, ev.clientY, [
        { icon: 'add', label: 'Place pattern here', fn: () => { proj.clips.push({ id: 'c' + Math.random().toString(36).slice(2, 7), channelId: ch.id, bar, length: 4, name: ch.name + ' ' + (bar + 1) + '–' + (bar + 4) }); snapshot(true); drawArrange(root); } },
        { icon: 'scale', label: 'Set loop to this bar', fn: () => { project.loop = { on: true, start: bar, end: bar + 1 }; window.dispatchEvent(new CustomEvent('daw:transport')); } },
      ]);
    }
  });

  root.addEventListener('click', ev => {
    const id = ev.target.id;
    if (id === 'arrNew') {
      const bar = Math.max(0, ...proj.clips.map(c => c.bar + c.length));
      const ch = project.channels[project.selChannel];
      proj.clips.push({ id: 'c' + Math.random().toString(36).slice(2, 7), channelId: ch.id, bar, length: 4, name: ch.name + ' ' + (bar + 1) + '–' + (bar + 4) });
      snapshot(true);
      drawArrange(root);
    } else if (id === 'arrDup') {
      const ch = project.channels[project.selChannel];
      const last = proj.clips.filter(c => c.channelId === ch.id).pop();
      if (!last) return;
      proj.clips.push({ ...last, id: 'c' + Math.random().toString(36).slice(2, 7), bar: last.bar + last.length, name: ch.name + ' ' + (last.bar + 1 + last.length) + '–' + (last.bar + last.length + last.length) });
      snapshot(true);
      drawArrange(root);
    } else if (id === 'arrClear') {
      proj.clips = [];
      snapshot(true);
      drawArrange(root);
    } else if (id === 'arrMode') {
      const order = ['read', 'touch', 'latch', 'write'];
      arr.mode = order[(order.indexOf(arr.mode) + 1) % order.length];
      const l = $('#arrModeLabel', root);
      if (l) l.textContent = arr.mode[0].toUpperCase() + arr.mode.slice(1);
      drawArrange(root);
    } else if (id === 'arrZoomIn') {
      arr.pxPerStep = Math.min(40, arr.pxPerStep * 1.25);
      drawArrange(root);
    } else if (id === 'arrZoomOut') {
      arr.pxPerStep = Math.max(2, arr.pxPerStep / 1.25);
      drawArrange(root);
    }
  });

  root.addEventListener('change', ev => {
    if (ev.target.id !== 'arrLane') return;
    arr.lane = ev.target.value;
    drawArrange(root);
  });
}

function humaniseClip(ch) {
  for (let i = 0; i < ch.pattern.length; i++) {
    if (ch.pattern[i] > 0) ch.pattern[i] = Math.max(20, Math.min(127, ch.pattern[i] + (Math.random() * 44 - 22)));
  }
}
function chipOf(ch) { return ch; }

/* ============================ virtual keys ============================ */

const KEYBOARD_MAP = {
  KeyZ: 0, KeyS: 1, KeyX: 2, KeyD: 3, KeyC: 4, KeyV: 5, KeyG: 6,
  KeyB: 7, KeyH: 8, KeyN: 9, KeyJ: 10, Comma: 11,
  KeyQ: 12, Digit2: 13, KeyW: 14, Digit3: 15, KeyE: 16, KeyR: 17,
  Digit5: 18, KeyT: 19, Digit6: 20, KeyY: 21, Digit7: 22, KeyU: 23,
  KeyI: 24, Digit9: 25, KeyO: 26, Digit0: 27, KeyP: 28,
};

export const octave = { base: 4 };
export function setOctave(d) {
  octave.base = Math.max(0, Math.min(8, octave.base + d));
  window.dispatchEvent(new CustomEvent('daw:redraw'));
}

export function renderKeys(root, pane) {
  root.innerHTML = `<div class="vkeys"><div class="vkeys-inner" id="vkInner"></div></div>`;
  const inner = $('#vkInner', root);
  const draw = () => {
    let html = '';
    const start = (octave.base + 1) * 12;
    for (let i = 0; i < 29; i++) {
      const p = start + i;
      const pc = ((p % 12) + 12) % 12;
      const black = [1, 3, 6, 8, 10].includes(pc);
      const scaleOn = project.scale.on;
      const cls = [
        'vk', black ? 'vk-black' : 'vk-white',
        scaleOn ? (pc === project.scale.root ? 'is-root' : inScale(p, project.scale.root, project.scale.mode) ? '' : 'is-out') : '',
      ].join(' ');
      const hit = ev => {
        const ch = project.channels[project.selChannel];
        const vel = 90 + Math.round((1 - (ev.clientY - ev.currentTarget.getBoundingClientRect().top) / ev.currentTarget.clientHeight) * 36);
        ev.currentTarget.classList.add('is-down');
        engine.trigger(ch.id, { vel, dur: 0.6, note: p });
        recordNote(ch, p, vel);
        const up = () => ev.currentTarget.classList.remove('is-down');
        setTimeout(up, 140);
      };
      html += `<button class="${cls}" data-pitch="${p}" aria-label="${noteName(p)}" aria-pressed="false"
        data-act-key="${p}">${!black ? `<span class="vk-lab">${noteName(p).replace(/[0-9-]/g, '')}</span>` : ''}</button>`;
    }
    inner.innerHTML = html;
    hydrateIcons(inner);
    $$('.vk', inner).forEach(b => b.addEventListener('pointerdown', e => {
      e.preventDefault();
      b.focus();
      const p = +b.dataset.pitch;
      const ch = project.channels[project.selChannel];
      const vel = 96;
      b.classList.add('is-down');
      engine.trigger(ch.id, { vel, dur: 0.55, note: p });
      recordNote(ch, p, vel);
      const up = () => { b.classList.remove('is-down'); b.removeEventListener('pointerup', up); };
      b.addEventListener('pointerup', up);
      setTimeout(() => b.classList.remove('is-down'), 200);
    }));
  };
  root.__draw = draw;
  draw();
}

/* write a note into the armed channel's piano roll when the transport runs */
export function recordNote(ch, pitch, vel) {
  if (!project.playing || !ch.armed) return;
  const step = Math.round(project.playhead);
  ch.notes.push({ step, len: Math.max(1, project.stepsPerBeat / 4), pitch, vel: Math.round(vel) });
  ch.notes.sort((a, b) => a.step - b.step);
  snapshot(false);
  window.dispatchEvent(new CustomEvent('daw:redraw'));
}

export const KEYBOARD = KEYBOARD_MAP;

/* ============================ sample editor ============================ */

export function renderEditor(root, pane) {
  const ch = project.channels[project.selChannel];
  root.innerHTML = `
    <div class="editor">
      <div class="editor-top">
        <span class="pane-title" style="font-size:11px;font-weight:620">${esc(ch ? ch.name : 'Sample')}</span>
        <button class="pill" data-ed="trim"><span class="ic" data-ic="scissors"></span>Trim</button>
        <button class="pill" data-ed="fadein"><span class="ic" data-ic="minus-circle"></span>Fade in</button>
        <button class="pill" data-ed="fadeout"><span class="ic" data-ic="minus-circle"></span>Fade out</button>
        <button class="pill" data-ed="norm"><span class="ic" data-ic="bolt"></span>Normalise</button>
        <button class="pill" data-ed="reverse"><span class="ic" data-ic="refresh"></span>Reverse</button>
        <span style="flex:1"></span>
        <button class="pill" data-ed="gain"><span class="ic" data-ic="volume-loud"></span>Gain</button>
      </div>
      <div class="editor-wave">
        <canvas class="editor-wave-canvas" id="edCanv"></canvas>
        <div class="editor-fades" id="edFades"></div>
      </div>
      <div class="editor-foot">
        <span id="edInfo">Selection: none</span>
        <span class="sp"></span>
        <span id="edDur">0.000 s · 0 passes</span>
      </div>
    </div>`;
  const cv = $_('#edCanv', root);
  const sel = { start: 0.2, end: 0.8 };
  const draw = () => {
    const dpr = Math.min(2, window.devicePixelRatio || 1);
    const w = cv.clientWidth, h = cv.clientHeight;
    if (!w || !h) return;
    cv.width = w * dpr; cv.height = h * dpr; cv.style.width = w + 'px'; cv.style.height = h + 'px';
    const c = cv.getContext('2d'); c.setTransform(dpr, 0, 0, dpr, 0, 0);
    c.clearRect(0, 0, w, h);
    /* a synthetic waveform derived from the channel's pattern density */
    const pat = ch ? ch.pattern : [];
    const n = 240;
    c.fillStyle = cssv1('--accent');
    for (let i = 0; i < n; i++) {
      const t = i / n;
      const idx = Math.floor(t * 64);
      const amp = (pat[idx] || 0) / 127;
      const env = Math.sin(Math.PI * t) * 0.7 + 0.3;
      const jitter = Math.abs(Math.sin(i * 2.7) * Math.cos(i * 0.9)) * 0.4;
      const bh = (amp * env * (0.5 + jitter)) * (h / 2 - 6);
      const x = t * w;
      c.fillRect(x, h / 2 - bh, Math.max(1, w / n - 0.5), bh * 2);
    }
    /* selection */
    c.fillStyle = cssv1('--accent-tint');
    c.fillRect(sel.start * w, 0, (sel.end - sel.start) * w, h);
    c.strokeStyle = cssv1('--daw-track'); c.lineWidth = 1;
    for (let i = 0; i < 8; i++) { const x = (i / 8) * w; c.beginPath(); c.moveTo(x + .5, 0); c.lineTo(x + .5, h); c.stroke(); }
    const info = $_('#edInfo', root);
    if (info) info.textContent = `Selection: ${(sel.start * 2).toFixed(2)} – ${(sel.end * 2).toFixed(2)} s`;
  };
  root.__draw = draw;
  draw();
}
function cssv1(n) { return getComputedStyle(document.documentElement).getPropertyValue(n).trim(); }

/* ============================ fx chain ============================ */


export function renderFx(root, pane) {
  const ch = project.channels[project.selChannel] || project.channels[0];
  const draw = () => {
    let html = `<div class="fxchain">
      <div class="fxchain-top"><span class="pane-title" style="font-size:11px;font-weight:620">${esc(ch.name)}</span>
      <span style="flex:1"></span><span class="tb-label">Insert chain</span></div>
      <div class="fxchain-body scroll">`;
    (ch.inserts || []).forEach((fx, i) => {
      const def = FX_DEFS[fx.id];
      html += `<div class="fxrow${fx.bypass ? ' is-bypassed' : ''}" data-fxi="${i}">
        <span class="tbtn" data-fxt="power" data-fxi="${i}" aria-label="Bypass ${esc(def.name)}" aria-pressed="${!fx.bypass}"><span class="ic" data-ic="bolt"></span></span>
        <span class="fxrow-name"><b>${esc(def.name)}</b><small>${esc(def.note || '')}</small></span>
        <span class="fxrow-params">`;
      (def.params || []).forEach(p => {
        const cur = (fx.params.find(x => x.id === p.id) || {}).value ?? p.value;
        html += `<span class="fxparam"><span class="cap">${esc(p.label)}</span>
          <span class="knob knob-sm" data-fxp="${p.id}" data-fxi="${i}" role="slider" tabindex="0"
            aria-label="${esc(def.name)} ${esc(p.label)}" aria-valuenow="${cur}" aria-valuetext="${fmtP(p, cur)}"><canvas class="knob-cv"></canvas><span class="knob-face"></span></span>
          <span class="val" data-fxread="${p.id}">${fmtP(p, cur)}</span></span>`;
      });
      html += `</span>
        <button class="iconbtn quiet sm" data-fxrm="${i}" aria-label="Remove ${esc(def.name)}"><span class="ic" data-ic="trash-bin-minimalistic"></span></button>
      </div>`;
    });
    html += `<button class="fx-add" id="fxAdd"><span class="ic" data-ic="add"></span>Add effect</button></div></div>`;
    root.innerHTML = html;
    hydrateIcons(root);
    $$('.knob[data-fxp]', root).forEach(kn => {
      const fx = ch.inserts[+kn.dataset.fxi];
      const p = FX_DEFS[fx.id].params.find(x => x.id === kn.dataset.fxp);
      const cur = (fx.params.find(x => x.id === p.id) || {}).value ?? p.value;
      new Knob(kn, {
        min: p.min, max: p.max, step: p.step, value: cur,
        label: FX_DEFS[fx.id].name + ' ' + p.label,
        format: v => fmtP(p, v),
        onChange: v => {
          const e = fx.params.find(x => x.id === p.id);
          if (e) e.value = v; else fx.params.push({ id: p.id, value: v });
          const r = kn.closest('.fxparam').querySelector('[data-fxread]');
          if (r) r.textContent = fmtP(p, v);
          engine.rebuildChannel(ch.id);
        },
      });
    });
  };
  root.__draw = draw;
  draw();

  root.addEventListener('click', ev => {
    const rm = ev.target.closest('[data-fxrm]');
    if (rm) {
      const i = +rm.dataset.fxrm;
      ch.inserts.splice(i, 1);
      snapshot(false);
      engine.rebuildChannel(ch.id);
      draw();
      return;
    }
    const pw = ev.target.closest('[data-fxt]');
    if (pw) {
      const i = +pw.dataset.fxi;
      ch.inserts[i].bypass = !ch.inserts[i].bypass;
      snapshot(false);
      engine.rebuildChannel(ch.id);
      draw();
      return;
    }
    if (ev.target.closest('#fxAdd')) {
      window.dispatchEvent(new CustomEvent('daw:fxadd', { detail: { ch, root } }));
    }
  });
}

function fmtP(p, v) {
  if (p.opts) return p.opts[Math.round(v)] ?? String(v);
  return p.fmt ? p.fmt(v) : String(Math.round(v * 100) / 100);
}
/* ============================ renderer registry ============================ */
paneRenderers.browser = renderBrowser;
paneRenderers.rack = renderRack;
paneRenderers.roll = renderRoll;
paneRenderers.mixer = renderMixer;
paneRenderers.arrange = renderArrange;
paneRenderers.keys = renderKeys;
paneRenderers.editor = renderEditor;
paneRenderers.fx = renderFx;

/* open a pane that is not part of any preset: editor, fx chain, keys */
export function openPane(id, float = true) {
  if (panes[id]) { maximizePane(id); return panes[id]; }
  return buildPane(id, float ? 'float' : 'docked');
}
export function closePane(id) {
  const p = panes[id];
  if (!p) return;
  p.node.remove();
  delete panes[id];
  notify();
}
export function isOpen(id) { return !!panes[id]; }
export function listPanes() { return panes; }
