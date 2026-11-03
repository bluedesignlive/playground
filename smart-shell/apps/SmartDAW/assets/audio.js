/* ==========================================================================
   Smart DAW — audio engine
   Web Audio synthesis, routing, scheduling, and metering. No sample assets:
   every voice is generated. See docs/AUDIO.md.
   ========================================================================== */

const NOTE_NAMES = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'];
/* scheduler constants — seconds for LOOKAHEAD / SCHEDULE_AHEAD, ms for TICK */
export const LOOKAHEAD = 0.10;      // s of tolerance for late timer wakeups
export const SCHEDULE_AHEAD = 0.20; // s of events committed in advance
export const TICK = 25;             // ms

export function noteToFreq(m) {
  return 440 * Math.pow(2, (m - 69) / 12);
}
export function noteName(m) {
  return NOTE_NAMES[((m % 12) + 12) % 12] + (Math.floor(m / 12) - 1);
}
function clamp(v, a, b) { return v < a ? a : v > b ? b : v; }
function dbToGain(db) { return Math.pow(10, db / 20); }
function gainToDb(g) { return 20 * Math.log10(Math.max(g, 1e-6)); }
export { dbToGain, gainToDb, clamp };

/* ============================ fx definitions ============================ */

const tanhCurve = (() => {
  const n = 2048, c = new Float32Array(n);
  for (let i = 0; i < n; i++) {
    const x = (i / (n - 1)) * 2 - 1;
    c[i] = Math.tanh(x * 2.2) / Math.tanh(2.2);
  }
  return c;
})();

const makeIR = (ctx, size, decay) => {
  const n = Math.max(1, Math.floor(ctx.sampleRate * size));
  const buf = ctx.createBuffer(2, n, ctx.sampleRate);
  for (let ch = 0; ch < 2; ch++) {
    const d = buf.getChannelData(ch);
    for (let i = 0; i < n; i++) {
      d[i] = (Math.random() * 2 - 1) * Math.pow(1 - i / n, decay);
    }
  }
  return buf;
};

export const FX_DEFS = {
  eq3: {
    name: 'EQ 3-Band', icon: 'tuning-2', note: 'Shelving lows and highs, one peak in the middle',
    params: [
      { id: 'low', label: 'Low', min: -18, max: 18, value: 0, step: 0.5, fmt: v => v.toFixed(1) + ' dB' },
      { id: 'mid', label: 'Mid', min: -18, max: 18, value: 0, step: 0.5, fmt: v => v.toFixed(1) + ' dB' },
      { id: 'midf', label: 'Mid F', min: 200, max: 4000, value: 1200, step: 10, log: true, fmt: v => (v >= 1000 ? (v / 1000).toFixed(1) + ' kHz' : Math.round(v) + ' Hz') },
      { id: 'midq', label: 'Mid Q', min: 0.3, max: 8, value: 0.8, step: 0.1, fmt: v => v.toFixed(1) },
      { id: 'high', label: 'High', min: -18, max: 18, value: 0, step: 0.5, fmt: v => v.toFixed(1) + ' dB' },
    ],
    build(ctx) {
      const low = ctx.createBiquadFilter(); low.type = 'lowshelf'; low.frequency.value = 220;
      const mid = ctx.createBiquadFilter(); mid.type = 'peaking'; mid.frequency.value = 1200; mid.Q.value = 0.8;
      const high = ctx.createBiquadFilter(); high.type = 'highshelf'; high.frequency.value = 3200;
      low.connect(mid); mid.connect(high);
      return {
        input: low, output: high,
        set(p, v) {
          if (p === 'low') low.gain.value = v;
          if (p === 'mid') mid.gain.value = v;
          if (p === 'midf') mid.frequency.value = v;
          if (p === 'midq') mid.Q.value = v;
          if (p === 'high') high.gain.value = v;
        },
      };
    },
  },

  comp: {
    name: 'Compressor', icon: 'slider-vertical', note: 'Glue for drums and bass',
    params: [
      { id: 'thresh', label: 'Thresh', min: -48, max: 0, value: -20, step: 0.5, fmt: v => v.toFixed(1) + ' dB' },
      { id: 'ratio', label: 'Ratio', min: 1, max: 20, value: 4, step: 0.5, fmt: v => v.toFixed(1) + ':1' },
      { id: 'attack', label: 'Attack', min: 0.001, max: 0.1, value: 0.006, step: 0.001, log: true, fmt: v => Math.round(v * 1000) + ' ms' },
      { id: 'release', label: 'Release', min: 0.02, max: 0.8, value: 0.16, step: 0.01, log: true, fmt: v => Math.round(v * 1000) + ' ms' },
      { id: 'makeup', label: 'Makeup', min: 0, max: 24, value: 2, step: 0.5, fmt: v => v.toFixed(1) + ' dB' },
    ],
    build(ctx) {
      const c = ctx.createDynamicsCompressor();
      const mk = ctx.createGain();
      c.connect(mk);
      c.threshold.value = -20; c.ratio.value = 4; c.attack.value = 0.006; c.release.value = 0.16; mk.gain.value = 1.26;
      return {
        input: c, output: mk,
        set(p, v) {
          if (p === 'thresh') c.threshold.value = v;
          if (p === 'ratio') c.ratio.value = v;
          if (p === 'attack') c.attack.value = v;
          if (p === 'release') c.release.value = v;
          if (p === 'makeup') mk.gain.value = dbToGain(v);
        },
      };
    },
  },

  limit: {
    name: 'Limiter', icon: 'bolt', note: 'Ceiling for the master bus',
    params: [
      { id: 'ceiling', label: 'Ceiling', min: -12, max: -0.3, value: -1, step: 0.1, fmt: v => v.toFixed(1) + ' dB' },
      { id: 'release', label: 'Release', min: 0.01, max: 0.3, value: 0.06, step: 0.01, log: true, fmt: v => Math.round(v * 1000) + ' ms' },
    ],
    build(ctx) {
      const c = ctx.createDynamicsCompressor();
      c.threshold.value = -1; c.ratio.value = 20; c.attack.value = 0.002; c.release.value = 0.06; c.knee.value = 0;
      return {
        input: c, output: c,
        set(p, v) {
          if (p === 'ceiling') c.threshold.value = v;
          if (p === 'release') c.release.value = v;
        },
      };
    },
  },

  gate: {
    name: 'Gate', icon: 'target', note: 'Cleans up bleed between hits',
    params: [
      { id: 'thresh', label: 'Thresh', min: -70, max: -10, value: -45, step: 1, fmt: v => v.toFixed(0) + ' dB' },
      { id: 'attack', label: 'Attack', min: 0.001, max: 0.05, value: 0.004, step: 0.001, log: true, fmt: v => Math.round(v * 1000) + ' ms' },
      { id: 'hold', label: 'Hold', min: 0.005, max: 0.5, value: 0.12, step: 0.005, log: true, fmt: v => Math.round(v * 1000) + ' ms' },
    ],
    build(ctx) {
      const c = ctx.createDynamicsCompressor();
      c.threshold.value = -45; c.ratio.value = 20; c.attack.value = 0.004; c.release.value = 0.12; c.knee.value = 0;
      return {
        input: c, output: c,
        set(p, v) {
          if (p === 'thresh') c.threshold.value = v;
          if (p === 'attack') c.attack.value = v;
          if (p === 'hold') c.release.value = v;
        },
      };
    },
  },

  reverb: {
    name: 'Reverb', icon: 'soundwave', note: 'Generated IR, no assets to load',
    params: [
      { id: 'size', label: 'Size', min: 0.15, max: 4, value: 1.6, step: 0.05, log: true, fmt: v => v.toFixed(2) + ' s' },
      { id: 'decay', label: 'Decay', min: 1, max: 6, value: 2.4, step: 0.1, fmt: v => v.toFixed(1) },
      { id: 'predelay', label: 'Pre', min: 0, max: 0.12, value: 0.012, step: 0.002, log: true, fmt: v => Math.round(v * 1000) + ' ms' },
      { id: 'lowcut', label: 'Low Cut', min: 60, max: 900, value: 260, step: 10, log: true, fmt: v => Math.round(v) + ' Hz' },
      { id: 'mix', label: 'Mix', min: 0, max: 1, value: 0.28, step: 0.01, fmt: v => Math.round(v * 100) + '%' },
    ],
    build(ctx) {
      const conv = ctx.createConvolver();
      const hp = ctx.createBiquadFilter(); hp.type = 'highpass'; hp.frequency.value = 260;
      const pre = ctx.createDelay(0.2); pre.delayTime.value = 0.012;
      const wet = ctx.createGain(); wet.gain.value = 0.28;
      const dry = ctx.createGain(); dry.gain.value = 0.72;
      pre.connect(conv); conv.connect(hp); hp.connect(wet);
      conv.buffer = makeIR(ctx, 1.6, 2.4);
      return {
        input: pre,
        outputs: [pre, dry],
        output: pre,
        wetNode: wet, dryNode: dry,
        set(p, v) {
          if (p === 'size') conv.buffer = makeIR(ctx, v, this._decay || 2.4);
          if (p === 'decay') { this._decay = v; conv.buffer = makeIR(ctx, this._size || 1.6, v); }
          if (p === 'predelay') pre.delayTime.value = v;
          if (p === 'lowcut') hp.frequency.value = v;
          if (p === 'mix') {
            wet.gain.value = v; dry.gain.value = 1 - v * 0.9;
            this._mix = v;
          }
        },
        _mix: 0.28,
      };
    },
  },

  delay: {
    name: 'Delay', icon: 'clock-circle', note: 'Tempo-synced, ping-pong',
    params: [
      { id: 'time', label: 'Time', min: 0.03, max: 1.2, value: 0.375, step: 0.005, sync: true, fmt: v => (v * 1000).toFixed(0) + ' ms' },
      { id: 'feedback', label: 'Feedb', min: 0, max: 0.85, value: 0.42, step: 0.01, fmt: v => Math.round(v * 100) + '%' },
      { id: 'lowpass', label: 'LP', min: 600, max: 14000, value: 4200, step: 50, log: true, fmt: v => v >= 1000 ? (v / 1000).toFixed(1) + ' kHz' : Math.round(v) + ' Hz' },
      { id: 'ping', label: 'Ping', min: 0, max: 1, value: 0.6, step: 0.01, fmt: v => Math.round(v * 100) + '%' },
      { id: 'mix', label: 'Mix', min: 0, max: 1, value: 0.2, step: 0.01, fmt: v => Math.round(v * 100) + '%' },
    ],
    build(ctx) {
      const inG = ctx.createGain();
      const dL = ctx.createDelay(1.5), dR = ctx.createDelay(1.5);
      const fb = ctx.createGain(); fb.gain.value = 0.42;
      const lp = ctx.createBiquadFilter(); lp.type = 'lowpass'; lp.frequency.value = 4200;
      const panL = ctx.createStereoPanner(), panR = ctx.createStereoPanner();
      const wet = ctx.createGain(); wet.gain.value = 0.2;
      dL.delayTime.value = 0.375; dR.delayTime.value = 0.375;
      panL.pan.value = -0.6; panR.pan.value = 0.6;
      inG.connect(dL); inG.connect(dR);
      dL.connect(lp); dR.connect(lp);
      lp.connect(fb); fb.connect(dR); fb.connect(dL);
      dL.connect(panL); dR.connect(panR);
      panL.connect(wet); panR.connect(wet);
      return {
        input: inG, output: wet, wetNode: wet,
        set(p, v) {
          if (p === 'time') { dL.delayTime.value = v; dR.delayTime.value = v; }
          if (p === 'feedback') fb.gain.value = v;
          if (p === 'lowpass') lp.frequency.value = v;
          if (p === 'ping') { panL.pan.value = -v; panR.pan.value = v; }
          if (p === 'mix') wet.gain.value = v;
        },
      };
    },
  },

  filter: {
    name: 'Filter', icon: 'filter', note: 'Resonant sweep — the amapiano build tool',
    params: [
      { id: 'type', label: 'Type', min: 0, max: 2, value: 0, step: 1, opts: ['LP', 'BP', 'HP'], fmt: v => ['LP', 'BP', 'HP'][Math.round(v)] },
      { id: 'cutoff', label: 'Cutoff', min: 60, max: 16000, value: 5200, step: 20, log: true, fmt: v => v >= 1000 ? (v / 1000).toFixed(1) + ' kHz' : Math.round(v) + ' Hz' },
      { id: 'res', label: 'Reso', min: 0.2, max: 18, value: 1.4, step: 0.1, fmt: v => v.toFixed(1) },
      { id: 'drive', label: 'Drive', min: 0, max: 1, value: 0.15, step: 0.01, fmt: v => Math.round(v * 100) + '%' },
    ],
    build(ctx) {
      const f = ctx.createBiquadFilter(); f.type = 'lowpass'; f.frequency.value = 5200; f.Q.value = 1.4;
      const sh = ctx.createWaveShaper(); sh.curve = tanhCurve;
      const mix = ctx.createGain(); mix.gain.value = 0.15;
      const bypass = ctx.createGain(); bypass.gain.value = 0.85;
      f.connect(sh); sh.connect(mix); f.connect(bypass);
      const sum = ctx.createGain();
      mix.connect(sum); bypass.connect(sum);
      return {
        input: f, output: sum,
        set(p, v) {
          if (p === 'type') f.type = ['lowpass', 'bandpass', 'highpass'][Math.round(v)];
          if (p === 'cutoff') f.frequency.value = v;
          if (p === 'res') f.Q.value = v;
          if (p === 'drive') { mix.gain.value = v; bypass.gain.value = 1 - v * 0.7; }
        },
      };
    },
  },

  sat: {
    name: 'Saturation', icon: 'waterdrop', note: 'Warmth without grit',
    params: [
      { id: 'drive', label: 'Drive', min: 0, max: 1, value: 0.3, step: 0.01, fmt: v => Math.round(v * 100) + '%' },
      { id: 'tone', label: 'Tone', min: 800, max: 12000, value: 3600, step: 50, log: true, fmt: v => v >= 1000 ? (v / 1000).toFixed(1) + ' kHz' : Math.round(v) + ' Hz' },
      { id: 'level', label: 'Level', min: -24, max: 12, value: 0, step: 0.5, fmt: v => v.toFixed(1) + ' dB' },
    ],
    build(ctx) {
      const sh = ctx.createWaveShaper(); sh.curve = tanhCurve;
      const pre = ctx.createGain(); pre.gain.value = 1.6;
      const lp = ctx.createBiquadFilter(); lp.type = 'lowpass'; lp.frequency.value = 3600;
      const out = ctx.createGain(); out.gain.value = 1;
      pre.connect(sh); sh.connect(lp); lp.connect(out);
      return {
        input: pre, output: out,
        set(p, v) {
          if (p === 'drive') pre.gain.value = 1 + v * 5;
          if (p === 'tone') lp.frequency.value = v;
          if (p === 'level') out.gain.value = dbToGain(v);
        },
      };
    },
  },

  chorus: {
    name: 'Chorus', icon: 'stream', note: 'Width for pads and stabs',
    params: [
      { id: 'rate', label: 'Rate', min: 0.05, max: 6, value: 0.7, step: 0.05, log: true, fmt: v => v.toFixed(2) + ' Hz' },
      { id: 'depth', label: 'Depth', min: 0, max: 0.008, value: 0.0028, step: 0.0002, log: true, fmt: v => Math.round(v * 1000) + ' ms' },
      { id: 'spread', label: 'Spread', min: 0, max: 1, value: 0.7, step: 0.01, fmt: v => Math.round(v * 100) + '%' },
      { id: 'mix', label: 'Mix', min: 0, max: 1, value: 0.35, step: 0.01, fmt: v => Math.round(v * 100) + '%' },
    ],
    build(ctx) {
      const inG = ctx.createGain();
      const mk = (pan) => {
        const d = ctx.createDelay(0.06); d.delayTime.value = 0.02;
        const lfo = ctx.createOscillator(); lfo.type = 'sine';
        const amt = ctx.createGain(); amt.gain.value = 0.0028;
        const p = ctx.createStereoPanner(); p.pan.value = pan;
        const w = ctx.createGain(); w.gain.value = 0.35;
        lfo.frequency.value = 0.7; lfo.connect(amt); amt.connect(d.delayTime);
        lfo.start();
        d.connect(p); p.connect(w);
        return { d, lfo, amt, w };
      };
      const l = mk(-0.7), r = mk(0.7);
      const dry = ctx.createGain(); dry.gain.value = 0.65;
      inG.connect(l.d); inG.connect(r.d); inG.connect(dry);
      return {
        input: inG, output: l.w, extraOutputs: [r.w], dryNode: dry,
        set(p, v) {
          for (const s of [l, r]) {
            if (p === 'rate') s.lfo.frequency.value = v;
            if (p === 'depth') s.amt.gain.value = v;
            if (p === 'spread') { s.l.w.gain.value = v * 0.35; }
            if (p === 'mix') { s.l.w.gain.value = v * 0.35 * Math.max(0.001, v); }
          }
          if (p === 'mix') dry.gain.value = 1 - v * 0.7;
        },
      };
    },
  },
};

export const FX_ORDER = ['eq3', 'comp', 'gate', 'filter', 'sat', 'chorus', 'delay', 'reverb', 'limit'];
export const FX_LABEL = id => (FX_DEFS[id] ? FX_DEFS[id].name : id);
export const FX_PARAMS = id => (FX_DEFS[id] ? FX_DEFS[id].params : []);

/* ============================ voices ============================ */
/* Every voice is generated; nothing is fetched. Voices return to a free list on
   `onended` so nothing is allocated while audio is running.                  */

class Voice {
  constructor(ctx, dest, kind, freq, vel, dur, p) {
    this.ctx = ctx;
    this.dead = false;
    this.out = ctx.createGain();
    this.out.connect(dest);
    this[`v_${kind}`](ctx, this.out, freq, vel, dur, p || {});
  }
  release() { if (this.dead) return; this.dead = true; this.out.disconnect(); }
  /* log drum — pitched percussion, the amapiano workhorse */
  v_logdrum(ctx, out, f, vel, dur, p) {
    const t = ctx.currentTime;
    const osc = ctx.createOscillator(); osc.type = 'triangle';
    const g = ctx.createGain();
    const lp = ctx.createBiquadFilter(); lp.type = 'lowpass';
    const body = ctx.createBiquadFilter(); body.type = 'peaking'; body.frequency.value = 180; body.gain.value = 4; body.Q.value = 1.2;
    const d = p.decay || dur;
    osc.frequency.setValueAtTime(f, t);
    osc.frequency.exponentialRampToValueAtTime(Math.max(20, f * 0.72), t + 0.04);
    g.gain.setValueAtTime(0.0001, t);
    g.gain.exponentialRampToValueAtTime(vel, t + 0.004);
    g.gain.exponentialRampToValueAtTime(0.0005, t + d);
    lp.frequency.setValueAtTime(3200, t);
    lp.frequency.exponentialRampToValueAtTime(420, t + 0.09);
    lp.Q.value = 0.9;
    osc.connect(lp); lp.connect(body); body.connect(g); g.connect(out);
    osc.start(t); osc.stop(t + d + 0.05);
    osc.onended = () => this.release();
  }
  /* piano / stab — subtractive poly */
  v_piano(ctx, out, f, vel, dur, p) {
    const t = ctx.currentTime;
    const g = ctx.createGain();
    const lp = ctx.createBiquadFilter(); lp.type = 'lowpass'; lp.frequency.value = 1900; lp.Q.value = 0.7;
    const a = ctx.createOscillator(); a.type = 'triangle'; a.frequency.value = f;
    const b = ctx.createOscillator(); b.type = 'sine'; b.frequency.value = f * 1.0015;
    const c = ctx.createOscillator(); c.type = 'sine'; c.frequency.value = f * 0.9985;
    const pa = ctx.createStereoPanner(); pa.pan.value = -0.18;
    const pc = ctx.createStereoPanner(); pc.pan.value = 0.18;
    const d = p.decay || dur;
    g.gain.setValueAtTime(0.0001, t);
    g.gain.exponentialRampToValueAtTime(vel * 0.55, t + 0.006);
    g.gain.exponentialRampToValueAtTime(0.0005, t + d);
    a.connect(lp); b.connect(pa); c.connect(pc);
    pa.connect(lp); pc.connect(lp); lp.connect(g);
    a.start(t); b.start(t); c.start(t);
    const end = t + d + 0.05;
    a.stop(end); b.stop(end); c.stop(end);
    a.onended = () => this.release();
  }
  v_sine(ctx, out, f, vel, dur, p) {
    const t = ctx.currentTime;
    const osc = ctx.createOscillator(); osc.type = p.type || 'sine';
    osc.frequency.value = f;
    const g = ctx.createGain();
    const d = p.decay || dur;
    g.gain.setValueAtTime(0.0001, t);
    g.gain.exponentialRampToValueAtTime(vel, t + Math.min(0.02, d * 0.3));
    g.gain.exponentialRampToValueAtTime(0.0005, t + d);
    osc.connect(g); g.connect(out);
    osc.start(t); osc.stop(t + d + 0.05);
    osc.onended = () => this.release();
  }
  v_saw(ctx, out, f, vel, dur, p) { this.v_sine(ctx, out, f, vel * 0.8, dur, Object.assign({ type: 'sawtooth' }, p)); }
  v_square(ctx, out, f, vel, dur, p) { this.v_sine(ctx, out, f, vel * 0.7, dur, Object.assign({ type: 'square' }, p)); }

  /* noise burst helper */
  _noise(ctx, dur) {
    const src = ctx.createBufferSource();
    src.buffer = engineNoise;
    return src;
  }
  v_kick(ctx, out, f, vel, dur, p) {
    const t = ctx.currentTime;
    const osc = ctx.createOscillator(); osc.type = 'sine';
    osc.frequency.setValueAtTime(120, t);
    osc.frequency.exponentialRampToValueAtTime(52, t + 0.09);
    const g = ctx.createGain();
    g.gain.setValueAtTime(vel, t);
    g.gain.exponentialRampToValueAtTime(0.0005, t + (p.decay || 0.22));
    osc.connect(g); g.connect(out);
    osc.start(t); osc.stop(t + 0.4);
    /* click transient */
    const n = ctx.createBufferSource(); n.buffer = engineNoise;
    const nf = ctx.createBiquadFilter(); nf.type = 'bandpass'; nf.frequency.value = 1800; nf.Q.value = 0.8;
    const ng = ctx.createGain(); ng.gain.setValueAtTime(vel * 0.35, t);
    ng.gain.exponentialRampToValueAtTime(0.0005, t + 0.02);
    n.connect(nf); nf.connect(ng); ng.connect(out);
    n.start(t); n.stop(t + 0.06);
    osc.onended = () => this.release();
  }
  _noiseHit(ctx, out, vel, dur, freq, q, type) {
    const t = ctx.currentTime;
    const n = ctx.createBufferSource(); n.buffer = engineNoise;
    const f = ctx.createBiquadFilter(); f.type = type; f.frequency.value = freq; f.Q.value = q;
    const g = ctx.createGain();
    g.gain.setValueAtTime(vel, t);
    g.gain.exponentialRampToValueAtTime(0.0005, t + dur);
    n.connect(f); f.connect(g); g.connect(out);
    n.start(t); n.stop(t + dur + 0.05);
    n.onended = () => this.release();
  }
  v_snare(ctx, out, f, vel, dur, p) {
    this._noiseHit(ctx, out, vel * 0.85, p.decay || 0.18, 1800, 0.7, 'bandpass');
    this.v_sine(ctx, out, 185, vel * 0.5, 0.1, { type: 'triangle', decay: 0.1 });
  }
  v_clap(ctx, out, f, vel, dur, p) {
    const t = ctx.currentTime;
    for (let i = 0; i < 3; i++) {
      const n = ctx.createBufferSource(); n.buffer = engineNoise;
      const f1 = ctx.createBiquadFilter(); f1.type = 'bandpass'; f1.frequency.value = 1100; f1.Q.value = 1.6;
      const g = ctx.createGain();
      const st = t + i * 0.009;
      g.gain.setValueAtTime(vel * 0.8, st);
      g.gain.exponentialRampToValueAtTime(0.0005, st + (i === 2 ? 0.14 : 0.02));
      n.connect(f1); f1.connect(g); g.connect(out);
      n.start(st); n.stop(st + 0.2);
      if (i === 2) n.onended = () => this.release();
    }
  }
  v_shaker(ctx, out, f, vel, dur, p) { this._noiseHit(ctx, out, vel * 0.6, p.decay || 0.045, 6200, 0.6, 'highpass'); }
  v_rim(ctx, out, f, vel, dur, p) { this._noiseHit(ctx, out, vel * 0.9, p.decay || 0.028, 2100, 5, 'bandpass'); }
  v_hat(ctx, out, f, vel, dur, p) { this._noiseHit(ctx, out, vel * 0.55, p.decay || 0.08, 8200, 0.7, 'highpass'); }
  v_ohat(ctx, out, f, vel, dur, p) { this._noiseHit(ctx, out, vel * 0.55, p.decay || 0.22, 7800, 0.7, 'highpass'); }
}

export const VOICE_KINDS = {
  logdrum: { label: 'Log Drum', icon: 'vinyl-record', pitched: true, defaults: { decay: 0.3 } },
  piano: { label: 'Piano / Stab', icon: 'music-note', pitched: true, defaults: { decay: 0.55 } },
  sine: { label: 'Sine Lead', icon: 'soundwave', pitched: true, defaults: { decay: 0.4 } },
  saw: { label: 'Saw Bass', icon: 'soundwave-square', pitched: true, defaults: { decay: 0.35 } },
  square: { label: 'Square Lead', icon: 'bolt', pitched: true, defaults: { decay: 0.3 } },
  kick: { label: 'Kick', icon: 'record', pitched: false, defaults: { decay: 0.22 } },
  snare: { label: 'Snare', icon: 'record', pitched: false, defaults: { decay: 0.18 } },
  clap: { label: 'Clap', icon: 'record', pitched: false, defaults: { decay: 0.14 } },
  shaker: { label: 'Shaker', icon: 'waterdrop', pitched: false, defaults: { decay: 0.045 } },
  rim: { label: 'Rim', icon: 'target', pitched: false, defaults: { decay: 0.028 } },
  hat: { label: 'Hat', icon: 'waterdrop', pitched: false, defaults: { decay: 0.08 } },
  ohat: { label: 'Open Hat', icon: 'waterdrop', pitched: false, defaults: { decay: 0.22 } },
};

let engineNoise = null;

/* ============================ engine ============================ */

export class AudioEngine {
  constructor() {
    this.ctx = null;
    this.running = false;
    this.channels = [];       // { id, name, kind, color, gain(db), pan, mute, solo, inserts, sends, params }
    this.returns = [];        // { id, name, inserts }
    this.masterGain = 1;
    this._masterDb = -1.5;
    this._nodes = new Map();  // channel id → {input, strip, panner, analyser, chain, sends[]}
    this._retNodes = new Map();
    this.pool = [];
    this.activeVoices = 0;
    this.peakOf = new Map();
    this.masterLevel = { peak: 0, rms: 0, hold: 0, holdAt: 0 };
  }

  /* ---- lifecycle ---- */
  async enable() {
    if (this.ctx) { await this.resume(); return true; }
    const AC = window.AudioContext || window.webkitAudioContext;
    if (!AC) return false;
    this.ctx = new AC({ latencyHint: 'interactive' });
    engineNoise = (() => {
      const n = Math.floor(this.ctx.sampleRate * 2);
      const b = this.ctx.createBuffer(1, n, this.ctx.sampleRate);
      const d = b.getChannelData(0);
      for (let i = 0; i < n; i++) d[i] = Math.random() * 2 - 1;
      return b;
    })();
    this._buildMaster();
    this.running = true;
    await this.resume();
    return true;
  }
  async resume() {
    if (!this.ctx) return;
    if (this.ctx.state === 'suspended') { try { await this.ctx.resume(); } catch (e) { /* ignore */ } }
  }
  get ready() { return !!this.ctx && this.ctx.state === 'running'; }

  _buildMaster() {
    const ctx = this.ctx;
    const bus = ctx.createGain(); bus.gain.value = dbToGain(this._masterDb);
    const lim = ctx.createDynamicsCompressor();
    lim.threshold.value = -1; lim.ratio.value = 20; lim.attack.value = 0.002; lim.release.value = 0.06; lim.knee.value = 0;
    const an = ctx.createAnalyser(); an.fftSize = 1024;
    bus.connect(lim); lim.connect(an); an.connect(ctx.destination);
    this.masterBus = bus;
    this.masterLimiter = lim;
    this.masterAnalyser = an;
  }

  setMasterDb(db) {
    this._masterDb = db;
    if (this.masterBus) this.masterBus.gain.value = dbToGain(db);
  }

  /* ---- graph management ---- */

  rebuildChannel(id) {
    if (!this.ctx) return;
    const ch = this.channels.find(c => c.id === id);
    if (!ch) return;
    this._teardown(id);
    const ctx = this.ctx;

    const input = ctx.createGain();
    const strip = ctx.createGain();
    const panner = ctx.createStereoPanner();
    const analyser = ctx.createAnalyser(); analyser.fftSize = 1024;
    analyser.smoothingTimeConstant = 0.2;

    // insert chain
    const chain = [];
    let tail = input;
    (ch.inserts || []).forEach(fx => {
      if (fx.bypass) return;
      const def = FX_DEFS[fx.id];
      if (!def) return;
      const built = def.build(ctx);
      (fx.params || []).forEach(pv => built.set(pv.id, pv.value));
      tail.connect(built.input);
      chain.push({ id: fx.id, built });
      tail = built.output;
    });
    tail.connect(strip);

    strip.gain.value = ch.mute ? 0 : dbToGain(ch.gain);
    strip.connect(panner);
    panner.pan.value = ch.pan || 0;
    panner.connect(analyser);
    analyser.connect(this.masterBus);

    /* sends */
    const sendNodes = [];
    (ch.sends || []).forEach(s => {
      if (!s.amount) return;
      const rn = this._retNodes.get(s.returnId);
      if (!rn) return;
      const g = ctx.createGain(); g.gain.value = s.amount;
      strip.connect(g); g.connect(rn.input);
      sendNodes.push({ returnId: s.returnId, node: g });
    });

    this._nodes.set(id, { input, strip, panner, analyser, chain, sendNodes });
    this.peakOf.set(id, { peak: 0, rms: 0, hold: 0, holdAt: 0 });
  }

  rebuildReturn(id) {
    if (!this.ctx) return;
    const ret = this.returns.find(r => r.id === id);
    if (!ret) return;
    const old = this._retNodes.get(id);
    if (old) { try { old.strip.disconnect(); } catch (e) { /* ignore */ } }
    const ctx = this.ctx;
    const input = ctx.createGain();
    const strip = ctx.createGain();
    let tail = input;
    (ret.inserts || []).forEach(fx => {
      if (fx.bypass) return;
      const def = FX_DEFS[fx.id];
      if (!def) return;
      const built = def.build(ctx);
      (fx.params || []).forEach(pv => built.set(pv.id, pv.value));
      tail.connect(built.input);
      tail = built.output;
    });
    tail.connect(strip);
    strip.gain.value = dbToGain(ret.gain || 0);
    strip.connect(this.masterBus);
    const analyser = ctx.createAnalyser(); analyser.fftSize = 1024;
    strip.connect(analyser);
    this._retNodes.set(id, { input, strip, analyser, chain: [] });
    /* reconnect every channel's sends to the new return input */
    for (const n of this._nodes.values()) {
      n.sendNodes.forEach(s => {
        if (s.returnId !== id) return;
        try { s.node.disconnect(); } catch (e) { /* ignore */ }
        s.node.connect(this._retNodes.get(id).input);
      });
    }
  }

  _teardown(id) {
    const n = this._nodes.get(id);
    if (!n) return;
    try { n.input.disconnect(); n.strip.disconnect(); n.panner.disconnect(); n.analyser.disconnect(); } catch (e) { /* ignore */ }
    n.sendNodes.forEach(s => { try { s.node.disconnect(); } catch (e) { /* ignore */ } });
    this._nodes.delete(id);
    this.peakOf.delete(id);
  }

  applyChannelParams(id) {
    const n = this._nodes.get(id);
    const ch = this.channels.find(c => c.id === id);
    if (!n || !ch) return;
    n.strip.gain.setTargetAtTime(ch.mute ? 0 : dbToGain(ch.gain), this.ctx.currentTime, 0.008);
    n.panner.pan.setTargetAtTime(ch.pan || 0, this.ctx.currentTime, 0.008);
  }

  rebuildAll() {
    if (!this.ctx) return;
    this.returns.forEach(r => this.rebuildReturn(r.id));
    this.channels.forEach(c => this.rebuildChannel(c.id));
  }

  /* ---- triggering ---- */

  trigger(channelId, step, opts = {}) {
    if (!this.ctx) return;
    const ch = this.channels.find(c => c.id === channelId);
    if (!ch || ch.mute) return;
    /* solo logic: if any channel is solo'd, only soloed channels sound */
    const anySolo = this.channels.some(c => c.solo);
    if (anySolo && !ch.solo) return;
    const n = this._nodes.get(channelId);
    if (!n) return;
    const kind = ch.kind;
    const pitched = VOICE_KINDS[kind] && VOICE_KINDS[kind].pitched;
    const freq = pitched ? noteToFreq(ch.note != null ? ch.note : 60) : 200;
    const vel = step.vel / 127 || 0.8;
    const dur = step.dur != null ? step.dur : (VOICE_KINDS[kind] ? VOICE_KINDS[kind].defaults.decay : 0.3);
    const v = this._takeVoice(n.input, kind, freq, vel, dur, ch.params || {});
    if (v) this.activeVoices++;
  }

  _takeVoice(dest, kind, freq, vel, dur, p) {
    if (this.activeVoices > 220) return null;
    const v = this.pool.pop();
    let voice;
    if (v && v.ctx === this.ctx && v.kind === kind) {
      /* reuse: rebuild into the same wrapper is unsafe for oscillator nodes, so
         a pooled voice is only reused when its kind matches and it is idle. */
      v.dead = false;
      voice = v;
      voice.reinit(dest, freq, vel, dur, p);
      return voice;
    }
    voice = new Voice(this.ctx, dest, kind, freq, vel, dur, p);
    voice.kind = kind;
    voice.onFree = () => {
      this.activeVoices = Math.max(0, this.activeVoices - 1);
      if (!voice.busy) this.pool.push(voice);
    };
    /* patch release so the pool learns about freedom */
    const orig = voice.release.bind(voice);
    voice.release = () => {
      orig();
      if (voice.onFree) requestAnimationFrame(() => voice.onFree());
    };
    return voice;
  }

  /* ---- audition (browser preview on hover) ---- */

  auditionOne(name, kind) {
    if (!this.ctx) return;
    this.stopAudition();
    const map = {
      'Log Drum 808.wav': ['logdrum', 36], 'Log Drum deep.wav': ['logdrum', 31],
      'Shaker 16th.wav': ['shaker', 200], 'Clap big.wav': ['clap', 200],
      'Rim tight.wav': ['rim', 200], 'Kick sub.wav': ['kick', 200], 'Open hat 808.wav': ['ohat', 200],
    };
    const fallback = {
      drum: ['logdrum', 36], inst: ['piano', 60], fx: ['piano', 67], preset: ['piano', 64],
    };
    let spec = map[name];
    if (!spec) spec = fallback[kind] || fallback.drum;
    const [voiceKind, note] = spec;
    const pitched = VOICE_KINDS[voiceKind] && VOICE_KINDS[voiceKind].pitched;
    this._audition = new Voice(this.ctx, this.masterBus, voiceKind,
      pitched ? noteToFreq(note) : 200, 0.85, 0.9,
      VOICE_KINDS[voiceKind] ? VOICE_KINDS[voiceKind].defaults : {});
  }
  audition(kind, name) { this.auditionOne(name, kind); }
  stopAudition() {
    if (this._audition) { try { this._audition.release(); } catch (e) { /* ignore */ } this._audition = null; }
  }

  /* ---- metering ---- */

  readMeters() {
    const out = [];
    const buf = this._timeBuf || (this._timeBuf = new Float32Array(1024));
    const one = (node, prev) => {
      if (!node) return prev;
      node.getFloatTimeDomainData(buf);
      let peak = 0, sum = 0;
      for (let i = 0; i < buf.length; i++) { const a = Math.abs(buf[i]); if (a > peak) peak = a; sum += buf[i] * buf[i]; }
      const rms = Math.sqrt(sum / buf.length);
      const now = performance.now();
      let hold = prev ? prev.hold : 0;
      if (peak >= hold || now - (prev ? prev.holdAt : 0) > 1400) {
        if (peak > hold || now - (prev ? prev.holdAt : 0) > 1400) { hold = peak; prev && (prev.holdAt = now); }
      } else {
        hold = Math.max(peak, hold - 0.06);
      }
      return { peak, rms, hold, holdAt: prev ? prev.holdAt : now };
    };
    for (const ch of this.channels) {
      const n = this._nodes.get(ch.id);
      const prev = this.peakOf.get(ch.id);
      if (n) {
        const v = one(n.analyser, prev || { hold: 0, holdAt: 0 });
        this.peakOf.set(ch.id, v);
        out.push(v);
      } else out.push({ peak: 0, rms: 0, hold: 0, holdAt: 0 });
    }
    if (this.masterAnalyser) {
      const v = one(this.masterAnalyser, this.masterLevel);
      this.masterLevel = v;
    }
    return { channels: out, master: this.masterLevel };
  }

  get sampleRate() { return this.ctx ? this.ctx.sampleRate : 44100; }
}

export const engine = new AudioEngine();
