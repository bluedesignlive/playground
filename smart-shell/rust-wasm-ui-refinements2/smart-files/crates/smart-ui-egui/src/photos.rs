//! Smart Photos — library grid, albums, a viewer with spring zoom/pan, filmstrip, crossfading slideshow and a
//! non-destructive-feeling editor (live adjustments, filter presets with previews, compare, histogram, save a copy).
//! Pure Rust: images are decoded with the `image` crate and edited on the CPU.
use crate::app::*;
use crate::chrome::*;
use crate::icons::Ic;
use crate::paint::*;
use crate::theme::*;
use egui::{pos2, vec2, Align2, Color32, Id, Rect, Sense, TextureHandle, TextureOptions, Ui, Vec2};
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

    pub fn photos_cmd(&mut self, c: &str) {
        let cur = self.shell.photos.open.clone();
        match c {
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
            "slideshow" => {
                if self.shell.photos.open.is_none() {
                    if let Some(e) = self.shell.photos.all.first().cloned() {
                        self.photos_show(e.loc);
                    }
                }
                self.shell.photos.slideshow = !self.shell.photos.slideshow;
                self.shell.photos.slide_t = self.time;
            }
            "info" => self.shell.photos.info = !self.shell.photos.info,
            "edit" => self.shell.photos.edit = !self.shell.photos.edit,
            "reveal" => {
                if let Some(l) = cur.and_then(|l| l.parent()) {
                    self.acts.push(Act::OpenApp(crate::shell::AppId::Files));
                    self.acts.push(Act::Nav(Route::Loc(l)));
                }
            }
            "wallpaper" => {
                if let Some(l) = cur {
                    self.acts.push(Act::WallpaperFrom(l));
                }
            }
            "save" => self.photos_save(),
            _ => {}
        }
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

    pub fn draw_photos(&mut self, ui: &mut Ui, rect: Rect, active: bool) {
        let ctx = ui.ctx().clone();
        let th = self.th.clone();
        if !self.shell.photos.scanned {
            self.photos_scan();
        }
        let p = ui.painter().clone();
        // current album list
        let list: Vec<Entry> = {
            let ph = &self.shell.photos;
            ph.all.iter().filter(|e| match &ph.album {
                Album::All => true,
                Album::Favorites => ph.favs.contains(&e.loc),
                Album::Folder(f) => e.loc.parent().as_ref() == Some(f),
            }).cloned().collect()
        };
        // albums
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
        let viewer = self.shell.photos.open.is_some();
        let vs = spring_to(&ctx, "ph-viewer", if viewer { 1.0 } else { 0.0 }, 200.0, 26.0).clamp(0.0, 1.0);
        let side_w = if rect.width() > 700.0 { 196.0 } else { 0.0 };

        // ── library sidebar ──
        if side_w > 0.0 {
            let side = Rect::from_min_size(rect.min + vec2(10.0, 8.0), vec2(side_w - 14.0, rect.height() - 16.0));
            p.text(pos2(side.left() + 10.0, side.top() + 12.0), Align2::LEFT_CENTER, "LIBRARY", f_semi(10.0), th.ink4);
            let mut y = side.top() + 28.0;
            let mut rows: Vec<(String, &'static str, usize, Album)> = vec![("All photos".into(), "gallery", self.shell.photos.all.len(), Album::All), ("Favorites".into(), "heart", self.shell.photos.favs.len(), Album::Favorites)];
            let mut folder_rows: Vec<(String, &'static str, usize, Album)> = folders.iter().take(8).map(|(l, n)| (self.loc_title(l), "folder", *n, Album::Folder(l.clone()))).collect();
            for (i, (name, icon, n, album)) in rows.drain(..).chain(std::iter::once(("".into(), "", 0, Album::All))).chain(folder_rows.drain(..)).enumerate() {
                if name.is_empty() {
                    y += 8.0;
                    p.text(pos2(side.left() + 10.0, y + 12.0), Align2::LEFT_CENTER, "FOLDERS", f_semi(10.0), th.ink4);
                    y += 26.0;
                    continue;
                }
                let r = Rect::from_min_size(pos2(side.left(), y), vec2(side.width(), 34.0));
                let on = self.shell.photos.album == album;
                let resp = self.hit(ui, r, ("ph-album", i), Sense::click());
                let a = self.anim(ui, ("ph-album-h", i), resp.hovered());
                if on {
                    grad_fill(&p, r, 11.0, th.accent_tint, th.accent_tint2);
                    rim(&p, r, 11.0, 1.0, alpha(Color32::WHITE, if th.dark { 0.1 } else { 0.7 }), true, 14.0);
                } else if a > 0.01 {
                    p.rect_filled(r, cr(11.0), alpha(th.row_hover, a));
                }
                self.ico(&p, icon, if on || a > 0.5 { Ic::Duo } else { Ic::Outline }, pos2(r.left() + 20.0, r.center().y), 16.0, if on { th.accent_ink } else { th.ink3 });
                p.text(pos2(r.left() + 38.0, r.center().y), Align2::LEFT_CENTER, self.fit(&p, &name, &f_medium(T_BODY), r.width() - 78.0), if on { f_semi(T_BODY) } else { f_medium(T_BODY) }, if on { th.ink1 } else { th.ink2 });
                p.text(pos2(r.right() - 10.0, r.center().y), Align2::RIGHT_CENTER, n.to_string(), f_medium(T_META), th.ink4);
                if resp.clicked() {
                    self.shell.photos.album = album;
                }
                y += 36.0;
            }
        }

        // ── grid ──
        let grid = Rect::from_min_max(pos2(rect.left() + side_w, rect.top()), rect.max);
        let tb = Rect::from_min_size(grid.min + vec2(8.0, 6.0), vec2(grid.width() - 22.0, 44.0));
        let title = match &self.shell.photos.album {
            Album::All => "All photos".to_string(),
            Album::Favorites => "Favorites".to_string(),
            Album::Folder(l) => self.loc_title(l),
        };
        p.text(pos2(tb.left() + 4.0, tb.center().y - 7.0), Align2::LEFT_CENTER, &title, f_bold(T_TITLE + 2.0), th.ink1);
        let total: u64 = list.iter().map(|e| e.size).sum();
        p.text(pos2(tb.left() + 4.0, tb.center().y + 12.0), Align2::LEFT_CENTER, format!("{} photo{} \u{b7} {}", list.len(), if list.len() == 1 { "" } else { "s" }, format_size(total)), f_medium(T_META), th.ink3);
        // controls (right to left)
        let mut rx = tb.right();
        let ss = Rect::from_min_size(pos2(rx - 118.0, tb.center().y - 17.0), vec2(118.0, 34.0));
        if rect.width() > 560.0 {
            if self.label_button(ui, ss, Some("play"), "Slideshow", "ph-slide", Btn::Tonal).clicked() {
                self.push_act(Act::App("photos.slideshow".into()));
            }
            rx = ss.left() - 10.0;
        }
        let slider = Rect::from_min_size(pos2(rx - 120.0, tb.center().y - 6.0), vec2(120.0, 12.0));
        if rect.width() > 640.0 {
            let resp = self.hit(ui, slider.expand2(vec2(8.0, 10.0)), "ph-size", Sense::click_and_drag());
            if resp.dragged() || resp.clicked() {
                if let Some(pp) = resp.interact_pointer_pos() {
                    self.shell.photos.tile = 120.0 + ((pp.x - slider.left()) / slider.width()).clamp(0.0, 1.0) * 200.0;
                }
            }
            well(&p, &th, slider, 6.0);
            let f = (self.shell.photos.tile - 120.0) / 200.0;
            let fill = Rect::from_min_size(slider.min, vec2((slider.width() * f).max(12.0), 12.0));
            let (c0, c1) = th.accent_grad();
            grad_fill(&p, fill, 6.0, c0, c1);
            let knob = Rect::from_center_size(pos2(slider.left() + slider.width() * f, slider.center().y), vec2(18.0, 18.0) * if resp.dragged() { 1.15 } else { 1.0 });
            raised(&p, &th, knob, 9.0, resp.hovered(), resp.dragged());
            self.ico(&p, "gallery", Ic::Outline, pos2(slider.left() - 16.0, slider.center().y), 14.0, th.ink3);
        }
        let tile = spring_to(&ctx, "ph-tile", self.shell.photos.tile, 180.0, 22.0).clamp(100.0, 340.0);
        let area = Rect::from_min_max(pos2(grid.left() + 8.0, tb.bottom() + 4.0), pos2(grid.right() - 8.0, grid.bottom() - 6.0));
        if list.is_empty() {
            let c = area.center();
            self.ico(&p, "heart", Ic::Duo, c - vec2(0.0, 26.0), 44.0, alpha(th.ink4, 0.7));
            p.text(c + vec2(0.0, 16.0), Align2::CENTER_CENTER, if self.shell.photos.album == Album::Favorites { "No favorites yet" } else { "No photos here" }, f_semi(T_TITLE), th.ink2);
            p.text(c + vec2(0.0, 38.0), Align2::CENTER_CENTER, if self.shell.photos.album == Album::Favorites { "Tap the heart on any photo to keep it close." } else { "Images from your folders appear here." }, f_medium(T_BODY), th.ink3);
        } else {
            let gap = 12.0;
            let cols = (((area.width() - 6.0 + gap) / (tile + gap)).floor() as usize).max(1);
            let cw = ((area.width() - 6.0 - gap * (cols as f32 - 1.0)) / cols as f32).floor();
            let ch = (cw * 0.78).round();
            let rows = list.len().div_ceil(cols);
            let content_h = rows as f32 * (ch + gap) + 8.0;
            let mut cui = ui.new_child(egui::UiBuilder::new().max_rect(area).layout(egui::Layout::top_down(egui::Align::Min)));
            cui.set_clip_rect(area.intersect(ui.clip_rect()));
            let mut budget = 2u32;
            let mut open_loc: Option<Location> = None;
            let mut fav_toggle: Option<Location> = None;
            egui::ScrollArea::vertical().id_salt("photos-grid").auto_shrink([false, false]).show_viewport(&mut cui, |ui, vp| {
                let top = ui.max_rect().top();
                ui.allocate_space(vec2(area.width() - 12.0, content_h));
                let p = ui.painter().clone();
                for (i, e) in list.iter().enumerate() {
                    let (rw, cl) = (i / cols, i % cols);
                    let y = rw as f32 * (ch + gap);
                    if y + ch < vp.top() - 40.0 || y > vp.bottom() + 40.0 {
                        continue;
                    }
                    let base = Rect::from_min_size(pos2(area.left() + 3.0 + cl as f32 * (cw + gap), top + y + 4.0), vec2(cw, ch));
                    let resp = self.hit(ui, base, ("ph-card", &e.loc), Sense::click());
                    let hov = resp.hovered() && !viewer;
                    let a = spring_to(ui.ctx(), ("ph-h", &e.loc), if hov { 1.0 } else { 0.0 }, 300.0, 24.0);
                    let r = base.expand2(vec2(2.5, 2.5) * a).translate(vec2(0.0, -3.0 * a));
                    elevate(&p, &th, r, 14.0, if a > 0.05 { 3 } else { 2 });
                    match self.photos_thumb(ui.ctx(), &e.loc, &mut budget) {
                        Some((tex, sz)) => {
                            let uv = cover_uv(sz[0], sz[1], r);
                            textured_rrect(&p, r, 14.0, tex, uv, Color32::WHITE);
                        }
                        None => {
                            grad_fill(&p, r, 14.0, alpha(th.accent, 0.12), alpha(th.accent, 0.04));
                            self.ico(&p, "gallery", Ic::Duo, r.center(), 26.0, alpha(th.ink4, 0.6));
                            ui.ctx().request_repaint();
                        }
                    }
                    rim(&p, r, 14.0, 1.2, alpha(Color32::WHITE, if th.dark { 0.16 } else { 0.6 }), true, 26.0);
                    if a > 0.02 {
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
                    if resp.clicked() && !matches!(fav_toggle, Some(ref l) if *l == e.loc) {
                        open_loc = Some(e.loc.clone());
                    }
                }
            });
            if let Some(l) = fav_toggle {
                if !self.shell.photos.favs.remove(&l) {
                    self.shell.photos.favs.insert(l);
                }
                open_loc = None;
            }
            if let Some(l) = open_loc {
                self.shell.photos.open = Some(l);
                self.photos_reset_view();
            }
        }

        // ── viewer overlay ──
        if vs > 0.01 {
            self.photos_load_cur(&ctx);
            if self.shell.photos.dirty {
                self.photos_reprocess(&ctx);
            }
            self.draw_photo_viewer(ui, rect, vs, active, &list);
        }
    }

    fn draw_photo_viewer(&mut self, ui: &mut Ui, rect: Rect, vs: f32, active: bool, list: &[Entry]) {
        let ctx = ui.ctx().clone();
        let th = self.th.clone();
        let mut vui = ui.new_child(egui::UiBuilder::new().max_rect(rect));
        vui.set_opacity(vs);
        let ui = &mut vui;
        let p = ui.painter().clone();
        let stage_bg = Rect::from_min_max(rect.min + vec2(0.0, 0.0), rect.max);
        let cr_ = R_WIN - 2.0;
        let bgp = p.clone();
        // deep stage that swallows clicks behind it
        ui.allocate_rect(stage_bg, Sense::click_and_drag());
        {
            let r = Rect::from_min_max(stage_bg.min, stage_bg.max);
            let round = egui::CornerRadius { nw: 0, ne: 0, sw: cr_ as u8, se: cr_ as u8 };
            bgp.add(egui::epaint::RectShape::filled(r, round, hex("#080c14")));
            bgp.add(egui::epaint::RectShape::filled(r, round, alpha(th.accent, 0.04)));
        }
        let info_w = 300.0 * spring_to(&ctx, "ph-info", if self.shell.photos.info || self.shell.photos.edit { 1.0 } else { 0.0 }, 220.0, 26.0).clamp(0.0, 1.0);
        let panel = Rect::from_min_max(pos2(rect.right() - info_w, rect.top()), rect.max);
        let stage = Rect::from_min_max(pos2(rect.left(), rect.top() + 54.0), pos2(rect.right() - info_w, rect.bottom() - 118.0));

        // keys
        let mut nav = 0;
        let mut close = false;
        if active {
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
                    match t.as_str() {
                        "i" => self.shell.photos.info = !self.shell.photos.info,
                        "e" => self.shell.photos.edit = !self.shell.photos.edit,
                        "f" => {
                            if let Some(l) = self.shell.photos.open.clone() {
                                if !self.shell.photos.favs.remove(&l) {
                                    self.shell.photos.favs.insert(l);
                                }
                            }
                        }
                        "r" => self.shell.photos.rot += 1,
                        "0" => {
                            self.shell.photos.zoom = 1.0;
                            self.shell.photos.pan = Vec2::ZERO
                        }
                        "s" => {
                            self.shell.photos.slideshow = !self.shell.photos.slideshow;
                            self.shell.photos.slide_t = self.time;
                        }
                        "+" | "=" => self.shell.photos.zoom = (self.shell.photos.zoom * 1.25).min(10.0),
                        "-" => self.shell.photos.zoom = (self.shell.photos.zoom / 1.25).max(0.3),
                        _ => {}
                    }
                }
            }
        }
        if self.shell.photos.slideshow && self.time - self.shell.photos.slide_t > 3.6 {
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
        let spin = spring_to(&ctx, "ph-rot", self.shell.photos.rot as f32, 160.0, 18.0);
        let _ = spin;
        if let Some(cur) = &self.shell.photos.cur {
            let (w, h) = if rot.rem_euclid(2) == 1 { (cur.h as f32, cur.w as f32) } else { (cur.w as f32, cur.h as f32) };
            let fit = (stage.width() / w).min(stage.height() / h).min(1.6) * 0.98;
            let size = vec2(w, h) * fit * zoom;
            let ctr = stage.center() + self.shell.photos.pan;
            let r = Rect::from_center_size(ctr, size);
            let sp = p.with_clip_rect(stage.expand(2.0));
            // crossfade from the previous photo (slideshow / arrows)
            let t = self.shell.photos.prev.as_ref().map(|(_, t0)| ((self.time - t0) / 0.55) as f32).unwrap_or(1.0);
            if t < 1.0 {
                ctx.request_repaint();
                if let Some((pt, _)) = &self.shell.photos.prev {
                    let pr = Rect::from_center_size(stage.center(), vec2(stage.width() * 0.9, stage.height() * 0.9));
                    draw_tex(&sp, pt.id(), pr, 0, false, false, alpha(Color32::WHITE, 1.0 - t));
                }
            }
            shadow(&sp, r, 4.0, 14.0, 40.0, -6.0, alpha(Color32::BLACK, 0.6 * smooth01(t)));
            let tex = if compare { cur.orig.id() } else { cur.tex.id() };
            draw_tex(&sp, tex, r, rot, fh, fv, alpha(Color32::WHITE, smooth01(t)));
            if compare {
                let b = Rect::from_center_size(pos2(r.center().x, r.top() + 22.0), vec2(86.0, 26.0));
                grad_fill(&p, b, 13.0, alpha(Color32::BLACK, 0.6), alpha(Color32::BLACK, 0.5));
                p.text(b.center(), Align2::CENTER_CENTER, "ORIGINAL", f_bold(10.0), Color32::WHITE);
            }
            // pan / zoom interactions
            let resp = ui.interact(stage, Id::new("ph-stage"), Sense::click_and_drag());
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
            if resp.hovered() {
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
                p.circle_filled(stage.center() + vec2((k as f32 - 1.0) * 18.0, 0.0), 4.0 + 2.0 * (t * 6.0 + k as f32).sin(), alpha(Color32::WHITE, 0.6));
            }
            ctx.request_repaint();
        }

        // arrows
        for (dir, icon) in [(-1, "alt-arrow-left"), (1, "alt-arrow-right")] {
            let c = pos2(if dir < 0 { stage.left() + 34.0 } else { stage.right() - 34.0 }, stage.center().y);
            let r = Rect::from_center_size(c, vec2(44.0, 44.0));
            let near = ctx.pointer_hover_pos().is_some_and(|pp| (pp.x - c.x).abs() < 120.0 && stage.contains(pp));
            let a = spring_to(&ctx, ("ph-arrow", dir), if near { 1.0 } else { 0.0 }, 260.0, 24.0).clamp(0.0, 1.0);
            if a > 0.02 {
                let resp = self.hit(ui, r, ("ph-nav", dir), Sense::click());
                p.circle_filled(r.center(), 22.0 * (0.9 + 0.1 * a), alpha(Color32::BLACK, 0.45 * a));
                p.circle_stroke(r.center(), 22.0 * (0.9 + 0.1 * a), egui::Stroke::new(1.0, alpha(Color32::WHITE, 0.2 * a)));
                self.ico(&p, icon, if resp.hovered() { Ic::Duo } else { Ic::Outline }, r.center(), 20.0, alpha(Color32::WHITE, a));
                if resp.clicked() {
                    self.photos_nav(list, dir);
                }
            }
        }

        // top bar
        let name = self.shell.photos.open.as_ref().map(|l| l.name()).unwrap_or_default();
        let idx = list.iter().position(|e| Some(&e.loc) == self.shell.photos.open.as_ref()).map(|i| i + 1).unwrap_or(0);
        let back = Rect::from_min_size(rect.min + vec2(14.0, 10.0), vec2(104.0, 34.0));
        if self.label_button(ui, back, Some("alt-arrow-left"), "Library", "ph-back", Btn::Raised).clicked() || close {
            self.shell.photos.open = None;
            self.shell.photos.slideshow = false;
            self.shell.photos.edit = false;
            self.shell.photos.info = false;
        }
        p.text(pos2(stage.center().x, rect.top() + 22.0), Align2::CENTER_CENTER, self.fit(&p, &name, &f_semi(T_BODY), stage.width() - 380.0), f_semi(T_BODY), Color32::WHITE);
        p.text(pos2(stage.center().x, rect.top() + 40.0), Align2::CENTER_CENTER, format!("{idx} of {}", list.len()), f_medium(T_META), hex("#8c98b3"));
        let mut rx = stage.right() - 12.0;
        let fav = self.shell.photos.open.as_ref().is_some_and(|l| self.shell.photos.favs.contains(l));
        let btns: [(&'static str, &str, bool, &str); 3] = [("tuning-2", "ph-b-edit", self.shell.photos.edit, "Edit  E"), ("info-circle", "ph-b-info", self.shell.photos.info, "Info  I"), ("heart", "ph-b-fav", fav, "Favorite  F")];
        for (icon, id, on, tip) in btns {
            let r = Rect::from_center_size(pos2(rx - 17.0, rect.top() + 27.0), vec2(34.0, 34.0));
            let resp = self.hit(ui, r, id, Sense::click());
            let a = self.anim(ui, (id, "h"), resp.hovered());
            if on && id != "ph-b-fav" {
                raised_accent(&p, &th, r, 17.0, resp.hovered(), resp.is_pointer_button_down_on());
            } else {
                grad_fill(&p, r, 17.0, alpha(Color32::WHITE, 0.10 + 0.08 * a), alpha(Color32::WHITE, 0.04 + 0.05 * a));
                rim(&p, r, 17.0, 1.0, alpha(Color32::WHITE, 0.16), true, 16.0);
            }
            let col = if on && id == "ph-b-fav" { hex("#ff6b8a") } else if on { th.on_accent() } else { Color32::WHITE };
            self.ico(&p, icon, if on || a > 0.5 { if id == "ph-b-fav" && on { Ic::Filled } else { Ic::Duo } } else { Ic::Outline }, r.center(), 17.0, col);
            resp.clone().on_hover_text(tip);
            if resp.clicked() {
                match id {
                    "ph-b-edit" => {
                        self.shell.photos.edit = !self.shell.photos.edit;
                        if self.shell.photos.edit {
                            self.shell.photos.info = false;
                        }
                    }
                    "ph-b-info" => {
                        self.shell.photos.info = !self.shell.photos.info;
                        if self.shell.photos.info {
                            self.shell.photos.edit = false;
                        }
                    }
                    _ => {
                        if let Some(l) = self.shell.photos.open.clone() {
                            if !self.shell.photos.favs.remove(&l) {
                                self.shell.photos.favs.insert(l);
                            }
                        }
                    }
                }
            }
            rx -= 42.0;
        }

        // tool pill
        let tools: [(&'static str, &str); 7] = [("rotate-left", "rotl"), ("restart", "rotr"), ("flip-horizontal", "fliph"), ("flip-vertical", "flipv"), ("magnifer-zoom-in", "zin"), ("full-screen", "fit"), (if self.shell.photos.slideshow { "pause" } else { "play" }, "slide")];
        let pw = tools.len() as f32 * 40.0 + 20.0;
        let pill = Rect::from_center_size(pos2(stage.center().x, rect.bottom() - 100.0), vec2(pw, 44.0));
        shadow(&p, pill, 22.0, 6.0, 18.0, -4.0, alpha(Color32::BLACK, 0.5));
        grad_fill(&p, pill, 22.0, alpha(hex("#232c40"), 0.92), alpha(hex("#161d2d"), 0.94));
        rim(&p, pill, 22.0, 1.2, alpha(Color32::WHITE, 0.22), true, 40.0);
        for (i, (icon, id)) in tools.iter().enumerate() {
            let r = Rect::from_center_size(pill.left_center() + vec2(30.0 + i as f32 * 40.0, 0.0), vec2(34.0, 34.0));
            let resp = self.hit(ui, r, ("ph-tool", *id), Sense::click());
            let a = self.anim(ui, ("ph-tool-h", *id), resp.hovered());
            if a > 0.01 {
                p.circle_filled(r.center(), 16.0, alpha(Color32::WHITE, 0.10 * a));
            }
            let flipped = (*id == "fliph" && fh) || (*id == "flipv" && fv);
            self.ico(&p, icon, if a > 0.5 || flipped { Ic::Duo } else { Ic::Outline }, r.center(), 18.0, if flipped { th.accent } else { mix(hex("#aab6d0"), Color32::WHITE, a) });
            if resp.clicked() {
                let ph = &mut self.shell.photos;
                match *id {
                    "rotl" => ph.rot -= 1,
                    "rotr" => ph.rot += 1,
                    "fliph" => ph.flip_h = !ph.flip_h,
                    "flipv" => ph.flip_v = !ph.flip_v,
                    "zin" => ph.zoom = (ph.zoom * 1.4).min(10.0),
                    "fit" => {
                        ph.zoom = 1.0;
                        ph.pan = Vec2::ZERO;
                    }
                    _ => {
                        ph.slideshow = !ph.slideshow;
                        ph.slide_t = self.time;
                    }
                }
            }
        }
        // slideshow progress
        if self.shell.photos.slideshow {
            let f = (((self.time - self.shell.photos.slide_t) / 3.6) as f32).clamp(0.0, 1.0);
            let b = Rect::from_min_size(pill.left_bottom() + vec2(14.0, 4.0), vec2(pill.width() - 28.0, 3.0));
            p.rect_filled(b, cr(1.5), alpha(Color32::WHITE, 0.12));
            p.rect_filled(Rect::from_min_size(b.min, vec2(b.width() * f, 3.0)), cr(1.5), th.accent);
        }

        // filmstrip
        let strip = Rect::from_min_max(pos2(rect.left() + 12.0, rect.bottom() - 62.0), pos2(rect.right() - info_w - 12.0, rect.bottom() - 10.0));
        let (tw_, gap) = (62.0f32, 8.0f32);
        let cur_i = idx.saturating_sub(1);
        let target = strip.center().x - (cur_i as f32 * (tw_ + gap) + tw_ / 2.0);
        let off = spring_to(&ctx, "ph-strip", target, 160.0, 22.0);
        let sp = p.with_clip_rect(strip);
        let mut budget = 2u32;
        let mut go: Option<Location> = None;
        for (i, e) in list.iter().enumerate() {
            let x = off + i as f32 * (tw_ + gap);
            if x + tw_ < strip.left() - 4.0 || x > strip.right() + 4.0 {
                continue;
            }
            let on = i == cur_i;
            let s = spring_to(&ctx, ("ph-fs", &e.loc), if on { 1.0 } else { 0.0 }, 300.0, 22.0);
            let r = Rect::from_min_size(pos2(x, strip.top() + 4.0), vec2(tw_, strip.height() - 8.0)).expand2(vec2(2.0, 4.0) * s);
            let resp = ui.interact(r.intersect(strip), Id::new(("ph-film", i)), Sense::click());
            match self.photos_thumb(&ctx, &e.loc, &mut budget) {
                Some((tex, sz)) => textured_rrect(&sp, r, 9.0, tex, cover_uv(sz[0], sz[1], r), alpha(Color32::WHITE, 0.55 + 0.45 * s.clamp(0.0, 1.0) + if resp.hovered() { 0.2 } else { 0.0 })),
                None => grad_fill(&sp, r, 9.0, alpha(Color32::WHITE, 0.1), alpha(Color32::WHITE, 0.04)),
            }
            if on {
                sp.rect_stroke(r, cr(9.0), egui::Stroke::new(2.0, th.accent), egui::StrokeKind::Outside);
            }
            if resp.clicked() && !on {
                go = Some(e.loc.clone());
            }
        }
        if let Some(l) = go {
            let d = list.iter().position(|e| e.loc == l).map(|j| j as i32 - cur_i as i32).unwrap_or(0);
            self.photos_nav(list, d);
        }

        // side panel: info or edit
        if info_w > 4.0 {
            let pp = p.with_clip_rect(panel);
            grad_fill(&pp, Rect::from_min_max(panel.min - vec2(20.0, 0.0), panel.max), 0.0, alpha(hex("#121a2b"), 0.97), alpha(hex("#0d1422"), 0.98));
            pp.line_segment([panel.left_top(), panel.left_bottom()], egui::Stroke::new(1.0, alpha(Color32::WHITE, 0.08)));
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
        p.text(pos2(r.left(), r.top() + 10.0), Align2::LEFT_CENTER, "Info", f_bold(T_TITLE), Color32::WHITE);
        let mut y = r.top() + 40.0;
        // histogram
        let hr = Rect::from_min_size(pos2(r.left(), y), vec2(r.width(), 70.0));
        grad_fill(p, hr, 12.0, alpha(Color32::WHITE, 0.06), alpha(Color32::WHITE, 0.02));
        let bw = (hr.width() - 16.0) / 48.0;
        for (i, v) in cur.hist.iter().enumerate() {
            let h = (v.sqrt() * (hr.height() - 14.0)).max(1.5);
            let b = Rect::from_min_max(pos2(hr.left() + 8.0 + i as f32 * bw, hr.bottom() - 7.0 - h), pos2(hr.left() + 8.0 + (i as f32 + 0.8) * bw, hr.bottom() - 7.0));
            p.rect_filled(b, cr(1.5), alpha(mix(hex("#5b87f0"), hex("#ffffff"), i as f32 / 60.0), 0.9));
        }
        y += 86.0;
        let mut kv = vec![("Name", cur.loc.name()), ("Dimensions", format!("{} \u{d7} {} px (preview)", cur.w, cur.h)), ("Size", format_size(ent.as_ref().map(|e| e.size).unwrap_or(cur.bytes))), ("Kind", ent.as_ref().map(|e| e.kind_label()).unwrap_or_default()), ("Modified", ent.as_ref().and_then(|e| e.modified).map(format_full_date).unwrap_or_default()), ("Where", cur.loc.parent().map(|l| self.pretty_path(&l)).unwrap_or_default())];
        let tags = self.lib(|l| l.tags_of(&cur.loc).iter().map(|t| t.name.clone()).collect::<Vec<_>>());
        if !tags.is_empty() {
            kv.push(("Tags", tags.join(", ")));
        }
        for (k, v) in kv {
            p.text(pos2(r.left(), y + 8.0), Align2::LEFT_CENTER, k, f_semi(T_META), hex("#7d89a3"));
            let t = self.fit(p, &v, &f_medium(T_BODY), r.width());
            p.text(pos2(r.left(), y + 26.0), Align2::LEFT_CENTER, t, f_medium(T_BODY), hex("#e4ecfb"));
            y += 46.0;
        }
        let b = Rect::from_min_size(pos2(r.left(), r.bottom() - 36.0), vec2(r.width(), 34.0));
        if self.label_button(ui, b, Some("gallery-wide"), "Set as wallpaper", "ph-wall", Btn::Tonal).clicked() {
            self.push_act(Act::WallpaperFrom(cur.loc.clone()));
        }
        let _ = th;
    }

    fn draw_photo_edit(&mut self, ui: &mut Ui, r: Rect, p: &egui::Painter) {
        let th = self.th.clone();
        p.text(pos2(r.left(), r.top() + 10.0), Align2::LEFT_CENTER, "Edit", f_bold(T_TITLE), Color32::WHITE);
        // compare + reset
        let cmp = Rect::from_min_size(pos2(r.right() - 168.0, r.top() - 4.0), vec2(104.0, 30.0));
        let resp = self.hit(ui, cmp, "ph-compare", Sense::click_and_drag());
        self.shell.photos.compare = resp.is_pointer_button_down_on();
        grad_fill(p, cmp, 15.0, alpha(Color32::WHITE, if self.shell.photos.compare { 0.2 } else { 0.08 }), alpha(Color32::WHITE, 0.03));
        rim(p, cmp, 15.0, 1.0, alpha(Color32::WHITE, 0.18), true, 14.0);
        p.text(cmp.center(), Align2::CENTER_CENTER, "Hold: original", f_semi(10.5), Color32::WHITE);
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
        p.text(pos2(r.left(), y + 6.0), Align2::LEFT_CENTER, "FILTERS", f_semi(10.0), hex("#7d89a3"));
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
            textured_rrect(p, rr, 9.0, tex, cover_uv(1.0, 1.0, rr), alpha(Color32::WHITE, if resp.hovered() || on { 1.0 } else { 0.85 }));
            if on {
                p.rect_stroke(rr, cr(9.0), egui::Stroke::new(2.0, th.accent), egui::StrokeKind::Outside);
            }
            p.text(pos2(rc.center().x, rc.bottom() + 12.0), Align2::CENTER_CENTER, PRESETS[i].0, if on { f_semi(10.5) } else { f_medium(10.5) }, if on { Color32::WHITE } else { hex("#9aa6c0") });
            if resp.clicked() {
                let ph = &mut self.shell.photos;
                ph.preset = i;
                ph.adj = PRESETS[i].1;
                ph.dirty = true;
            }
        }
        y += 2.0 * (tw_ * 0.85 + 26.0) + 8.0;
        p.text(pos2(r.left(), y + 6.0), Align2::LEFT_CENTER, "ADJUST", f_semi(10.0), hex("#7d89a3"));
        y += 24.0;
        let sliders: [(&str, &str, fn(&mut Adjust) -> &mut f32); 5] = [("Exposure", "sun-2", |a| &mut a.exposure), ("Contrast", "tuning-2", |a| &mut a.contrast), ("Saturation", "palette", |a| &mut a.saturation), ("Warmth", "bolt", |a| &mut a.warmth), ("Vignette", "layers-minimalistic", |a| &mut a.vignette)];
        for (label, icon, f) in sliders {
            let mut adj = self.shell.photos.adj;
            let v = *f(&mut adj);
            let signed = label != "Vignette";
            let track = Rect::from_min_size(pos2(r.left() + 2.0, y + 24.0), vec2(r.width() - 4.0, 8.0));
            let resp = self.hit(ui, track.expand2(vec2(6.0, 10.0)), ("ph-adj", label), Sense::click_and_drag());
            if resp.double_clicked() {
                *f(&mut adj) = 0.0;
            } else if resp.dragged() || resp.clicked() {
                if let Some(pp) = resp.interact_pointer_pos() {
                    let t = ((pp.x - track.left()) / track.width()).clamp(0.0, 1.0);
                    *f(&mut adj) = if signed { t * 2.0 - 1.0 } else { t };
                }
            }
            if adj != self.shell.photos.adj {
                let ph = &mut self.shell.photos;
                ph.adj = adj;
                ph.dirty = true;
                ph.preset = PRESETS.iter().position(|(_, a)| *a == adj).unwrap_or(usize::MAX);
            }
            self.ico(p, icon, Ic::Outline, pos2(r.left() + 8.0, y + 8.0), 14.0, hex("#9aa6c0"));
            p.text(pos2(r.left() + 24.0, y + 8.0), Align2::LEFT_CENTER, label, f_medium(T_BODY), hex("#dbe4f5"));
            p.text(pos2(r.right(), y + 8.0), Align2::RIGHT_CENTER, format!("{:+.0}", (if signed { v * 100.0 } else { v * 100.0 }).round()), f_medium(T_META), hex("#7d89a3"));
            let shown = spring_to(ui.ctx(), ("ph-sl", label), v, 380.0, 30.0);
            well(p, &th, track, 4.0);
            let (a, b) = if signed { (track.center().x.min(track.left() + track.width() * (shown * 0.5 + 0.5)), track.center().x.max(track.left() + track.width() * (shown * 0.5 + 0.5))) } else { (track.left(), track.left() + track.width() * shown) };
            if b - a > 0.5 {
                let (c0, c1) = th.accent_grad();
                grad_fill(p, Rect::from_min_max(pos2(a, track.top()), pos2(b, track.bottom())), 4.0, c0, c1);
            }
            let kx = track.left() + track.width() * if signed { shown * 0.5 + 0.5 } else { shown };
            let knob = Rect::from_center_size(pos2(kx, track.center().y), vec2(16.0, 16.0) * if resp.dragged() { 1.15 } else { 1.0 });
            raised(p, &th, knob, 8.0, resp.hovered(), resp.dragged());
            y += 44.0;
        }
        let b = Rect::from_min_size(pos2(r.left(), r.bottom() - 36.0), vec2(r.width(), 34.0));
        if self.label_button(ui, b, Some("download-minimalistic"), "Save edited copy", "ph-save", Btn::Accent).clicked() {
            self.photos_save();
        }
    }
}

fn smooth01(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
