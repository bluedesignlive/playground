/* smart-daw app.js — state, audio engine, undo/redo, project IO, rendering, keyboard, AI */
/* Tokens strictly from smart.toml; skeuomorphic feel only in knobs/faders. */

(function(){
  const $ = s => document.querySelector(s);
  const $$ = s => Array.from(document.querySelectorAll(s));

  /* ============ build DAW grid (injected into #viewMount) ============ */
  function buildGrid(){
    const mount = $('#viewMount');
    if (!mount) return;
    mount.innerHTML = `
      <div class="daw-grid">
        <aside class="daw-pane docked" id="pane-browser" aria-label="Browser">
          <div class="daw-pane-header">
            <span class="daw-pane-title">Browser</span>
            <div class="daw-pane-actions">
              <button class="pane-btn" title="Float/dock"><span class="ic" data-ic="widget-outline"></span></button>
              <button class="pane-btn" title="Close"><span class="ic" data-ic="close-circle-outline"></span></button>
            </div>
          </div>
          <div class="daw-pane-content" id="pane-browser-content">
            <div class="browser-list" id="browserList"></div>
          </div>
        </aside>

        <aside class="daw-pane docked" id="pane-channel" aria-label="Channel Rack">
          <div class="daw-pane-header">
            <span class="daw-pane-title">Channel Rack</span>
            <div class="daw-pane-actions">
              <button class="pane-btn" title="Float/dock"><span class="ic" data-ic="widget-outline"></span></button>
              <button class="pane-btn" title="Close"><span class="ic" data-ic="close-circle-outline"></span></button>
            </div>
          </div>
          <div class="daw-pane-content" id="pane-channel-content">
            <div class="rack-body" id="rackBody"></div>
          </div>
        </aside>

        <aside class="daw-pane docked" id="pane-piano" aria-label="Piano Roll">
          <div class="daw-pane-header">
            <span class="daw-pane-title">Piano Roll</span>
            <div class="daw-pane-actions">
              <button class="iconbtn pane-btn" title="Scale highlight" id="btn-scale-highlight" aria-pressed="true"><span class="ic" data-ic="stars-minimalistic-outline"></span></button>
              <button class="iconbtn pane-btn" title="Snap to scale" id="btn-snap-scale" aria-pressed="false"><span class="ic" data-ic="magnifer-outline"></span></button>
              <button class="pane-btn" title="Float/dock"><span class="ic" data-ic="widget-outline"></span></button>
              <button class="pane-btn" title="Close"><span class="ic" data-ic="close-circle-outline"></span></button>
            </div>
          </div>
          <div class="daw-pane-content" id="pane-piano-content">
            <div class="piano-roll">
              <div class="piano-keyboard" id="piano-keyboard"></div>
              <div class="piano-main">
                <div class="piano-ruler" id="piano-ruler"></div>
                <div class="piano-view">
                  <div class="piano-grid" id="piano-grid"></div>
                  <div class="piano-notes" id="piano-notes"></div>
                  <div class="playhead" id="playhead"></div>
                  <div class="loop-bar" id="loop-bar"></div>
                </div>
                <div class="velocity-lane" id="velocity-lane">
                  <div class="velocity-bars" id="velocity-bars"></div>
                </div>
              </div>
              <div class="piano-ctrls">
                <label class="search" style="width:170px;margin-right:auto">
                  <span class="ic" data-ic="magnifer-outline"></span>
                  <input id="piano-search" type="text" placeholder="Search notes" autocomplete="off" spellcheck="false">
                </label>
                <div class="seg" role="group" aria-label="Grid">
                  <button class="iconbtn pane-btn" id="btn-grid-4" title="1/4"><span class="ic" data-ic="widget-outline"></span></button>
                  <button class="iconbtn pane-btn" id="btn-grid-8" title="1/8"><span class="ic" data-ic="widget-outline"></span></button>
                  <button class="iconbtn pane-btn" id="btn-grid-16" title="1/16"><span class="ic" data-ic="widget-outline"></span></button>
                </div>
              </div>
            </div>
          </div>
        </aside>

        <aside class="daw-pane docked" id="pane-transport" aria-label="Transport">
          <div class="daw-pane-content">
            <div class="transport-group" id="transportControls">
              <button class="tr-btn" id="tr-stop" title="Stop"><span class="ic" data-ic="minimize-outline"></span></button>
              <button class="tr-btn" id="tr-play" title="Play (Space)"><span class="ic" data-ic="arrow-right-up-outline"></span></button>
              <button class="tr-btn danger" id="tr-record" title="Record (R)"><span class="ic" data-ic="close-circle-outline"></span></button>
              <button class="tr-btn" id="tr-loop" title="Loop (L)"><span class="ic" data-ic="refresh-outline"></span></button>
              <button class="tr-btn" id="tr-metro" title="Metronome"><span class="ic" data-ic="bell-outline"></span></button>
              <div class="tr-sep"></div>
              <div class="tr-field">
                <span class="ic" data-ic="clock-circle-outline" style="--ic:14px"></span>
                <label style="font-size:var(--t-micro);color:var(--ink-3);margin-right:4px">BPM</label>
                <input id="tr-field-bpm" type="number" value="124" min="20" max="300" aria-label="BPM">
              </div>
              <div class="tr-sep"></div>
              <div class="tr-field" id="song-pos" title="Song position">1.1.000 | 0:00</div>
              <div class="tr-sep"></div>
              <button class="tr-btn" id="btn-loop-in" title="Loop in (I)"><span class="ic" data-ic="alt-arrow-left-outline"></span></button>
              <button class="tr-btn" id="btn-loop-out" title="Loop out (O)"><span class="ic" data-ic="alt-arrow-right-outline"></span></button>
              <div class="tr-loop" id="loop-range">0.00 — 4.00</div>
              <div class="tr-sep"></div>
              <button class="tr-btn" id="btn-export" title="Export project"><span class="ic" data-ic="arrow-right-up-outline"></span></button>
              <button class="tr-btn" id="btn-import" title="Import project"><span class="ic" data-ic="import-outline"></span></button>
              <div class="tr-ai" id="ai-pill" title="AI suggestion">
                <span class="ic" data-ic="stars-minimalistic-outline"></span>
                <span class="ai-label">AI: ready</span>
              </div>
            </div>
          </div>
        </aside>

        <aside class="daw-pane docked" id="pane-mixer" aria-label="Mixer">
          <div class="daw-pane-header">
            <span class="daw-pane-title">Mixer</span>
            <div class="daw-pane-actions">
              <button class="pane-btn" title="Float/dock"><span class="ic" data-ic="widget-outline"></span></button>
              <button class="pane-btn" title="Close"><span class="ic" data-ic="close-circle-outline"></span></button>
            </div>
          </div>
          <div class="daw-pane-content" id="pane-mixer-content">
            <div class="mixer-body" id="mixerBody"></div>
          </div>
        </aside>

        <aside class="daw-pane docked" id="pane-playlist" aria-label="Playlist" style="display:none">
          <div class="daw-pane-header">
            <span class="daw-pane-title">Playlist</span>
            <div class="daw-pane-actions">
              <button class="pane-btn" title="Float/dock"><span class="ic" data-ic="widget-outline"></span></button>
              <button class="pane-btn" title="Close"><span class="ic" data-ic="close-circle-outline"></span></button>
            </div>
          </div>
          <div class="daw-pane-content" id="pane-playlist-content">
            <div class="playlist-body">
              <div class="playlist-ruler" id="playlist-ruler"></div>
              <div class="playlist-area" id="playlist-area"></div>
              <div class="automation-lane" id="automation-lane">
                <span class="ic" data-ic="widget-outline" style="--ic:12px"></span>
                <span>Automation: Volume</span>
              </div>
            </div>
          </div>
        </aside>
      </div>
    `;
  }

  /* ============ theme ============ */
  const btnTheme = $('#btn-theme');
  let dark = false;
  if (btnTheme) {
    btnTheme.addEventListener('click', () => {
      dark = !dark;
      document.documentElement.setAttribute('data-theme', dark ? 'dark' : 'light');
      btnTheme.setAttribute('aria-pressed', dark);
      btnTheme.textContent = dark ? '◑' : '◐';
    });
  }

  /* ============ audio engine ============ */
  let audioCtx = null;
  let masterGainNode = null;
  let analyserNode = null;
  const channelNodes = new Map();

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

  function getMeterLevel(){
    if (!analyserNode) return 0;
    const data = new Uint8Array(analyserNode.frequencyBinCount);
    analyserNode.getByteFrequencyData(data);
    let sum = 0;
    for (let i = 0; i < data.length; i++) sum += data[i];
    return sum / (data.length * 255);
  }

  /* ============ state ============ */
  const state = {
    bpm: 124, playing: false, recording: false, position: 0, ppqn: 96,
    loopStart: 0, loopEnd: 4, timeSig: [4,4], gridDiv: 16,
    selectedNoteId: null, scaleActive: true, scaleMode: 'C:major', snapToScale: false,
    metro: true, masterVol: 0, view: 'piano', playlistVisible: false,
    sidebarCollapsed: false,
  };
  const NOTE_NAMES = ['C','C#','D','D#','E','F','F#','G','G#','A','A#','B'];
  const LOW = 36, HIGH = 83;
  const ROW_H = 22;
  const BAR_W = 160;

  const channels = [
    {id:1, name:'Kick', color:'#e74c3c', type:'drum', sound:'kick', mute:false, solo:false, vol:-3, pan:0, pattern:[1,0,0,0, 1,0,0,0, 1,0,0,0, 1,0,0,0], stepVel:[100,0,0,0, 100,0,0,0, 100,0,0,0, 100,0,0,0]},
    {id:2, name:'Snare', color:'#3498db', type:'drum', sound:'snare', mute:false, solo:false, vol:-6, pan:0, pattern:[0,0,0,0, 0,0,1,0, 0,0,0,0, 0,0,1,0], stepVel:[0,0,0,0, 0,0,100,0, 0,0,0,0, 0,0,100,0]},
    {id:3, name:'HiHat', color:'#f1c40f', type:'drum', sound:'hihat', mute:false, solo:false, vol:-9, pan:-0.15, pattern:[1,0,1,0, 1,0,1,0, 1,0,1,0, 1,0,1,0], stepVel:[80,0,80,0, 80,0,80,0, 80,0,80,0, 80,0,80,0]},
    {id:4, name:'LogDrum', color:'#e67e22', type:'drum', sound:'kick', mute:false, solo:false, vol:-4, pan:0.1, pattern:[0,0,0,0, 0,0,0,0, 1,0,0,0, 0,0,0,0], stepVel:[0,0,0,0, 0,0,0,0, 100,0,0,0, 0,0,0,0]},
    {id:5, name:'Bass', color:'#2ecc71', type:'melodic', sound:'bass', mute:false, solo:false, vol:-2, pan:0, notes:[{id:101,midi:36,x:0,w:BAR_W,vel:100},{id:102,midi:43,x:BAR_W,w:BAR_W,vel:80},{id:103,midi:38,x:BAR_W*2,w:BAR_W,vel:90},{id:104,midi:41,x:BAR_W*3,w:BAR_W,vel:100}]},
    {id:6, name:'Chords', color:'#9b59b6', type:'melodic', sound:'chord', mute:false, solo:false, vol:-5, pan:0, notes:[{id:201,midi:48,x:0,w:BAR_W*2,vel:90},{id:202,midi:52,x:0,w:BAR_W*2,vel:90},{id:203,midi:55,x:0,w:BAR_W*2,vel:90},{id:204,midi:60,x:BAR_W*2,w:BAR_W*2,vel:90},{id:205,midi:64,x:BAR_W*2,w:BAR_W*2,vel:90},{id:206,midi:67,x:BAR_W*2,w:BAR_W*2,vel:90}]},
    {id:7, name:'Lead', color:'#1abc9c', type:'melodic', sound:'lead', mute:false, solo:false, vol:-3, pan:0.2, notes:[{id:301,midi:60,x:BAR_W*4,w:BAR_W,vel:90},{id:302,midi:64,x:BAR_W*5,w:BAR_W/2,vel:90},{id:303,midi:67,x:BAR_W*5.5,w:BAR_W/2,vel:90},{id:304,midi:69,x:BAR_W*6,w:BAR_W,vel:100}]},
    {id:8, name:'Pad', color:'#34495e', type:'melodic', sound:'pad', mute:false, solo:false, vol:-6, pan:-0.2, notes:[{id:401,midi:48,x:0,w:BAR_W*8,vel:60},{id:402,midi:52,x:0,w:BAR_W*8,vel:60},{id:403,midi:55,x:0,w:BAR_W*8,vel:60}]},
  ];

  const sampleLibrary = [
    {id:'kick_808', name:'Kick 808', file:'mocks/audio/kick_808.wav', img:'mocks/images/kick_808.svg', type:'drum'},
    {id:'kick_logdrum', name:'Kick LogDrum', file:'mocks/audio/kick_logdrum.wav', img:'mocks/images/kick_logdrum.svg', type:'drum'},
    {id:'snare_clap', name:'Snare Clap', file:'mocks/audio/snare_clap.wav', img:'mocks/images/snare_clap.svg', type:'drum'},
    {id:'snare_rimshot', name:'Snare Rimshot', file:'mocks/audio/snare_rimshot.wav', img:'mocks/images/snare_rimshot.svg', type:'drum'},
    {id:'hihat_closed', name:'HiHat Closed', file:'mocks/audio/hihat_closed.wav', img:'mocks/images/hihat_closed.svg', type:'drum'},
    {id:'hihat_open', name:'HiHat Open', file:'mocks/audio/hihat_open.wav', img:'mocks/images/hihat_open.svg', type:'drum'},
    {id:'hihat_pedal', name:'HiHat Pedal', file:'mocks/audio/hihat_pedal.wav', img:'mocks/images/hihat_pedal.svg', type:'drum'},
    {id:'logdrum_conga', name:'LogDrum Conga', file:'mocks/audio/logdrum_conga.wav', img:'mocks/images/logdrum_conga.svg', type:'drum'},
    {id:'bass_deep', name:'Bass Deep', file:'mocks/audio/bass_deep.wav', img:'mocks/images/bass_deep.svg', type:'melodic'},
    {id:'chord_stab', name:'Chord Stab', file:'mocks/audio/chord_stab.wav', img:'mocks/images/chord_stab.svg', type:'melodic'},
    {id:'piano_lead', name:'Piano Lead', file:'mocks/audio/piano_lead.wav', img:'mocks/images/piano_lead.svg', type:'melodic'},
    {id:'pad_warm', name:'Pad Warm', file:'mocks/audio/pad_warm.wav', img:'mocks/images/pad_warm.svg', type:'melodic'},
    {id:'cowbell', name:'Cowbell', file:'mocks/audio/cowbell.wav', img:'mocks/images/cowbell.svg', type:'drum'},
    {id:'shaker', name:'Shaker', file:'mocks/audio/shaker.wav', img:'mocks/images/shaker.svg', type:'drum'},
    {id:'adlib_hey', name:'Adlib Hey', file:'mocks/audio/adlib_hey.wav', img:'mocks/images/adlib_hey.svg', type:'melodic'},
    {id:'vocal_chop', name:'Vocal Chop', file:'mocks/audio/vocal_chop.wav', img:'mocks/images/vocal_chop.svg', type:'melodic'},
  ];

  /* ============ undo/redo ============ */
  const undoStack = [], redoStack = [];
  const MAX_UNDO = 50;
  function snapshot(){ return JSON.parse(JSON.stringify({channels, state})); }
  function pushUndo(){
    undoStack.push(snapshot());
    if (undoStack.length > MAX_UNDO) undoStack.shift();
    redoStack.length = 0;
  }
  function undo(){
    if (undoStack.length === 0) return;
    redoStack.push(snapshot());
    const prev = undoStack.pop();
    channels.length = 0; channels.push(...prev.channels);
    Object.assign(state, prev.state);
    renderAll();
    addHistory('Undo');
  }
  function redo(){
    if (redoStack.length === 0) return;
    undoStack.push(snapshot());
    const nxt = redoStack.pop();
    channels.length = 0; channels.push(...nxt.channels);
    Object.assign(state, nxt.state);
    renderAll();
    addHistory('Redo');
  }

  /* ============ project save/load ============ */
  function exportProject(){
    const data = {version:1, channels, state, exportedAt: new Date().toISOString()};
    const blob = new Blob([JSON.stringify(data,null,2)], {type:'application/json'});
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a'); a.href=url; a.download='smart-daw-project.json'; a.click();
    URL.revokeObjectURL(url);
    addHistory('Project exported');
  }
  function importProject(file){
    const r = new FileReader();
    r.onload = e=>{
      try {
        const data = JSON.parse(e.target.result);
        channels.length = 0; channels.push(...data.channels);
        Object.assign(state, data.state);
        renderAll();
        pushUndo();
        addHistory('Project imported: ' + file.name);
      } catch(err){ alert('Invalid project file'); }
    };
    r.readAsText(file);
  }

  /* ============ AI ============ */
  let aiHistory = [];
  const cmd = $('#cmd-palette');
  const cmdInput = $('#cmd-palette .cmd-search');
  const cmdResults = $('#cmd-results');
  const btnAI = $('#btn-ai');
  const aiPill = $('#ai-pill');
  const aiHistoryEl = $('#ai-history');
  const aiHistoryList = $('#ai-history-list');
  const aiPanel = $('#aiPanel');

  function openCmd(){ if(cmd) cmd.classList.add('open'); if(cmdInput) cmdInput.focus(); renderCmd(''); }
  function closeCmd(){ if(cmd) cmd.classList.remove('open'); if(cmdInput) cmdInput.value=''; if(cmdResults) cmdResults.innerHTML=''; }

  const AI_RESPONSES = [
    'Created log drum pattern in C minor — 4 bars, step 1/16',
    'Added deep house chord stab every 2 bars',
    'Sidechained bass to kick (25% duck)',
    'Humanized hi-hats by 12%',
    'Built amapiano starter template',
    'Quantized selected notes to 1/16 grid',
    'Changed scale to A minor and snap-to-scale on',
    'Exported project',
    'Imported project',
  ];

  function renderCmd(q){
    if(!cmdResults) return;
    const f = AI_RESPONSES.filter(r => !q || r.toLowerCase().includes(q.toLowerCase()));
    cmdResults.innerHTML = f.map(r =>
      `<div class="cmd-item" data-ai="${r}"><span>${r}</span><kbd>⏎</kbd></div>`
    ).join('');
    $$('.cmd-item').forEach(it => {
      it.addEventListener('click', () => applyAI(it.dataset.ai));
    });
  }

  function applyAI(text){
    closeCmd();
    addHistory(text);
    const label = text.replace(/^(Created|Added|Sidechained|Humanized|Built|Quantized|Changed|Exported|Imported)\s+/,'').split('—')[0].trim();
    if (aiPill) {
      const lbl = aiPill.querySelector('.ai-label');
      if (lbl) lbl.textContent = 'AI: "' + label + '"';
      aiPill.classList.add('ring');
      setTimeout(() => aiPill.classList.remove('ring'), 900);
    }
    if (text.includes('scale')) {
      state.scaleActive = true;
      const bs = $('#btn-scale-highlight');
      if (bs) bs.setAttribute('aria-pressed', 'true');
      applyScale();
    }
    if (text.includes('log drum') || text.includes('drum')) {
      renderRack();
      renderPianoNotes();
    }
    if (text === 'Exported project') { exportProject(); }
    if (text === 'Imported project') { document.getElementById('proj-import').click(); }
  }

  function addHistory(text){
    aiHistory.unshift({text, time: new Date().toLocaleTimeString([], {hour:'2-digit', minute:'2-digit'})});
    if (aiHistory.length > 8) aiHistory.pop();
    renderHistory();
  }
  function renderHistory(){
    if (!aiHistoryList) return;
    aiHistoryList.innerHTML = aiHistory.map(h =>
      `<div class="ai-history-item"><span>${h.time} — ${h.text}</span><span class="ai-history-undo" title="Revert">↩</span></div>`
    ).join('');
  }

  /* ============ render ============ */
  function midiY(midi){ return (HIGH - midi) * ROW_H; }
  function renderAll(){ renderRack(); renderBrowser(); renderPianoNotes(); renderVelocityLane(); renderMixer(); updateTransportUI(); updateLoopBar(); renderPlaylist(); }

  /* ---- browser ---- */
  function renderBrowser(){
    const list = $('#browserList');
    if (!list) return;
    list.innerHTML = '';
    sampleLibrary.forEach(s => {
      const item = document.createElement('button');
      item.className = 'browser-item';
      item.dataset.id = s.id;
      item.innerHTML =
        '<span class="ic" style="--ic:16px;color:var(--ink-3)" data-ic="document-text-outline"></span>' +
        '<span>' + s.name + '</span>' +
        '<span class="tail">' + s.type + '</span>' +
        '<div class="wave">' +
        '<div class="bar" style="height:40%"></div>' +
        '<div class="bar" style="height:70%"></div>' +
        '<div class="bar" style="height:50%"></div>' +
        '<div class="bar" style="height:80%"></div>' +
        '</div>' +
        '<button class="browser-audition" title="Audition ' + s.name + '"><span class="ic" style="--ic:13px" data-ic="arrow-right-up-outline"></span></button>';
      list.appendChild(item);
    });

    list.addEventListener('click', e => {
      const item = e.target.closest('.browser-item');
      if (!item) return;
      $$('.browser-item').forEach(i => i.classList.remove('sel'));
      item.classList.add('add');
      const id = item.dataset.id;
      const s = sampleLibrary.find(x => x.id === id);
      if (s) {
        const btn = item.querySelector('.browser-audition');
        if (btn) {
          btn.classList.add('playing');
          playSample(s.file);
          setTimeout(() => btn.classList.remove('playing'), 1500);
        }
      }
    });
    list.addEventListener('mouseenter', e => {
      const item = e.target.closest('.browser-item');
      if (item) {
        const btn = item.querySelector('.browser-audition');
        if (btn) btn.style.opacity = '1';
      }
    });
    list.addEventListener('mouseleave', e => {
      const item = e.target.closest('.browser-item');
      if (item && !item.classList.contains('sel')) {
        const btn = item.querySelector('.browser-audition');
        if (btn) btn.style.opacity = '0';
      }
    });
  }

  let samplePlayer = null;
  function playSample(file){
    if (samplePlayer) { samplePlayer.stop(); samplePlayer.disconnect(); samplePlayer = null; }
    const c = ctx();
    fetch(file).then(r => r.arrayBuffer()).then(buf => {
      c.decodeAudioData(buf).then(ab => {
        samplePlayer = c.createBufferSource();
        samplePlayer.buffer = ab;
        samplePlayer.connect(masterGainNode);
        samplePlayer.start(0);
      });
    });
  }

  /* ---- channel rack ---- */
  const rackGrid = $('#rackBody');
  function renderRack(){
    if (!rackGrid) return;
    rackGrid.innerHTML = '';
    channels.forEach(ch => {
      const row = document.createElement('div');
      row.className = 'rack-channel';
      row.style.setProperty('--ch-color', ch.color);
      row.innerHTML =
        '<div class="rack-color" style="background:' + ch.color + ';" title="' + ch.name + '"></div>' +
        '<span class="rack-name">' + ch.name + '</span>' +
        '<div class="rack-controls">' +
        '<button class="rack-ctrl mute ' + (ch.mute ? 'active' : '') + '" data-ch="' + ch.id + '" title="Mute">M</button>' +
        '<button class="rack-ctrl solo ' + (ch.solo ? 'active' : '') + '" data-ch="' + ch.id + '" title="Solo">S</button>' +
        '<button class="rack-ctrl arm ' + (ch.arm ? 'active' : '') + '" data-ch="' + ch.id + '" title="Arm">R</button>' +
        '</div>';
      const steps = document.createElement('div');
      steps.className = 'rack-steps';
      (ch.pattern || []).forEach((on, i) => {
        const b = document.createElement('button');
        b.className = 'rack-step' + (on ? ' on' : '');
        b.dataset.ch = ch.id;
        b.dataset.step = i;
        if (on) {
          b.dataset.vel = (ch.stepVel && ch.stepVel[i]) || 100;
          b.style.opacity = ((ch.stepVel && ch.stepVel[i]) || 100) / 100;
        } else {
          b.style.opacity = 0.3;
        }
        steps.appendChild(b);
      });
      rackGrid.appendChild(row);
      rackGrid.appendChild(steps);
    });
    const addBtn = document.createElement('button');
    addBtn.className = 'channel-btn';
    addBtn.innerHTML = '<span class="ic" style="--ic:13px" data-ic="add-outline"></span>Add channel';
    rackGrid.appendChild(addBtn);
  }

  rackGrid.addEventListener('click', e => {
    const step = e.target.closest('.rack-step');
    if (step) {
      const ch = channels.find(c => c.id == parseInt(step.dataset.ch));
      const i = parseInt(step.dataset.step);
      const on = !step.classList.contains('on');
      step.classList.toggle('on', on);
      if (!ch.pattern) ch.pattern = [];
      ch.pattern[i] = on ? 1 : 0;
      if (on && (!ch.stepVel || ch.stepVel[i] === undefined)) {
        if (!ch.stepVel) ch.stepVel = [];
        ch.stepVel[i] = 100;
      }
      step.dataset.vel = (ch.stepVel && ch.stepVel[i]) || '';
      step.style.opacity = on ? ((ch.stepVel && ch.stepVel[i]) || 100) / 100 : 0.3;
      pushUndo();
      return;
    }
    const ctrl = e.target.closest('.rack-ctrl');
    if (ctrl) {
      const ch = channels.find(c => c.id == parseInt(ctrl.dataset.ch));
      ch[ctrl.dataset.act || ctrl.classList.contains('mute') ? 'mute' : ctrl.classList.contains('solo') ? 'solo' : 'arm'] = !ch[ctrl.classList.contains('mute') ? 'mute' : ctrl.classList.contains('solo') ? 'solo' : 'arm'];
      ctrl.classList.toggle('active', ctrl.classList.contains('mute') ? ch.mute : ctrl.classList.contains('solo') ? ch.solo : ch.arm);
      pushUndo();
    }
  });

  rackGrid.addEventListener('contextmenu', e => {
    e.preventDefault();
    const step = e.target.closest('.rack-step');
    if (!step) return;
    const ch = channels.find(c => c.id == parseInt(step.dataset.ch));
    const i = parseInt(step.dataset.step);
    const v = prompt('Step velocity (0-127)', ch.stepVel && ch.stepVel[i] ? ch.stepVel[i] : '100');
    if (v !== null) {
      const nv = Math.max(0, Math.min(127, parseInt(v || 100)));
      if (!ch.stepVel) ch.stepVel = [];
      ch.stepVel[i] = nv;
      step.dataset.vel = nv;
      if (nv > 0) {
        step.classList.add('on');
        ch.pattern[i] = 1;
        step.style.opacity = nv / 100;
      } else {
        step.classList.remove('on');
        ch.pattern[i] = 0;
        step.style.opacity = 0.3;
      }
      pushUndo();
    }
  });

  /* ---- piano roll ---- */
  const pianoKeyboard = $('#piano-keyboard');
  const pianoRuler = $('#piano-ruler');
  const pianoNotes = $('#piano-notes');
  const pianoArea = $('#piano-grid');

  pianoKeyboard.innerHTML = '';
  for (let midi = HIGH; midi >= LOW; midi--) {
    const octave = Math.floor(midi / 12) - 1;
    const name = NOTE_NAMES[midi % 12];
    const isSharp = name.includes('#');
    const key = document.createElement('div');
    key.className = 'piano-key ' + (isSharp ? 'black' : 'white');
    key.dataset.midi = midi;
    key.style.height = ROW_H + 'px';
    if (!isSharp) key.textContent = name + octave;
    pianoKeyboard.appendChild(key);
  }

  pianoRuler.innerHTML = '';
  for (let b = 0; b < 16; b++) {
    const bar = document.createElement('div');
    bar.className = 'piano-bar';
    bar.style.minWidth = BAR_W + 'px';
    bar.textContent = (b + 1);
    pianoRuler.appendChild(bar);
  }

  pianoArea.style.setProperty('--roll-rows', (HIGH - LOW + 1));
  pianoArea.style.setProperty('--roll-grid', ROW_H + 'px');

  const SCALES = { chromatic: [], 'C:major': [0,2,4,5,7,9,11], 'A:minor': [0,2,3,5,7,8,10], 'F:lydian': [0,2,4,6,7,9,11], 'C:pentatonic': [0,2,4,7,9] };
  const scaleSelect = $('#scale-select');
  if (scaleSelect) {
    scaleSelect.innerHTML = Object.keys(SCALES).map(k => '<option value="' + k + '">' + k + '</option>').join('');
    scaleSelect.value = state.scaleMode;
  }

  function applyScale(){
    const iv = SCALES[state.scaleMode] || [];
    $$('.piano-key[data-midi]').forEach(el => {
      const midi = parseInt(el.dataset.midi);
      const pc = ((midi % 12) + 12) % 12;
      const isRoot = pc === 0;
      el.classList.remove('scale-on', 'scale-root');
      if (!state.scaleActive) return;
      if (isRoot) { el.classList.add('scale-root'); return; }
      if (iv.length === 0) return;
      if (iv.includes(pc)) el.classList.add('scale-on');
    });
  }
  applyScale();

  if (scaleSelect) {
    scaleSelect.addEventListener('change', e => { state.scaleMode = e.target.value; applyScale(); });
  }
  const btnScale = $('#btn-scale-highlight');
  if (btnScale) {
    btnScale.addEventListener('click', () => {
      state.scaleActive = !state.scaleActive;
      btnScale.setAttribute('aria-pressed', state.scaleActive);
      applyScale();
    });
  }
  const btnSnap = $('#btn-snap-scale');
  if (btnSnap) {
    btnSnap.addEventListener('click', () => {
      state.snapToScale = !state.snapToScale;
      btnSnap.setAttribute('aria-pressed', state.snapToScale);
    });
  }

  // piano key click -> play note
  pianoKeyboard.addEventListener('mousedown', e => {
    const key = e.target.closest('.piano-key');
    if (!key) return;
    const midi = parseInt(key.dataset.midi);
    playNote(midi, 90, 0.5, 1);
    key.classList.add('selected');
    setTimeout(() => key.classList.remove('selected'), 150);
  });

  function allNotes(){
    let out = [];
    channels.forEach(ch => {
      if (ch.type === 'melodic' && ch.notes) ch.notes.forEach(n => out.push({...n, chId: ch.id}));
    });
    return out;
  }

  function renderPianoNotes(){
    if (!pianoNotes) return;
    pianoNotes.innerHTML = '';
    allNotes().forEach(n => {
      const el = document.createElement('div');
      el.className = 'piano-note' + (state.selectedNoteId === n.id ? ' selected' : '');
      const ch = channels.find(c => c.id === n.chId);
      el.style.setProperty('--note-color', ch ? ch.color : '#e74c3c');
      el.style.left = n.x + 'px';
      el.style.top = midiY(n.midi) + 'px';
      el.style.width = Math.max(n.w, 20) + 'px';
      el.style.height = (ROW_H - 2) + 'px';
      el.textContent = NOTE_NAMES[n.midi % 12];
      el.dataset.id = n.id;
      el.dataset.chid = n.chId;
      pianoNotes.appendChild(el);
    });
    renderVelocityLane();
  }

  function renderVelocityLane(){
    const area = $('#velocity-bars');
    if (!area) return;
    area.innerHTML = '';
    const notes = allNotes();
    notes.forEach(n => {
      const bar = document.createElement('div');
      bar.style.cssText = 'position:absolute;left:' + n.x + 'px;width:' + Math.max(n.w, 4) + 'px;height:' + (n.vel / 127 * 100) + '%;background:var(--accent);border-radius:2px 2px 0 0;opacity:0.6;transition:height 140ms var(--ease-out)';
      bar.title = 'Velocity: ' + n.vel;
      area.appendChild(bar);
    });
  }

  function snapX(x){
    const cell = BAR_W / (state.gridDiv / 4);
    return Math.round(x / cell) * cell;
  }
  function snapMidi(midi){
    if (!state.snapToScale || state.scaleMode === 'chromatic') return midi;
    const pc = ((midi % 12) + 12) % 12;
    const iv = SCALES[state.scaleMode] || [];
    if (!iv.length) return midi;
    let best = midi, bd = 99;
    for (let off = -12; off <= 12; off++) {
      const m = midi + off;
      const p = ((m % 12) + 12) % 12;
      if (iv.includes(p)) {
        const d = Math.abs(off);
        if (d < bd) { bd = d; best = m; }
      }
    }
    return best;
  }

  let draggingNote = null, drawing = null, resizing = null, editingVel = null;
  pianoArea.addEventListener('mousedown', e => {
    const rect = pianoArea.getBoundingClientRect();
    const x = e.clientX - rect.left + pianoArea.scrollLeft;
    const y = e.clientY - rect.top + pianoArea.scrollTop;
    const note = e.target.closest('.piano-note');
    if (note && !e.altKey) {
      const id = parseInt(note.dataset.id);
      if (e.offsetX > note.clientWidth - 6 && !e.shiftKey) {
        resizing = { id, startX: x };
        return;
      }
      if (e.offsetY > note.clientHeight - 6 && !e.shiftKey) {
        editingVel = { id, startVel: null };
        return;
      }
      if (e.shiftKey) {
        if (state.selectedNoteId === id) state.selectedNoteId = null;
        else state.selectedNoteId = id;
      } else {
        state.selectedNoteId = id;
      }
      draggingNote = { id, startX: null, startMidi: null };
      return;
    }
    if (e.target === pianoArea || e.target === pianoGrid || e.target === pianoRuler) {
      const cell = BAR_W / (state.gridDiv / 4);
      const sx = snapX(x);
      const row = Math.floor(y / ROW_H);
      const midi = HIGH - row;
      const smidi = snapMidi(midi);
      const ch = channels.find(c => c.type === 'melodic') || channels[0];
      if (!ch.notes) ch.notes = [];
      const nn = { id: Date.now(), chId: ch.id, midi: smidi, x: sx, w: cell, vel: 90 };
      ch.notes.push(nn);
      state.selectedNoteId = nn.id;
      renderPianoNotes();
      drawing = { id: nn.id, startX: sx };
      pushUndo();
      playNote(smidi, 90, 0.3, getChannelNode(ch.id).gain.gain.value);
    }
  });

  window.addEventListener('mousemove', e => {
    const rect = pianoArea.getBoundingClientRect();
    const rawX = e.clientX - rect.left + pianoArea.scrollLeft;
    const rawY = e.clientY - rect.top + pianoArea.scrollTop;
    if (draggingNote) {
      const n = allNotes().find(z => z.id === draggingNote.id);
      if (!n) return;
      if (draggingNote.startX === null) { draggingNote.startX = n.x; draggingNote.startMidi = n.midi; }
      n.x = snapX(rawX);
      n.midi = snapMidi(HIGH - Math.floor(rawY / ROW_H));
      renderPianoNotes();
    }
    if (drawing) {
      const n = allNotes().find(z => z.id === drawing.id);
      if (n) { n.w = Math.max(20, rawX - n.x); renderPianoNotes(); }
    }
    if (resizing) {
      const n = allNotes().find(z => z.id === resizing.id);
      if (n) {
        n.w = Math.max(20, rawX - n.x);
        renderPianoNotes();
      }
    }
    if (editingVel) {
      const n = allNotes().find(z => z.id === editingVel.id);
      if (n) {
        const el = document.querySelector('.piano-note[data-id="' + editingVel.id + '"]');
        if (el) {
          const ar = pianoArea.getBoundingClientRect();
          const relY = (e.clientY - ar.top + pianoArea.scrollTop) - parseFloat(el.style.top);
          const pct = Math.max(0, Math.min(1, 1 - relY / (ROW_H - 2)));
          n.vel = Math.round(pct * 127);
          renderVelocityLane();
        }
      }
    }
  });

  window.addEventListener('mouseup', () => {
    if (draggingNote) {
      const n = allNotes().find(z => z.id === draggingNote.id);
      if (n) { playNote(n.midi, n.vel, 0.2, getChannelNode(n.chId).gain.gain.value); }
      pushUndo();
    }
    if (resizing) pushUndo();
    if (editingVel) pushUndo();
    draggingNote = null; drawing = null; resizing = null; editingVel = null;
  });

  pianoArea.addEventListener('dblclick', e => {
    const note = e.target.closest('.piano-note');
    if (!note) return;
    const id = parseInt(note.dataset.id);
    const n = allNotes().find(z => z.id === id);
    if (!n) return;
    const v = prompt('Velocity (0-127)', n.vel);
    if (v !== null) {
      n.vel = Math.max(0, Math.min(127, parseInt(v || n.vel)));
      renderPianoNotes();
      pushUndo();
    }
  });

  /* ---- transport ---- */
  let animFrame;
  let meterRAF = null;

  function fmtBeats(beats){
    const bar = Math.floor(beats / 4) + 1;
    const beat = Math.floor(beats % 4) + 1;
    const tick = Math.floor((beats % 1) * 1000);
    return bar + '.' + beat + '.' + tick;
  }
  function fmtTime(beats){
    const sec = beats / (state.bpm / 60);
    const m = Math.floor(sec / 60);
    const s = Math.floor(sec % 60).toString().padStart(2, '0');
    return m + ':' + s;
  }
  function updateTransportUI(){
    const sp = $('#song-pos');
    if (sp) sp.textContent = fmtBeats(state.position) + '  |  ' + fmtTime(state.position);
    const ph = $('#playhead');
    if (ph) ph.style.left = (state.position * (BAR_W / 4)) + 'px';
    const playBtn = $('#tr-play');
    if (playBtn) playBtn.classList.toggle('active', state.playing);
    const recBtn = $('#tr-record');
    if (recBtn) recBtn.classList.toggle('active', state.recording);
    const metroBtn = $('#tr-metro');
    if (metroBtn) metroBtn.classList.toggle('active', state.metro);
    const loopBtn = $('#tr-loop');
    if (loopBtn) loopBtn.classList.toggle('active', state.loopEnd > state.loopStart);
    const lr = $('#loop-range');
    if (lr) lr.textContent = fmtBeats(state.loopStart) + ' — ' + fmtBeats(state.loopEnd);
  }
  function updateLoopBar(){
    const lb = $('#loop-bar');
    if (!lb) return;
    lb.style.left = (state.loopStart * (BAR_W / 4)) + 'px';
    lb.style.width = ((state.loopEnd - state.loopStart) * (BAR_W / 4)) + 'px';
  }

  function transportTick(){
    if (!state.playing) return;
    const bps = state.bpm / 60;
    state.position += bps * (16.667 / 1000);
    if (state.position > state.loopEnd) state.position = state.loopStart;
    updateTransportUI();
    if (state.metro) {
      const beat16 = Math.floor(state.position * 4) % 16;
      const beat4 = Math.floor(state.position) % 4;
      const last16 = transportTick._last16 || -1;
      if (beat16 !== last16) {
        transportTick._last16 = beat16;
        if (beat16 % 4 === 0) playMetronome(beat4 === 0);
      }
    }
    const step16 = Math.floor(state.position * 4) % 16;
    channels.filter(c => c.type === 'drum').forEach(ch => {
      if (ch.mute) return;
      if (ch.pattern && ch.pattern[step16]) {
        playDrum(ch.sound, (ch.stepVel && ch.stepVel[step16]) || 100, getChannelNode(ch.id).gain.gain.value);
      }
    });
    allNotes().forEach(n => {
      const sb = n.x / (BAR_W / 4);
      const eb = sb + n.w / (BAR_W / 4);
      if (!n._triggered && state.position >= sb && state.position < eb) {
        n._triggered = true;
        playNote(n.midi, n.vel, (eb - sb) * (60 / state.bpm), getChannelNode(n.chId).gain.gain.value);
      }
      if (state.position < sb) n._triggered = false;
    });
    animFrame = requestAnimationFrame(transportTick);
  }

  $('#tr-play').addEventListener('click', () => {
    state.playing = !state.playing;
    updateTransportUI();
    if (state.playing) {
      transportTick._last16 = -1;
      transportTick();
    }
  });
  $('#tr-stop').addEventListener('click', () => {
    state.playing = false;
    state.position = 0;
    updateTransportUI();
    if (animFrame) cancelAnimationFrame(animFrame);
    transportTick._last16 = -1;
  });
  $('#tr-record').addEventListener('click', () => {
    state.recording = !state.recording;
    $('#tr-record').classList.toggle('active', state.recording);
  });
  $('#tr-loop').addEventListener('click', () => {
    if (state.loopEnd <= state.loopStart) state.loopEnd = state.loopStart + 4;
    $('#tr-loop').classList.toggle('active', state.loopEnd > state.loopStart);
    updateLoopBar();
  });
  $('#tr-metro').addEventListener('click', () => {
    state.metro = !state.metro;
    $('#tr-metro').classList.toggle('active', state.metro);
  });

  const bpmInput = $('#tr-field-bpm');
  bpmInput.value = state.bpm;
  bpmInput.addEventListener('change', e => { state.bpm = parseInt(bpmInput.value) || 124; });

  $('#btn-loop-in').addEventListener('click', () => {
    state.loopStart = state.position;
    updateTransportUI();
    updateLoopBar();
    addHistory('Loop in: ' + state.loopStart.toFixed(2));
  });
  $('#btn-loop-out').addEventListener('click', () => {
    state.loopEnd = state.position;
    updateTransportUI();
    updateLoopBar();
    addHistory('Loop out: ' + state.loopEnd.toFixed(2));
  });

  /* ---- mixer render ---- */
  function renderMixer(){
    const area = $('#mixerBody');
    if (!area) return;
    area.innerHTML = '';

    const masterStrip = document.createElement('div');
    masterStrip.className = 'mixer-strip master';
    masterStrip.innerHTML =
      '<div class="mixer-name">Master</div>' +
      '<div class="meter-wrap"><div class="meter-bar" id="meter-master" style="height:0%"></div></div>' +
      '<div class="mixer-fader">' +
      '<div class="skeu-fader-track"></div>' +
      '<div class="skeu-fader-thumb" style="top:70%" data-fader="master"></div>' +
      '<div class="skeu-fader-value">0.0 dB</div>' +
      '</div>' +
      '<div class="mixer-knobs">' +
      '<div class="knob-wrap"><div class="skeu-knob" data-knob="master-gain" style="--knob-rot:135deg"><div class="pointer"></div><div class="value-ring"></div></div><div class="skeu-knob-label">Gain</div></div>' +
      '<div class="knob-wrap"><div class="skeu-knob" data-knob="master-pan" style="--knob-rot:90deg"><div class="pointer"></div><div class="value-ring"></div></div><div class="skeu-knob-label">Pan</div></div>' +
      '</div>' +
      '<div class="mixer-actions">' +
      '<button class="mixer-btn mute" title="Mute">M</button>' +
      '<button class="mixer-btn solo" title="Solo">S</button>' +
      '</div>' +
      '<div class="insert-slot">FX 1</div>';
    area.appendChild(masterStrip);

    channels.forEach(ch => {
      const strip = document.createElement('div');
      strip.className = 'mixer-strip';
      strip.style.setProperty('--ch-color', ch.color);
      strip.innerHTML =
        '<div class="mixer-name">' + ch.name + '</div>' +
        '<div class="meter-wrap"><div class="meter-bar" id="meter-' + ch.id + '" style="height:0%"></div></div>' +
        '<div class="mixer-fader">' +
        '<div class="skeu-fader-track"></div>' +
        '<div class="skeu-fader-thumb" style="top:' + (70 + (ch.vol || 0) * 1.5) + '%" data-fader="' + ch.id + '"></div>' +
        '<div class="skeu-fader-value">' + (ch.vol || 0).toFixed(1) + ' dB</div>' +
        '</div>' +
        '<div class="mixer-knobs">' +
        '<div class="knob-wrap"><div class="skeu-knob" data-knob="ch' + ch.id + '-gain" style="--knob-rot:135deg"><div class="pointer"></div><div class="value-ring"></div></div><div class="skeu-knob-label">Gain</div></div>' +
        '<div class="knob-wrap"><div class="skeu-knob" data-knob="ch' + ch.id + '-pan" style="--knob-rot:90deg"><div class="pointer"></div><div class="value-ring"></div></div><div class="skeu-knob-label">Pan</div></div>' +
        '</div>' +
        '<div class="mixer-actions">' +
        '<button class="mixer-btn mute ' + (ch.mute ? 'active' : '') + '" title="Mute">M</button>' +
        '<button class="mixer-btn solo ' + (ch.solo ? 'active' : '') + '" title="Solo">S</button>' +
        '<button class="mixer-btn arm ' + (ch.arm ? 'active' : '') + '" title="Arm">R</button>' +
        '</div>' +
        '<div class="insert-slot">FX 1</div>';
      area.appendChild(strip);
    });
    initMixerControls();
  }

  function initMixerControls(){
    $$('.skeu-fader-thumb').forEach(thumb => {
      thumb.removeEventListener('mousedown', thumb._mstart);
      let active = false, startY, startTop;
      const start = e => {
        active = true; startY = e.clientY;
        startTop = parseFloat(getComputedStyle(thumb).top) || 0;
        thumb.setAttribute('data-active', '1');
      };
      const move = e => {
        if (!active) return;
        e.preventDefault();
        let top = startTop + (e.clientY - startY);
        top = Math.max(0, Math.min(160 - 34, top));
        thumb.style.top = top + 'px';
        const pct = 1 - top / (160 - 34);
        const db = (pct * 24 - 12).toFixed(1);
        thumb.nextElementSibling.textContent = db + ' dB';
      };
      const end = () => { active = false; thumb.removeAttribute('data-active'); pushUndo(); };
      thumb._mstart = start;
      thumb.addEventListener('mousedown', start);
      window.addEventListener('mousemove', move);
      window.addEventListener('mouseup', end);
      thumb.addEventListener('dblclick', () => {
        thumb.style.top = '50%';
        thumb.nextElementSibling.textContent = '0.0 dB';
        pushUndo();
      });
    });

    $$('.skeu-knob').forEach(knob => {
      knob.removeEventListener('mousedown', knob._kstart);
      let active = false, startY, startRot;
      const getRot = () => {
        const m = (getComputedStyle(knob).getPropertyValue('--knob-rot') || '0deg').match(/([-\d.]+)/);
        return parseFloat(m[1]) || 0;
      };
      const setRot = deg => {
        knob.style.setProperty('--knob-rot', ((deg % 360) + 360) % 360 + 'deg');
      };
      const start = e => {
        active = true; startY = e.clientY; startRot = getRot();
        knob.classList.add('active');
      };
      const move = e => {
        if (!active) return;
        let rot = startRot + (startY - e.clientY) * 2;
        setRot(rot);
      };
      const end = () => { active = false; knob.classList.remove('active'); pushUndo(); };
      knob._kstart = start;
      knob.addEventListener('mousedown', start);
      window.addEventListener('mousemove', move);
      window.addEventListener('mouseup', end);
      knob.addEventListener('wheel', e => {
        e.preventDefault();
        setRot(getRot() - e.deltaY * 0.5);
      }, { passive: false });
      knob.addEventListener('dblclick', () => setRot(90));
      knob.addEventListener('mousemove', e => { showTooltip(knob, e); });
      knob.addEventListener('mouseleave', hideTooltip);
    });

    $$('.mute,.solo,.arm').forEach(b => {
      b.removeEventListener('click', b._tclick);
      b._tclick = () => {
        b.classList.toggle('active');
        pushUndo();
      };
      b.addEventListener('click', b._tclick);
    });
  }

  /* ---- meter animation ---- */
  function animateMeters(){
    if (!meterRAF) return;
    channels.forEach(ch => {
      const bar = $('#meter-' + ch.id);
      if (!bar) return;
      let h = state.playing ? (Math.random() * (ch.mute ? 5 : 40) + (ch.solo ? 20 : 0)) : 0;
      const cur = parseFloat(bar.style.height || 0);
      h = Math.max(cur - 2, h);
      bar.style.height = Math.min(100, h) + '%';
    });
    const master = $('#meter-master');
    if (master) {
      let h = state.playing ? Math.random() * 35 : 0;
      const cur = parseFloat(master.style.height || 0);
      master.style.height = Math.min(100, Math.max(cur - 2, h)) + '%';
    }
    meterRAF = requestAnimationFrame(animateMeters);
  }

  $('#tr-play').addEventListener('click', () => {
    if (state.playing) {
      if (!meterRAF) meterRAF = requestAnimationFrame(animateMeters);
    } else {
      if (meterRAF) cancelAnimationFrame(meterRAF);
      meterRAF = null;
      channels.forEach(ch => {
        const b = $('#meter-' + ch.id);
        if (b) b.style.height = '0%';
      });
      const m = $('#meter-master');
      if (m) m.style.height = '0%';
    }
  });

  /* ---- tooltip (re-export for daw.js) ---- */
  const tooltip = $('#daw-tooltip');
  let ttTimer;
  function showTooltip(el, e){
    clearTimeout(ttTimer);
    ttTimer = setTimeout(() => {
      tooltip.textContent = (el.dataset.knob || el.dataset.fader || el.title || '');
      tooltip.classList.add('show');
      positionTooltip(e);
    }, 400);
  }
  function hideTooltip(){
    clearTimeout(ttTimer);
    tooltip.classList.remove('show');
  }
  function positionTooltip(e){
    tooltip.style.left = (e.clientX + 14) + 'px';
    tooltip.style.top = (e.clientY + 14) + 'px';
  }

  /* ---- playlist ---- */
  const btnPlaylist = $('#btn-playlist');
  const playlist = $('#pane-playlist');
  if (btnPlaylist && playlist) {
    btnPlaylist.addEventListener('click', () => {
      const on = playlist.style.display === 'none';
      playlist.style.display = on ? 'flex' : 'none';
      btnPlaylist.setAttribute('aria-pressed', on);
      state.playlistVisible = on;
      if (on) renderPlaylist();
    });
  }
  document.addEventListener('keydown', e => {
    if ((e.metaKey || e.ctrlKey) && e.key === 'p') {
      e.preventDefault();
      if (btnPlaylist) btnPlaylist.click();
    }
  });

  function renderPlaylist(){
    const area = $('#playlist-clips');
    if (!area) return;
    area.innerHTML = '';
    let left = 0;
    channels.forEach((ch, i) => {
      if (ch.type !== 'drum') return;
      const clip = document.createElement('div');
      clip.className = 'playlist-clip';
      clip.style.setProperty('--clip-color', ch.color);
      clip.style.left = left + 'px';
      clip.style.top = (i * 60) + 'px';
      clip.style.width = (BAR_W * 4) + 'px';
      clip.style.height = '52px';
      clip.textContent = ch.name + ' pattern';
      clip.dataset.chId = ch.id;
      area.appendChild(clip);
      left += BAR_W * 4;
    });
    const autoLane = document.createElement('div');
    autoLane.style.cssText = 'position:absolute;bottom:0;left:0;right:0;height:80px;border-top:1px solid var(--hair);background:var(--panel-bottom)';
    autoLane.innerHTML = '<div style="padding:4px 8px;font-size:var(--t-micro);color:var(--ink-3)">Automation: Volume</div>';
    $('#playlist-area').appendChild(autoLane);
  }

  $('#playlist-area').addEventListener('mousedown', e => {
    const clip = e.target.closest('.playlist-clip');
    if (!clip) return;
    let sx = parseFloat(clip.style.left), sy = parseFloat(clip.style.top), px = e.clientX, py = e.clientY;
    const move = ev => {
      clip.style.left = (sx + ev.clientX - px) + 'px';
      clip.style.top = (sy + ev.clientY - py) + 'px';
    };
    const up = () => {
      window.removeEventListener('mousemove', move);
      window.removeEventListener('mouseup', up);
    };
    window.addEventListener('mousemove', move);
    window.addEventListener('mouseup', up);
  });

  /* ---- AI command palette ---- */
  if (btnAI) btnAI.addEventListener('click', openCmd);
  document.addEventListener('keydown', e => {
    if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
      e.preventDefault();
      openCmd();
    }
    if (e.key === 'Escape') {
      closeCmd();
      if (aiHistoryEl) aiHistoryEl.classList.remove('open');
    }
  });
  if (cmdInput) {
    cmdInput.addEventListener('input', e => renderCmd(e.target.value));
    cmdInput.addEventListener('keydown', e => { if (e.key === 'Escape') closeCmd(); });
  }
  if (cmdResults) {
    cmdResults.addEventListener('click', e => {
      const item = e.target.closest('.cmd-item');
      if (item) applyAI(item.dataset.ai);
    });
  }

  const aiHistoryClose = document.getElementById('ai-history-close');
  if (aiHistoryClose) {
    aiHistoryClose.addEventListener('click', () => { if (aiHistoryEl) aiHistoryEl.classList.remove('open'); });
  }
  if (aiPill) {
    aiPill.addEventListener('click', () => {
      if (aiHistoryEl) aiHistoryEl.classList.toggle('open');
    });
  }
  if (aiHistoryList) {
    aiHistoryList.addEventListener('click', e => {
      const btn = e.target.closest('.ai-history-undo');
      if (btn) btn.closest('.ai-history-item').remove();
    });
  }

  /* ---- sampler / fx toggles ---- */
  const btnSampler = $('#btn-sampler');
  const paneSampler = $('#pane-sampler');
  if (btnSampler && paneSampler) {
    btnSampler.addEventListener('click', () => {
      const on = !paneSampler.classList.contains('open');
      paneSampler.classList.toggle('open', on);
      paneSampler.style.display = on ? 'flex' : 'none';
      btnSampler.setAttribute('aria-pressed', on);
    });
  }
  const btnFx = $('#btn-fx');
  const paneFx = $('#pane-fx');
  if (btnFx && paneFx) {
    btnFx.addEventListener('click', () => {
      const on = !paneFx.classList.contains('open');
      paneFx.classList.toggle('open', on);
      paneFx.style.display = on ? 'flex' : 'none';
      btnFx.setAttribute('aria-pressed', on);
    });
  }

  /* ---- project import input ---- */
  const projInput = document.createElement('input');
  projInput.type = 'file';
  projInput.accept = '.json';
  projInput.id = 'proj-import';
  projInput.style.display = 'none';
  projInput.addEventListener('change', e => {
    if (e.target.files[0]) importProject(e.target.files[0]);
  });
  document.body.appendChild(projInput);
  const btnImport = $('#btn-import');
  if (btnImport) btnImport.addEventListener('click', () => projInput.click());
  const btnExport = $('#btn-export');
  if (btnExport) btnExport.addEventListener('click', exportProject);

  /* ---- pane float/dock + drag ---- */
  $$('.daw-pane-header').forEach(header => {
    const pane = header.closest('.daw-pane');
    const floatBtn = header.querySelector('[title="Float/dock"]');
    let isFloating = pane.classList.contains('floating');
    let dragging = false, startX, startY, startLeft, startTop;
    if (floatBtn) {
      floatBtn.addEventListener('click', () => {
        isFloating = !isFloating;
        pane.classList.toggle('floating', isFloating);
        pane.classList.toggle('docked', !isFloating);
        if (isFloating) {
          pane.style.position = 'fixed';
          pane.style.left = '220px';
          pane.style.top = '140px';
          pane.style.width = '420px';
          pane.style.height = '460px';
        } else {
          pane.style.position = '';
          pane.style.left = '';
          pane.style.top = '';
          pane.style.width = '';
          pane.style.height = '';
        }
      });
    }
    header.addEventListener('mousedown', e => {
      if (e.target.tagName === 'BUTTON' || e.target.tagName === 'SELECT' || e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA') return;
      if (!isFloating) return;
      dragging = true;
      startX = e.clientX; startY = e.clientY;
      const cs = getComputedStyle(pane);
      startLeft = parseFloat(cs.left) || 0;
      startTop = parseFloat(cs.top) || 0;
      pane.style.zIndex = 70;
    });
    window.addEventListener('mousemove', e => {
      if (!dragging) return;
      pane.style.left = (startLeft + e.clientX - startX) + 'px';
      pane.style.top = (startTop + e.clientY - startY) + 'px';
    });
    window.addEventListener('mouseup', () => {
      dragging = false;
      pane.style.zIndex = 60;
    });
  });

  $$('[title="Close"]').forEach(btn => {
    btn.addEventListener('click', () => {
      btn.closest('.daw-pane').style.display = 'none';
    });
  });

  /* ---- sidebar collapse ---- */
  const sbCollapse = $('#sbCollapse');
  const windowEl = $('#win');
  if (sbCollapse && windowEl) {
    sbCollapse.addEventListener('click', () => {
      state.sidebarCollapsed = !state.sidebarCollapsed;
      windowEl.classList.toggle('is-collapsed', state.sidebarCollapsed);
    });
  }

  /* ---- global keyboard shortcuts ---- */
  document.addEventListener('keydown', e => {
    // Undo/redo
    if ((e.metaKey || e.ctrlKey) && e.key === 'z') {
      e.preventDefault();
      if (e.shiftKey) redo(); else undo();
    }
    // Transport
    if (e.code === 'Space') {
      e.preventDefault();
      $('#tr-play').click();
    }
    if (e.key === 'r' || e.key === 'R') {
      e.preventDefault();
      $('#tr-record').click();
    }
    if (e.key === 'l' || e.key === 'L') {
      e.preventDefault();
      $('#tr-loop').click();
    }
    if (e.key === 'i' || e.key === 'I') {
      e.preventDefault();
      $('#btn-loop-in').click();
    }
    if (e.key === 'o' || e.key === 'O') {
      e.preventDefault();
      $('#btn-loop-out').click();
    }
    if (e.key === 't' || e.key === 'T') {
      e.preventDefault();
      // tap tempo
    }
    // Scale
    if (e.key === 's' || e.key === 'S') {
      e.preventDefault();
      state.scaleActive = !state.scaleActive;
      if (btnScale) btnScale.setAttribute('aria-pressed', state.scaleActive);
      applyScale();
    }
    // Grid division
    const k = e.key;
    if (k >= '1' && k <= '9') {
      const g = parseInt(k) * 2;
      if (g <= 64) {
        state.gridDiv = g;
        renderPianoNotes();
      }
    }
    // Delete note
    if (e.key === 'Delete' || e.key === 'Backspace') {
      if (state.selectedNoteId !== null) {
        channels.forEach(c => {
          if (c.notes) c.notes = c.notes.filter(z => z.id !== state.selectedNoteId);
        });
        state.selectedNoteId = null;
        renderPianoNotes();
        pushUndo();
      }
    }
  });

  /* ---- init ---- */
  function init(){
    buildGrid();
    ensureAudio();
    renderRack();
    renderBrowser();
    renderPianoNotes();
    renderMixer();
    updateTransportUI();
    updateLoopBar();
    renderPlaylist();
    console.log('Smart DAW initialized');
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }

  window.smartDaw = {
    state, channels, allNotes, playNote, playDrum, renderRack, renderPianoNotes,
    renderMixer, renderAll, exportProject, importProject, undo, redo, plugins: [],
    onDrop: (data, zoneId) => {
      console.log('Drop:', data, 'at', zoneId);
      addHistory('Dropped: ' + data);
    }
  };
})();
