//! Application state, navigation, event pump and actions. Drawing lives in `chrome`, `content`, `overlays`.
use crate::icons::Icons;
use crate::theme::*;
use egui::{Context, Pos2, Rect, TextureHandle, Vec2};
use smart_core::apps::AppInfo;
use smart_core::tools;
use smart_core::*;
use std::collections::{HashMap, HashSet};

pub const WALLPAPERS: [(&str, &[u8], &[u8]); 4] = [
    ("Cobalt Mist", include_bytes!("../../../assets/wallpaper.jpg"), include_bytes!("../../../assets/wallpaper_blur.jpg")),
    ("Dusk", include_bytes!("../../../assets/wallpaper_dusk.jpg"), include_bytes!("../../../assets/wallpaper_dusk_blur.jpg")),
    ("Meadow", include_bytes!("../../../assets/wallpaper_meadow.jpg"), include_bytes!("../../../assets/wallpaper_meadow_blur.jpg")),
    ("Graphite", include_bytes!("../../../assets/wallpaper_graphite.jpg"), include_bytes!("../../../assets/wallpaper_graphite_blur.jpg")),
];

pub const IS_WEB: bool = cfg!(target_arch = "wasm32");

#[derive(Clone, PartialEq, Debug)]
pub enum Route {
    Loc(Location),
    Recents,
    Tag(String),
}

#[derive(Clone)]
pub struct Confirm {
    pub title: String,
    pub body: String,
    pub ok: String,
    pub danger: bool,
    pub act: Box<Act>,
}

#[derive(Clone)]
pub enum Act {
    Nav(Route),
    Back,
    Forward,
    Up,
    Refresh,
    Open(Location),
    OpenSel,
    OpenWith(Location, AppInfo),
    SetDefault(Location, AppInfo),
    Preview(Location),
    Info,
    NewFolder,
    NewFile,
    Rename(Location),
    Trash,
    DeleteForever,
    Copy,
    Cut,
    Paste,
    Duplicate,
    SelectAll,
    ToggleTag(Location, String),
    CopyTo(bool),
    CopyPath,
    Reveal,
    ToggleSidebar,
    SetGrid(bool),
    SetSort(SortKey),
    ToggleAsc,
    ToggleTheme,
    ToggleHidden,
    ToggleFoldersFirst,
    Modal(Modal),
    ArchiveConfirm(Location),
    CloseModal,
    Palette,
    Ask,
    FocusSearch,
    ClearSearch,
    Mount(String),
    Unmount(String),
    Eject(String),
    SetWallpaper(usize),
    Confirm(Confirm),
    TogglePin(Location),
    Play(Location),
    Toast(String),
    Move(Vec<Location>, Location),
    Organise,
    ApplyOrganise,
    TagOld,
    FindDuplicates,
    FindLarge,
    FindOld,
    RunSearch(String),
    WinClose,
    WinMin,
    WinZoom,
    WinOpen,
    Settings,
    SettingsTab(usize),
    ToggleExtensions,
    ToggleConfirmDelete,
    ToggleReduceMotion,
    ToggleAutoplay,
    SetAnimSecs(f32),
    SetGridZoom(f32),
    ResetLibrary,
    CopyConfig,
    WallpaperFrom(Location),
    ExtractHere(Location),
    ArchiveBrowse(Location),
    ArchiveRetry,
    ArchiveDoExtract,
    ArchiveCompress,
    ArchiveCreate,
    ArchiveClose,
    ArchiveDest(Location),
    ArchiveToggleDest,
    SetArchiveName(String),
    SetArchivePassword(String),
    ArchiveLevel(u8),
    OpenTerminal,
    OpenNewWindow,
    ClearSelection,
    ToggleSelectMode,
    /// Several actions in order (used by the desktop menus: open the window, then act).
    Seq(Vec<Act>),
    /// Command for one of the suite apps ("term.new_tab", "photos.fit" …).
    App(String),
    OpenApp(crate::shell::AppId),
    NewTab(Option<Route>),
    CloseTab(usize),
    SwitchTab(usize),
    CopyText(String),
    OpenInTerminal(Location),
    OpenInPhotos(Location),
    TogglePlayPause(Location),
    CopyDialogClose,
    SearchSheet(bool),
    ToggleEverywhere,
    ToggleContents,
    ToggleGridOverlay,
    ToggleContentSearch,
    NewTag,
    CopyTokens,
    Connect(String),
    Job(JobKind),
}

#[derive(Clone)]
pub enum Modal {
    None,
    Preview(Location),
    Info(Vec<Location>),
    OpenWith(Location),
    Wallpaper,
    About,
    Shortcuts,
    Confirm(Confirm),
    Devices,
    Transfer,
    Phone,
    Tokens,
    NewTag,
    Connect,
    Settings,
    Archive,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum MenuKind {
    Item,
    Background,
    Device,
    Place,
    More,
    Sort,
    Workspace,
    Crumb,
}

#[derive(Clone)]
pub struct Menu {
    pub kind: MenuKind,
    pub pos: Pos2,
    pub target: Option<Location>,
    pub device: Option<String>,
    pub sub: Option<usize>,
    pub age: f32,
}

pub struct Toast {
    pub text: String,
    pub ok: bool,
    pub age: f32,
}

#[derive(Clone, PartialEq)]
pub enum JobState {
    Running,
    Done,
    Failed(String),
}

pub struct JobView {
    pub id: u64,
    pub title: String,
    pub done: u64,
    pub total: u64,
    pub current: String,
    pub state: JobState,
    pub age: f32,
}

#[derive(Default)]
pub struct SearchState {
    pub query: String,
    pub sheet: bool,
    pub req: u64,
    pub hits: Vec<Hit>,
    pub done: bool,
    pub scanned: usize,
    pub ms: i64,
    pub cached: bool,
    pub everywhere: bool,
    pub last_sent: String,
    pub dirty_at: f64,
    pub focus_next: bool,
    pub focused: bool,
}

pub struct Rename {
    pub loc: Location,
    pub text: String,
    pub first: bool,
}

pub struct Drag {
    pub items: Vec<Location>,
    pub label: String,
}

pub struct Tex {
    pub tex: TextureHandle,
    pub orig: Option<(u32, u32)>,
}

pub struct Msg {
    pub user: bool,
    pub text: String,
    pub files: Vec<Location>,
    pub actions: Vec<(String, Act)>,
}

#[derive(Default)]
pub struct AiState {
    pub open: bool,
    pub scroll: f32,
    pub input: String,
    pub msgs: Vec<Msg>,
    pub anim: f32,
    pub pending_org: Option<(Location, Vec<(String, Vec<Location>)>)>,
    pub pending_old: Vec<Location>,
}

#[derive(Default)]
pub struct PaletteState {
    pub open: bool,
    pub cat: usize,
    pub query: String,
    pub sel: usize,
    pub req: u64,
    pub hits: Vec<Hit>,
    pub last_sent: String,
    pub focus: bool,
    pub anim: f32,
}

pub struct TransferState {
    pub srcs: Vec<Location>,
    pub mv: bool,
    pub dest: Location,
    pub folders: Vec<Entry>,
    pub req: u64,
}

pub struct Section {
    pub title: String,
    pub folders: bool,
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, PartialEq)]
pub enum ArchiveMode {
    Browse,
    Create { srcs: Vec<Location> },
}

pub struct ArchiveView {
    pub loc: Location,
    pub entries: Vec<smart_core::archive::ZipEntry>,
    pub password: String,
    pub error: Option<String>,
    pub mode: ArchiveMode,
    pub dest: Location,
    pub name: String,
    pub level: u8,
    pub browsing_dest: bool,
}

#[derive(Clone)]
pub struct TabState {
    pub route: Route,
    pub hist: Vec<Route>,
    pub hist_i: usize,
}

pub struct SmartApp {
    pub tabs: Vec<TabState>,
    pub tab_i: usize,
    pub shell: crate::shell::Shell,
    pub be: Backend,
    pub th: Theme,
    pub icons: Icons,
    pub walls: Vec<(TextureHandle, TextureHandle, [f32; 2])>,
    pub route: Route,
    pub hist: Vec<Route>,
    pub hist_i: usize,
    pub entries: Vec<Entry>,
    pub items: Vec<Entry>,
    pub sections: Vec<Section>,
    pub listing_req: u64,
    pub loading: bool,
    pub error: Option<String>,
    pub summaries: HashMap<Location, (usize, u64)>,
    pub place_counts: HashMap<Location, usize>,
    pub thumbs: HashMap<Location, Tex>,
    pub big_thumbs: HashMap<Location, Tex>,
    pub thumb_failed: HashSet<Location>,
    pub sel: HashSet<Location>,
    /// Selection mode: ticks are shown on every item and a plain click toggles. Plain click otherwise only focuses.
    pub select_mode: bool,
    pub anchor: Option<usize>,
    pub focus: Option<usize>,
    pub search: SearchState,
    pub modal: Modal,
    pub menu: Option<Menu>,
    pub palette: PaletteState,
    pub ai: AiState,
    pub toasts: Vec<Toast>,
    pub jobs: Vec<JobView>,
    pub clipboard: Option<(Vec<Location>, bool)>,
    pub rename: Option<Rename>,
    pub drag: Option<Drag>,
    pub devices: Vec<Device>,
    pub places: Vec<Place>,
    pub transfer: Option<TransferState>,
    pub new_tag_name: String,
    pub connect_url: String,
    pub win_closed: bool,
    pub win_min: bool,
    pub win_zoom: bool,
    pub win_off: Vec2,
    pub sidebar_open: f32,
    pub grid_overlay: bool,
    pub cols: usize,
    pub scroll_to: Option<usize>,
    pub select_after_list: Vec<Location>,
    pub acts: Vec<Act>,
    pub waveform: Option<(Location, Vec<f32>)>,
    pub text_preview: Option<(Location, String)>,
    pub audio_loc: Option<Location>,
    pub pending_copy: Option<String>,
    pub time: f64,
    pub dt: f32,
    pub settings_tab: usize,
    pub archive: Option<ArchiveView>,
    pub copy_dialog: Option<u64>,
    pub custom_wall: Option<(TextureHandle, [f32; 2])>,
    pub sidebar_scroll: f32,
    pub sidebar_h: f32,
    pub last_drop: Option<String>,
    pub copy_dialog_anim: f32,
    pub last_transfer: Option<(Vec<Location>, Location)>,
}

#[derive(Clone)]
pub struct JobSnapshot {
    pub id: u64,
    pub title: String,
    pub done: u64,
    pub total: u64,
    pub current: Option<String>,
    pub age: f32,
    pub state: JobState,
}

impl JobView {
    pub fn clone_job_view(&self) -> JobSnapshot {
        JobSnapshot { id: self.id, title: self.title.clone(), done: self.done, total: self.total, current: if self.current.is_empty() { None } else { Some(self.current.clone()) }, age: self.age, state: self.state.clone() }
    }
}

impl SmartApp {
    pub fn new(cc: &eframe::CreationContext<'_>, be_override: Option<Backend>) -> Self {
        let ctx = &cc.egui_ctx;
        install_fonts(ctx);
        ctx.set_visuals(egui::Visuals::light());
        ctx.global_style_mut(|s| {
            s.visuals.selection.bg_fill = c(47, 111, 235, 0.28);
            s.visuals.text_cursor.stroke = egui::Stroke::new(1.5, hex("#2f6feb"));
        });
        let stored: Option<String> = cc.storage.and_then(|s| s.get_string("smart-library"));
        let be = be_override.unwrap_or_else(|| {
            #[cfg(target_arch = "wasm32")]
            {
                Backend::new_demo(stored.as_deref().map(Library::from_json).unwrap_or_default())
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                let _ = &stored;
                if std::env::args().any(|a| a == "--demo") { Backend::new_demo(Library::default()) } else { Backend::new_native() }
            }
        });
        let walls = WALLPAPERS
            .iter()
            .map(|(n, sharp, blur)| {
                let load = |b: &[u8], tag: &str| {
                    let im = image::load_from_memory(b).unwrap().to_rgba8();
                    let sz = [im.width() as usize, im.height() as usize];
                    (ctx.load_texture(format!("wall-{n}-{tag}"), egui::ColorImage::from_rgba_unmultiplied(sz, im.as_raw()), egui::TextureOptions::LINEAR), [sz[0] as f32, sz[1] as f32])
                };
                let (a, sa) = load(sharp, "s");
                let (b, _) = load(blur, "b");
                (a, b, sa)
            })
            .collect();
        let st = be.library.read().unwrap().settings.clone();
        let home = be.home();
        let places = be.places();
        let devices = be.devices();
        let start = places.iter().find(|p| p.id == "pictures").map(|p| p.loc.clone()).filter(|l| be.stat(l).is_ok()).unwrap_or(home);
        let mut app = Self {
            tabs: vec![],
            tab_i: 0,
            shell: crate::shell::Shell::new(be.home()),
            th: if st.dark { Theme::dark() } else { Theme::light() },
            icons: Icons::new(ctx),
            walls,
            route: Route::Loc(start.clone()),
            hist: vec![Route::Loc(start)],
            hist_i: 0,
            entries: vec![],
            items: vec![],
            sections: vec![],
            listing_req: 0,
            loading: true,
            error: None,
            summaries: HashMap::new(),
            place_counts: HashMap::new(),
            thumbs: HashMap::new(),
            big_thumbs: HashMap::new(),
            thumb_failed: HashSet::new(),
            sel: HashSet::new(),
            select_mode: false,
            anchor: None,
            focus: None,
            search: SearchState::default(),
            modal: Modal::None,
            menu: None,
            palette: PaletteState::default(),
            ai: AiState::default(),
            toasts: vec![],
            jobs: vec![],
            clipboard: None,
            rename: None,
            drag: None,
            devices,
            places,
            transfer: None,
            new_tag_name: String::new(),
            connect_url: String::new(),
            win_closed: false,
            win_min: false,
            win_zoom: false,
            win_off: Vec2::ZERO,
            sidebar_open: if st.sidebar_collapsed { 0.0 } else { 1.0 },
            grid_overlay: false,
            cols: 4,
            scroll_to: None,
            select_after_list: vec![],
            acts: vec![],
            waveform: None,
            text_preview: None,
            audio_loc: None,
            pending_copy: None,
            time: 0.0,
            dt: 0.016,
            settings_tab: 0,
            archive: None,
            copy_dialog: None,
            custom_wall: None,
            sidebar_scroll: 0.0,
            sidebar_h: 0.0,
            last_drop: None,
            copy_dialog_anim: 0.0,
            last_transfer: None,
            be,
        };
        app.tabs = vec![TabState { route: app.route.clone(), hist: app.hist.clone(), hist_i: 0 }];
        for p in app.places.clone() {
            app.be.summarize(&p.loc);
        }
        app.load();
        app
    }

    // ───────────── settings helpers ─────────────
    pub fn set<R>(&self, f: impl FnOnce(&mut Settings) -> R) -> R {
        f(&mut self.be.library.write().unwrap().settings)
    }
    pub fn st(&self) -> Settings {
        self.be.library.read().unwrap().settings.clone()
    }
    pub fn lib<R>(&self, f: impl FnOnce(&Library) -> R) -> R {
        f(&self.be.library.read().unwrap())
    }
    pub fn current_loc(&self) -> Option<Location> {
        match &self.route {
            Route::Loc(l) => Some(l.clone()),
            _ => None,
        }
    }
    pub fn target_dir(&self) -> Location {
        self.current_loc().unwrap_or_else(|| self.be.home())
    }

    // ───────────── navigation ─────────────
    pub fn navigate(&mut self, r: Route) {
        if self.hist.get(self.hist_i) == Some(&r) && !self.entries.is_empty() {
            return;
        }
        self.hist.truncate(self.hist_i + 1);
        self.hist.push(r.clone());
        self.hist_i = self.hist.len() - 1;
        self.go(r);
    }
    fn go(&mut self, r: Route) {
        self.route = r;
        self.sel.clear();
        self.select_mode = false;
        self.anchor = None;
        self.focus = None;
        self.rename = None;
        self.clear_search();
        self.scroll_to = Some(0);
        self.load();
    }
    pub fn can_back(&self) -> bool {
        self.hist_i > 0
    }
    pub fn can_fwd(&self) -> bool {
        self.hist_i + 1 < self.hist.len()
    }
    pub fn load(&mut self) {
        self.loading = true;
        self.error = None;
        match self.route.clone() {
            Route::Loc(l) => {
                self.listing_req = self.be.list(&l);
                self.be.watch(&l);
            }
            Route::Recents => {
                let recents = self.lib(|l| l.recents.clone());
                let items: Vec<Entry> = recents.iter().filter_map(|r| self.be.stat(&r.loc).ok()).collect();
                self.set_entries(items);
            }
            Route::Tag(id) => {
                let locs = self.lib(|l| l.tagged(&id));
                let items: Vec<Entry> = locs.iter().filter_map(|l| self.be.stat(l).ok()).collect();
                self.set_entries(items);
            }
        }
    }
    pub fn title(&self) -> String {
        match &self.route {
            Route::Recents => "Recents".into(),
            Route::Tag(id) => self.lib(|l| l.tags.iter().find(|t| &t.id == id).map(|t| t.name.clone())).unwrap_or_else(|| id.clone()),
            Route::Loc(l) => self.loc_title(l),
        }
    }
    pub fn loc_title(&self, l: &Location) -> String {
        if let Some(p) = self.places.iter().find(|p| &p.loc == l) {
            return p.title.clone();
        }
        if l.is_root() {
            if let Some(d) = self.devices.iter().find(|d| d.loc.as_ref() == Some(l)) {
                return d.name.clone();
            }
            return if l.fs == "file" { "Computer".into() } else { l.fs.clone() };
        }
        l.name()
    }
    pub fn crumbs(&self) -> Vec<(String, Route)> {
        match &self.route {
            Route::Recents => vec![("Smart".into(), Route::Loc(self.be.home())), ("Recents".into(), Route::Recents)],
            Route::Tag(_) => vec![("Tags".into(), Route::Recents), (self.title(), self.route.clone())],
            Route::Loc(l) => {
                let home = self.be.home();
                let mut v: Vec<(String, Route)> = vec![];
                if l.starts_with(&home) {
                    v.push(("Smart".into(), Route::Loc(home.clone())));
                    for a in l.ancestors().iter().filter(|a| a.starts_with(&home) && **a != home) {
                        v.push((a.name(), Route::Loc(a.clone())));
                    }
                    if *l == home {
                        v.push(("Home".into(), Route::Loc(home)));
                    }
                } else {
                    for a in l.ancestors() {
                        v.push((self.loc_title(&a), Route::Loc(a)));
                    }
                }
                v
            }
        }
    }

    // ───────────── listing / view ─────────────
    pub fn set_entries(&mut self, e: Vec<Entry>) {
        self.entries = e;
        self.loading = false;
        self.rebuild_view();
        let want: Vec<Location> = std::mem::take(&mut self.select_after_list);
        if !want.is_empty() {
            self.sel = want.iter().cloned().collect();
            if let Some(i) = self.items.iter().position(|e| want.contains(&e.loc)) {
                self.focus = Some(i);
                self.anchor = Some(i);
                self.scroll_to = Some(i);
            }
        }
        let alive: HashSet<Location> = self.items.iter().map(|e| e.loc.clone()).collect();
        self.sel.retain(|l| alive.contains(l));
        let todo: Vec<Location> = self.items.iter().filter(|e| e.is_dir && !self.summaries.contains_key(&e.loc)).take(80).map(|e| e.loc.clone()).collect();
        for l in todo {
            self.be.summarize(&l);
        }
    }
    fn group_idx(e: &Entry, by_kind: bool) -> usize {
        if e.is_dir {
            return 0;
        }
        if !by_kind {
            return 1;
        }
        match e.kind {
            Kind::Image | Kind::Vector => 1,
            Kind::Audio => 2,
            Kind::Video => 3,
            Kind::Pdf | Kind::Document | Kind::Text | Kind::Code => 4,
            _ => 5,
        }
    }
    pub fn rebuild_view(&mut self) {
        let s = self.st();
        let mut v: Vec<Entry> = self.entries.iter().filter(|e| s.show_hidden || !e.hidden).cloned().collect();
        sort_entries(&mut v, s.sort, s.ascending, s.folders_first);
        let by_kind = s.sort == SortKey::Name && s.folders_first;
        if s.folders_first {
            v.sort_by_key(|e| Self::group_idx(e, by_kind)); // stable: name order kept inside groups
        }
        let titles = ["Folders", "Images", "Audio", "Video", "Documents", "Files"];
        let mut secs: Vec<Section> = vec![];
        for (i, e) in v.iter().enumerate() {
            let g = Self::group_idx(e, by_kind);
            let t = if !s.folders_first { "Items".to_string() } else if by_kind { titles[g].to_string() } else if g == 0 { "Folders".into() } else { "Files".into() };
            if secs.last().map(|x| x.title != t).unwrap_or(true) {
                if let Some(l) = secs.last_mut() {
                    l.end = i;
                }
                secs.push(Section { title: t, folders: g == 0 && s.folders_first, start: i, end: v.len() });
            }
        }
        self.items = v;
        self.sections = secs;
    }
    pub fn selected(&self) -> Vec<Entry> {
        let mut v: Vec<Entry> = self.items.iter().filter(|e| self.sel.contains(&e.loc)).cloned().collect();
        if v.is_empty() {
            v = self.search.hits.iter().filter(|h| self.sel.contains(&h.entry.loc)).map(|h| h.entry.clone()).collect();
        }
        v
    }
    pub fn total_bytes(&self) -> u64 {
        self.items.iter().map(|e| if e.is_dir { self.summaries.get(&e.loc).map(|s| s.1).unwrap_or(0) } else { e.size }).sum()
    }
    pub fn tag_colors(&self, l: &Location) -> Vec<egui::Color32> {
        self.lib(|lib| lib.tags_of(l).iter().map(|t| hex(&t.color)).collect())
    }

    // ───────────── search ─────────────
    pub fn clear_search(&mut self) {
        self.be.cancel_search();
        self.search.query.clear();
        self.search.hits.clear();
        self.search.last_sent.clear();
        self.search.done = false;
    }
    pub fn search_active(&self) -> bool {
        !self.search.query.trim().is_empty()
    }
    pub fn run_search(&mut self) {
        let q = self.search.query.trim().to_string();
        if q.is_empty() {
            self.clear_search();
            return;
        }
        let scope = if self.search.everywhere || self.current_loc().is_none() { self.be.home() } else { self.target_dir() };
        let contents = self.st().search_contents;
        self.search.hits.clear();
        self.search.done = false;
        self.search.last_sent = q.clone();
        self.search.req = self.be.search(&q, &scope, contents);
        self.sel.clear();
        self.focus = None;
    }

    // ───────────── event pump ─────────────
    pub fn pump(&mut self, ctx: &Context) {
        let mut repaint = false;
        for ev in self.be.poll() {
            repaint = true;
            match ev {
                Event::Listed { req, loc, result } => {
                    if req == self.listing_req {
                        match result {
                            Ok(v) => self.set_entries(v),
                            Err(e) => {
                                self.error = Some(e);
                                self.loading = false;
                                self.entries.clear();
                                self.rebuild_view();
                            }
                        }
                    } else if self.transfer.as_ref().is_some_and(|t| t.req == req) {
                        if let (Some(t), Ok(mut v)) = (self.transfer.as_mut(), result) {
                            v.retain(|e| e.is_dir && !e.hidden);
                            sort_entries(&mut v, SortKey::Name, true, true);
                            t.folders = v;
                            t.dest = loc;
                        }
                    }
                }
                Event::SearchHits { req, hits } => {
                    if req == self.search.req && !self.palette.open {
                        self.search.hits.extend(hits);
                        self.search.hits.sort_by(|a, b| b.score.cmp(&a.score));
                        self.search.hits.truncate(400);
                    } else if req == self.palette.req {
                        self.palette.hits.extend(hits);
                        self.palette.hits.sort_by(|a, b| b.score.cmp(&a.score));
                        self.palette.hits.truncate(8);
                    }
                }
                Event::SearchDone { req, scanned, elapsed_ms, cached } => {
                    if req == self.search.req || req == self.palette.req {
                        self.search.done = true;
                        self.search.scanned = scanned;
                        self.search.ms = elapsed_ms;
                        self.search.cached = cached;
                    }
                }
                Event::Job(j) => self.on_job(j),
                Event::Thumb(t) => {
                    if t.size[0] == 0 {
                        self.thumb_failed.insert(t.loc);
                    } else {
                        let img = egui::ColorImage::from_rgba_unmultiplied(t.size, &t.rgba);
                        let tex = ctx.load_texture(format!("th-{}-{}", t.loc.path, t.edge), img, egui::TextureOptions::LINEAR);
                        let e = Tex { tex, orig: t.original };
                        if t.edge > 600 {
                            self.big_thumbs.insert(t.loc, e);
                        } else {
                            self.thumbs.insert(t.loc, e);
                        }
                    }
                }
                Event::Devices(d) => self.devices = d,
                Event::Changed(l) => {
                    if self.current_loc().as_ref() == Some(&l) && !self.loading {
                        self.listing_req = self.be.list(&l);
                    }
                }
                Event::Summary { loc, items, bytes } => {
                    if self.places.iter().any(|p| p.loc == loc) {
                        self.place_counts.insert(loc.clone(), items);
                    }
                    self.summaries.insert(loc, (items, bytes));
                }
                Event::DeviceOp { op, result, id } => match result {
                    Ok(loc) => {
                        let name = self.devices.iter().find(|d| d.id == id).map(|d| d.name.clone()).unwrap_or(id);
                        match op {
                            "mount" => {
                                self.toast(format!("Mounted {}", name), true);
                                if let Some(l) = loc {
                                    self.navigate(Route::Loc(l));
                                    self.modal = Modal::None;
                                }
                            }
                            "eject" => self.toast(format!("{} can now be safely removed", name), true),
                            _ => self.toast(format!("Unmounted {}", name), true),
                        }
                        if op != "mount" {
                            if let Route::Loc(l) = self.route.clone() {
                                if l.fs != "file" && l.fs != "mem" {
                                    let h = self.be.home();
                                    self.navigate(Route::Loc(h));
                                }
                            }
                        }
                        self.be.refresh_devices();
                    }
                    Err(e) => self.toast(e, false),
                },
                Event::Message { text, ok } => self.toast(text, ok),
                Event::ArchiveDone { loc, ok, detail } => {
                    if ok {
                        self.toast(format!("{} \u{b7} {}", loc.name(), detail), true);
                    }
                    if let Route::Loc(cur) = self.route.clone() {
                        if cur == loc.parent().unwrap_or(cur.clone()) {
                            self.listing_req = self.be.list(&cur);
                        }
                    }
                }
            }
        }
        if repaint || self.be.busy() {
            ctx.request_repaint();
        }
    }

    fn on_job(&mut self, j: JobEvent) {
        match j {
            JobEvent::Started { id, title } => self.jobs.push(JobView { id, title, done: 0, total: 1, current: String::new(), state: JobState::Running, age: 0.0 }),
            JobEvent::Progress { id, done, total, current } => {
                if let Some(v) = self.jobs.iter_mut().find(|v| v.id == id) {
                    v.done = done;
                    v.total = total.max(1);
                    v.current = current;
                }
            }
            JobEvent::Finished { id, created, touched } => {
                if let Some(v) = self.jobs.iter_mut().find(|v| v.id == id) {
                    v.state = JobState::Done;
                    v.done = v.total;
                    v.age = 0.0;
                    if self.copy_dialog == Some(id) {
                        self.copy_dialog = None;
                    }
                }
                if let Some(cur) = self.current_loc() {
                    if touched.contains(&cur) {
                        self.select_after_list = created;
                        self.listing_req = self.be.list(&cur);
                    }
                }
                self.be.refresh_devices();
            }
            JobEvent::Failed { id, error } => {
                if let Some(v) = self.jobs.iter_mut().find(|v| v.id == id) {
                    v.state = JobState::Failed(error.clone());
                    v.age = 0.0;
                }
                if error != "cancelled" {
                    self.toast(error, false);
                }
                if let Some(cur) = self.current_loc() {
                    self.listing_req = self.be.list(&cur);
                }
            }
        }
    }

    pub fn toast(&mut self, text: impl Into<String>, ok: bool) {
        self.toasts.push(Toast { text: text.into(), ok, age: 0.0 });
        if self.toasts.len() > 4 {
            self.toasts.remove(0);
        }
    }

    pub fn sel_locs(&self) -> Vec<Location> {
        self.selected().iter().map(|e| e.loc.clone()).collect()
    }

    // ───────────── actions ─────────────
    pub fn act(&mut self, a: Act) {
        match a {
            Act::Nav(r) => {
                self.modal = Modal::None;
                self.navigate(r)
            }
            Act::Back => {
                if self.can_back() {
                    self.hist_i -= 1;
                    let r = self.hist[self.hist_i].clone();
                    self.go(r);
                }
            }
            Act::Forward => {
                if self.can_fwd() {
                    self.hist_i += 1;
                    let r = self.hist[self.hist_i].clone();
                    self.go(r);
                }
            }
            Act::Up => {
                if let Some(cur) = self.current_loc() {
                    if let Some(p) = cur.parent() {
                        self.navigate(Route::Loc(p));
                        self.select_after_list = vec![cur];
                    }
                }
            }
            Act::Refresh => {
                self.thumbs.clear();
                self.thumb_failed.clear();
                for e in &self.entries {
                    self.be.forget_thumb(&e.loc);
                }
                self.load();
                self.be.refresh_devices();
            }
            Act::Open(l) => self.open_loc(l),
            Act::OpenSel => {
                for e in self.selected().into_iter().take(8) {
                    self.open_loc(e.loc);
                }
            }
            Act::OpenWith(l, app) => {
                self.be.open_with(&l, &app);
                self.modal = Modal::None;
            }
            Act::SetDefault(l, app) => match self.be.set_default_app(&l, &app) {
                Ok(()) => self.toast(format!("{} is now the default for .{}", app.name, ext_of(&l.name())), true),
                Err(e) => self.toast(e, false),
            },
            Act::Preview(l) => {
                self.menu = None;
                self.begin_preview(l);
            }
            Act::Info => {
                let s = self.sel_locs();
                let s = if s.is_empty() { vec![self.target_dir()] } else { s };
                for l in &s {
                    self.be.summarize(l);
                }
                self.menu = None;
                self.modal = Modal::Info(s);
            }
            Act::NewFolder | Act::NewFile => {
                let d = self.target_dir();
                let r = if matches!(a, Act::NewFolder) { self.be.create_folder(&d, "untitled folder") } else { self.be.create_file(&d, "untitled.txt") };
                match r {
                    Ok(l) => {
                        self.select_after_list = vec![l.clone()];
                        self.rename = Some(Rename { text: l.name(), loc: l, first: true });
                        self.load();
                    }
                    Err(e) => self.toast(e, false),
                }
            }
            Act::Rename(l) => {
                self.rename = Some(Rename { text: l.name(), loc: l.clone(), first: true });
                self.sel = [l].into_iter().collect();
            }
            Act::Trash | Act::DeleteForever => {
                let forever = matches!(a, Act::DeleteForever);
                let s = self.sel_locs();
                if s.is_empty() {
                    return;
                }
                let can_trash = self.be.registry.get(&s[0].fs).map(|v| v.supports_trash()).unwrap_or(false);
                let confirm = self.st().confirm_delete;
                if (forever || !can_trash) && confirm {
                    let n = if s.len() == 1 { format!("\u{201c}{}\u{201d}", s[0].name()) } else { format!("{} items", s.len()) };
                    self.modal = Modal::Confirm(Confirm { title: format!("Delete {} permanently?", n), body: "This can\u{2019}t be undone \u{2014} it won\u{2019}t go to the Trash.".into(), ok: "Delete".into(), danger: true, act: Box::new(Act::Job(JobKind::Delete { srcs: s })) });
                } else {
                    self.submit_job(JobKind::Trash { srcs: s });
                    self.sel.clear();
                }
            }
            Act::Job(k) => {
                self.modal = Modal::None;
                self.archive = None;
                self.submit_job(k);
                self.sel.clear();
            }
            Act::Copy | Act::Cut => {
                let s = self.sel_locs();
                if !s.is_empty() {
                    let n = s.len();
                    let cut = matches!(a, Act::Cut);
                    self.clipboard = Some((s, cut));
                    self.toast(format!("{} {} item{}", if cut { "Cut" } else { "Copied" }, n, if n == 1 { "" } else { "s" }), true);
                }
            }
            Act::Paste => {
                if let (Some((srcs, cut)), Some(dest)) = (self.clipboard.clone(), self.current_loc()) {
                    self.submit_job(if cut { JobKind::Move { srcs, dest } } else { JobKind::Copy { srcs, dest } });
                    if cut {
                        self.clipboard = None;
                    }
                }
            }
            Act::Duplicate => {
                let s = self.sel_locs();
                if !s.is_empty() {
                    self.submit_job(JobKind::Duplicate { srcs: s });
                }
            }
            Act::SelectAll => {
                self.sel = self.items.iter().map(|e| e.loc.clone()).collect();
                self.select_mode = true;
            }
            Act::ToggleSelectMode => {
                self.select_mode = !self.select_mode;
                if !self.select_mode {
                    self.sel.clear();
                }
            }
            Act::ToggleTag(l, id) => {
                let targets = if self.sel.contains(&l) { self.sel_locs() } else { vec![l.clone()] };
                let all_have = targets.iter().all(|t| self.lib(|lib| lib.has_tag(t, &id)));
                for t in targets {
                    if all_have || !self.lib(|lib| lib.has_tag(&t, &id)) {
                        self.be.library.write().unwrap().toggle_tag(&t, &id);
                    }
                }
                if let Route::Tag(_) = self.route {
                    self.load();
                }
            }
            Act::CopyTo(mv) => {
                let s = self.sel_locs();
                if s.is_empty() {
                    return;
                }
                let dest = self.target_dir();
                let req = self.be.list(&dest);
                self.transfer = Some(TransferState { srcs: s, mv, dest, folders: vec![], req });
                self.menu = None;
                self.modal = Modal::Transfer;
            }
            Act::CopyPath => {
                let s = self.selected();
                let txt = if s.is_empty() { self.target_dir().path } else { s.iter().map(|e| e.loc.path.clone()).collect::<Vec<_>>().join("\n") };
                self.pending_copy = Some(txt);
                self.toast("Path copied", true);
            }
            Act::Reveal => {
                if let Some(e) = self.selected().first() {
                    self.be.reveal_in_system(&e.loc);
                }
            }
            Act::ToggleSidebar => self.set(|s| s.sidebar_collapsed = !s.sidebar_collapsed),
            Act::SetGrid(g) => self.set(|s| s.grid = g),
            Act::SetSort(k) => {
                self.set(|s| {
                    if s.sort == k {
                        s.ascending = !s.ascending
                    } else {
                        s.sort = k;
                        s.ascending = true
                    }
                });
                self.rebuild_view();
            }
            Act::ToggleAsc => {
                self.set(|s| s.ascending = !s.ascending);
                self.rebuild_view();
            }
            Act::ToggleTheme => {
                let d = self.set(|s| {
                    s.dark = !s.dark;
                    s.dark
                });
                self.th = if d { Theme::dark() } else { Theme::light() };
            }
            Act::ToggleHidden => {
                self.set(|s| s.show_hidden = !s.show_hidden);
                self.rebuild_view();
            }
            Act::ToggleFoldersFirst => {
                self.set(|s| s.folders_first = !s.folders_first);
                self.rebuild_view();
            }
            Act::Modal(m) => {
                self.menu = None;
                if let Modal::Devices | Modal::Phone = m {
                    self.be.refresh_devices();
                }
                self.palette.open = false;
                self.modal = m;
            }
            Act::CloseModal => {
                self.modal = Modal::None;
                self.transfer = None;
            }
            Act::Palette => {
                let open = !self.palette.open;
                self.palette = PaletteState { open, focus: true, ..Default::default() };
                self.menu = None;
            }
            Act::Ask => {
                self.ai.open = !self.ai.open;
            }
            Act::FocusSearch => self.search.focus_next = true,
            Act::ClearSearch => self.clear_search(),
            Act::Mount(id) => self.be.mount(&id),
            Act::Unmount(id) => self.be.unmount(&id),
            Act::Eject(id) => self.be.eject(&id),
            Act::SetWallpaper(i) => self.set(|s| s.wallpaper = i),
            Act::Confirm(c) => self.modal = Modal::Confirm(c),
            Act::TogglePin(l) => self.be.library.write().unwrap().toggle_pin(&l),
            Act::Play(l) => self.play(l),
            Act::Toast(t) => self.toast(t, true),
            Act::Move(srcs, dest) => {
                self.be.submit(JobKind::Move { srcs, dest });
            }
            Act::Organise => self.ai_organise(),
            Act::ApplyOrganise => self.ai_apply_organise(),
            Act::TagOld => {
                let old = std::mem::take(&mut self.ai.pending_old);
                let n = old.len();
                for l in old {
                    if !self.lib(|lib| lib.has_tag(&l, "archive")) {
                        self.be.library.write().unwrap().toggle_tag(&l, "archive");
                    }
                }
                self.ai_say(format!("Tagged {} files as Archive.", n), vec![("Show Archive".into(), Act::Nav(Route::Tag("archive".into())))], vec![]);
            }
            Act::FindDuplicates => self.ai_duplicates(),
            Act::FindLarge => {
                self.ai_user("Find large files");
                self.ai_say("Looking for anything over 50 MB across your home folder.", vec![("Show results".into(), Act::RunSearch("size:>50mb".into()))], vec![]);
            }
            Act::FindOld => self.ai_old(),
            Act::RunSearch(q) => {
                self.ai.open = false;
                self.palette.open = false;
                self.modal = Modal::None;
                self.search.query = q;
                self.search.everywhere = true;
                self.run_search();
            }
            Act::WinClose => {
                if IS_WEB {
                    self.win_closed = true;
                } else {
                    self.be.save_library();
                    std::process::exit(0);
                }
            }
            Act::WinMin => self.win_min = true,
            Act::WinZoom => self.win_zoom = !self.win_zoom,
            Act::WinOpen => {
                self.win_closed = false;
                self.win_min = false;
            }
            Act::ToggleGridOverlay => self.grid_overlay = !self.grid_overlay,
            Act::ToggleContentSearch => {
                self.set(|s| s.search_contents = !s.search_contents);
                self.run_search();
            }
            Act::NewTag => {
                let n = self.new_tag_name.trim().to_string();
                if !n.is_empty() {
                    let colors = ["#e0457b", "#7a5af8", "#0ea5a4", "#d9534f", "#5b8def"];
                    let i = self.lib(|l| l.tags.len()) % colors.len();
                    self.be.library.write().unwrap().add_tag(&n, colors[i]);
                }
                self.new_tag_name.clear();
                self.modal = Modal::None;
            }
            Act::CopyTokens => {
                self.pending_copy = Some(include_str!("../../../assets/tokens.toml").to_string());
                self.toast("smart.toml copied to the clipboard", true);
            }
            Act::Connect(url) => {
                self.be.connect_server(&url);
                self.modal = Modal::None;
            }
            Act::ArchiveConfirm(l) => {
                self.archive_open(l);
            }
            Act::Settings => {
                self.menu = None;
                self.ai.open = false;
                self.modal = Modal::Settings;
            }
            Act::SettingsTab(i) => self.settings_tab = i,
            Act::ToggleExtensions => {
                self.set(|s| s.show_extensions = !s.show_extensions);
            }
            Act::ToggleConfirmDelete => {
                self.set(|s| s.confirm_delete = !s.confirm_delete);
            }
            Act::ToggleReduceMotion => {
                self.set(|s| s.reduce_motion = !s.reduce_motion);
            }
            Act::ToggleAutoplay => {
                self.set(|s| s.autoplay = !s.autoplay);
            }
            Act::SetAnimSecs(v) => {
                self.set(|s| s.copy_animation_secs = v);
            }
            Act::SetGridZoom(v) => {
                self.set(|s| s.thumb_scale = v);
            }
            Act::ResetLibrary => {
                let mut lib = Library::default();
                lib.settings = self.st();
                *self.be.library.write().unwrap() = lib;
                self.toast("Preferences reset \u{2014} theme and layout kept", true);
            }
            Act::CopyConfig => {
                self.pending_copy = Some(self.be.library.read().unwrap().to_json());
                self.toast("Library JSON copied to the clipboard", true);
            }
            Act::WallpaperFrom(l) => self.apply_wallpaper_image(l),
            Act::ExtractHere(l) => {
                let Some(parent) = l.parent() else { return };
                let stem = l.name().trim_end_matches(".zip").to_string();
                let folder = self.unique_archive_name(&parent, &stem);
                let folder = folder.trim_end_matches(".zip").to_string();
                if self.be.create_folder(&parent, &folder).is_ok() {
                    let dest = parent.join(&folder);
                    let pw = self.archive.as_ref().map(|a| a.password.clone()).filter(|p| !p.is_empty());
                    self.be.archive_extract(&l, &dest, pw);
                    self.sel.clear();
                }
            }
            Act::ArchiveBrowse(l) => {
                self.archive_open(l);
                self.modal = Modal::Archive;
            }
            Act::ArchiveRetry => {
                if let Some(a) = self.archive.as_mut() {
                    let pw = a.password.clone();
                    let loc = a.loc.clone();
                    match self.be.archive_entries(&loc, if pw.is_empty() { None } else { Some(&pw) }) {
                        Ok(e) => {
                            a.entries = e;
                            a.error = None;
                        }
                        Err(e) => a.error = Some(e),
                    }
                }
            }
            Act::ArchiveDoExtract => {
                let Some(a) = self.archive.as_ref() else { return };
                let (loc, dest, pw) = (a.loc.clone(), a.dest.clone(), a.password.clone());
                self.be.archive_extract(&loc, &dest, if pw.is_empty() { None } else { Some(pw) });
                self.archive = None;
                self.modal = Modal::None;
                self.toast(format!("Extracting into {}\u{2026}", dest.name()), true);
                self.sel.clear();
            }
            Act::ArchiveCompress => {
                let srcs = self.sel_locs();
                if !srcs.is_empty() {
                    self.archive_compress(srcs);
                }
            }
            Act::ArchiveCreate => {
                let Some(a) = self.archive.as_ref() else { return };
                let ArchiveMode::Create { srcs } = a.mode.clone() else { return };
                let dest = a.dest.join(a.name.trim());
                let name = a.name.trim().to_string();
                let pw = if a.password.is_empty() { None } else { Some(a.password.clone()) };
                let level = a.level;
                if !name.ends_with(".zip") {
                    if let Some(a) = self.archive.as_mut() {
                        a.error = Some("Give the archive a .zip name".into());
                    }
                    return;
                }
                self.archive = None;
                self.modal = Modal::None;
                self.select_after_list = vec![dest.clone()];
                self.be.archive_compress(srcs, dest, pw, level);
            }
            Act::ArchiveClose => {
                self.archive = None;
                self.modal = Modal::None;
            }
            Act::ArchiveDest(d) => {
                if let Some(a) = self.archive.as_mut() {
                    let browse = a.mode == ArchiveMode::Browse;
                    a.dest = d.clone();
                    if browse {
                        a.name = a.loc.name().trim_end_matches(".zip").to_string();
                    }
                    a.browsing_dest = false;
                }
            }
            Act::ArchiveToggleDest => {
                if let Some(a) = self.archive.as_mut() {
                    a.browsing_dest = !a.browsing_dest;
                }
            }
            Act::SetArchiveName(n) => {
                if let Some(a) = self.archive.as_mut() {
                    a.name = n;
                }
            }
            Act::SetArchivePassword(p) => {
                if let Some(a) = self.archive.as_mut() {
                    a.password = p;
                }
            }
            Act::ArchiveLevel(l) => {
                if let Some(a) = self.archive.as_mut() {
                    a.level = l;
                }
            }
            Act::OpenTerminal => {
                let l = self.current_loc().unwrap_or_else(|| self.be.home());
                if IS_WEB || self.be.demo {
                    self.act(Act::OpenInTerminal(l));
                } else {
                    self.be.open_terminal(&l);
                }
            }
            Act::Seq(v) => {
                for x in v {
                    self.act(x);
                }
            }
            Act::OpenApp(id) => self.open_app(id),
            Act::App(cmd) => self.app_cmd(&cmd),
            Act::NewTab(r) => self.new_tab(r),
            Act::CloseTab(i) => self.close_tab(i),
            Act::SwitchTab(i) => self.switch_tab(i),
            Act::CopyText(t) => {
                self.pending_copy = Some(t);
                self.toast("Copied to clipboard", true);
            }
            Act::OpenInTerminal(l) => {
                self.open_app(crate::shell::AppId::Terminal);
                self.shell.term.cd_to(l);
            }
            Act::OpenInPhotos(l) => {
                self.open_app(crate::shell::AppId::Photos);
                self.photos_show(l);
            }
            Act::OpenNewWindow => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if let Ok(exe) = std::env::current_exe() {
                        let _ = std::process::Command::new(exe).spawn();
                    }
                }
                #[cfg(target_arch = "wasm32")]
                {
                    if let Some(w) = web_sys::window() {
                        let _ = w.open_with_url_and_target("", "_blank");
                    }
                }
            }
            Act::ClearSelection => {
                self.select_mode = false;
                self.sel.clear();
                self.focus = None;
            }
            Act::TogglePlayPause(l) => self.toggle_play(l),
            Act::CopyDialogClose => {
                if let Some(id) = self.copy_dialog.take() {
                    self.be.cancel_job(id);
                }
            }
            Act::SearchSheet(v) => {
                self.search.sheet = v;
                if v {
                    self.search.focus_next = true;
                }
            }
            Act::ToggleEverywhere => {
                self.search.everywhere = !self.search.everywhere;
                self.run_search();
            }
            Act::ToggleContents => {
                self.set(|s| s.search_contents = !s.search_contents);
                self.run_search();
            }
        }
    }

    pub fn sel_entries(&self) -> Vec<Entry> {
        self.selected()
    }
    pub fn sel_all_images(&self) -> bool {
        let v = self.selected();
        !v.is_empty() && v.iter().all(|e| matches!(e.kind, Kind::Image | Kind::Vector) && !e.is_dir)
    }
    pub fn sel_all_audio(&self) -> bool {
        let v = self.selected();
        !v.is_empty() && v.iter().all(|e| e.kind == Kind::Audio)
    }
    pub fn sel_has_archive(&self) -> bool {
        self.selected().iter().any(|e| e.kind == Kind::Archive)
    }
    pub fn sel_has_paced(&self) -> bool {
        self.selected().iter().any(|e| e.paced)
    }
    pub fn sel_has_dir(&self) -> bool {
        self.selected().iter().any(|e| e.is_dir)
    }
    pub fn sel_total(&self) -> u64 {
        self.selected().iter().map(|e| e.size).sum()
    }

    /// Copy/move facade: the demo 5 GB files get the paced transfer + progress dialog.
    pub fn submit_job(&mut self, kind: JobKind) {
        let animate = matches!(kind, JobKind::Copy { .. } | JobKind::Move { .. }) && self.be.has_paced_sources(&kind);
        if animate {
            let secs = self.st().copy_animation_secs;
            if let JobKind::Copy { srcs, dest } | JobKind::Move { srcs, dest } = &kind {
                self.last_transfer = Some((srcs.clone(), dest.clone()));
            }
            self.copy_dialog_anim = 0.0;
            self.copy_dialog = Some(self.be.submit_paced(kind, secs));
        } else {
            self.be.submit(kind);
        }
    }

    pub fn toggle_play(&mut self, l: Location) {
        let same = self.audio_loc.as_ref() == Some(&l);
        if same && self.be.audio.is_loaded() {
            if self.be.audio.is_playing() {
                self.be.audio.pause();
            } else {
                self.be.audio.resume();
            }
        } else {
            self.play(l);
        }
    }

    pub fn archive_open(&mut self, loc: Location) {
        let parent = loc.parent().unwrap_or_else(|| self.target_dir());
        let mut v = ArchiveView {
            loc: loc.clone(),
            entries: vec![],
            password: String::new(),
            error: None,
            mode: ArchiveMode::Browse,
            dest: parent,
            name: loc.name(),
            level: 6,
            browsing_dest: false,
        };
        match self.be.archive_entries(&loc, None) {
            Ok(e) => v.entries = e,
            Err(err) => v.error = Some(err),
        }
        self.archive = Some(v);
        self.modal = Modal::Archive;
    }

    pub fn archive_compress(&mut self, srcs: Vec<Location>) {
        let dir = self.target_dir();
        let base = if srcs.len() == 1 {
            srcs[0].name().rsplit_once('.').map(|(a, _)| a.to_string()).unwrap_or_else(|| srcs[0].name())
        } else {
            self.title()
        };
        let name = self.unique_archive_name(&dir, &base);
        self.archive = Some(ArchiveView { loc: dir.join(&name), entries: vec![], password: String::new(), error: None, mode: ArchiveMode::Create { srcs }, dest: dir, name, level: 6, browsing_dest: false });
        self.modal = Modal::Archive;
    }

    pub fn unique_archive_name(&self, dir: &Location, base: &str) -> String {
        let base = base.trim();
        let base = if base.is_empty() { "Archive" } else { base };
        let v = self.be.registry.get(&dir.fs);
        for i in 0..500 {
            let n = if i == 0 { format!("{}.zip", base) } else { format!("{} {}.zip", base, i + 1) };
            if v.as_ref().map(|v| !v.exists(&dir.join(&n).path)).unwrap_or(true) {
                return n;
            }
        }
        format!("Archive {}.zip", crate::model::now_secs())
    }

    pub fn apply_wallpaper_image(&mut self, loc: Location) {
        let bytes = match self.be.read_bytes(&loc, 48 << 20) {
            Ok(b) => b,
            Err(e) => return self.toast(e, false),
        };
        match image::load_from_memory(&bytes) {
            Ok(img) => {
                let thumb = img.thumbnail(2400, 2400).to_rgba8();
                let sz = [thumb.width() as f32, thumb.height() as f32];
                let ci = egui::ColorImage::from_rgba_unmultiplied([thumb.width() as usize, thumb.height() as usize], thumb.as_raw());
                let tex = self.icons.ctx.load_texture(format!("custom-wall-{}", self.time), ci, egui::TextureOptions::LINEAR);
                self.custom_wall = Some((tex, sz));
                self.set(|s| s.wallpaper = 4);
                self.modal = Modal::None;
                self.toast(format!("{} is now your desktop wallpaper", loc.name()), true);
            }
            Err(_) => self.toast("That image could not be decoded", false),
        }
    }

    pub fn open_loc(&mut self, l: Location) {
        let Ok(e) = self.be.stat(&l) else {
            self.toast("That item is no longer available", false);
            return;
        };
        if e.is_dir {
            self.navigate(Route::Loc(l));
            return;
        }
        match e.kind {
            Kind::Image | Kind::Vector | Kind::Audio | Kind::Text | Kind::Code => self.begin_preview(l),
            _ => {
                self.be.open(&l);
            }
        }
    }

    pub fn begin_preview(&mut self, l: Location) {
        self.be.library.write().unwrap().touch_recent(&l);
        if let Ok(e) = self.be.stat(&l) {
            match e.kind {
                Kind::Image => {
                    if !self.big_thumbs.contains_key(&l) {
                        self.be.request_thumb(&l, 1800);
                    }
                }
                Kind::Audio => {
                    let w = self.make_waveform(&l);
                    self.waveform = Some((l.clone(), w));
                }
                Kind::Text | Kind::Code => {
                    let t = self.be.read_text(&l, 12_000).unwrap_or_else(|e| e);
                    self.text_preview = Some((l.clone(), t));
                }
                _ => {}
            }
        }
        if self.st().autoplay {
            if let Some(e) = self.entries.iter().find(|e| e.loc == l) {
                if e.kind == Kind::Audio {
                    self.play(l.clone());
                }
            }
        }
        self.modal = Modal::Preview(l);
    }

    fn make_waveform(&self, l: &Location) -> Vec<f32> {
        let n = 96;
        let Ok(b) = self.be.read_bytes(l, 8 << 20) else { return vec![0.2; n] };
        let start = b.windows(4).position(|w| w == b"data").map(|i| i + 8).unwrap_or(44).min(b.len());
        let samples: Vec<i16> = b[start..].chunks_exact(2).map(|c| i16::from_le_bytes([c[0], c[1]])).collect();
        if samples.len() < n * 4 || !l.name().to_lowercase().ends_with(".wav") {
            return (0..n).map(|i| 0.25 + 0.55 * (((b.get(i * 7 % b.len().max(1)).copied().unwrap_or(90) as f32) / 255.0 * 6.28).sin().abs())).collect();
        }
        let per = samples.len() / n;
        let mut v: Vec<f32> = (0..n).map(|i| samples[i * per..(i + 1) * per].iter().map(|s| (*s as f32).abs()).fold(0.0, f32::max) / 32768.0).collect();
        let m = v.iter().cloned().fold(0.05, f32::max);
        v.iter_mut().for_each(|x| *x = (*x / m).clamp(0.04, 1.0));
        v
    }

    pub fn play(&mut self, l: Location) {
        match self.be.read_bytes(&l, 256 << 20) {
            Ok(b) => match self.be.audio.play(b, &l.name()) {
                Ok(()) => self.audio_loc = Some(l),
                Err(e) => self.toast(e, false),
            },
            Err(e) => self.toast(e, false),
        }
    }

    pub fn drop_on(&mut self, dest: Location, copy: bool) {
        if let Some(d) = self.drag.take() {
            let srcs: Vec<Location> = d.items.into_iter().filter(|s| s.parent().as_ref() != Some(&dest) && *s != dest && !dest.starts_with(s)).collect();
            if srcs.is_empty() {
                return;
            }
            self.submit_job(if copy { JobKind::Copy { srcs, dest } } else { JobKind::Move { srcs, dest } });
            self.sel.clear();
        }
    }

    // ───────────── “Ask Smart” — on-device helpers ─────────────
    pub fn ai_say(&mut self, text: impl Into<String>, actions: Vec<(String, Act)>, files: Vec<Location>) {
        self.ai.msgs.push(Msg { user: false, text: text.into(), files, actions });
    }
    pub fn ai_user(&mut self, text: impl Into<String>) {
        self.ai.msgs.push(Msg { user: true, text: text.into(), files: vec![], actions: vec![] });
    }
    fn ai_organise(&mut self) {
        self.ai_user("Organise this folder");
        let plan = tools::organise_by_kind(&self.items);
        if plan.is_empty() {
            self.ai_say("There are no loose files to organise here.", vec![], vec![]);
            return;
        }
        let mut groups: Vec<(String, Vec<Location>)> = vec![];
        for (l, g) in &plan {
            match groups.iter_mut().find(|x| &x.0 == g) {
                Some(x) => x.1.push(l.clone()),
                None => groups.push((g.clone(), vec![l.clone()])),
            }
        }
        let summary = groups.iter().map(|(g, v)| format!("{} \u{2192} {}", g, v.len())).collect::<Vec<_>>().join("  \u{b7}  ");
        let files: Vec<Location> = plan.iter().map(|p| p.0.clone()).take(6).collect();
        self.ai.pending_org = Some((self.target_dir(), groups));
        self.ai_say(format!("Here\u{2019}s the plan \u{2014} nothing moves until you apply it.\n{}", summary), vec![("Apply plan".into(), Act::ApplyOrganise)], files);
    }
    fn ai_apply_organise(&mut self) {
        if let Some((dir, groups)) = self.ai.pending_org.take() {
            let mut n = 0;
            for (g, srcs) in groups {
                let target = dir.join(&g);
                if self.be.stat(&target).is_err() && self.be.create_folder(&dir, &g).is_err() {
                    continue;
                }
                n += srcs.len();
                self.submit_job(JobKind::Move { srcs, dest: target });
            }
            self.ai_say(format!("Done \u{2014} filing {} files into folders by kind.", n), vec![], vec![]);
        }
    }
    fn ai_duplicates(&mut self) {
        self.ai_user("Find duplicates");
        let groups = tools::find_duplicates(&self.be.registry, &self.items);
        if groups.is_empty() {
            self.ai_say("No duplicates in this folder \u{2014} every file is unique.", vec![], vec![]);
            return;
        }
        let extra: Vec<Location> = groups.iter().flat_map(|g| g.iter().skip(1).map(|e| e.loc.clone())).collect();
        let wasted: u64 = groups.iter().map(|g| g[0].size * (g.len() as u64 - 1)).sum();
        let shown: Vec<Location> = groups.iter().flat_map(|g| g.iter().map(|e| e.loc.clone())).take(8).collect();
        self.ai_say(
            format!("Found {} duplicate group{} wasting {}. I\u{2019}d keep the first copy of each and trash the rest.", groups.len(), if groups.len() == 1 { "" } else { "s" }, format_size(wasted)),
            vec![(format!("Trash {} copies", extra.len()), Act::Confirm(Confirm { title: format!("Move {} duplicates to Trash?", extra.len()), body: "The first copy of each file stays where it is.".into(), ok: "Move to Trash".into(), danger: true, act: Box::new(Act::Job(JobKind::Trash { srcs: extra })) }))],
            shown,
        );
    }
    fn ai_old(&mut self) {
        self.ai_user("Find old files");
        let old = tools::older_than(&self.items, 90);
        if old.is_empty() {
            self.ai_say("Nothing here is older than 90 days.", vec![], vec![]);
        } else {
            let bytes: u64 = old.iter().map(|e| e.size).sum();
            let files = old.iter().map(|e| e.loc.clone()).take(8).collect();
            self.ai.pending_old = old.iter().map(|e| e.loc.clone()).collect();
            self.ai_say(format!("{} files haven\u{2019}t been touched in 90+ days ({}). Tag them Archive?", old.len(), format_size(bytes)), vec![("Tag as Archive".into(), Act::TagOld)], files);
        }
    }
    pub fn ai_send(&mut self) {
        let t = self.ai.input.trim().to_string();
        if t.is_empty() {
            return;
        }
        self.ai.input.clear();
        let l = t.to_lowercase();
        if l.contains("duplicate") {
            return self.ai_duplicates();
        }
        if l.contains("organi") || l.contains("tidy") {
            return self.ai_organise();
        }
        self.ai_user(t.clone());
        match Query::from_natural(&t) {
            Some((q, why)) => self.ai_say(format!("I read that as {}. Searching your files on this device.", why), vec![("Show results".into(), Act::RunSearch(q))], vec![]),
            None => self.ai_say("I can search by kind, date, size and tag \u{2014} try \u{201c}large videos from last month\u{201d} or \u{201c}documents tagged design\u{201d}. I can also tidy this folder or find duplicates.", vec![], vec![]),
        }
    }
}

// ───────────── tabs ─────────────
impl SmartApp {
    pub fn route_label(&self, r: &Route) -> String {
        match r {
            Route::Recents => "Recents".into(),
            Route::Tag(id) => self.lib(|l| l.tags.iter().find(|t| &t.id == id).map(|t| t.name.clone())).unwrap_or_else(|| id.clone()),
            Route::Loc(l) => self.loc_title(l),
        }
    }
    fn tab_store(&mut self) {
        let (r, h, i) = (self.route.clone(), self.hist.clone(), self.hist_i);
        if let Some(t) = self.tabs.get_mut(self.tab_i) {
            t.route = r;
            t.hist = h;
            t.hist_i = i;
        }
    }
    fn tab_load(&mut self, i: usize) {
        let t = self.tabs[i].clone();
        self.tab_i = i;
        self.hist = t.hist;
        self.hist_i = t.hist_i;
        self.go(t.route);
    }
    pub fn new_tab(&mut self, r: Option<Route>) {
        self.tab_store();
        let r = r.unwrap_or_else(|| self.route.clone());
        self.tabs.insert(self.tab_i + 1, TabState { route: r.clone(), hist: vec![r.clone()], hist_i: 0 });
        self.tab_load(self.tab_i + 1);
        self.toast(format!("New tab \u{b7} {}", self.route_label(&r)), true);
    }
    pub fn close_tab(&mut self, i: usize) {
        if self.tabs.len() <= 1 || i >= self.tabs.len() {
            return;
        }
        self.tab_store();
        self.tabs.remove(i);
        let next = if i < self.tab_i { self.tab_i - 1 } else { self.tab_i.min(self.tabs.len() - 1) };
        self.tab_load(next);
    }
    pub fn switch_tab(&mut self, i: usize) {
        if i < self.tabs.len() && i != self.tab_i {
            self.tab_store();
            self.tab_load(i);
        }
    }
}
