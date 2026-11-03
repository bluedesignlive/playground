/* ==========================================================================
   Smart DAW — control primitives
   Knob, fader, meter. One light: one top highlight + one rim, light from above.
   The value ring is the only accent. See docs/COMPONENTS.md §2, §3, §8.
   ========================================================================== */

const $ = (s, r = document) => r.querySelector(s);
const $$ = (s, r = document) => Array.from(r.querySelectorAll(s));
const el = (tag, cls, html) => {
  const n = document.createElement(tag);
  if (cls) n.className = cls;
  if (html != null) n.innerHTML = html;
  return n;
};
export { $, $$, el };

/* ---------- icons ---------- */
export function hydrateIcons(root = document) {
  $$('[data-ic]', root).forEach(n => {
    if (n.dataset.icDone) return;
    const name = n.dataset.ic;
    n.innerHTML =
      `<svg class="o" aria-hidden="true"><use href="#i-${name}-outline"></use></svg>` +
      `<svg class="d" aria-hidden="true"><use href="#i-${name}-duotone"></use></svg>` +
      `<svg class="f" aria-hidden="true"><use href="#i-${name}-filled"></use></svg>`;
    n.dataset.icDone = '1';
    n.setAttribute('aria-hidden', 'true');
  });
}

/* ---------- format helpers ---------- */
export function fmtFxParam(param, raw) {
  if (param.opts) return param.opts[Math.round(raw)] ?? String(raw);
  return param.fmt ? param.fmt(raw) : String(raw);
}
export function fmtTime(v) {
  const m = Math.floor(v / 60), s = Math.floor(v % 60), ms = Math.floor((v % 1) * 1000);
  return `${m}:${String(s).padStart(2, '0')}.${String(ms).padStart(3, '0')}`;
}
export function esc(s) {
  return String(s).replace(/[&<>"']/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]));
}

/* ---------- drag primitive ---------- */
/* one pointer capture routine for all continuous controls */
function beginDrag(ev, onMove, onUp) {
  ev.preventDefault();
  const move = e => onMove(e);
  const up = e => {
    window.removeEventListener('pointermove', move);
    window.removeEventListener('pointerup', up);
    window.removeEventListener('pointercancel', up);
    onUp && onUp(e);
  };
  window.addEventListener('pointermove', move);
  window.addEventListener('pointerup', up);
  window.addEventListener('pointercancel', up);
}

/* ==========================================================================
   KNOB
   ========================================================================== */

const KNOB_START = -135, KNOB_SWEEP = 270;

export class Knob {
  constructor(host, opts) {
    this.host = host;
    this.min = opts.min ?? 0;
    this.max = opts.max ?? 1;
    this.step = opts.step ?? 0.01;
    this.value = opts.value ?? this.min;
    this.label = opts.label || 'Value';
    this.format = opts.format || (v => String(v));
    this.defaultValue = opts.value ?? this.min;
    this.onChange = opts.onChange || (() => {});
    this.tip = null;

    this.host.classList.add('knob');
    this.host.setAttribute('role', 'slider');
    if (!this.host.getAttribute('aria-label')) this.host.setAttribute('aria-label', this.label);
    this.host.setAttribute('aria-valuemin', String(this.min));
    this.host.setAttribute('aria-valuemax', String(this.max));
    this.cv = this.host.querySelector('canvas');
    if (!this.cv) { this.cv = el('canvas'); this.host.appendChild(this.cv); }
    this.cv.className = 'knob-cv';
    this.face = this.host.querySelector('.knob-face');
    if (!this.face) { this.face = el('span', 'knob-face'); this.host.appendChild(this.face); }

    this._bind();
    this.render();
  }

  get norm() { return (this.value - this.min) / (this.max - this.min); }
  get angle() { return KNOB_START + KNOB_SWEEP * clamp01(this.norm); }

  set(v, silent) {
    let x = v;
    if (this.step) x = Math.round(x / this.step) * this.step;
    x = Math.min(this.max, Math.max(this.min, x));
    if (Math.abs(x - this.value) < 1e-9) return false;
    this.value = x;
    this.render();
    if (!silent) this.onChange(x);
    return true;
  }
  add(d, silent) { return this.set(this.value + d, silent); }

  render() {
    const dpr = Math.min(2, window.devicePixelRatio || 1);
    const box = this.host.clientWidth || 36;
    if (this.cv.width !== Math.round(box * dpr)) {
      this.cv.width = Math.round(box * dpr);
      this.cv.height = Math.round(box * dpr);
    }
    const c = this.cv.getContext('2d');
    c.setTransform(dpr, 0, 0, dpr, 0, 0);
    c.clearRect(0, 0, box, box);
    const cx = box / 2, cy = box / 2, r = box / 2 - 1.5;

    /* ring track */
    c.lineWidth = 3;
    c.lineCap = 'round';
    c.strokeStyle = cssv('--daw-ring-track');
    c.beginPath();
    c.arc(cx, cy, r, rad(KNOB_START), rad(KNOB_START + KNOB_SWEEP));
    c.stroke();

    /* value arc — the one accent */
    const a = this.angle;
    if (Math.abs(a - KNOB_START) > 0.01) {
      c.strokeStyle = cssv('--accent-glow');
      c.lineWidth = 3;
      c.beginPath();
      c.arc(cx, cy, r, rad(KNOB_START), rad(a));
      c.stroke();
    }

    /* pointer on the face */
    const fr = r - 3.5;
    const px = cx + Math.cos(rad(a)) * fr;
    const py = cy + Math.sin(rad(a)) * fr;
    c.fillStyle = cssv('--ink-2');
    c.beginPath();
    c.arc(px, py, 1.6, 0, Math.PI * 2);
    c.fill();

    this.host.setAttribute('aria-valuenow', String(this.value));
    this.host.setAttribute('aria-valuetext', this.format(this.value));
  }

  _bind() {
    const h = this.host;
    h.addEventListener('pointerdown', ev => {
      if (h.getAttribute('aria-disabled') === 'true') return;
      h.focus();
      this._y = ev.clientY;
      this._x = ev.clientX;
      this._start = this.value;
      this._fine = ev.shiftKey;
      h.classList.add('is-dragging');
      this._showTip();
      beginDrag(ev, e => {
        const dy = this._y - e.clientY;
        const dx = e.clientX - this._x;
        const fine = e.shiftKey;
        const range = this.max - this.min;
        const span = (fine ? 900 : 220);
        let v = this._start + (dy * range) / span - (dx * range) / 900;
        this.set(v);
        this._showTip();
      }, () => {
        h.classList.remove('is-dragging');
        this._hideTip();
      });
    });
    h.addEventListener('dblclick', () => this.set(this.defaultValue));
    h.addEventListener('wheel', ev => {
      if (h.getAttribute('aria-disabled') === 'true') return;
      ev.preventDefault();
      const fine = ev.shiftKey;
      const range = this.max - this.min;
      const inc = range / (ev.deltaY < 0 ? (fine ? 240 : 60) : -(fine ? 240 : 60));
      this.set(this.value + inc * Math.sign(ev.deltaY));
    }, { passive: false });
    h.addEventListener('keydown', ev => {
      const big = (this.max - this.min) / 10;
      const map = {
        ArrowUp: this.step, ArrowDown: -this.step,
        ArrowRight: this.step, ArrowLeft: -this.step,
        PageUp: big, PageDown: -big,
        Home: this.min, End: this.max,
      };
      if (ev.key in map) {
        ev.preventDefault();
        if (ev.key === 'Home' || ev.key === 'End') this.set(map[ev.key]);
        else this.add(map[ev.key]);
      }
    });
  }

  _showTip() {
    if (!this.tip) {
      this.tip = el('div', 'knob-tip');
      document.body.appendChild(this.tip);
    }
    const b = this.host.getBoundingClientRect();
    this.tip.innerHTML = `<b>${esc(this.label)}</b> · ${esc(this.format(this.value))}`;
    this.tip.style.left = b.left + b.width / 2 + 'px';
    this.tip.style.top = b.top - 6 + 'px';
    this.tip.style.transform = 'translate(-50%,-100%)';
    this.tip.classList.add('show');
  }
  _hideTip() {
    if (this.tip) this.tip.classList.remove('show');
  }
}

/* ==========================================================================
   FADER
   ========================================================================== */

const FADER_MIN = -60, FADER_MAX = 6, FADER_UNITY = 0;

export class Fader {
  constructor(host, opts = {}) {
    this.host = host;
    this.range = opts.range || [FADER_MIN, FADER_MAX];
    this.value = opts.value != null ? opts.value : 0;
    this.onChange = opts.onChange || (() => {});
    this.defaultValue = 0;
    this.host.classList.add('fader');
    this.host.setAttribute('role', 'slider');
    this.host.setAttribute('aria-valuemin', String(this.range[0]));
    this.host.setAttribute('aria-valuemax', String(this.range[1]));
    this.host.setAttribute('aria-orientation', 'vertical');
    if (!this.host.getAttribute('aria-label')) this.host.setAttribute('aria-label', 'Level');

    this.track = this.host.querySelector('.fader-track');
    this.fill = this.host.querySelector('.fader-fill');
    this.thumb = this.host.querySelector('.fader-thumb');
    const unity = this.host.querySelector('.fader-unity');
    if (!this.track) { this.track = el('i', 'fader-track'); this.host.appendChild(this.track); }
    if (!this.fill) { this.fill = el('i', 'fader-fill'); this.host.appendChild(this.fill); }
    this.u = unity || el('i', 'fader-unity');
    if (!this.u.parentNode) this.host.appendChild(this.u);
    if (!this.thumb) { this.thumb = el('i', 'fader-thumb'); this.host.appendChild(this.thumb); }
    this._bind();
    this.render();
  }

  set(v, silent) {
    const x = Math.min(this.range[1], Math.max(this.range[0], v));
    if (Math.abs(x - this.value) < 1e-9) return false;
    this.value = x;
    this.render();
    if (!silent) this.onChange(x);
    return true;
  }
  get db() { return this.value; }
  get pct() {
    const [a, b] = this.range;
    return (this.value - a) / (b - a);
  }

  render() {
    const h = this.host.clientHeight || 140;
    const t = h - 14;                       /* thumb travel in px */
    const unityPct = (FADER_UNITY - this.range[0]) / (this.range[1] - this.range[0]);
    const y = t * (1 - this.pct);
    this.thumb.style.bottom = y + 'px';
    const uy = t * (1 - unityPct);
    this.u.style.bottom = (uy + 7) + 'px';
    this.fill.style.bottom = uy + 'px';
    this.fill.style.height = Math.max(0, uy - y) + 'px';
    this.host.setAttribute('aria-valuenow', this.value.toFixed(1));
    this.host.setAttribute('aria-valuetext', this.fmt(this.value));
  }
  fmt(v) { return v <= this.range[0] + 0.5 ? '−∞ dB' : (v > 0 ? '+' : '') + v.toFixed(1) + ' dB'; }

  _bind() {
    this.host.addEventListener('pointerdown', ev => {
      ev.preventDefault();
      this.host.focus();
      const b = this.host.getBoundingClientRect();
      this._y = ev.clientY;
      this._start = this.value;
      this.host.classList.add('is-dragging');
      beginDrag(ev, e => {
        const dy = this._y - e.clientY;
        const fine = e.shiftKey;
        const range = this.range[1] - this.range[0];
        const span = (b.height - 14) * (fine ? 3.2 : 1);
        this.set(this._start + (dy * range) / span);
      }, () => this.host.classList.remove('is-dragging'));
    });
    this.host.addEventListener('dblclick', () => this.set(this.defaultValue));
    this.host.addEventListener('wheel', ev => {
      ev.preventDefault();
      this.set(this.value - Math.sign(ev.deltaY) * (ev.shiftKey ? 0.2 : 1));
    }, { passive: false });
    this.host.addEventListener('keydown', ev => {
      const map = {
        ArrowUp: 1, ArrowDown: -1, ArrowRight: 1, ArrowLeft: -1,
        PageUp: 6, PageDown: -6,
        Home: this.range[0], End: this.range[1],
        Backspace: 0, Delete: 0,
      };
      if (ev.key in map) {
        ev.preventDefault();
        if (ev.key === 'Home' || ev.key === 'End') this.set(map[ev.key]);
        else this.set(this.value + map[ev.key]);
      }
    });
  }
}

/* ==========================================================================
   meter — flat bars, no springs, collapse instantly when silent
   ========================================================================== */
export function setMeter(node, level, peak, hold) {
  const db = gainToDb01(level);
  const bar = node.firstElementChild;
  const cap = node.querySelector('.meter-cap');
  const pct = clamp01((db + 60) / 66) * 100;
  bar.style.height = pct.toFixed(1) + '%';
  bar.classList.toggle('warn', pct > 78 && pct <= 96);
  bar.classList.toggle('peak', pct > 96);
  if (cap) {
    const cdb = gainToDb01(hold);
    const cpct = clamp01((cdb + 60) / 66) * 100;
    cap.style.bottom = cpct.toFixed(1) + '%';
    cap.style.opacity = cdb > 0.0005 ? '.9' : '0';
  }
}
export function gainToDb01(g) { return 20 * Math.log10(Math.max(g, 1e-6)); }
export function clamp01(v) { return v < 0 ? 0 : v > 1 ? 1 : v; }

/* ---------- css var reader (canvas needs computed values) ---------- */
const cssCache = new Map();
export function cssv(name) {
  if (cssCache.has(name)) return cssCache.get(name);
  const v = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
  cssCache.set(name, v);
  return v;
}
export function cssvTheme(sel, name) {
  const k = sel + '|' + name;
  if (cssCache.has(k)) return cssCache.get(k);
  const v = getComputedStyle(document.querySelector(sel) || document.documentElement).getPropertyValue(name).trim();
  cssCache.set(k, v);
  return v;
}

/* token colours move with the theme; invalidate on switch */
export function invalidateCssCache() { cssCache.clear(); }
export { cssCache };

function rad(d) { return (d * Math.PI) / 180; }
export { rad };
