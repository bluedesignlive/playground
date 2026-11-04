//! Smart Photos — library grid, albums, a viewer with spring zoom/pan, filmstrip, crossfading slideshow and a
//! non-destructive-feeling editor (live adjustments, filter presets with previews, compare, histogram, save a copy).
//! Pure Rust: images are decoded with the `image` crate and edited on the CPU.
use crate::app::*;
use crate::appkit::{Frame, SRow, SVal};
use crate::chrome::*;
use crate::overlays::{mi, MI};
use crate::icons::Ic;
use crate::paint::*;
use crate::theme::*;
use egui::{pos2, vec2, Align2, Color32, Id, Pos2, Rect, Sense, TextureHandle, TextureOptions, Ui, Vec2};
use smart_core::*;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Adjust {
    pub exposure: f32,
    pub contrast: f32,
    pub saturation: f32,
    pub warmth: f32,
    pub vignette: f32,
}

impl Default for Adjust {
    fn default() -> Self {
        Self { exposure: 0.0, contrast: 0.0, saturation: 0.0, warmth: 0.0, vignette: 0.0 }
    }
}

const PRESETS: [(&str, Adjust); 8] = [
    ("Original", Adjust { exposure: 0.0, contrast: 0.0, saturation: 0.0, warmth: 0.0, vignette: 0.0 }),
    ("Vivid", Adjust { exposure: 0.05, contrast: 0.18, saturation: 0.45, warmth: 0.05, vignette: 0.0 }),
    ("Warm", Adjust { exposure: 0.05, contrast: 0.05, saturation: 0.10, warmth: 0.55, vignette: 0.10 }),
    ("Cool", Adjust { exposure: 0.0, contrast: 0.08, saturation: 0.0, warmth: -0.55, vignette: 0.10 }),
    ("Mono", Adjust { exposure: 0.0, contrast: 0.12, saturation: -1.0, warmth: 0.0, vignette: 0.0 }),
    ("Noir", Adjust { exposure: -0.10, contrast: 0.40, saturation: -1.0, warmth: 0.0, vignette: 0.55 }),
    ("Fade", Adjust { exposure: 0.15, contrast: -0.25, saturation: -0.30, warmth: 0.10, vignette: 0.0 }),
    ("Dusk", Adjust { exposure: -0.10, contrast: 0.15, saturation: 0.20, warmth: -0.25, vignette: 0.40 }),
];

#[derive(Clone, PartialEq)]
pub enum Album {
    All,
    Favorites,
    Folder(Location),
}

pub struct Loaded {
    pub loc: Location,
    pub rgba: Vec<u8>,
    pub w: usize,
    pub h: usize,
    pub tex: TextureHandle,
    pub orig: TextureHandle,
    pub hist: [f32; 48],
    pub presets: Vec<TextureHandle>,
    pub bytes: u64,
}

pub struct Photos {
    pub scanned: bool,
    pub all: Vec<Entry>,
    pub album: Album,
    pub thumbs: HashMap<Location, (TextureHandle, [f32; 2])>,
    pub failed: HashSet<Location>,
    pub favs: HashSet<Location>,
    pub open: Option<Location>,
    pub cur: Option<Loaded>,
    pub prev: Option<(TextureHandle, f64)>,
    pub info: bool,
    pub edit: bool,
    pub zoom: f32,
    pub pan: Vec2,
    pub rot: i32,
    pub flip_h: bool,
    pub flip_v: bool,
    pub adj: Adjust,
    pub preset: usize,
    pub dirty: bool,
    pub tile: f32,
    pub slideshow: bool,
    pub slide_t: f64,
    pub compare: bool,
    pub grid_scroll_to: Option<f32>,
    pub sort: usize,
    pub fill: bool,
    pub captions: bool,
    pub gap: f32,
    pub slide_secs: f32,
    pub filmstrip: bool,
    pub tools: bool,
    pub wheel_zoom: bool,
    pub search: String,
    pub search_open: bool,
    pub search_focus: bool,
    pub ctx: Option<Location>,
    pub folders: Vec<(Location, usize)>,
    pub strip_x: f32,
    pub strip_v: f32,
    pub strip_goal: f32,
    pub strip_idx: usize,
}

impl Default for Photos {
    fn default() -> Self {
        Self {
            scanned: false,
            all: vec![],
            album: Album::All,
            thumbs: HashMap::new(),
            failed: HashSet::new(),
            favs: HashSet::new(),
            open: None,
            cur: None,
            prev: None,
            info: false,
            edit: false,
            zoom: 1.0,
            pan: Vec2::ZERO,
            rot: 0,
            flip_h: false,
            flip_v: false,
            adj: Adjust::default(),
            preset: 0,
            dirty: false,
            tile: 210.0,
            slideshow: false,
            slide_t: 0.0,
            compare: false,
            grid_scroll_to: None,
            sort: 0,
            fill: true,
            captions: false,
            gap: 12.0,
            slide_secs: 4.0,
            filmstrip: true,
            tools: true,
            wheel_zoom: true,
            search: String::new(),
            search_open: false,
            search_focus: false,
            ctx: None,
            folders: vec![],
            strip_x: 0.0,
            strip_v: 0.0,
            strip_goal: 0.0,
            strip_idx: usize::MAX,
        }
    }
}

fn process(src: &[u8], w: usize, h: usize, a: &Adjust) -> Vec<u8> {
    let mut out = vec![255u8; src.len()];
    let ex = 2f32.powf(a.exposure);
    let k = 1.0 + a.contrast * 1.1;
    let sat = 1.0 + a.saturation;
    let warm = a.warmth * 0.10;
    let (cx, cy) = (w as f32 / 2.0, h as f32 / 2.0);
    let maxd = (cx * cx + cy * cy).sqrt().max(1.0);
    let vig = a.vignette;
    for y in 0..h {
        let dy = (y as f32 - cy) / maxd;
        for x in 0..w {
            let i = (y * w + x) * 4;
            let (mut r, mut g, mut b) = (src[i] as f32 / 255.0, src[i + 1] as f32 / 255.0, src[i + 2] as f32 / 255.0);
            r *= ex;
            g *= ex;
            b *= ex;
            r = (r - 0.5) * k + 0.5;
            g = (g - 0.5) * k + 0.5;
            b = (b - 0.5) * k + 0.5;
            let l = 0.2126 * r + 0.7152 * g + 0.0722 * b;
            r = l + (r - l) * sat + warm;
            g = l + (g - l) * sat + warm * 0.15;
            b = l + (b - l) * sat - warm;
            if vig > 0.0 {
                let dx = (x as f32 - cx) / maxd;
                let d = (dx * dx + dy * dy).sqrt();
                let t = ((d - 0.30) / 0.70).clamp(0.0, 1.0);
                let v = 1.0 - vig * t * t * (3.0 - 2.0 * t);
                r *= v;
                g *= v;
                b *= v;
            }
            out[i] = (r.clamp(0.0, 1.0) * 255.0) as u8;
            out[i + 1] = (g.clamp(0.0, 1.0) * 255.0) as u8;
            out[i + 2] = (b.clamp(0.0, 1.0) * 255.0) as u8;
        }
    }
    out
}

fn uvs(rot: i32, fh: bool, fv: bool) -> [egui::Pos2; 4] {
    let base = [pos2(0.0, 0.0), pos2(1.0, 0.0), pos2(1.0, 1.0), pos2(0.0, 1.0)];
    let k = rot.rem_euclid(4) as usize;
    let mut u = [base[0], base[0], base[0], base[0]];
    for i in 0..4 {
        u[i] = base[(i + 4 - k) % 4];
    }
    if fh {
        u.swap(0, 1);
        u.swap(2, 3);
    }
    if fv {
        u.swap(0, 3);
        u.swap(1, 2);
    }
    u
}

fn draw_tex(p: &egui::Painter, tex: egui::TextureId, r: Rect, rot: i32, fh: bool, fv: bool, tint: Color32) {
    let u = uvs(rot, fh, fv);
    let mut m = egui::epaint::Mesh::with_texture(tex);
    let c = [r.left_top(), r.right_top(), r.right_bottom(), r.left_bottom()];
    for i in 0..4 {
        m.vertices.push(egui::epaint::Vertex { pos: c[i], uv: u[i], color: tint });
    }
    m.indices.extend([0, 1, 2, 0, 2, 3]);
    p.add(egui::Shape::mesh(m));
}

impl SmartApp {
    fn photos_scan(&mut self) {
        let mut found: Vec<Entry> = vec![];
        let mut stack = vec![self.be.home()];
        let mut budget = 6000;
        while let Some(d) = stack.pop() {
            let Some(fs) = self.be.registry.get(&d.fs) else { continue };
            let Ok(list) = fs.list(&d.path) else { continue };
            for e in list {
                if budget == 0 {
                    break;
                }
                budget -= 1;
                if e.hidden {
                    continue;
                }
                if e.is_dir {
                    stack.push(e.loc.clone());
                } else if e.kind == Kind::Image {
                    found.push(e);
                }
            }
        }
        found.sort_by(|a, b| b.modified.cmp(&a.modified).then(natural_cmp(&a.name, &b.name)));
        self.shell.photos.all = found;
        self.shell.photos.scanned = true;
    }

    pub fn photos_first(&mut self) -> Option<Location> {
        if !self.shell.photos.scanned {
            self.photos_scan();
        }
        self.shell.photos.all.first().map(|e| e.loc.clone())
    }

    pub fn photos_show(&mut self, l: Location) {
        if !self.shell.photos.scanned {
            self.photos_scan();
        }
        self.shell.photos.open = Some(l);
        self.photos_reset_view();
    }

    fn photos_reset_view(&mut self) {
        let ph = &mut self.shell.photos;
        ph.zoom = 1.0;
        ph.pan = Vec2::ZERO;
        ph.rot = 0;
        ph.flip_h = false;
        ph.flip_v = false;
        ph.adj = Adjust::default();
        ph.preset = 0;
        ph.dirty = false;
        ph.cur = None;
    }

    fn photos_target(&self) -> Option<Location> {
        self.shell.photos.ctx.clone().or_else(|| self.shell.photos.open.clone())
    }

    fn photos_toggle_fav(&mut self, l: Location) {
        if !self.shell.photos.favs.remove(&l) {
            self.shell.photos.favs.insert(l);
        }
    }

    pub fn photos_cmd(&mut self, c: &str) {
        let tgt = self.photos_target();
        let (name, arg) = c.split_once(':').unwrap_or((c, ""));
        match name {
            "fit" => {
                self.shell.photos.zoom = 1.0;
                self.shell.photos.pan = Vec2::ZERO;
            }
            "actual" => {
                if let Some(l) = &self.shell.photos.cur {
                    let z = (l.w.max(l.h) as f32 / 700.0).clamp(1.0, 6.0);
                    self.shell.photos.zoom = z;
                }
            }
            "zoomin" => self.shell.photos.zoom = (self.shell.photos.zoom * 1.4).min(10.0),
            "zoomout" => self.shell.photos.zoom = (self.shell.photos.zoom / 1.4).max(0.4),
            "rotl" => self.shell.photos.rot -= 1,
            "rotr" => self.shell.photos.rot += 1,
            "fliph" => self.shell.photos.flip_h = !self.shell.photos.flip_h,
            "flipv" => self.shell.photos.flip_v = !self.shell.photos.flip_v,
            "slideshow" => {
                if self.shell.photos.open.is_none() {
                    let first = self.photos_list().first().cloned();
                    if let Some(e) = first {
                        self.photos_show(e.loc);
                    }
                }
                self.shell.photos.slideshow = !self.shell.photos.slideshow;
                self.shell.photos.slide_t = self.time;
            }
            "slide_from" => {
                if let Some(l) = tgt.clone() {
                    self.photos_show(l);
                    self.shell.photos.slideshow = true;
                    self.shell.photos.slide_t = self.time;
                }
            }
            "view" => {
                if let Some(l) = tgt.clone() {
                    self.photos_show(l);
                }
            }
            "view_idx" => {
                if let Some(e) = arg.parse::<usize>().ok().and_then(|i| self.shell.photos.all.get(i).cloned()) {
                    self.photos_show(e.loc);
                }
            }
            "album" => {
                let ph = &mut self.shell.photos;
                ph.open = None;
                ph.album = match arg {
                    "all" => Album::All,
                    "fav" => Album::Favorites,
                    n => n.parse::<usize>().ok().and_then(|i| ph.folders.get(i).map(|f| Album::Folder(f.0.clone()))).unwrap_or(Album::All),
                };
            }
            "fav" => {
                if let Some(l) = tgt.clone() {
                    self.photos_toggle_fav(l);
                }
            }
            "info" => {
                self.shell.photos.info = !self.shell.photos.info;
                if self.shell.photos.info {
                    self.shell.photos.edit = false;
                }
            }
            "edit" => {
                if self.shell.photos.open.is_none() {
                    if let Some(l) = tgt.clone() {
                        self.photos_show(l);
                    }
                }
                self.shell.photos.edit = !self.shell.photos.edit;
                if self.shell.photos.edit {
                    self.shell.photos.info = false;
                }
            }
            "back" => {
                let ph = &mut self.shell.photos;
                ph.open = None;
                ph.slideshow = false;
                ph.edit = false;
                ph.info = false;
            }
            "reveal" => {
                if let Some(l) = tgt.and_then(|l| l.parent()) {
                    self.acts.push(Act::OpenApp(crate::shell::AppId::Files));
                    self.acts.push(Act::Nav(Route::Loc(l)));
                }
            }
            "wallpaper" => {
                if let Some(l) = tgt {
                    self.acts.push(Act::WallpaperFrom(l));
                }
            }
            "sort" => self.shell.photos.sort = arg.parse().unwrap_or(0),
            "tile" => self.shell.photos.tile = arg.parse().unwrap_or(210.0),
            "captions" => self.shell.photos.captions = !self.shell.photos.captions,
            "fill" => self.shell.photos.fill = !self.shell.photos.fill,
            "search" => {
                self.shell.photos.search_open = true;
                self.shell.photos.search_focus = true;
            }
            "refresh" => {
                self.shell.photos.scanned = false;
                self.toast("Library refreshed", true);
            }
            "save" => self.photos_save(),
            _ => {}
        }
        self.shell.photos.ctx = None;
    }

    pub fn photos_list(&self) -> Vec<Entry> {
        let ph = &self.shell.photos;
        let q = ph.search.trim().to_lowercase();
        let mut v: Vec<Entry> = ph
            .all
            .iter()
            .filter(|e| match &ph.album {
                Album::All => true,
                Album::Favorites => ph.favs.contains(&e.loc),
                Album::Folder(f) => e.loc.parent().as_ref() == Some(f),
            })
            .filter(|e| q.is_empty() || e.name.to_lowercase().contains(&q))
            .cloned()
            .collect();
        match ph.sort {
            0 => {}
            1 => v.reverse(),
            2 => v.sort_by(|a, b| natural_cmp(&a.name, &b.name)),
            _ => v.sort_by(|a, b| b.size.cmp(&a.size)),
        }
        v
    }

    fn photos_save(&mut self) {
        let Some(cur) = &self.shell.photos.cur else { return };
        let ph = &self.shell.photos;
        let data = process(&cur.rgba, cur.w, cur.h, &ph.adj);
        let mut img = image::RgbaImage::from_raw(cur.w as u32, cur.h as u32, data).unwrap();
        for _ in 0..ph.rot.rem_euclid(4) {
            img = image::imageops::rotate90(&img);
        }
        if ph.flip_h {
            img = image::imageops::flip_horizontal(&img);
        }
        if ph.flip_v {
            img = image::imageops::flip_vertical(&img);
        }
        let rgb = image::DynamicImage::ImageRgba8(img).to_rgb8();
        let mut bytes = vec![];
        if image::codecs::jpeg::JpegEncoder::new_with_quality(&mut bytes, 92).encode_image(&rgb).is_err() {
            self.toast("Could not encode the image", false);
            return;
        }
        let Some(dir) = cur.loc.parent() else { return };
        let stem = cur.loc.name();
        let stem = stem.rsplit_once('.').map(|x| x.0.to_string()).unwrap_or(stem);
        let mut name = format!("{stem} edited.jpg");
        let mut n = 2;
        while self.be.stat(&dir.join(&name)).is_ok() {
            name = format!("{stem} edited {n}.jpg");
            n += 1;
        }
        let dest = dir.join(&name);
        let res = self.be.create_file(&dir, &name).and_then(|_| self.be.registry.get(&dest.fs).ok_or("unavailable".to_string())?.write(&dest.path, &bytes).map_err(|e| e.to_string()));
        match res {
            Ok(()) => {
                self.toast(format!("Saved \u{201c}{name}\u{201d}"), true);
                self.acts.push(Act::Refresh);
                self.shell.photos.scanned = false;
            }
            Err(e) => self.toast(format!("Save failed: {e}"), false),
        }
    }

    fn photos_thumb(&mut self, ctx: &egui::Context, l: &Location, budget: &mut u32) -> Option<(egui::TextureId, [f32; 2])> {
        if let Some((t, s)) = self.shell.photos.thumbs.get(l) {
            return Some((t.id(), *s));
        }
        if *budget == 0 || self.shell.photos.failed.contains(l) {
            return None;
        }
        *budget -= 1;
        let img = self.be.read_bytes(l, 24 * 1024 * 1024).ok().and_then(|b| image::load_from_memory(&b).ok());
        let Some(img) = img else {
            self.shell.photos.failed.insert(l.clone());
            return None;
        };
        let t = img.thumbnail(420, 420).to_rgba8();
        let sz = [t.width() as usize, t.height() as usize];
        let tex = ctx.load_texture(format!("ph-t-{}", l.path), egui::ColorImage::from_rgba_unmultiplied(sz, t.as_raw()), TextureOptions::LINEAR);
        let id = tex.id();
        self.shell.photos.thumbs.insert(l.clone(), (tex, [sz[0] as f32, sz[1] as f32]));
        Some((id, [sz[0] as f32, sz[1] as f32]))
    }

    fn photos_load_cur(&mut self, ctx: &egui::Context) {
        let Some(l) = self.shell.photos.open.clone() else { return };
        if self.shell.photos.cur.as_ref().is_some_and(|c| c.loc == l) {
            return;
        }
        let prev = self.shell.photos.cur.as_ref().map(|c| c.tex.clone());
        let bytes = self.be.read_bytes(&l, 48 * 1024 * 1024);
        let size = bytes.as_ref().map(|b| b.len() as u64).unwrap_or(0);
        let Some(img) = bytes.ok().and_then(|b| image::load_from_memory(&b).ok()) else {
            self.toast("Could not open that image", false);
            self.shell.photos.open = None;
            return;
        };
        let img = if img.width().max(img.height()) > 1600 { img.resize(1600, 1600, image::imageops::FilterType::Triangle) } else { img };
        let rgba = img.to_rgba8();
        let (w, h) = (rgba.width() as usize, rgba.height() as usize);
        let mk = |data: &[u8], w: usize, h: usize, name: &str| ctx.load_texture(name.to_string(), egui::ColorImage::from_rgba_unmultiplied([w, h], data), TextureOptions::LINEAR);
        let orig = mk(rgba.as_raw(), w, h, &format!("ph-o-{}", l.path));
        let mut hist = [0f32; 48];
        for px in rgba.as_raw().chunks_exact(4).step_by(3) {
            let lum = (0.2126 * px[0] as f32 + 0.7152 * px[1] as f32 + 0.0722 * px[2] as f32) / 255.0;
            hist[((lum * 47.0) as usize).min(47)] += 1.0;
        }
        let mx = hist.iter().cloned().fold(1.0, f32::max);
        for v in hist.iter_mut() {
            *v /= mx;
        }
        // tiny preview of every preset
        let small = image::imageops::thumbnail(&rgba, 120, 120);
        let (sw, sh) = (small.width() as usize, small.height() as usize);
        let presets = PRESETS.iter().enumerate().map(|(i, (_, a))| mk(&process(small.as_raw(), sw, sh, a), sw, sh, &format!("ph-p{i}-{}", l.path))).collect();
        let adj = self.shell.photos.adj;
        let tex = if adj == Adjust::default() { orig.clone() } else { mk(&process(rgba.as_raw(), w, h, &adj), w, h, &format!("ph-e-{}", l.path)) };
        self.shell.photos.cur = Some(Loaded { loc: l, rgba: rgba.into_raw(), w, h, tex, orig, hist, presets, bytes: size });
        if let Some(p) = prev {
            self.shell.photos.prev = Some((p, self.time));
        }
    }

    fn photos_reprocess(&mut self, ctx: &egui::Context) {
        let adj = self.shell.photos.adj;
        if let Some(c) = &mut self.shell.photos.cur {
            c.tex = if adj == Adjust::default() { c.orig.clone() } else { ctx.load_texture(format!("ph-e-{}", c.loc.path), egui::ColorImage::from_rgba_unmultiplied([c.w, c.h], &process(&c.rgba, c.w, c.h, &adj)), TextureOptions::LINEAR) };
        }
        self.shell.photos.dirty = false;
    }

    fn photos_nav(&mut self, list: &[Entry], delta: i32) {
        let Some(cur) = self.shell.photos.open.clone() else { return };
        if let Some(i) = list.iter().position(|e| e.loc == cur) {
            let n = list.len() as i32;
            let j = (i as i32 + delta).rem_euclid(n) as usize;
            self.shell.photos.open = Some(list[j].loc.clone());
            let (adj, preset) = (Adjust::default(), 0);
            let ph = &mut self.shell.photos;
            ph.zoom = 1.0;
            ph.pan = Vec2::ZERO;
            ph.rot = 0;
            ph.flip_h = false;
            ph.flip_v = false;
            ph.adj = adj;
            ph.preset = preset;
            ph.slide_t = self.time;
        }
    }

    // ───────────── context menus ─────────────
    fn photos_item_menu(&self, loc: &Location) -> Vec<MI> {
        let fav = self.shell.photos.favs.contains(loc);
        let a = |s: &str| Act::App(format!("photos.{s}"));
        vec![
            mi("gallery", "Open", a("view")).k("Enter"),
            mi("heart", if fav { "Remove from favorites" } else { "Add to favorites" }, a("fav")).k("F").q(),
            mi("tuning-2", "Edit\u{2026}", a("edit")).k("E").q(),
            mi("play", "Slideshow from here", a("slide_from")),
            mi("gallery-wide", "Set as wallpaper", Act::WallpaperFrom(loc.clone())).gap(),
            mi("folder-open", "Show in Files", a("reveal")),
            mi("link", "Copy path", Act::CopyText(loc.path.clone())).q(),
        ]
    }

    pub(crate) fn photos_library_menu(&self) -> Vec<MI> {
        let ph = &self.shell.photos;
        let a = |s: &str| Act::App(format!("photos.{s}"));
        let sorts: Vec<MI> = SORTS.iter().enumerate().map(|(i, n)| mi(["clock-circle", "history", "text", "database"][i], *n, a(&format!("sort:{i}"))).on(ph.sort == i)).collect();
        let sizes: Vec<MI> = [("Small", 140.0f32), ("Medium", 210.0), ("Large", 300.0)].iter().map(|(n, v)| mi("gallery", *n, a(&format!("tile:{v}"))).on((ph.tile - v).abs() < 30.0)).collect();
        vec![
            mi("sort-from-top-to-bottom", "Sort by", Act::Toast(String::new())).sub(sorts),
            mi("widget", "Thumbnail size", Act::Toast(String::new())).sub(sizes),
            mi("text", "Show captions", a("captions")).on(ph.captions),
            mi("crop-minimalistic", "Fill thumbnails", a("fill")).on(ph.fill),
            mi("play", "Start slideshow", a("slideshow")).gap(),
            mi("magnifer", "Search photos", a("search")).k("Ctrl+F"),
            mi("refresh", "Refresh library", a("refresh")),
            mi("settings-minimalistic", "Photos settings\u{2026}", Act::App("settings".into())).k("Ctrl+,").gap(),
            mi("command", "Command palette", Act::Palette).k("Ctrl+K"),
        ]
    }

    pub(crate) fn photos_viewer_menu(&self) -> Vec<MI> {
        let ph = &self.shell.photos;
        let fav = ph.open.as_ref().is_some_and(|l| ph.favs.contains(l));
        let a = |s: &str| Act::App(format!("photos.{s}"));
        vec![
            mi("rotate-left", "Rotate left", a("rotl")).q(),
            mi("restart", "Rotate right", a("rotr")).k("R").q(),
            mi("flip-horizontal", "Flip horizontally", a("fliph")).on(ph.flip_h).q(),
            mi("flip-vertical", "Flip vertically", a("flipv")).on(ph.flip_v).q(),
            mi("heart", if fav { "Remove from favorites" } else { "Add to favorites" }, a("fav")).k("F").q(),
            mi("full-screen", "Zoom to fit", a("fit")).k("0").gap(),
            mi("magnifer-zoom-in", "Actual size", a("actual")),
            mi("magnifer-zoom-in", "Zoom in", a("zoomin")).k("+"),
            mi("tuning-2", if ph.edit { "Close editor" } else { "Edit\u{2026}" }, a("edit")).k("E").gap(),
            mi("info-circle", if ph.info { "Hide info" } else { "Show info" }, a("info")).k("I"),
            mi(if ph.slideshow { "pause" } else { "play" }, if ph.slideshow { "Pause slideshow" } else { "Play slideshow" }, a("slideshow")).k("S"),
            mi("gallery-wide", "Set as wallpaper", a("wallpaper")).gap(),
            mi("folder-open", "Show in Files", a("reveal")),
            mi("link", "Copy path", Act::CopyText(ph.open.as_ref().map(|l| l.path.clone()).unwrap_or_default())),
            mi("alt-arrow-left", "Back to library", a("back")).k("Esc").gap(),
        ]
    }

    // ───────────── drawing ─────────────
    pub fn draw_photos(&mut self, ui: &mut Ui, fr: &Frame) {
        let ctx = ui.ctx().clone();
        if !self.shell.photos.scanned {
            self.photos_scan();
        }
        let list = self.photos_list();
        let mut folders: Vec<(Location, usize)> = vec![];
        for e in &self.shell.photos.all {
            if let Some(par) = e.loc.parent() {
                match folders.iter_mut().find(|f| f.0 == par) {
                    Some(f) => f.1 += 1,
                    None => folders.push((par, 1)),
                }
            }
        }
        folders.sort_by(|a, b| b.1.cmp(&a.1));
        self.shell.photos.folders = folders.clone();
        let viewer = self.shell.photos.open.is_some();
        let vs = spring_to(&ctx, "ph-viewer", if viewer { 1.0 } else { 0.0 }, 200.0, 26.0).clamp(0.0, 1.0);
        self.photos_sidebar(ui, fr, &folders);
        self.photos_grid(ui, fr, &list, viewer);
        if vs > 0.01 {
            self.photos_load_cur(&ctx);
            if self.shell.photos.dirty {
                self.photos_reprocess(&ctx);
            }
            self.draw_photo_viewer(ui, fr, vs, &list);
        }
        self.photos_toolbar(ui, fr, &list, vs);
    }

    fn photos_sidebar(&mut self, ui: &mut Ui, fr: &Frame, folders: &[(Location, usize)]) {
        if fr.side.width() < 24.0 {
            return;
        }
        let th = self.th.clone();
        let saved = ui.clip_rect();
        ui.set_clip_rect(fr.side.intersect(saved));
        let p = ui.painter().clone();
        let left = fr.win.left();
        let gear = Rect::from_center_size(pos2(left + SIDEBAR_W - 28.0, fr.win.top() + APPBAR_H / 2.0), vec2(30.0, 30.0));
        if self.icon_button(ui, gear, "settings-minimalistic", "ph-gear", Btn::Quiet, true).on_hover_text("Photos settings  Ctrl+,").clicked() {
            self.shell.settings = Some((crate::shell::AppId::Photos, 0));
        }
        // identity row
        let ws = Rect::from_min_size(pos2(left + 12.0, fr.win.top() + 54.0), vec2(SIDEBAR_W - 24.0, 42.0));
        let av = Rect::from_min_size(ws.min + vec2(7.0, 7.0), vec2(28.0, 28.0));
        shadow(&p, av, 9.0, 2.0, 6.0, -1.0, c(255, 122, 89, 0.45));
        grad_fill(&p, av, 9.0, hex("#ff9fb8"), hex("#ff7a59"));
        rim(&p, av, 9.0, 1.0, Color32::from_white_alpha(130), true, 14.0);
        self.ico(&p, "gallery", Ic::Duo, av.center(), 16.0, Color32::WHITE);
        p.text(pos2(av.right() + 9.0, ws.center().y - 7.5), Align2::LEFT_CENTER, "Photos", f_semi(13.0), th.ink1);
        p.text(pos2(av.right() + 9.0, ws.center().y + 8.0), Align2::LEFT_CENTER, format!("{} in library", self.shell.photos.all.len()), f_medium(11.0), th.ink3);
        // storage card
        let total: u64 = self.shell.photos.all.iter().map(|e| e.size).sum();
        let card = Rect::from_min_size(pos2(left + 12.0, fr.side.bottom() - 14.0 - 62.0), vec2(SIDEBAR_W - 24.0, 62.0));
        elevate(&p, &th, card, 14.0, 2);
        grad_fill(&p, card, 14.0, th.card_top, th.card_bottom);
        rim(&p, card, 14.0, 1.2, th.rim, true, 30.0);
        p.text(card.left_top() + vec2(14.0, 18.0), Align2::LEFT_CENTER, "Library size", f_semi(T_META), th.ink1);
        p.text(card.right_top() + vec2(-14.0, 18.0), Align2::RIGHT_CENTER, format_size(total), f_semi(T_META), th.ink2);
        let bar = Rect::from_min_size(card.left_top() + vec2(14.0, 32.0), vec2(card.width() - 28.0, 8.0));
        well(&p, &th, bar, 4.0);
        let frac = (self.shell.photos.favs.len() as f32 / self.shell.photos.all.len().max(1) as f32).clamp(0.0, 1.0).max(0.04);
        grad_fill(&p, Rect::from_min_size(bar.min, vec2(bar.width() * frac, 8.0)), 4.0, hex("#ff9fb8"), hex("#ff7a59"));
        p.text(card.left_top() + vec2(14.0, 50.0), Align2::LEFT_CENTER, format!("{} favorites \u{b7} {} albums", self.shell.photos.favs.len(), folders.len()), f_medium(T_LABEL), th.ink3);
        // nav
        let nav = Rect::from_min_max(pos2(left, ws.bottom() + 6.0), pos2(left + SIDEBAR_W, card.top() - 8.0));
        let rows_n = 2 + folders.len();
        let content_h = 12.0 + 22.0 + 2.0 * 33.0 + 14.0 + 23.0 + folders.len().max(1) as f32 * 33.0 + 12.0;
        let _ = rows_n;
        let off = self.vscroll(ui, nav, "ph-side", content_h);
        ui.set_clip_rect(nav.intersect(saved));
        let pp = p.with_clip_rect(nav.intersect(saved));
        let x0 = nav.left() + 12.0;
        let w = SIDEBAR_W - 24.0;
        let mut y = nav.top() - off + 12.0;
        let label = |y: f32, s: &str| pp.text(pos2(nav.left() + 20.0, y + 7.0), Align2::LEFT_CENTER, s.to_uppercase(), f_semi(T_LABEL), th.ink4);
        label(y, "Library");
        y += 22.0;
        let mut rows: Vec<(String, &'static str, usize, Album, String)> = vec![("All photos".into(), "gallery", self.shell.photos.all.len(), Album::All, "all".into()), ("Favorites".into(), "heart", self.shell.photos.favs.len(), Album::Favorites, "fav".into())];
        let mut sec2: Vec<(String, &'static str, usize, Album, String)> = vec![];
        for (i, (l, n)) in folders.iter().enumerate() {
            sec2.push((self.loc_title(l), "folder", *n, Album::Folder(l.clone()), i.to_string()));
        }
        for (si, sec) in [rows.drain(..).collect::<Vec<_>>(), sec2].into_iter().enumerate() {
            if si == 1 {
                y += 14.0;
                label(y, "Albums");
                y += 23.0;
            }
            for (name, icon, n, album, key) in sec {
                let rect = Rect::from_min_size(pos2(x0, y), vec2(w, 32.0));
                if rect.bottom() > nav.top() - 4.0 && rect.top() < nav.bottom() + 4.0 {
                    let on = self.shell.photos.album == album;
                    let resp = self.sb_row(ui, rect, icon, &name, on, Some(n.to_string()), false, &format!("ph-album-{key}"));
                    if resp.clicked() {
                        self.push_act(Act::App(format!("photos.album:{key}")));
                    }
                    if resp.secondary_clicked() {
                        if let (Album::Folder(l), Some(pos)) = (&album, ui.ctx().pointer_latest_pos()) {
                            let loc = l.clone();
                            self.open_app_menu(pos, vec![mi("gallery", "Open album", Act::App(format!("photos.album:{key}"))), mi("folder-open", "Show in Files", Act::Seq(vec![Act::OpenApp(crate::shell::AppId::Files), Act::Nav(Route::Loc(loc.clone()))])), mi("link", "Copy path", Act::CopyText(loc.path.clone()))]);
                        }
                    }
                }
                y += 33.0;
            }
        }
        ui.set_clip_rect(saved);
    }

    fn photos_toolbar(&mut self, ui: &mut Ui, fr: &Frame, list: &[Entry], vs: f32) {
        let th = self.th.clone();
        let cy = fr.bar.center().y;
        let a = |s: &str| Act::App(format!("photos.{s}"));
        if vs < 0.5 {
            let mut bui = ui.new_child(egui::UiBuilder::new().max_rect(fr.bar));
            bui.set_opacity((1.0 - vs * 2.0).clamp(0.0, 1.0));
            let ui = &mut bui;
            let p = ui.painter().clone();
            let mut rx = fr.right_x;
            let avail = rx - fr.lead_x;
            // more
            let more = Rect::from_min_size(pos2(rx - 34.0, cy - 17.0), vec2(34.0, 34.0));
            if self.icon_button(ui, more, "menu-dots", "ph-more", Btn::Raised, true).on_hover_text("More").clicked() {
                let items = self.photos_library_menu();
                self.open_app_menu(pos2(more.right() - 232.0, more.bottom() + 8.0), items);
            }
            rx = more.left() - 8.0;
            // slideshow (collapses to an icon)
            let wide = avail > 640.0;
            let ss = if wide { Rect::from_min_size(pos2(rx - 118.0, cy - 17.0), vec2(118.0, 34.0)) } else { Rect::from_min_size(pos2(rx - 34.0, cy - 17.0), vec2(34.0, 34.0)) };
            let ssc = if wide { self.label_button(ui, ss, Some("play"), "Slideshow", "ph-slide", Btn::Tonal).clicked() } else { self.icon_button(ui, ss, "play", "ph-slide-i", Btn::Tonal, true).on_hover_text("Slideshow").clicked() };
            if ssc {
                self.push_act(a("slideshow"));
            }
            rx = ss.left() - 8.0;
            // sort
            let sort = Rect::from_min_size(pos2(rx - 34.0, cy - 17.0), vec2(34.0, 34.0));
            if self.icon_button(ui, sort, "sort-vertical", "ph-sort", Btn::Raised, true).on_hover_text(format!("Sort: {}", SORTS[self.shell.photos.sort.min(3)])).clicked() {
                let items = self.photos_library_menu();
                let sub = items.first().map(|i| i.sub.clone()).unwrap_or_default();
                self.open_app_menu(pos2(sort.left(), sort.bottom() + 8.0), sub);
            }
            rx = sort.left() - 8.0;
            // zoom slider
            if avail > 780.0 {
                let track = Rect::from_min_size(pos2(rx - 104.0, cy - 4.0), vec2(104.0, 8.0));
                let t = (self.shell.photos.tile - 120.0) / 220.0;
                let (nt, resp) = self.slider(ui, track, t.clamp(0.0, 1.0), "ph-zoom");
                if resp.dragged() || resp.clicked() {
                    self.shell.photos.tile = 120.0 + nt * 220.0;
                }
                self.ico(&p, "gallery", Ic::Outline, pos2(track.left() - 14.0, cy), 12.0, th.ink4);
                self.ico(&p, "gallery", Ic::Outline, pos2(track.right() + 16.0, cy), 18.0, th.ink4);
                rx = track.left() - 40.0;
            }
            // search: a capsule when there is room, otherwise an icon that expands on demand
            let want_open = self.shell.photos.search_open || !self.shell.photos.search.is_empty();
            let room = rx - fr.lead_x - 190.0;
            if avail > 880.0 || (want_open && room > 120.0) {
                let sw = if avail > 880.0 { 196.0 } else { 160.0 };
                let sr = Rect::from_min_size(pos2(rx - sw, cy - 17.0), vec2(sw, 34.0));
                well(&p, &th, sr, 17.0);
                self.ico(&p, "magnifer", Ic::Outline, pos2(sr.left() + 17.0, cy), 15.0, th.ink3);
                let mut q = std::mem::take(&mut self.shell.photos.search);
                let te = ui.put(Rect::from_min_max(sr.min + vec2(32.0, 6.0), sr.max - vec2(30.0, 6.0)), egui::TextEdit::singleline(&mut q).id(Id::new("ph-search")).frame(egui::Frame::NONE).hint_text(egui::RichText::new("Search photos").color(th.ink4)).font(f_medium(T_BODY)).text_color(th.ink1).vertical_align(egui::Align::Center).margin(egui::Margin::ZERO));
                if self.shell.photos.search_focus {
                    te.request_focus();
                    self.shell.photos.search_focus = false;
                }
                if te.has_focus() {
                    p.rect_stroke(sr.shrink(0.5), cr(17.0), egui::Stroke::new(1.5, alpha(th.accent, 0.55)), egui::StrokeKind::Inside);
                }
                if !q.is_empty() {
                    let xr = Rect::from_center_size(pos2(sr.right() - 16.0, cy), vec2(22.0, 22.0));
                    if self.icon_button(ui, xr, "close-circle", "ph-search-x", Btn::Quiet, true).clicked() {
                        q.clear();
                    }
                }
                self.shell.photos.search = q;
                rx = sr.left() - 10.0;
            } else {
                let sr = Rect::from_min_size(pos2(rx - 34.0, cy - 17.0), vec2(34.0, 34.0));
                if self.icon_button(ui, sr, "magnifer", "ph-search-i", Btn::Raised, true).on_hover_text("Search  Ctrl+F").clicked() {
                    self.push_act(a("search"));
                }
                rx = sr.left() - 10.0;
            }
            // title
            let ph = &self.shell.photos;
            let title = match &ph.album {
                Album::All => "All photos".to_string(),
                Album::Favorites => "Favorites".to_string(),
                Album::Folder(l) => self.loc_title(l),
            };
            let total: u64 = list.iter().map(|e| e.size).sum();
            let tw_ = (rx - fr.lead_x).max(40.0);
            p.text(pos2(fr.lead_x, cy - 8.0), Align2::LEFT_CENTER, self.fit(&p, &title, &f_bold(T_TITLE + 2.0), tw_), f_bold(T_TITLE + 2.0), th.ink1);
            p.text(pos2(fr.lead_x, cy + 11.0), Align2::LEFT_CENTER, self.fit(&p, &format!("{} photo{} \u{b7} {}{}", list.len(), if list.len() == 1 { "" } else { "s" }, format_size(total), if ph.search.is_empty() { String::new() } else { format!(" \u{b7} matching \u{201c}{}\u{201d}", ph.search) }), &f_medium(T_META), tw_), f_medium(T_META), th.ink3);
        } else {
            let mut bui = ui.new_child(egui::UiBuilder::new().max_rect(fr.bar));
            bui.set_opacity((vs * 2.0 - 1.0).clamp(0.0, 1.0));
            let ui = &mut bui;
            let p = ui.painter().clone();
            let back = Rect::from_min_size(pos2(fr.lead_x, cy - 17.0), vec2(104.0, 34.0));
            if self.label_button(ui, back, Some("alt-arrow-left"), "Library", "ph-back", Btn::Raised).clicked() {
                self.push_act(a("back"));
            }
            let ph = &self.shell.photos;
            let fav = ph.open.as_ref().is_some_and(|l| ph.favs.contains(l));
            let (edit, info) = (ph.edit, ph.info);
            let name = ph.open.as_ref().map(|l| l.name()).unwrap_or_default();
            let idx = list.iter().position(|e| Some(&e.loc) == ph.open.as_ref()).map(|i| i + 1).unwrap_or(0);
            let mut rx = fr.right_x;
            let btns: [(&'static str, &str, bool, &str, &str); 4] = [("menu-dots", "ph-v-more", false, "More", "more"), ("tuning-2", "ph-v-edit", edit, "Edit  E", "edit"), ("info-circle", "ph-v-info", info, "Info  I", "info"), ("heart", "ph-v-fav", fav, "Favorite  F", "fav")];
            let compact = fr.right_x - fr.lead_x < 520.0;
            for (icon, id, on, tip, cmd) in btns.iter().rev().rev() {
                if compact && *cmd != "more" && *cmd != "fav" {
                    continue;
                }
                let r = Rect::from_min_size(pos2(rx - 34.0, cy - 17.0), vec2(34.0, 34.0));
                let kind = if *cmd == "fav" || !*on { Btn::Raised } else { Btn::Active };
                let resp = self.icon_button(ui, r, icon, id, kind, true).on_hover_text(*tip);
                if *cmd == "fav" && *on {
                    self.ico(&p, "heart", Ic::Filled, r.center(), 17.0, hex("#ff6b8a"));
                }
                if resp.clicked() {
                    if *cmd == "more" {
                        let items = self.photos_viewer_menu();
                        self.open_app_menu(pos2(r.right() - 236.0, r.bottom() + 8.0), items);
                    } else {
                        self.push_act(a(cmd));
                    }
                }
                rx = r.left() - 8.0;
            }
            let mid = Rect::from_min_max(pos2(back.right() + 12.0, fr.bar.top()), pos2(rx, fr.bar.bottom()));
            p.text(pos2(mid.center().x, cy - 8.0), Align2::CENTER_CENTER, self.fit(&p, &name, &f_semi(T_BODY), mid.width()), f_semi(T_BODY), th.ink1);
            p.text(pos2(mid.center().x, cy + 10.0), Align2::CENTER_CENTER, format!("{idx} of {}", list.len()), f_medium(T_META), th.ink3);
        }
    }

    fn photos_grid(&mut self, ui: &mut Ui, fr: &Frame, list: &[Entry], viewer: bool) {
        let ctx = ui.ctx().clone();
        let th = self.th.clone();
        let p = ui.painter().clone();
        let area = Rect::from_min_max(pos2(fr.stage.left() + 12.0, fr.stage.top() + 2.0), pos2(fr.stage.right() - 6.0, fr.stage.bottom() - 10.0));
        let tile = spring_to(&ctx, "ph-tile", self.shell.photos.tile, 180.0, 22.0).clamp(100.0, 360.0);
        if list.is_empty() {
            let c0 = area.center();
            let fav = self.shell.photos.album == Album::Favorites;
            self.ico(&p, if fav { "heart" } else { "gallery" }, Ic::Duo, c0 - vec2(0.0, 26.0), 44.0, alpha(th.ink4, 0.7));
            p.text(c0 + vec2(0.0, 16.0), Align2::CENTER_CENTER, if fav { "No favorites yet" } else { "No photos here" }, f_semi(T_TITLE), th.ink2);
            p.text(c0 + vec2(0.0, 38.0), Align2::CENTER_CENTER, if fav { "Tap the heart on any photo to keep it close." } else { "Images from your folders appear here." }, f_medium(T_BODY), th.ink3);
            return;
        }
        let bg = self.hit(ui, area, "ph-bg", Sense::click());
        if bg.secondary_clicked() && !viewer {
            if let Some(pos) = ctx.pointer_latest_pos() {
                let items = self.photos_library_menu();
                self.open_app_menu(pos, items);
            }
        }
        let gap = self.shell.photos.gap;
        let caption = self.shell.photos.captions;
        let fill = self.shell.photos.fill;
        let cols = (((area.width() - 6.0 + gap) / (tile + gap)).floor() as usize).max(1);
        let cw = ((area.width() - 14.0 - gap * (cols as f32 - 1.0)) / cols as f32).floor();
        let ch = (cw * 0.78).round();
        let cap_h = if caption { 24.0 } else { 0.0 };
        let rows = list.len().div_ceil(cols);
        let content_h = rows as f32 * (ch + cap_h + gap) + 12.0;
        let mut cui = ui.new_child(egui::UiBuilder::new().max_rect(area).layout(egui::Layout::top_down(egui::Align::Min)));
        cui.set_clip_rect(area.intersect(ui.clip_rect()));
        let mut budget = 2u32;
        let mut open_loc: Option<Location> = None;
        let mut fav_toggle: Option<Location> = None;
        let mut menu: Option<(Pos2, Location)> = None;
        egui::ScrollArea::vertical().id_salt("photos-grid").auto_shrink([false, false]).show_viewport(&mut cui, |ui, vp| {
            let top = ui.max_rect().top();
            ui.allocate_space(vec2(area.width() - 12.0, content_h));
            let p = ui.painter().clone();
            for (i, e) in list.iter().enumerate() {
                let (rw, cl) = (i / cols, i % cols);
                let y = rw as f32 * (ch + cap_h + gap);
                if y + ch < vp.top() - 40.0 || y > vp.bottom() + 40.0 {
                    continue;
                }
                let base = Rect::from_min_size(pos2(area.left() + 3.0 + cl as f32 * (cw + gap), top + y + 4.0), vec2(cw, ch));
                let resp = self.hit(ui, base.union(Rect::from_min_size(base.left_bottom(), vec2(cw, cap_h))), ("ph-card", &e.loc), Sense::click());
                let hov = resp.hovered() && !viewer;
                let a = spring_to(ui.ctx(), ("ph-h", &e.loc), if hov { 1.0 } else { 0.0 }, 300.0, 24.0);
                let r = base.expand2(vec2(2.5, 2.5) * a).translate(vec2(0.0, -3.0 * a));
                elevate(&p, &th, r, 14.0, if a > 0.05 { 3 } else { 2 });
                match self.photos_thumb(ui.ctx(), &e.loc, &mut budget) {
                    Some((tex, sz)) => {
                        if fill {
                            textured_rrect(&p, r, 14.0, tex, cover_uv(sz[0], sz[1], r), Color32::WHITE);
                        } else {
                            grad_fill(&p, r, 14.0, th.card_top, th.card_bottom);
                            let k = (r.width() / sz[0]).min(r.height() / sz[1]) * 0.94;
                            let ir = Rect::from_center_size(r.center(), vec2(sz[0], sz[1]) * k);
                            textured_rrect(&p, ir, 8.0, tex, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
                        }
                    }
                    None => {
                        grad_fill(&p, r, 14.0, alpha(th.accent, 0.12), alpha(th.accent, 0.04));
                        self.ico(&p, "gallery", Ic::Duo, r.center(), 26.0, alpha(th.ink4, 0.6));
                        ui.ctx().request_repaint();
                    }
                }
                rim(&p, r, 14.0, 1.2, alpha(Color32::WHITE, if th.dark { 0.16 } else { 0.6 }), true, 26.0);
                if caption {
                    p.text(pos2(base.left() + 4.0, base.bottom() + 13.0), Align2::LEFT_CENTER, self.fit(&p, &e.name, &f_medium(T_META), cw - 8.0), f_medium(T_META), th.ink2);
                } else if a > 0.02 {
                    let cap = Rect::from_min_max(pos2(r.left(), r.bottom() - 44.0), r.max);
                    let sh = p.with_clip_rect(cap);
                    grad_fill(&sh, r, 14.0, alpha(Color32::BLACK, 0.0), alpha(Color32::BLACK, 0.62 * a));
                    p.text(pos2(r.left() + 12.0, r.bottom() - 24.0 + (1.0 - a) * 6.0), Align2::LEFT_CENTER, self.fit(&p, &e.name, &f_semi(T_BODY), r.width() - 24.0), f_semi(T_BODY), alpha(Color32::WHITE, a));
                    p.text(pos2(r.left() + 12.0, r.bottom() - 9.0 + (1.0 - a) * 6.0), Align2::LEFT_CENTER, format!("{} \u{b7} {}", format_size(e.size), e.modified.map(format_date).unwrap_or_default()), f_medium(T_META), alpha(Color32::WHITE, 0.8 * a));
                }
                let fav = self.shell.photos.favs.contains(&e.loc);
                if a > 0.05 || fav {
                    let hb = Rect::from_center_size(r.right_top() + vec2(-22.0, 22.0), vec2(30.0, 30.0));
                    let hr = self.hit(ui, hb, ("ph-heart", &e.loc), Sense::click());
                    let pop = spring_to(ui.ctx(), ("ph-fav", &e.loc), if fav { 1.0 } else { 0.0 }, 380.0, 15.0);
                    p.circle_filled(hb.center(), 14.0 * (0.9 + 0.1 * a), alpha(Color32::BLACK, 0.34));
                    self.ico(&p, "heart", if fav { Ic::Filled } else { Ic::Outline }, hb.center(), 16.0 * (1.0 + 0.35 * (pop - 1.0).max(0.0) + if hr.hovered() { 0.1 } else { 0.0 }), if fav { hex("#ff6b8a") } else { Color32::WHITE });
                    if hr.clicked() {
                        fav_toggle = Some(e.loc.clone());
                    }
                }
                if resp.secondary_clicked() {
                    if let Some(pos) = ui.ctx().pointer_latest_pos() {
                        menu = Some((pos, e.loc.clone()));
                    }
                }
                if resp.double_clicked() || (resp.clicked() && !matches!(fav_toggle, Some(ref l) if *l == e.loc)) {
                    open_loc = Some(e.loc.clone());
                }
            }
        });
        if let Some(l) = fav_toggle {
            self.photos_toggle_fav(l);
            open_loc = None;
        }
        if let Some((pos, l)) = menu {
            self.shell.photos.ctx = Some(l.clone());
            let items = self.photos_item_menu(&l);
            self.open_app_menu(pos, items);
            open_loc = None;
        }
        if let Some(l) = open_loc {
            self.shell.photos.open = Some(l);
            self.photos_reset_view();
        }
    }

    fn draw_photo_viewer(&mut self, ui: &mut Ui, fr: &Frame, vs: f32, list: &[Entry]) {
        let ctx = ui.ctx().clone();
        let th = self.th.clone();
        let active = fr.active;
        let mut vui = ui.new_child(egui::UiBuilder::new().max_rect(fr.stage));
        vui.set_opacity(vs);
        let ui = &mut vui;
        let p = ui.painter().clone();
        // opaque, theme-aware backing so the library never shows through the viewer
        {
            let back = if th.dark { hex("#141b29") } else { hex("#f3f6fb") };
            let rr = R_WIN as u8;
            let round = egui::CornerRadius { nw: 0, ne: 0, sw: if fr.has_side() { 0 } else { rr }, se: rr };
            p.add(egui::epaint::RectShape::filled(fr.stage.expand2(vec2(0.0, 0.0)), round, back));
            p.add(egui::epaint::RectShape::filled(fr.stage, round, alpha(th.accent, 0.03)));
        }
        // deep stage that swallows clicks behind it
        ui.allocate_rect(fr.stage, Sense::click_and_drag());
        let info_w = 304.0 * spring_to(&ctx, "ph-info", if self.shell.photos.info || self.shell.photos.edit { 1.0 } else { 0.0 }, 220.0, 26.0).clamp(0.0, 1.0);
        let strip_h = if self.shell.photos.filmstrip { 62.0 } else { 0.0 };
        let outer = Rect::from_min_max(pos2(fr.stage.left() + 12.0, fr.stage.top() + 2.0), pos2(fr.stage.right() - 12.0 - if info_w > 4.0 { info_w + 6.0 } else { 0.0 }, fr.stage.bottom() - 12.0));
        let canvas = Rect::from_min_max(outer.min, pos2(outer.right(), outer.bottom() - if strip_h > 0.0 { strip_h + 10.0 } else { 0.0 }));
        well(&p, &th, canvas, 18.0);
        let panel = Rect::from_min_max(pos2(fr.stage.right() - info_w - 6.0, fr.stage.top() + 2.0), pos2(fr.stage.right() - 12.0, fr.stage.bottom() - 12.0));

        // keys
        let mut nav = 0;
        let mut close = false;
        let typing = ctx.egui_wants_keyboard_input();
        if active && !typing {
            ctx.input(|i| {
                use egui::Key;
                if i.key_pressed(Key::ArrowRight) {
                    nav = 1;
                }
                if i.key_pressed(Key::ArrowLeft) {
                    nav = -1;
                }
                if i.key_pressed(Key::Escape) {
                    close = true;
                }
            });
            let ev = ctx.input(|i| i.events.clone());
            for e in ev {
                if let egui::Event::Text(t) = e {
                    let cmd = match t.as_str() {
                        "i" => "info",
                        "e" => "edit",
                        "f" => "fav",
                        "r" => "rotr",
                        "0" => "fit",
                        "s" => "slideshow",
                        "+" | "=" => "zoomin",
                        "-" => "zoomout",
                        _ => "",
                    };
                    if !cmd.is_empty() {
                        self.photos_cmd(cmd);
                    }
                }
            }
        }
        if close {
            self.photos_cmd("back");
        }
        if self.shell.photos.slideshow && self.time - self.shell.photos.slide_t > self.shell.photos.slide_secs as f64 {
            nav = 1;
        }
        if self.shell.photos.slideshow {
            ctx.request_repaint_after(std::time::Duration::from_millis(30));
        }
        if nav != 0 {
            self.photos_nav(list, nav);
        }

        // image
        let compare = self.shell.photos.compare;
        let (rot, fh, fv) = (self.shell.photos.rot, self.shell.photos.flip_h, self.shell.photos.flip_v);
        let zoom = spring_to(&ctx, "ph-zoom", self.shell.photos.zoom, 220.0, 26.0);
        if let Some(cur) = &self.shell.photos.cur {
            let (w, h) = if rot.rem_euclid(2) == 1 { (cur.h as f32, cur.w as f32) } else { (cur.w as f32, cur.h as f32) };
            let fit = (canvas.width() / w).min(canvas.height() / h).min(1.6) * 0.96;
            let size = vec2(w, h) * fit * zoom;
            let r = Rect::from_center_size(canvas.center() + self.shell.photos.pan, size);
            let sp = p.with_clip_rect(canvas.expand(2.0));
            // crossfade from the previous photo (slideshow / arrows)
            let t = self.shell.photos.prev.as_ref().map(|(_, t0)| ((self.time - t0) / 0.55) as f32).unwrap_or(1.0);
            if t < 1.0 {
                ctx.request_repaint();
                if let Some((pt, _)) = &self.shell.photos.prev {
                    let pr = Rect::from_center_size(canvas.center(), vec2(canvas.width() * 0.9, canvas.height() * 0.9));
                    draw_tex(&sp, pt.id(), pr, 0, false, false, alpha(Color32::WHITE, 1.0 - t));
                }
            }
            shadow(&sp, r, 4.0, 14.0, 40.0, -6.0, alpha(Color32::BLACK, (if th.dark { 0.6 } else { 0.32 }) * smooth01(t)));
            let tex = if compare { cur.orig.id() } else { cur.tex.id() };
            draw_tex(&sp, tex, r, rot, fh, fv, alpha(Color32::WHITE, smooth01(t)));
            if compare {
                let b = Rect::from_center_size(pos2(r.center().x, r.top() + 22.0), vec2(86.0, 26.0));
                grad_fill(&p, b, 13.0, alpha(Color32::BLACK, 0.6), alpha(Color32::BLACK, 0.5));
                p.text(b.center(), Align2::CENTER_CENTER, "ORIGINAL", f_bold(10.0), Color32::WHITE);
            }
            let resp = ui.interact(canvas, Id::new("ph-stage"), Sense::click_and_drag());
            if resp.dragged() && self.shell.photos.zoom > 1.0 {
                self.shell.photos.pan += resp.drag_delta();
                ctx.set_cursor_icon(egui::CursorIcon::Grabbing);
            } else if resp.hovered() && self.shell.photos.zoom > 1.0 {
                ctx.set_cursor_icon(egui::CursorIcon::Grab);
            }
            if resp.double_clicked() {
                let z = if self.shell.photos.zoom > 1.05 { 1.0 } else { 2.4 };
                self.shell.photos.zoom = z;
                if z == 1.0 {
                    self.shell.photos.pan = Vec2::ZERO;
                }
            }
            if resp.secondary_clicked() {
                if let Some(pos) = ctx.pointer_latest_pos() {
                    self.shell.photos.ctx = None;
                    let items = self.photos_viewer_menu();
                    self.open_app_menu(pos, items);
                }
            }
            if resp.hovered() && self.shell.photos.wheel_zoom {
                let (sy, pinch) = ctx.input(|i| (i.smooth_scroll_delta.y, i.zoom_delta()));
                if sy.abs() > 0.1 || (pinch - 1.0).abs() > 0.001 {
                    let nz = (self.shell.photos.zoom * (sy * 0.0035).exp() * pinch).clamp(0.4, 10.0);
                    self.shell.photos.zoom = nz;
                    if nz <= 1.0 {
                        self.shell.photos.pan *= 0.8;
                    }
                }
            }
        } else {
            let t = self.time as f32;
            for k in 0..3 {
                p.circle_filled(canvas.center() + vec2((k as f32 - 1.0) * 18.0, 0.0), 4.0 + 2.0 * (t * 6.0 + k as f32).sin(), alpha(th.ink3, 0.6));
            }
            ctx.request_repaint();
        }

        // arrows
        for (dir, icon) in [(-1, "alt-arrow-left"), (1, "alt-arrow-right")] {
            let c0 = pos2(if dir < 0 { canvas.left() + 34.0 } else { canvas.right() - 34.0 }, canvas.center().y);
            let r = Rect::from_center_size(c0, vec2(44.0, 44.0));
            let near = ctx.pointer_hover_pos().is_some_and(|pp| (pp.x - c0.x).abs() < 120.0 && canvas.contains(pp));
            let a = spring_to(&ctx, ("ph-arrow", dir), if near { 1.0 } else { 0.0 }, 260.0, 24.0).clamp(0.0, 1.0);
            if a > 0.02 {
                let resp = self.hit(ui, r, ("ph-nav", dir), Sense::click());
                shadow(&p, r, 22.0, 3.0, 10.0, -2.0, alpha(Color32::BLACK, 0.3 * a));
                p.circle_filled(r.center(), 22.0 * (0.9 + 0.1 * a), alpha(Color32::BLACK, 0.5 * a));
                self.ico(&p, icon, if resp.hovered() { Ic::Duo } else { Ic::Outline }, r.center(), 20.0, alpha(Color32::WHITE, a));
                if resp.clicked() {
                    self.photos_nav(list, dir);
                }
            }
        }

        // tool pill (raised, theme-aware)
        let idx = list.iter().position(|e| Some(&e.loc) == self.shell.photos.open.as_ref()).map(|i| i + 1).unwrap_or(0);
        if self.shell.photos.tools {
            let tools: [(&'static str, &str, &str); 7] = [("rotate-left", "rotl", "Rotate left"), ("restart", "rotr", "Rotate right"), ("flip-horizontal", "fliph", "Flip horizontally"), ("flip-vertical", "flipv", "Flip vertically"), ("magnifer-zoom-in", "zoomin", "Zoom in"), ("full-screen", "fit", "Fit to window"), (if self.shell.photos.slideshow { "pause" } else { "play" }, "slideshow", "Slideshow  S")];
            let pw = tools.len() as f32 * 40.0 + 14.0;
            let pill = Rect::from_center_size(pos2(canvas.center().x, canvas.bottom() - 34.0), vec2(pw.min(canvas.width() - 20.0), 44.0));
            raised(&p, &th, pill, 22.0, false, false);
            for (i, (icon, id, tip)) in tools.iter().enumerate() {
                let r = Rect::from_center_size(pill.left_center() + vec2(27.0 + i as f32 * 40.0, 0.0), vec2(34.0, 34.0));
                if r.right() > pill.right() {
                    break;
                }
                let resp = self.hit(ui, r, ("ph-tool", *id), Sense::click());
                let a = self.anim(ui, ("ph-tool-h", *id), resp.hovered());
                if resp.is_pointer_button_down_on() {
                    grad_fill(&p, r, 17.0, alpha(th.shadow, if th.dark { 0.3 } else { 0.10 }), alpha(th.shadow, 0.04));
                } else if a > 0.01 {
                    grad_fill(&p, r, 17.0, alpha(th.accent, 0.16 * a), alpha(th.accent, 0.07 * a));
                    rim(&p, r, 17.0, 1.0, alpha(Color32::WHITE, (if th.dark { 0.14 } else { 0.9 }) * a), true, 12.0);
                }
                let on = (*id == "fliph" && fh) || (*id == "flipv" && fv);
                self.ico(&p, icon, if a > 0.5 || on { Ic::Duo } else { Ic::Outline }, r.center(), 18.0, if on { th.accent } else { mix(th.ink2, th.accent_ink, a) });
                resp.clone().on_hover_text(*tip);
                if resp.clicked() {
                    self.push_act(Act::App(format!("photos.{id}")));
                }
            }
            if self.shell.photos.slideshow {
                let f = (((self.time - self.shell.photos.slide_t) / self.shell.photos.slide_secs as f64) as f32).clamp(0.0, 1.0);
                let b = Rect::from_min_size(pill.left_bottom() + vec2(16.0, 6.0), vec2(pill.width() - 32.0, 3.0));
                p.rect_filled(b, cr(1.5), alpha(th.ink3, 0.25));
                p.rect_filled(Rect::from_min_size(b.min, vec2(b.width() * f, 3.0)), cr(1.5), th.accent);
            }
        }

        // filmstrip: wheel + drag scrollable, follows the current photo with a spring
        if strip_h > 0.0 {
            let strip = Rect::from_min_max(pos2(outer.left(), outer.bottom() - strip_h), outer.max);
            let (tw_, gap) = (62.0f32, 8.0f32);
            let total = list.len() as f32 * (tw_ + gap) - gap;
            let max_x = (total - strip.width()).max(0.0);
            let cur_i = idx.saturating_sub(1);
            if self.shell.photos.strip_idx != cur_i {
                self.shell.photos.strip_idx = cur_i;
                self.shell.photos.strip_goal = (cur_i as f32 * (tw_ + gap) + tw_ / 2.0 - strip.width() / 2.0).clamp(0.0, max_x);
            }
            let resp = ui.interact(strip, Id::new("ph-strip"), Sense::click_and_drag());
            let dt = ctx.input(|i| i.stable_dt).min(0.05);
            let mut free = false;
            if resp.dragged() {
                self.shell.photos.strip_x = (self.shell.photos.strip_x - resp.drag_delta().x).clamp(0.0, max_x);
                free = true;
                ctx.set_cursor_icon(egui::CursorIcon::Grabbing);
            }
            if resp.hovered() {
                let (dx, dy) = ctx.input(|i| (i.smooth_scroll_delta.x, i.smooth_scroll_delta.y));
                let d = if dx.abs() > dy.abs() { dx } else { dy };
                if d != 0.0 {
                    self.shell.photos.strip_x = (self.shell.photos.strip_x - d).clamp(0.0, max_x);
                    free = true;
                }
            }
            if free {
                self.shell.photos.strip_goal = self.shell.photos.strip_x;
                self.shell.photos.strip_v = 0.0;
            } else {
                let (x, g) = (self.shell.photos.strip_x, self.shell.photos.strip_goal.clamp(0.0, max_x));
                let mut v = self.shell.photos.strip_v;
                let mut nx = x;
                for _ in 0..3 {
                    v += (-170.0 * (nx - g) - 24.0 * v) * dt / 3.0;
                    nx += v * dt / 3.0;
                }
                if (nx - g).abs() < 0.2 && v.abs() < 1.0 {
                    nx = g;
                    v = 0.0;
                } else {
                    ctx.request_repaint();
                }
                self.shell.photos.strip_x = nx.clamp(0.0, max_x);
                self.shell.photos.strip_v = v;
            }
            let off = -self.shell.photos.strip_x + if total < strip.width() { (strip.width() - total) / 2.0 } else { 0.0 };
            let sp = p.with_clip_rect(strip.expand2(vec2(0.0, 6.0)));
            let ptr = ctx.pointer_hover_pos();
            let mut budget = 2u32;
            let mut hit_i: Option<usize> = None;
            for (i, e) in list.iter().enumerate() {
                let x = strip.left() + off + i as f32 * (tw_ + gap);
                if x + tw_ < strip.left() - 4.0 || x > strip.right() + 4.0 {
                    continue;
                }
                let on = i == cur_i;
                let base = Rect::from_min_size(pos2(x, strip.top() + 4.0), vec2(tw_, strip.height() - 8.0));
                let hov = resp.hovered() && ptr.is_some_and(|pp| base.contains(pp));
                let s = spring_to(&ctx, ("ph-fs", &e.loc), if on { 1.0 } else if hov { 0.4 } else { 0.0 }, 300.0, 22.0);
                let r = base.expand2(vec2(2.0, 4.0) * s);
                match self.photos_thumb(&ctx, &e.loc, &mut budget) {
                    Some((tex, sz)) => textured_rrect(&sp, r, 9.0, tex, cover_uv(sz[0], sz[1], r), alpha(Color32::WHITE, 0.6 + 0.4 * s.clamp(0.0, 1.0))),
                    None => grad_fill(&sp, r, 9.0, alpha(th.ink3, 0.2), alpha(th.ink3, 0.1)),
                }
                rim(&sp, r, 9.0, 1.0, alpha(Color32::WHITE, if th.dark { 0.18 } else { 0.7 }), true, 14.0);
                if on {
                    sp.rect_stroke(r.expand(1.0), cr(10.0), egui::Stroke::new(2.0, th.accent), egui::StrokeKind::Outside);
                }
                if hov {
                    hit_i = Some(i);
                }
            }
            if hit_i.is_some() {
                ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if resp.clicked() {
                if let Some(i) = hit_i {
                    if i != cur_i {
                        self.photos_nav(list, i as i32 - cur_i as i32);
                    }
                }
            }
            if resp.secondary_clicked() {
                if let (Some(i), Some(pos)) = (hit_i, ctx.pointer_latest_pos()) {
                    let l = list[i].loc.clone();
                    self.shell.photos.ctx = Some(l.clone());
                    let items = self.photos_item_menu(&l);
                    self.open_app_menu(pos, items);
                }
            }
            // soft fade at the edges signals there is more to scroll
            if max_x > 1.0 {
                let fade = |x: f32, a0: f32, a1: f32| {
                    let r = Rect::from_min_size(pos2(x, strip.top()), vec2(26.0, strip.height()));
                    let mut m = egui::epaint::Mesh::default();
                    let (xl, xr) = (r.left(), r.right());
                    let cl = alpha(th.window_top, a0);
                    let cr_ = alpha(th.window_top, a1);
                    for (xx, col) in [(xl, cl), (xr, cr_)] {
                        m.vertices.push(egui::epaint::Vertex { pos: pos2(xx, r.top()), uv: egui::epaint::WHITE_UV, color: col });
                        m.vertices.push(egui::epaint::Vertex { pos: pos2(xx, r.bottom()), uv: egui::epaint::WHITE_UV, color: col });
                    }
                    m.indices.extend([0, 1, 2, 1, 3, 2]);
                    sp.add(egui::Shape::mesh(m));
                };
                if self.shell.photos.strip_x > 2.0 {
                    fade(strip.left(), 0.9, 0.0);
                }
                if self.shell.photos.strip_x < max_x - 2.0 {
                    fade(strip.right() - 26.0, 0.0, 0.9);
                }
            }
        }

        // side panel: info or edit (a raised card, same as every other surface)
        if info_w > 4.0 {
            let pp = p.with_clip_rect(panel.expand(30.0).intersect(fr.stage.expand(30.0)));
            elevate(&pp, &th, panel, 18.0, 2);
            grad_fill(&pp, panel, 18.0, th.card_top, th.card_bottom);
            rim(&pp, panel, 18.0, 1.2, th.rim, true, 60.0);
            let inner = Rect::from_min_max(panel.min + vec2(16.0, 14.0), panel.max - vec2(16.0, 14.0));
            if self.shell.photos.edit {
                self.draw_photo_edit(ui, inner, &pp);
            } else {
                self.draw_photo_info(ui, inner, &pp);
            }
        }
    }

    fn draw_photo_info(&mut self, ui: &mut Ui, r: Rect, p: &egui::Painter) {
        let th = self.th.clone();
        let Some(cur) = &self.shell.photos.cur else { return };
        let ent = self.be.stat(&cur.loc).ok();
        p.text(pos2(r.left(), r.top() + 10.0), Align2::LEFT_CENTER, "Info", f_bold(T_TITLE), th.ink1);
        let mut y = r.top() + 40.0;
        let hr = Rect::from_min_size(pos2(r.left(), y), vec2(r.width(), 70.0));
        well(p, &th, hr, 12.0);
        let bw = (hr.width() - 16.0) / 48.0;
        for (i, v) in cur.hist.iter().enumerate() {
            let h = (v.sqrt() * (hr.height() - 14.0)).max(1.5);
            let b = Rect::from_min_max(pos2(hr.left() + 8.0 + i as f32 * bw, hr.bottom() - 7.0 - h), pos2(hr.left() + 8.0 + (i as f32 + 0.8) * bw, hr.bottom() - 7.0));
            p.rect_filled(b, cr(1.5), alpha(mix(th.accent, th.ink2, i as f32 / 60.0), 0.9));
        }
        y += 86.0;
        let mut kv = vec![("Name", cur.loc.name()), ("Dimensions", format!("{} \u{d7} {} px (preview)", cur.w, cur.h)), ("Size", format_size(ent.as_ref().map(|e| e.size).unwrap_or(cur.bytes))), ("Kind", ent.as_ref().map(|e| e.kind_label()).unwrap_or_default()), ("Modified", ent.as_ref().and_then(|e| e.modified).map(format_full_date).unwrap_or_default()), ("Where", cur.loc.parent().map(|l| self.pretty_path(&l)).unwrap_or_default())];
        let tags = self.lib(|l| l.tags_of(&cur.loc).iter().map(|t| t.name.clone()).collect::<Vec<_>>());
        if !tags.is_empty() {
            kv.push(("Tags", tags.join(", ")));
        }
        for (k, v) in kv {
            p.text(pos2(r.left(), y + 8.0), Align2::LEFT_CENTER, k, f_semi(T_META), th.ink3);
            let t = self.fit(p, &v, &f_medium(T_BODY), r.width());
            p.text(pos2(r.left(), y + 26.0), Align2::LEFT_CENTER, t, f_medium(T_BODY), th.ink1);
            y += 46.0;
        }
        let b = Rect::from_min_size(pos2(r.left(), r.bottom() - 36.0), vec2(r.width(), 34.0));
        if self.label_button(ui, b, Some("gallery-wide"), "Set as wallpaper", "ph-wall", Btn::Tonal).clicked() {
            self.push_act(Act::WallpaperFrom(cur.loc.clone()));
        }
    }

    fn draw_photo_edit(&mut self, ui: &mut Ui, r: Rect, p: &egui::Painter) {
        let th = self.th.clone();
        p.text(pos2(r.left(), r.top() + 10.0), Align2::LEFT_CENTER, "Edit", f_bold(T_TITLE), th.ink1);
        let cmp = Rect::from_min_size(pos2(r.right() - 168.0, r.top() - 4.0), vec2(104.0, 30.0));
        let resp = self.hit(ui, cmp, "ph-compare", Sense::click_and_drag());
        self.shell.photos.compare = resp.is_pointer_button_down_on();
        if self.shell.photos.compare {
            well(p, &th, cmp, 15.0);
        } else {
            raised(p, &th, cmp, 15.0, resp.hovered(), false);
        }
        p.text(cmp.center(), Align2::CENTER_CENTER, "Hold: original", f_semi(10.5), th.ink1);
        let rst = Rect::from_min_size(pos2(r.right() - 56.0, r.top() - 4.0), vec2(56.0, 30.0));
        if self.label_button(ui, rst, None, "Reset", "ph-reset", Btn::Quiet).clicked() {
            let ph = &mut self.shell.photos;
            ph.adj = Adjust::default();
            ph.preset = 0;
            ph.dirty = true;
            ph.rot = 0;
            ph.flip_h = false;
            ph.flip_v = false;
        }
        let mut y = r.top() + 40.0;
        p.text(pos2(r.left(), y + 6.0), Align2::LEFT_CENTER, "FILTERS", f_semi(10.0), th.ink4);
        y += 20.0;
        let tw_ = (r.width() - 3.0 * 8.0) / 4.0;
        let n_presets = self.shell.photos.cur.as_ref().map(|c| c.presets.len()).unwrap_or(0);
        for i in 0..n_presets.min(PRESETS.len()) {
            let rc = Rect::from_min_size(pos2(r.left() + (i % 4) as f32 * (tw_ + 8.0), y + (i / 4) as f32 * (tw_ * 0.85 + 26.0)), vec2(tw_, tw_ * 0.85));
            let resp = self.hit(ui, rc, ("ph-preset", i), Sense::click());
            let on = self.shell.photos.preset == i;
            let a = spring_to(ui.ctx(), ("ph-pset", i), if on { 1.0 } else { 0.0 }, 300.0, 22.0);
            let tex = self.shell.photos.cur.as_ref().map(|c| c.presets[i].id()).unwrap();
            let rr = rc.expand(1.5 * a);
            textured_rrect(p, rr, 9.0, tex, cover_uv(1.0, 1.0, rr), alpha(Color32::WHITE, if resp.hovered() || on { 1.0 } else { 0.88 }));
            if on {
                p.rect_stroke(rr, cr(9.0), egui::Stroke::new(2.0, th.accent), egui::StrokeKind::Outside);
            }
            p.text(pos2(rc.center().x, rc.bottom() + 12.0), Align2::CENTER_CENTER, PRESETS[i].0, if on { f_semi(10.5) } else { f_medium(10.5) }, if on { th.ink1 } else { th.ink3 });
            if resp.clicked() {
                let ph = &mut self.shell.photos;
                ph.preset = i;
                ph.adj = PRESETS[i].1;
                ph.dirty = true;
            }
        }
        y += 2.0 * (tw_ * 0.85 + 26.0) + 8.0;
        p.text(pos2(r.left(), y + 6.0), Align2::LEFT_CENTER, "ADJUST", f_semi(10.0), th.ink4);
        y += 24.0;
        let sliders: [(&str, &str, fn(&mut Adjust) -> &mut f32); 5] = [("Exposure", "sun-2", |a| &mut a.exposure), ("Contrast", "tuning-2", |a| &mut a.contrast), ("Saturation", "palette", |a| &mut a.saturation), ("Warmth", "bolt", |a| &mut a.warmth), ("Vignette", "layers-minimalistic", |a| &mut a.vignette)];
        for (label, icon, f) in sliders {
            let mut adj = self.shell.photos.adj;
            let v = *f(&mut adj);
            let signed = label != "Vignette";
            let track = Rect::from_min_size(pos2(r.left() + 4.0, y + 24.0), vec2(r.width() - 8.0, 8.0));
            let t0 = if signed { v * 0.5 + 0.5 } else { v };
            let (nt, resp) = self.slider(ui, track, t0, ("ph-adj", label));
            if resp.double_clicked() {
                *f(&mut adj) = 0.0;
            } else if resp.dragged() || resp.clicked() {
                *f(&mut adj) = if signed { nt * 2.0 - 1.0 } else { nt };
            }
            if adj != self.shell.photos.adj {
                let ph = &mut self.shell.photos;
                ph.adj = adj;
                ph.dirty = true;
                ph.preset = PRESETS.iter().position(|(_, a)| *a == adj).unwrap_or(usize::MAX);
            }
            self.ico(p, icon, Ic::Outline, pos2(r.left() + 8.0, y + 8.0), 14.0, th.ink3);
            p.text(pos2(r.left() + 24.0, y + 8.0), Align2::LEFT_CENTER, label, f_medium(T_BODY), th.ink1);
            p.text(pos2(r.right(), y + 8.0), Align2::RIGHT_CENTER, format!("{:+.0}", (v * 100.0).round()), f_medium(T_META), th.ink3);
            y += 44.0;
        }
        let b = Rect::from_min_size(pos2(r.left(), r.bottom() - 36.0), vec2(r.width(), 34.0));
        if self.label_button(ui, b, Some("download-minimalistic"), "Save edited copy", "ph-save", Btn::Accent).clicked() {
            self.photos_save();
        }
    }

    // ───────────── settings ─────────────
    pub fn photos_settings(&mut self, ui: &mut Ui, win: Rect) {
        let pages = [("tuning-2", "General"), ("play", "Viewer"), ("gallery", "Library")];
        let rows_for = |app: &SmartApp, page: usize| -> Vec<SRow> {
            let ph = &app.shell.photos;
            match page {
                0 => vec![
                    SRow::Choice { icon: "sort-vertical", label: "Sort photos by", sub: "Order used in every album", opts: &["Newest", "Oldest", "Name", "Size"], sel: ph.sort },
                    SRow::Choice { icon: "crop-minimalistic", label: "Thumbnails", sub: "Fill crops to a tile, Fit shows the whole photo", opts: &["Fill", "Fit"], sel: if ph.fill { 0 } else { 1 } },
                    SRow::Toggle { icon: "text", label: "Always show captions", sub: "File name under each thumbnail", on: ph.captions },
                    SRow::Slider { icon: "gallery", label: "Thumbnail size", sub: "Larger tiles, fewer columns", v: ph.tile, lo: 120.0, hi: 340.0, unit: " px" },
                    SRow::Slider { icon: "widget", label: "Grid spacing", sub: "Gap between thumbnails", v: ph.gap, lo: 4.0, hi: 24.0, unit: " px" },
                ],
                1 => vec![
                    SRow::Slider { icon: "play", label: "Slideshow interval", sub: "Seconds each photo stays on screen", v: ph.slide_secs, lo: 2.0, hi: 10.0, unit: " s" },
                    SRow::Toggle { icon: "layers-minimalistic", label: "Show filmstrip", sub: "Scrollable strip under the photo", on: ph.filmstrip },
                    SRow::Toggle { icon: "tuning-2", label: "Show tool bar", sub: "Rotate, flip and zoom controls", on: ph.tools },
                    SRow::Toggle { icon: "magnifer-zoom-in", label: "Scroll to zoom", sub: "Mouse wheel and pinch zoom the photo", on: ph.wheel_zoom },
                ],
                _ => vec![
                    SRow::Button { icon: "refresh", label: "Rescan library", sub: "Look for new and removed pictures", cta: "Rescan" },
                    SRow::Button { icon: "heart", label: "Clear favorites", sub: "Remove the heart from every photo", cta: "Clear" },
                    SRow::Button { icon: "trash-bin-minimalistic", label: "Reset thumbnails", sub: "Drop the thumbnail cache and rebuild", cta: "Reset" },
                ],
            }
        };
        if let Some((page, row, val)) = self.draw_app_settings(ui, win, crate::shell::AppId::Photos, &pages, &rows_for) {
            let ph = &mut self.shell.photos;
            match (page, row, val) {
                (0, 0, SVal::Idx(i)) => ph.sort = i,
                (0, 1, SVal::Idx(i)) => ph.fill = i == 0,
                (0, 2, SVal::Bool(b)) => ph.captions = b,
                (0, 3, SVal::F(v)) => ph.tile = v,
                (0, 4, SVal::F(v)) => ph.gap = v,
                (1, 0, SVal::F(v)) => ph.slide_secs = v.round(),
                (1, 1, SVal::Bool(b)) => ph.filmstrip = b,
                (1, 2, SVal::Bool(b)) => ph.tools = b,
                (1, 3, SVal::Bool(b)) => ph.wheel_zoom = b,
                (2, 0, _) => {
                    ph.scanned = false;
                    self.toast("Library rescanned", true);
                }
                (2, 1, _) => {
                    ph.favs.clear();
                    self.toast("Favorites cleared", true);
                }
                (2, 2, _) => {
                    ph.thumbs.clear();
                    ph.failed.clear();
                    self.toast("Thumbnails will rebuild", true);
                }
                _ => {}
            }
        }
    }
}

const SORTS: [&str; 4] = ["Newest first", "Oldest first", "Name", "Size"];

fn smooth01(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
