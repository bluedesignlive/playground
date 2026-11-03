/* smart-daw audio.js — Web Audio engine, scheduler, metering */
/* Tokens strictly from smart.toml; no raw values. */

(function(){
  const $ = s => document.querySelector(s);
  const $$ = s => Array.from(document.querySelectorAll(s));

  /* ============ audio context ============ */
  let audioCtx = null;
  let masterGainNode = null;
  let analyserNode = null;

  function ensureAudio(){
    if (!audioCtx){
      audioCtx = new (window.AudioContext || window.webkitAudioContext)();
      masterGainNode = audioCtx.createGain();
      masterGainNode.gain.value = 1;
      masterGainNode.connect(audioCtx.destination);
      
      analyserNode = audioCtx.createAnalyser();
      analyserNode.fftSize = 256;
      masterGainNode.connect(analyserNode);
    }
    return audioCtx;
  }

  function ctx(){ return ensureAudio(); }

  /* ============ channel routing ============ */
  const channelNodes = new Map();

  function getChannelNode(chId){
    if (channelNodes.has(chId)) return channelNodes.get(chId);
    const c = ctx();
    const gain = c.createGain(); gain.gain.value = 0.8;
    const pan = c.createStereoPanner(); pan.pan.value = 0;
    gain.connect(pan); pan.connect(masterGainNode);
    const node = {gain, pan};
    channelNodes.set(chId, node);
    return node;
  }

  /* ============ synth voices ============ */
  function playDrum(type, vel, chGain){
    const c = ctx(); const t = c.currentTime;
    const g = chGain || 1;
    if (type === 'kick'){
      const osc = c.createOscillator(); const env = c.createGain();
      osc.connect(env); env.connect(masterGainNode);
      osc.type='sine'; osc.frequency.setValueAtTime(150,t); osc.frequency.exponentialRampToValueAtTime(30,t+0.3);
      env.gain.setValueAtTime(0,t); env.gain.exponentialRampToValueAtTime(vel/100*g,t+0.01); env.gain.exponentialRampToValueAtTime(0.001,t+0.4);
      osc.start(t); osc.stop(t+0.5);
    } else if (type === 'snare'){
      const osc = c.createOscillator(); const env = c.createGain();
      osc.connect(env); env.connect(masterGainNode);
      osc.type='square'; osc.frequency.setValueAtTime(180,t);
      env.gain.setValueAtTime(0,t); env.gain.exponentialRampToValueAtTime(vel/100*0.6*g,t+0.01); env.gain.exponentialRampToValueAtTime(0.001,t+0.15);
      osc.start(t); osc.stop(t+0.2);
      const bs=c.sampleRate*0.15; const buf=c.createBuffer(1,bs,c.sampleRate); const d=buf.getChannelData(0);
      for(let i=0;i<bs;i++)d[i]=Math.random()*2-1;
      const n=c.createBufferSource(); n.buffer=buf; const ne=c.createGain();
      n.connect(ne); ne.connect(masterGainNode);
      ne.gain.setValueAtTime(0,t); ne.gain.exponentialRampToValueAtTime(vel/100*0.35*g,t+0.01); ne.gain.exponentialRampToValueAtTime(0.001,t+0.15);
      n.start(t); n.stop(t+0.2);
    } else if (type === 'hihat'){
      const osc = c.createOscillator(); const env = c.createGain();
      osc.connect(env); env.connect(masterGainNode);
      osc.type='square'; osc.frequency.setValueAtTime(8000,t);
      env.gain.setValueAtTime(0,t); env.gain.exponentialRampToValueAtTime(vel/100*0.3*g,t+0.01); env.gain.exponentialRampToValueAtTime(0.001,t+0.06);
      osc.start(t); osc.stop(t+0.08);
    }
  }

  function playNote(midi, vel, dur, chGain){
    const c = ctx(); const t = c.currentTime;
    const g = chGain || 1;
    const osc = c.createOscillator(); const env = c.createGain();
    osc.type='sine'; osc.frequency.setValueAtTime(440*Math.pow(2,(midi-69)/12),t);
    osc.connect(env); env.connect(masterGainNode);
    env.gain.setValueAtTime(0,t); env.gain.linearRampToValueAtTime(vel/100*0.4*g,t+0.05); env.gain.exponentialRampToValueAtTime(0.001,t+dur);
    osc.start(t); osc.stop(t+dur+0.05);
  }

  function playMetronome(accent){
    const c = ctx(); const t = c.currentTime;
    const osc = c.createOscillator(); const env = c.createGain();
    osc.connect(env); env.connect(masterGainNode);
    osc.type = accent ? 'sine' : 'square';
    osc.frequency.setValueAtTime(accent ? 1000 : 500, t);
    env.gain.setValueAtTime(0, t);
    env.gain.exponentialRampToValueAtTime(0.15, t + 0.01);
    env.gain.exponentialRampToValueAtTime(0.001, t + 0.05);
    osc.start(t); osc.stop(t + 0.08);
  }

  /* ============ metering ============ */
  function getMeterLevel(){
    if (!analyserNode) return 0;
    const data = new Uint8Array(analyserNode.frequencyBinCount);
    analyserNode.getByteFrequencyData(data);
    let sum = 0;
    for (let i = 0; i < data.length; i++) sum += data[i];
    return sum / (data.length * 255);
  }

  /* ============ export ============ */
  window.smartDawAudio = {
    ensureAudio, ctx, getChannelNode, playDrum, playNote, playMetronome, getMeterLevel
  };
})();
