//! The one object a frontend talks to. Commands go in as method calls, results come back from `poll()`.
//! No frontend types appear here — swap egui for gpui/iced/slint/Qt by rewriting only the view layer.
use crate::apps::{AppInfo, AppProvider};
use crate::audio::Audio;
use crate::devices::{Device, DeviceProvider};
use crate::exec::spawn;
use crate::library::{Library, SharedLibrary};
use crate::model::*;
use crate::ops::{self, JobCtx, JobEvent, JobKind};
use crate::registry::Registry;
use crate::search::{Hit, Query, SearchEngine, SearchMsg};
use crate::thumbs::{ThumbResult, Thumbs};
use crate::vfs::FsError;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct Place {
    pub id: String,
    pub title: String,
    pub icon: &'static str,
    pub loc: Location,
}

pub enum Event {
    Listed { req: u64, loc: Location, result: Result<Vec<Entry>, String> },
    SearchHits { req: u64, hits: Vec<Hit> },
    SearchDone { req: u64, scanned: usize, elapsed_ms: i64, cached: bool },
    Job(JobEvent),
    Thumb(ThumbResult),
    Devices(Vec<Device>),
    Changed(Location),
    Summary { loc: Location, items: usize, bytes: u64 },
    DeviceOp { id: String, op: &'static str, result: Result<Option<Location>, String> },
    Message { text: String, ok: bool },
    ArchiveDone { loc: Location, ok: bool, detail: String },
}

pub struct PacedJob {
    id: u64,
    kind: JobKind,
    started_ms: i64,
    secs: f32,
    last: i64,
}

pub struct Backend {
    pub registry: Arc<Registry>,
    pub library: SharedLibrary,
    pub audio: Audio,
    pub demo: bool,
    apps: Arc<dyn AppProvider>,
    providers: Vec<Arc<dyn DeviceProvider>>,
    search: SearchEngine,
    thumbs: Thumbs,
    devices: Arc<Mutex<Vec<Device>>>,
    tx: Sender<Event>,
    rx: Receiver<Event>,
    search_tx: Sender<SearchMsg>,
    search_rx: Receiver<SearchMsg>,
    search_cancel: Option<Arc<AtomicBool>>,
    jobs: Mutex<HashMap<u64, Arc<AtomicBool>>>,
    next: AtomicU64,
    paced: Mutex<Vec<PacedJob>>,
    #[cfg(not(target_arch = "wasm32"))]
    watcher: Option<notify::RecommendedWatcher>,
}

impl Backend {
    fn assemble(registry: Arc<Registry>, library: Library, apps: Arc<dyn AppProvider>, providers: Vec<Arc<dyn DeviceProvider>>, demo: bool) -> Self {
        let library = library.shared();
        let (tx, rx) = channel();
        let (search_tx, search_rx) = channel();
        let devices: Vec<Device> = providers.iter().flat_map(|p| p.list()).collect();
        Self {
            search: SearchEngine::new(registry.clone(), library.clone()),
            thumbs: Thumbs::new(registry.clone()),
            registry,
            library,
            audio: Audio::new(),
            demo,
            apps,
            providers,
            devices: Arc::new(Mutex::new(devices)),
            tx,
            rx,
            search_tx,
            search_rx,
            search_cancel: None,
            jobs: Mutex::new(HashMap::new()),
            next: AtomicU64::new(1),
            paced: Mutex::new(Vec::new()),
            #[cfg(not(target_arch = "wasm32"))]
            watcher: None,
        }
    }

    /// Real disk, real mounts, real default apps.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new_native() -> Self {
        use crate::devices::native::{adb, SystemDevices};
        let reg = Arc::new(Registry::new());
        reg.insert(Arc::new(crate::local::LocalVfs));
        let mut providers: Vec<Arc<dyn DeviceProvider>> = vec![Arc::new(SystemDevices)];
        if adb::adb_available() {
            providers.push(Arc::new(adb::AdbDevices));
        }
        let b = Self::assemble(reg, Library::load(), Arc::new(crate::apps::SystemApps), providers, false);
        // Phones are registered as soon as they are seen so they can be browsed immediately.
        for d in b.devices.lock().unwrap().iter() {
            if d.id.starts_with("adb:") {
                if let Some(p) = b.providers.iter().find(|p| p.owns(&d.id)) {
                    if let Ok(m) = p.mount(&d.id) {
                        if let Some(v) = m.vfs {
                            b.registry.insert(v);
                        }
                    }
                }
            }
        }
        b
    }

    /// Seeded in-memory world (browser build, screenshots, tests).
    #[cfg(feature = "demo")]
    pub fn new_demo(library: Library) -> Self {
        let (reg, dev) = crate::demo::build();
        Self::assemble(reg, library, Arc::new(crate::apps::DemoApps), vec![dev], true)
    }

    fn id(&self) -> u64 {
        self.next.fetch_add(1, Ordering::Relaxed)
    }

    // ───────── places ─────────
    pub fn home(&self) -> Location {
        #[cfg(feature = "demo")]
        if self.demo {
            return crate::demo::home();
        }
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(h) = dirs::home_dir() {
            return crate::local::path_to_loc(&h);
        }
        Location::new("mem", "/")
    }

    pub fn places(&self) -> Vec<Place> {
        let mk = |id: &str, title: &str, icon: &'static str, loc: Location| Place { id: id.into(), title: title.into(), icon, loc };
        let home = self.home();
        let mut v = vec![mk("home", "Home", "home", home.clone())];
        #[cfg(not(target_arch = "wasm32"))]
        if !self.demo {
            let add = |v: &mut Vec<Place>, id: &str, t: &str, i: &'static str, p: Option<std::path::PathBuf>| {
                if let Some(p) = p.filter(|p| p.is_dir()) {
                    v.push(mk(id, t, i, crate::local::path_to_loc(&p)));
                }
            };
            add(&mut v, "desktop", "Desktop", "monitor", dirs::desktop_dir());
            add(&mut v, "documents", "Documents", "document-text", dirs::document_dir());
            add(&mut v, "downloads", "Downloads", "download-minimalistic", dirs::download_dir());
            add(&mut v, "pictures", "Pictures", "gallery", dirs::picture_dir());
            add(&mut v, "music", "Music", "music-note", dirs::audio_dir());
            add(&mut v, "movies", "Movies", "clapperboard", dirs::video_dir());
            return v;
        }
        for (id, t, i) in [("desktop", "Desktop", "monitor"), ("documents", "Documents", "document-text"), ("downloads", "Downloads", "download-minimalistic"), ("pictures", "Pictures", "gallery"), ("music", "Music", "music-note"), ("movies", "Movies", "clapperboard")] {
            v.push(mk(id, t, i, home.join(t)));
        }
        v
    }

    // ───────── browsing ─────────
    pub fn list(&self, loc: &Location) -> u64 {
        let (id, reg, tx, loc) = (self.id(), self.registry.clone(), self.tx.clone(), loc.clone());
        spawn(move || {
            let result = match reg.get(&loc.fs) {
                Some(v) => v.list(&loc.path).map_err(|e| e.to_string()),
                None => Err("This location is no longer available".into()),
            };
            let _ = tx.send(Event::Listed { req: id, loc, result });
        });
        id
    }

    pub fn summarize(&self, loc: &Location) {
        let (reg, tx, loc) = (self.registry.clone(), self.tx.clone(), loc.clone());
        spawn(move || {
            if let Some(Ok(l)) = reg.get(&loc.fs).map(|v| v.list(&loc.path)) {
                let l: Vec<_> = l.into_iter().filter(|e| !e.hidden).collect();
                let _ = tx.send(Event::Summary { loc, items: l.len(), bytes: l.iter().map(|e| e.size).sum() });
            }
        });
    }

    pub fn stat(&self, loc: &Location) -> Result<Entry, String> {
        self.registry.get(&loc.fs).ok_or("unavailable")?.stat(&loc.path).map_err(|e| e.to_string())
    }

    pub fn capacity(&self, loc: &Location) -> Option<(u64, u64)> {
        self.registry.get(&loc.fs)?.capacity(&loc.path).map(|c| (c.total, c.free))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn watch(&mut self, loc: &Location) {
        use notify::{RecursiveMode, Watcher};
        self.watcher = None;
        let Some(path) = self.registry.get(&loc.fs).and_then(|v| v.real_path(&loc.path)) else { return };
        let (tx, l) = (self.tx.clone(), loc.clone());
        if let Ok(mut w) = notify::recommended_watcher(move |r: notify::Result<notify::Event>| {
            if r.is_ok() {
                let _ = tx.send(Event::Changed(l.clone()));
            }
        }) {
            if w.watch(&path, RecursiveMode::NonRecursive).is_ok() {
                self.watcher = Some(w);
            }
        }
    }
    #[cfg(target_arch = "wasm32")]
    pub fn watch(&mut self, _loc: &Location) {}

    // ───────── search ─────────
    pub fn search(&mut self, raw: &str, scope: &Location, contents: bool) -> u64 {
        self.cancel_search();
        let mut q = Query::parse(raw);
        if contents {
            q = q.into_content_search();
        }
        let id = self.id();
        self.search_cancel = Some(self.search.start(id, q, scope.clone(), self.search_tx.clone()));
        id
    }
    pub fn cancel_search(&mut self) {
        if let Some(c) = self.search_cancel.take() {
            c.store(true, Ordering::Relaxed);
        }
    }
    pub fn indexed_count(&self) -> usize {
        self.search.indexed_count()
    }

    // ───────── mutation (small, synchronous) ─────────
    fn vfs(&self, loc: &Location) -> Result<Arc<dyn crate::vfs::Vfs>, String> {
        self.registry.get(&loc.fs).ok_or_else(|| "This location is no longer available".to_string())
    }

    pub fn create_folder(&self, dir: &Location, name: &str) -> Result<Location, String> {
        let v = self.vfs(dir)?;
        let n = ops::unique_name(v.as_ref(), dir, name, None);
        let l = dir.join(&n);
        v.mkdir(&l.path).map_err(|e| e.to_string())?;
        self.search.invalidate(dir);
        Ok(l)
    }
    pub fn create_file(&self, dir: &Location, name: &str) -> Result<Location, String> {
        let v = self.vfs(dir)?;
        let n = ops::unique_name(v.as_ref(), dir, name, None);
        let l = dir.join(&n);
        v.write(&l.path, b"").map_err(|e| e.to_string())?;
        self.search.invalidate(dir);
        Ok(l)
    }
    pub fn rename(&self, loc: &Location, new_name: &str) -> Result<Location, String> {
        let name = new_name.trim();
        if name.is_empty() || name.contains('/') {
            return Err("That name isn\u{2019}t valid".into());
        }
        let Some(parent) = loc.parent() else { return Err("Can\u{2019}t rename this location".into()) };
        let to = parent.join(name);
        if to == *loc {
            return Ok(to);
        }
        self.vfs(loc)?.rename(&loc.path, &to.path).map_err(|e| match e {
            FsError::Exists => format!("\u{201c}{}\u{201d} already exists here", name),
            e => e.to_string(),
        })?;
        self.library.write().unwrap().rekey(loc, &to);
        self.search.invalidate(&parent);
        Ok(to)
    }
    pub fn read_bytes(&self, loc: &Location, limit: usize) -> Result<Vec<u8>, String> {
        self.vfs(loc)?.read(&loc.path, limit).map_err(|e| e.to_string())
    }
    pub fn read_text(&self, loc: &Location, limit: usize) -> Result<String, String> {
        let b = self.read_bytes(loc, limit)?;
        if b.iter().take(2048).any(|c| *c == 0) {
            return Err("Binary file".into());
        }
        Ok(String::from_utf8_lossy(&b).to_string())
    }
    pub fn write_text(&self, loc: &Location, text: &str) -> Result<(), String> {
        self.vfs(loc)?.write(&loc.path, text.as_bytes()).map_err(|e| e.to_string())
    }

    // ───────── jobs ─────────
    pub fn submit(&self, kind: JobKind) -> u64 {
        let id = self.id();
        let cancel = Arc::new(AtomicBool::new(false));
        self.jobs.lock().unwrap().insert(id, cancel.clone());
        let (reg, tx) = (self.registry.clone(), self.tx.clone());
        spawn(move || {
            let tx2 = tx.clone();
            let ctx = JobCtx { id, cancel, emit: Box::new(move |e| { let _ = tx2.send(Event::Job(e)); }) };
            ops::run(&reg, kind, &ctx);
        });
        id
    }
    /// Does this job involve the demo “virtual” files? Those get a paced transfer animation.
    pub fn has_paced_sources(&self, kind: &JobKind) -> bool {
        kind.srcs().iter().take(8).any(|l| self.registry.get(&l.fs).and_then(|v| v.stat(&l.path).ok()).is_some_and(|e| e.paced))
    }

    /// Copy/move with a deliberate, visible transfer (seconds of progress) — used for the 5 GB showcase files.
    pub fn submit_paced(&self, kind: JobKind, secs: f32) -> u64 {
        let id = self.id();
        let title = kind.title();
        let _ = self.tx.send(Event::Job(JobEvent::Started { id, title }));
        self.paced.lock().unwrap().push(PacedJob { id, kind, started_ms: crate::exec::now_millis(), secs: secs.clamp(0.2, 60.0), last: 0 });
        id
    }

    pub fn archive_entries(&self, loc: &Location, password: Option<&str>) -> Result<Vec<crate::archive::ZipEntry>, String> {
        crate::archive::preview_entries(&self.registry, loc, password).map_err(|e| e.to_string())
    }

    /// Extract on a worker thread with real progress; refreshes the viewer when finished.
    pub fn archive_extract(&self, src: &Location, dest: &Location, password: Option<String>) {
        let (reg, tx, src, dest, id) = (self.registry.clone(), self.tx.clone(), src.clone(), dest.clone(), self.id());
        let title = format!("Extracting {}", src.name());
        let _ = tx.send(Event::Job(JobEvent::Started { id, title: title.clone() }));
        spawn(move || {
            let tx2 = tx.clone();
            let r = crate::archive::extract(&reg, &src, &dest, password.as_deref(), |name, done, total| {
                let _ = tx2.send(Event::Job(JobEvent::Progress { id, done, total, current: name.to_string() }));
            });
            match r {
                Ok((n, bytes)) => {
                    let _ = tx.send(Event::Job(JobEvent::Finished { id, created: vec![dest.clone()], touched: vec![dest.clone()] }));
                    let _ = tx.send(Event::Message { text: format!("Extracted {} files ({}) into {}", n, crate::model::format_size(bytes), dest.name()), ok: true });
                    let _ = tx.send(Event::ArchiveDone { loc: src, ok: true, detail: format!("{} files", n) });
                }
                Err(e) => {
                    let _ = tx.send(Event::Job(JobEvent::Failed { id, error: e.to_string() }));
                }
            }
        });
    }

    pub fn archive_compress(&self, srcs: Vec<Location>, dest_zip: Location, password: Option<String>, level: u8) {
        let (reg, tx, id) = (self.registry.clone(), self.tx.clone(), self.id());
        let title = format!("Creating {}", dest_zip.name());
        let _ = tx.send(Event::Job(JobEvent::Started { id, title }));
        spawn(move || {
            let tx2 = tx.clone();
            let r = crate::archive::compress(&reg, &srcs, &dest_zip, password, level, |name, done, total| {
                let _ = tx2.send(Event::Job(JobEvent::Progress { id, done, total, current: name.to_string() }));
            });
            match r {
                Ok((n, _)) => {
                    let parent = dest_zip.parent().unwrap_or_else(|| dest_zip.clone());
                    let _ = tx.send(Event::Job(JobEvent::Finished { id, created: vec![dest_zip.clone()], touched: vec![parent] }));
                    let _ = tx.send(Event::Message { text: format!("Compressed {} files into {}", n, dest_zip.name()), ok: true });
                    let _ = tx.send(Event::ArchiveDone { loc: dest_zip, ok: true, detail: format!("{} files", n) });
                }
                Err(e) => {
                    let _ = tx.send(Event::Job(JobEvent::Failed { id, error: e.to_string() }));
                }
            }
        });
    }

    /// Native terminal at this folder (Linux/macOS).
    pub fn open_terminal(&self, loc: &Location) {
        let Some(p) = self.registry.get(&loc.fs).and_then(|v| v.real_path(&loc.path)) else { return };
        let dir = if p.is_dir() { p } else { p.parent().map(|x| x.to_path_buf()).unwrap_or(p) };
        #[cfg(not(target_arch = "wasm32"))]
        {
            for (cmd, args) in [("x-terminal-emulator", vec![]), ("gnome-terminal", vec!["--working-directory".to_string(), dir.to_string_lossy().to_string()]), ("konsole", vec!["--workdir".to_string(), dir.to_string_lossy().to_string()]), ("xfce4-terminal", vec!["--working-directory".to_string(), dir.to_string_lossy().to_string()])] {
                let mut c = std::process::Command::new(cmd);
                if args.is_empty() {
                    c.current_dir(&dir);
                } else {
                    c.args(&args);
                }
                if c.spawn().is_ok() {
                    return;
                }
            }
        }
        #[cfg(target_arch = "wasm32")]
        let _ = dir;
    }

    pub fn cancel_job(&self, id: u64) {
        if let Some(c) = self.jobs.lock().unwrap().get(&id) {
            c.store(true, Ordering::Relaxed);
        }
    }

    // ───────── thumbnails ─────────
    pub fn request_thumb(&self, loc: &Location, edge: u32) {
        self.thumbs.request(loc, edge);
    }
    pub fn forget_thumb(&self, loc: &Location) {
        self.thumbs.forget(loc);
    }

    // ───────── devices ─────────
    pub fn devices(&self) -> Vec<Device> {
        self.devices.lock().unwrap().clone()
    }
    pub fn refresh_devices(&self) {
        let (providers, tx, store) = (self.providers.clone(), self.tx.clone(), self.devices.clone());
        spawn(move || {
            let all: Vec<Device> = providers.iter().flat_map(|p| p.list()).collect();
            *store.lock().unwrap() = all.clone();
            let _ = tx.send(Event::Devices(all));
        });
    }
    fn device_op(&self, id: &str, op: &'static str) {
        let (providers, tx, reg, id) = (self.providers.clone(), self.tx.clone(), self.registry.clone(), id.to_string());
        let store = self.devices.clone();
        spawn(move || {
            let Some(p) = providers.iter().find(|p| p.owns(&id)) else {
                let _ = tx.send(Event::DeviceOp { id, op, result: Err("Unknown device".into()) });
                return;
            };
            let result = match op {
                "mount" => p.mount(&id).map(|m| {
                    if let Some(v) = m.vfs {
                        reg.insert(v);
                    }
                    Some(m.loc)
                }),
                "eject" => p.eject(&id).map(|_| None),
                _ => p.unmount(&id).map(|_| None),
            };
            let all: Vec<Device> = providers.iter().flat_map(|p| p.list()).collect();
            *store.lock().unwrap() = all.clone();
            let _ = tx.send(Event::DeviceOp { id, op, result });
            let _ = tx.send(Event::Devices(all));
        });
    }
    pub fn mount(&self, id: &str) {
        self.device_op(id, "mount")
    }
    pub fn unmount(&self, id: &str) {
        self.device_op(id, "unmount")
    }
    pub fn eject(&self, id: &str) {
        self.device_op(id, "eject")
    }

    /// Connect to a network location (smb://, sftp://, dav://, mtp://) through GVfs — real on Linux.
    pub fn connect_server(&self, url: &str) {
        let (tx, url) = (self.tx.clone(), url.to_string());
        #[cfg(not(target_arch = "wasm32"))]
        if !self.demo {
            spawn(move || {
                let r = std::process::Command::new("gio").args(["mount", &url]).output();
                let (text, ok) = match r {
                    Ok(o) if o.status.success() => (format!("Connected to {}", url), true),
                    Ok(o) => (String::from_utf8_lossy(&o.stderr).trim().to_string(), false),
                    Err(_) => ("Connecting to servers needs GVfs (gio) on this system".to_string(), false),
                };
                let _ = tx.send(Event::Message { text, ok });
            });
            return;
        }
        let _ = tx.send(Event::Message { text: format!("Demo: pretending to connect to {}", url), ok: true });
    }

    // ───────── open / open with ─────────
    pub fn apps_for(&self, loc: &Location) -> Vec<AppInfo> {
        self.apps.apps_for(&loc.name())
    }
    pub fn open(&self, loc: &Location) {
        self.launch(loc, None)
    }
    pub fn open_with(&self, loc: &Location, app: &AppInfo) {
        self.launch(loc, Some(app.clone()))
    }
    pub fn set_default_app(&self, loc: &Location, app: &AppInfo) -> Result<(), String> {
        self.apps.set_default(app, &loc.name())
    }
    fn launch(&self, loc: &Location, app: Option<AppInfo>) {
        self.library.write().unwrap().touch_recent(loc);
        let (reg, tx, apps, loc) = (self.registry.clone(), self.tx.clone(), self.apps.clone(), loc.clone());
        spawn(move || {
            let name = loc.name();
            let real = reg.get(&loc.fs).and_then(|v| v.real_path(&loc.path));
            #[cfg(not(target_arch = "wasm32"))]
            let real = real.or_else(|| {
                // Remote provider: stage a temporary local copy, then hand it to the OS.
                let v = reg.get(&loc.fs)?;
                let dir = std::env::temp_dir().join("smart-files-open");
                std::fs::create_dir_all(&dir).ok()?;
                let p = dir.join(&name);
                let mut r = v.open_read(&loc.path).ok()?;
                let mut w = std::fs::File::create(&p).ok()?;
                std::io::copy(&mut r, &mut w).ok()?;
                Some(p)
            });
            let res = match &app {
                Some(a) => apps.open_with(a, &name, real.as_deref()),
                None => apps.open_default(&name, real.as_deref()),
            };
            let (text, ok) = match res {
                Ok(t) => (t, true),
                Err(e) => (e, false),
            };
            let _ = tx.send(Event::Message { text, ok });
        });
    }
    /// Reveal in the system file manager (native only).
    pub fn reveal_in_system(&self, loc: &Location) {
        let Some(p) = self.registry.get(&loc.fs).and_then(|v| v.real_path(&loc.path)) else { return };
        let dir = if p.is_dir() { p } else { p.parent().map(|x| x.to_path_buf()).unwrap_or(p) };
        #[cfg(not(target_arch = "wasm32"))]
        {
            let cmd = if cfg!(target_os = "macos") { "open" } else if cfg!(windows) { "explorer" } else { "xdg-open" };
            let _ = std::process::Command::new(cmd).arg(dir).spawn();
        }
        #[cfg(target_arch = "wasm32")]
        let _ = dir;
    }

    // ───────── pump ─────────
    pub fn poll(&mut self) -> Vec<Event> {
        let mut out: Vec<Event> = vec![];
        // paced (showcase) transfers: progress is driven by wall-clock time so the UI animates smoothly
        let now = crate::exec::now_millis();
        let mut finished: Vec<(u64, JobKind)> = vec![];
        {
            let mut q = self.paced.lock().unwrap();
            q.retain_mut(|j| {
                let frac = ((now - j.started_ms) as f32 / 1000.0) / j.secs;
                if now - j.last >= 16 || frac >= 1.0 {
                    j.last = now;
                    let total = 1000u64;
                    let done = ((frac.clamp(0.0, 0.999) * total as f32) as u64).max(1);
                    out.push(Event::Job(JobEvent::Progress { id: j.id, done, total, current: j.kind.srcs().first().map(|l| l.name()).unwrap_or_default() }));
                }
                if frac >= 1.0 {
                    finished.push((j.id, j.kind.clone()));
                    return false;
                }
                true
            });
        }
        for (id, kind) in finished {
            match ops::run_sync(&self.registry, &kind) {
                Ok((created, touched)) => {
                    for t in &touched {
                        self.search.invalidate(t);
                    }
                    let n = created.len();
                    out.push(Event::Job(JobEvent::Finished { id, created, touched }));
                    let verb = match kind { JobKind::Move { .. } => "Moved", _ => "Copied" };
                    out.push(Event::Message { text: format!("{} {} item{}", verb, n, if n == 1 { "" } else { "s" }), ok: true });
                }
                Err(e) => {
                    out.push(Event::Job(JobEvent::Failed { id, error: e.to_string() }));
                }
            }
        }
        self.be_poll_rest(&mut out);
        out
    }

    fn be_poll_rest(&mut self, out: &mut Vec<Event>) {
        while let Ok(m) = self.search_rx.try_recv() {
            out.push(match m {
                SearchMsg::Hits { id, hits } => Event::SearchHits { req: id, hits },
                SearchMsg::Done { id, scanned, elapsed_ms, cached } => Event::SearchDone { req: id, scanned, elapsed_ms, cached },
            });
        }
        while let Ok(e) = self.rx.try_recv() {
            match &e {
                Event::Changed(l) => self.search.invalidate(l),
                Event::Job(JobEvent::Finished { touched, id, .. }) => {
                    for t in touched {
                        self.search.invalidate(t);
                    }
                    self.jobs.lock().unwrap().remove(id);
                }
                Event::Job(JobEvent::Failed { id, .. }) => {
                    self.jobs.lock().unwrap().remove(id);
                }
                _ => {}
            }
            out.push(e);
        }
        for t in self.thumbs.poll() {
            out.push(Event::Thumb(t));
        }
    }
    pub fn busy(&self) -> bool {
        self.thumbs.pending() || !self.jobs.lock().unwrap().is_empty()
    }

    pub fn save_library(&self) {
        #[cfg(not(target_arch = "wasm32"))]
        if !self.demo {
            self.library.read().unwrap().save();
        }
    }
}
