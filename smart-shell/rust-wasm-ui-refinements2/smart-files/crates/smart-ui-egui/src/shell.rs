//! Desktop shell for the web showcase: system status bar, dock with spring magnification and launch bounce,
//! and a tiny window manager (z-order, focus-on-press, drag, resize, maximise) shared by the app suite
//! (Files, Photos, Terminal). Everything here is drawn with the same depth primitives as the file manager.
use crate::app::*;
use crate::chrome::*;
use crate::icons::Ic;
use crate::overlays::{mi, MI};
use crate::paint::*;
use crate::theme::*;
use egui::{pos2, vec2, Align2, Color32, Id, Pos2, Rect, Sense, Ui};
use smart_core::*;
use std::collections::HashMap;

pub const TOPBAR_H: f32 = 30.0;
pub const DOCK_H: f32 = 98.0;
const TITLE_H: f32 = 46.0;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AppId {
    Files,
    Photos,
    Terminal,
}

impl AppId {
    pub fn name(self) -> &'static str {
        match self {
            AppId::Files => "Files",
            AppId::Photos => "Photos",
            AppId::Terminal => "Terminal",
        }
    }
    pub fn icon(self) -> &'static str {
        match self {
            AppId::Files => "folder",
            AppId::Photos => "gallery",
            AppId::Terminal => "terminal",
        }
    }
}

#[derive(Clone)]
pub struct AppWin {
    pub open: bool,
    pub min: bool,
    pub rect: Rect,
    pub restore: Option<Rect>,
    pub born: f64,
    pub placed: bool,
}

impl AppWin {
    fn new(w: f32, h: f32, open: bool) -> Self {
        Self { open, min: false, rect: Rect::from_min_size(pos2(0.0, 0.0), vec2(w, h)), restore: None, born: 0.0, placed: false }
    }
}

pub struct Shell {
    pub z: Vec<AppId>,
    pub photos_w: AppWin,
    pub term_w: AppWin,
    pub term: crate::terminal::Terminal,
    pub photos: crate::photos::Photos,
    pub menu: Option<usize>,
    pub qs: bool,
    pub wifi: bool,
    pub bt: bool,
    pub focus: bool,
    pub brightness: f32,
    pub volume: f32,
    pub launched: HashMap<AppId, f64>,
}

impl Shell {
    pub fn new(home: Location) -> Self {
        Self {
            z: vec![AppId::Terminal, AppId::Photos, AppId::Files],
            photos_w: AppWin::new(980.0, 640.0, false),
            term_w: AppWin::new(880.0, 540.0, false),
            term: crate::terminal::Terminal::new(home),
            photos: crate::photos::Photos::default(),
            menu: None,
            qs: false,
            wifi: true,
            bt: true,
            focus: false,
            brightness: 1.0,
            volume: 0.62,
            launched: HashMap::new(),
        }
    }
    pub fn active(&self) -> AppId {
        *self.z.last().unwrap_or(&AppId::Files)
    }
}

impl SmartApp {
    pub fn open_app(&mut self, id: AppId) {
        let t = self.time;
        match id {
            AppId::Files => {
                self.win_closed = false;
                self.win_min = false;
            }
            AppId::Photos => {
                if !self.shell.photos_w.open {
                    self.shell.photos_w.born = t;
                }
                self.shell.photos_w.open = true;
                self.shell.photos_w.min = false;
            }
            AppId::Terminal => {
                if !self.shell.term_w.open {
                    self.shell.term_w.born = t;
                }
                self.shell.term_w.open = true;
                self.shell.term_w.min = false;
            }
        }
        self.shell.launched.insert(id, t);
        self.raise(id);
    }

    /// Topmost window that is actually on screen.
    pub fn focused_app(&self) -> AppId {
        for id in self.shell.z.iter().rev() {
            let vis = match id {
                AppId::Files => !(self.win_closed || self.win_min),
                other => self.app_win(*other).map(|w| w.open && !w.min).unwrap_or(false),
            };
            if vis {
                return *id;
            }
        }
        AppId::Files
    }

    pub fn raise(&mut self, id: AppId) {
        self.shell.z.retain(|x| *x != id);
        self.shell.z.push(id);
    }

    pub fn app_cmd(&mut self, cmd: &str) {
        if let Some(c) = cmd.strip_prefix("term.") {
            self.shell.term.command(c);
        } else if let Some(c) = cmd.strip_prefix("photos.") {
            self.photos_cmd(c);
        }
    }

    fn app_win(&self, id: AppId) -> Option<&AppWin> {
        match id {
            AppId::Files => None,
            AppId::Photos => Some(&self.shell.photos_w),
            AppId::Terminal => Some(&self.shell.term_w),
        }
    }
    fn app_win_mut(&mut self, id: AppId) -> &mut AppWin {
        match id {
            AppId::Terminal => &mut self.shell.term_w,
            _ => &mut self.shell.photos_w,
        }
    }

    // ───────────── entry point ─────────────
    pub fn draw_window(&mut self, ui: &mut Ui) {
        if !IS_WEB {
            clear_drop_targets();
            return self.draw_files_window(ui);
        }
        clear_drop_targets();
        let screen = ui.max_rect();
        let p = ui.painter().clone();
        self.draw_desktop(&p, screen);
        self.shell_focus(ui, screen);
        for id in self.shell.z.clone() {
            match id {
                AppId::Files => {
                    if !(self.win_closed || self.win_min) {
                        self.draw_files_window(ui);
                    }
                }
                _ => self.draw_app(ui, id, screen),
            }
        }
        self.draw_dock(ui, screen);
        self.draw_topbar(ui, screen);
        let b = self.shell.brightness;
        if b < 0.99 {
            let lp = ui.ctx().layer_painter(egui::LayerId::new(egui::Order::Tooltip, Id::new("dim")));
            lp.rect_filled(screen, 0.0, Color32::from_black_alpha(((1.0 - b) * 150.0) as u8));
        }
    }

    /// Pressing inside a window raises it; the top bar and dock never steal focus.
    fn shell_focus(&mut self, ui: &Ui, screen: Rect) {
        let Some(pos) = ui.input(|i| if i.pointer.primary_pressed() { i.pointer.interact_pos() } else { None }) else { return };
        if pos.y < TOPBAR_H || pos.y > screen.bottom() - 70.0 {
            return;
        }
        for id in self.shell.z.clone().into_iter().rev() {
            let r = match id {
                AppId::Files => {
                    if self.win_closed || self.win_min {
                        continue;
                    }
                    self.window_rect(screen)
                }
                other => {
                    let w = self.app_win(other).unwrap();
                    if !w.open || w.min {
                        continue;
                    }
                    w.rect
                }
            };
            if r.contains(pos) {
                if self.focused_app() != id {
                    self.raise(id);
                }
                break;
            }
        }
    }

    // ───────────── app window frame ─────────────
    fn draw_app(&mut self, ui: &mut Ui, id: AppId, screen: Rect) {
        let (open, min) = {
            let w = self.app_win(id).unwrap();
            (w.open, w.min)
        };
        if !open || min {
            return;
        }
        let avail = Rect::from_min_max(pos2(screen.left(), screen.top() + TOPBAR_H), pos2(screen.right(), screen.bottom() - 6.0));
        let n_open = self.shell.z.iter().position(|x| *x == id).unwrap_or(0) as f32;
        {
            let w = self.app_win_mut(id);
            if !w.placed {
                let size = vec2(w.rect.width().min(avail.width() - 40.0), w.rect.height().min(avail.height() - DOCK_H * 0.6));
                let c = avail.center() + vec2(40.0 + n_open * 18.0, 10.0 + n_open * 14.0 - 30.0);
                w.rect = Rect::from_center_size(c, size);
                w.placed = true;
            }
        }
        let (mut rect, born, restore) = {
            let w = self.app_win(id).unwrap();
            (w.rect, w.born, w.restore)
        };
        let active = self.focused_app() == id;
        let th = self.th.clone();
        let age = (self.time - born) as f32;
        let pop = if self.lib(|l| l.settings.reduce_motion) { 1.0 } else { spring_out(age) };
        if age < 0.7 {
            ui.ctx().request_repaint();
        }
        let off = vec2(0.0, (1.0 - pop) * 28.0);
        let mut cui = ui.new_child(egui::UiBuilder::new().max_rect(screen));
        cui.set_opacity((age * 7.0).clamp(0.0, 1.0));
        let p = cui.painter().clone();
        let shown = rect.translate(off);
        elevate(&p, &th, shown, R_WIN, if active { 5 } else { 3 });
        // glass body (same recipe as the Files window)
        let (_, sz, blur) = self.active_wall();
        if let Some(b) = blur {
            let suv = cover_uv(sz[0], sz[1], screen);
            let f = |pt: Pos2| pos2(suv.min.x + (pt.x - screen.min.x) / screen.width() * suv.width(), suv.min.y + (pt.y - screen.min.y) / screen.height() * suv.height());
            textured_rrect(&p, shown, R_WIN, b, Rect::from_min_max(f(shown.min), f(shown.max)), Color32::from_white_alpha(if th.dark { 210 } else { 255 }));
        }
        grad_fill(&p, shown, R_WIN, th.window_top, th.window_bottom);
        // title bar
        let title = Rect::from_min_size(shown.min, vec2(shown.width(), TITLE_H));
        let tresp = cui.interact(title, Id::new(("apptitle", id)), Sense::click_and_drag());
        let mut maximise = tresp.double_clicked();
        if tresp.dragged() && restore.is_none() {
            let d = tresp.drag_delta();
            let w = self.app_win_mut(id);
            w.rect = w.rect.translate(d);
            w.rect = w.rect.translate(vec2(0.0, (avail.top() - w.rect.top()).max(0.0)));
            rect = w.rect;
        }
        // traffic lights
        let origin = shown.min + vec2(24.0, TITLE_H / 2.0);
        let cols = [("close", hex("#ff7c72"), hex("#f2554c")), ("min", hex("#ffd264"), hex("#f5b93c")), ("zoom", hex("#6fdc8c"), hex("#38bd62"))];
        let g_hover = cui.rect_contains_pointer(Rect::from_min_size(origin - vec2(10.0, 10.0), vec2(70.0, 20.0)));
        let mut do_close = false;
        let mut do_min = false;
        for (i, (n, a, b)) in cols.iter().enumerate() {
            let ctr = origin + vec2(20.0 * i as f32, 0.0);
            let hr = Rect::from_center_size(ctr, vec2(18.0, 18.0));
            let resp = cui.interact(hr, Id::new(("tl", id, *n)), Sense::click());
            let dot = Rect::from_center_size(ctr, vec2(12.0, 12.0) * if resp.hovered() { 1.14 } else { 1.0 });
            let (a, b) = if active { (*a, *b) } else { (alpha(th.ink4, 0.6), alpha(th.ink4, 0.45)) };
            shadow(&p, dot, 6.0, 0.5, 1.0, 0.0, c(20, 29, 48, 0.12));
            grad_fill(&p, dot, 6.0, a, b);
            rim(&p, dot, 6.0, 1.0, Color32::from_white_alpha(95), true, 7.0);
            if g_hover {
                let st = egui::Stroke::new(1.35, c(40, 20, 18, 0.55));
                match *n {
                    "close" => {
                        p.line_segment([ctr + vec2(-2.8, -2.8), ctr + vec2(2.8, 2.8)], st);
                        p.line_segment([ctr + vec2(2.8, -2.8), ctr + vec2(-2.8, 2.8)], st);
                    }
                    "min" => {
                        p.line_segment([ctr + vec2(-3.0, 0.0), ctr + vec2(3.0, 0.0)], st);
                    }
                    _ => {
                        p.line_segment([ctr + vec2(-2.6, 2.6), ctr + vec2(2.6, -2.6)], st);
                    }
                }
            }
            if resp.clicked() {
                match *n {
                    "close" => do_close = true,
                    "min" => do_min = true,
                    _ => maximise = true,
                }
            }
        }
        let tx = shown.center().x;
        let f = f_semi(T_BODY);
        let nm = id.name();
        let wname = self.tw(&p, nm, &f);
        self.ico(&p, id.icon(), Ic::Duo, pos2(tx - wname / 2.0 - 14.0, title.center().y), 16.0, if active { th.accent_ink } else { th.ink4 });
        p.text(pos2(tx + 4.0, title.center().y), Align2::CENTER_CENTER, nm, f, if active { th.ink1 } else { th.ink3 });
        p.line_segment([pos2(shown.left() + 1.0, title.bottom()), pos2(shown.right() - 1.0, title.bottom())], egui::Stroke::new(1.0, th.seam));
        // content
        let content = Rect::from_min_max(pos2(shown.left(), title.bottom() + 1.0), shown.max);
        let mut cp = cui.new_child(egui::UiBuilder::new().max_rect(content));
        cp.set_clip_rect(content.intersect(screen));
        match id {
            AppId::Terminal => self.draw_terminal(&mut cp, content, active),
            AppId::Photos => self.draw_photos(&mut cp, content, active),
            AppId::Files => {}
        }
        // edge + corner resize
        let min = vec2(560.0, 360.0);
        let zones = [
            (Rect::from_min_max(pos2(shown.right() - 7.0, shown.top() + 20.0), pos2(shown.right() + 3.0, shown.bottom() - 16.0)), egui::CursorIcon::ResizeHorizontal, (1.0, 0.0)),
            (Rect::from_min_max(pos2(shown.left() + 16.0, shown.bottom() - 7.0), pos2(shown.right() - 16.0, shown.bottom() + 3.0)), egui::CursorIcon::ResizeVertical, (0.0, 1.0)),
            (Rect::from_min_max(pos2(shown.right() - 18.0, shown.bottom() - 18.0), pos2(shown.right() + 4.0, shown.bottom() + 4.0)), egui::CursorIcon::ResizeNwSe, (1.0, 1.0)),
        ];
        for (k, (zr, cur, dir)) in zones.iter().enumerate() {
            let r = cui.interact(*zr, Id::new(("resize", id, k)), Sense::drag());
            if r.hovered() || r.dragged() {
                cui.ctx().set_cursor_icon(*cur);
            }
            if r.dragged() && restore.is_none() {
                let d = r.drag_delta();
                let w = self.app_win_mut(id);
                w.rect.max.x = (w.rect.max.x + d.x * dir.0).max(w.rect.min.x + min.x);
                w.rect.max.y = (w.rect.max.y + d.y * dir.1).max(w.rect.min.y + min.y);
            }
        }
        p.add(egui::Shape::Noop);
        rim(&p, shown, R_WIN, 1.4, if th.dark { c(255, 255, 255, if active { 0.14 } else { 0.07 }) } else { c(255, 255, 255, 0.9) }, true, 42.0);
        p.rect_stroke(shown.shrink(0.5), cr(R_WIN), egui::Stroke::new(1.0, if active { alpha(th.accent, 0.22) } else { alpha(Color32::WHITE, 0.05) }), egui::StrokeKind::Inside);
        // apply
        if do_close {
            self.app_win_mut(id).open = false;
        }
        if do_min {
            self.app_win_mut(id).min = true;
        }
        if maximise {
            let w = self.app_win_mut(id);
            if let Some(r) = w.restore.take() {
                w.rect = r;
            } else {
                w.restore = Some(w.rect);
                w.rect = avail.shrink2(vec2(8.0, 4.0));
            }
        }
        let _ = rect;
    }

    // ───────────── status bar ─────────────
    fn tb_menus(&self, app: AppId) -> Vec<(&'static str, Vec<MI>)> {
        let st = self.st();
        let sel_n = self.sel.len();
        match app {
            AppId::Files => vec![
                (
                    "File",
                    vec![
                        mi("add-folder", "New folder", Act::NewFolder).k("Ctrl+Shift+N"),
                        mi("add", "New tab", Act::NewTab(None)).k("Ctrl+T"),
                        mi("close-circle", "Close tab", Act::CloseTab(self.tab_i)).off(self.tabs.len() < 2),
                        mi("info-circle", "Get Info", Act::Info).k("Ctrl+I").gap().off(sel_n == 0),
                        mi("archive", "Compress to ZIP\u{2026}", Act::ArchiveCompress).off(sel_n == 0),
                        mi("terminal", "Open terminal here", Act::OpenTerminal).gap(),
                        mi("maximize", "Close window", Act::WinClose).gap(),
                    ],
                ),
                (
                    "Edit",
                    vec![
                        mi("copy", "Copy", Act::Copy).k("Ctrl+C").off(sel_n == 0),
                        mi("scissors", "Cut", Act::Cut).k("Ctrl+X").off(sel_n == 0),
                        mi("clipboard", "Paste", Act::Paste).k("Ctrl+V").off(self.clipboard.is_none()),
                        mi("checklist-minimalistic", "Select all", Act::SelectAll).k("Ctrl+A").gap(),
                        mi("check-circle", "Selection mode", Act::ToggleSelectMode).on(self.select_mode),
                    ],
                ),
                (
                    "View",
                    vec![
                        mi("widget", "As grid", Act::SetGrid(true)).on(st.grid).k("G"),
                        mi("list-check", "As list", Act::SetGrid(false)).on(!st.grid).k("L"),
                        mi("eye", "Show hidden files", Act::ToggleHidden).on(st.show_hidden).k("Ctrl+H").gap(),
                        mi("folder", "Folders first", Act::ToggleFoldersFirst).on(st.folders_first),
                        mi("sidebar-minimalistic", "Sidebar", Act::ToggleSidebar).k("Ctrl+\\").gap(),
                    ],
                ),
                (
                    "Go",
                    {
                        let mut v = vec![mi("alt-arrow-left", "Back", Act::Back).k("Alt+\u{2190}").off(!self.can_back()), mi("alt-arrow-right", "Forward", Act::Forward).k("Alt+\u{2192}").off(!self.can_fwd()), mi("arrow-up", "Enclosing folder", Act::Up).k("Alt+\u{2191}")];
                        for (i, pl) in self.places.iter().take(6).enumerate() {
                            let mut m = mi(pl.icon, pl.title.clone(), Act::Nav(Route::Loc(pl.loc.clone())));
                            if i == 0 {
                                m = m.gap();
                            }
                            v.push(m);
                        }
                        v
                    },
                ),
            ],
            AppId::Terminal => vec![
                ("Shell", vec![mi("add", "New session", Act::App("term.new".into())).k("Ctrl+T"), mi("close-circle", "Close session", Act::App("term.close".into())).k("Ctrl+W"), mi("trash-bin-minimalistic", "Clear", Act::App("term.clear".into())).k("Ctrl+L").gap()]),
                ("View", vec![mi("magnifer-zoom-in", "Bigger text", Act::App("term.bigger".into())).k("Ctrl +"), mi("magnifer", "Smaller text", Act::App("term.smaller".into())).k("Ctrl -"), mi("stars-minimalistic", if st.dark { "Light theme" } else { "Dark theme" }, Act::ToggleTheme).gap()]),
            ],
            AppId::Photos => vec![
                ("File", vec![mi("folder-open", "Show in Files", Act::App("photos.reveal".into())), mi("gallery-wide", "Set as wallpaper", Act::App("photos.wallpaper".into())), mi("download-minimalistic", "Save edited copy", Act::App("photos.save".into())).gap()]),
                (
                    "View",
                    vec![
                        mi("full-screen", "Fit to window", Act::App("photos.fit".into())).k("0"),
                        mi("maximize", "Actual size", Act::App("photos.actual".into())).k("1"),
                        mi("play", "Slideshow", Act::App("photos.slideshow".into())).k("S").gap(),
                        mi("info-circle", "Info panel", Act::App("photos.info".into())).k("I").on(self.shell.photos.info),
                        mi("tuning-2", "Edit panel", Act::App("photos.edit".into())).k("E").on(self.shell.photos.edit),
                    ],
                ),
            ],
        }
    }

    fn draw_topbar(&mut self, ui: &mut Ui, screen: Rect) {
        let ctx = ui.ctx().clone();
        let th = self.th.clone();
        let p = ui.painter().clone();
        let bar = Rect::from_min_size(screen.min, vec2(screen.width(), TOPBAR_H));
        let (top, bot) = if th.dark { (c(10, 14, 24, 0.66), c(10, 14, 24, 0.50)) } else { (c(255, 255, 255, 0.62), c(244, 248, 255, 0.48)) };
        grad_fill(&p, bar, 0.0, top, bot);
        p.line_segment([bar.left_bottom(), bar.right_bottom()], egui::Stroke::new(1.0, if th.dark { c(255, 255, 255, 0.07) } else { c(255, 255, 255, 0.7) }));
        p.line_segment([bar.left_bottom() + vec2(0.0, 1.0), bar.right_bottom() + vec2(0.0, 1.0)], egui::Stroke::new(1.0, alpha(th.shadow, if th.dark { 0.3 } else { 0.06 })));
        let cy = bar.center().y;
        let app = self.focused_app();
        let app = if app == AppId::Files && (self.win_closed || self.win_min) { AppId::Files } else { app };
        let ink = if th.dark { th.ink1 } else { th.ink1 };

        // logo → Smart menu
        let mut x = bar.left() + 10.0;
        let logo = Rect::from_center_size(pos2(x + 13.0, cy), vec2(26.0, 22.0));
        let lr = self.hit(ui, logo, "tb-logo", Sense::click());
        if lr.hovered() || self.shell.menu == Some(0) {
            p.rect_filled(logo, cr(7.0), th.row_hover);
        }
        let lg = Rect::from_center_size(logo.center(), vec2(17.0, 17.0));
        let (a, b) = th.accent_grad();
        grad_fill(&p, lg, 6.0, a, b);
        rim(&p, lg, 6.0, 1.0, Color32::from_white_alpha(140), true, 8.0);
        p.text(lg.center() + vec2(0.0, 0.5), Align2::CENTER_CENTER, "S", f_bold(10.5), Color32::WHITE);
        let mut anchors: Vec<Rect> = vec![logo];
        if lr.clicked() {
            self.shell.menu = if self.shell.menu == Some(0) { None } else { Some(0) };
        }
        x = logo.right() + 8.0;
        p.text(pos2(x, cy), Align2::LEFT_CENTER, app.name(), f_bold(12.5), ink);
        x += self.tw(&p, app.name(), &f_bold(12.5)) + 12.0;
        let menus = self.tb_menus(app);
        for (i, (label, _)) in menus.iter().enumerate() {
            let w = self.tw(&p, label, &f_medium(12.5)) + 18.0;
            let r = Rect::from_min_size(pos2(x, cy - 11.0), vec2(w, 22.0));
            let resp = self.hit(ui, r, ("tb-menu", i), Sense::click());
            let open = self.shell.menu == Some(i + 1);
            if resp.hovered() || open {
                p.rect_filled(r, cr(7.0), if open { th.row_press } else { th.row_hover });
            }
            p.text(r.center(), Align2::CENTER_CENTER, *label, f_medium(12.5), if open || resp.hovered() { ink } else { th.ink2 });
            if resp.clicked() {
                self.shell.menu = if open { None } else { Some(i + 1) };
            } else if resp.hovered() && self.shell.menu.is_some() && !open {
                self.shell.menu = Some(i + 1);
            }
            anchors.push(r);
            x += w + 2.0;
        }

        // right cluster: status icons + clock
        let mut rx = bar.right() - 12.0;
        let time = clock_hm();
        let date = clock_date();
        let tf = f_semi(12.5);
        let tw_ = self.tw(&p, &time, &tf);
        let dw = self.tw(&p, &date, &f_medium(12.5));
        let cluster_w = tw_ + dw + 16.0 + 4.0 * 26.0 + 14.0;
        let cluster = Rect::from_min_max(pos2(rx - cluster_w, cy - 12.0), pos2(rx + 4.0, cy + 12.0));
        let cresp = self.hit(ui, cluster, "tb-cluster", Sense::click());
        if cresp.hovered() || self.shell.qs {
            p.rect_filled(cluster, cr(8.0), if self.shell.qs { th.row_press } else { th.row_hover });
        }
        if cresp.clicked() {
            self.shell.qs = !self.shell.qs;
            self.shell.menu = None;
        }
        p.text(pos2(rx, cy), Align2::RIGHT_CENTER, &time, tf, ink);
        rx -= tw_ + 10.0;
        p.text(pos2(rx, cy), Align2::RIGHT_CENTER, &date, f_medium(12.5), th.ink2);
        rx -= dw + 14.0;
        let sh = &self.shell;
        let status: [(&'static str, bool); 4] = [("battery-charge-minimalistic", true), ("volume-small", sh.volume > 0.02), ("bluetooth", sh.bt), ("wi-fi", sh.wifi)];
        for (name, on) in status {
            self.ico(&p, name, Ic::Outline, pos2(rx - 9.0, cy), 15.0, if on { th.ink2 } else { alpha(th.ink4, 0.7) });
            rx -= 26.0;
        }

        // menu popovers
        if let Some(mi_idx) = self.shell.menu {
            let (items, anchor) = if mi_idx == 0 {
                (
                    vec![
                        mi("info-circle", "About Smart Shell", Act::Seq(vec![Act::OpenApp(AppId::Files), Act::Modal(Modal::About)])),
                        mi("settings-minimalistic", "Settings\u{2026}", Act::Seq(vec![Act::OpenApp(AppId::Files), Act::Settings])).k("Ctrl+,").gap(),
                        mi("stars-minimalistic", if self.st().dark { "Switch to light" } else { "Switch to dark" }, Act::ToggleTheme).k("D"),
                        mi("gallery-wide", "Wallpaper\u{2026}", Act::Seq(vec![Act::OpenApp(AppId::Files), Act::Modal(Modal::Wallpaper)])),
                        mi("command", "Command palette", Act::Seq(vec![Act::OpenApp(AppId::Files), Act::Palette])).k("Ctrl+K").gap(),
                    ],
                    anchors[0],
                )
            } else {
                (menus.get(mi_idx - 1).map(|m| m.1.clone()).unwrap_or_default(), anchors.get(mi_idx).copied().unwrap_or(anchors[0]))
            };
            if !items.is_empty() {
                let w = self.menu_width(&p, &items, 220.0);
                let mut mp = ui.new_child(egui::UiBuilder::new().max_rect(screen));
                let pop = spring_to(&ctx, ("tbpop", mi_idx), 1.0, 300.0, 24.0);
                mp.set_opacity(pop.clamp(0.0, 1.0));
                let (r, _, act) = self.paint_menu(&mut mp, pos2(anchor.left(), bar.bottom() + 4.0 - (1.0 - pop.min(1.0)) * 6.0), &items, w, "tbm", None);
                if let Some(a) = act {
                    self.shell.menu = None;
                    self.acts.push(a);
                } else if ctx.input(|i| i.pointer.primary_pressed()) && ctx.pointer_interact_pos().is_some_and(|pp| !r.contains(pp) && !bar.contains(pp)) {
                    self.shell.menu = None;
                }
            }
        } else {
            ctx.data_mut(|d| {
                for i in 0..8 {
                    d.remove::<(f32, f32)>(Id::new(("spring", ("tbpop", i))));
                }
            });
        }
        self.draw_quick_settings(ui, screen, bar);
    }

    fn draw_quick_settings(&mut self, ui: &mut Ui, screen: Rect, bar: Rect) {
        let ctx = ui.ctx().clone();
        let th = self.th.clone();
        let s = spring_to(&ctx, "qs", if self.shell.qs { 1.0 } else { 0.0 }, 260.0, 24.0);
        if s < 0.01 {
            return;
        }
        let w = 320.0;
        let panel = Rect::from_min_size(pos2(screen.right() - w - 10.0, bar.bottom() + 8.0 - (1.0 - s.min(1.0)) * 12.0), vec2(w, 326.0));
        let mut cui = ui.new_child(egui::UiBuilder::new().max_rect(screen));
        cui.set_opacity(s.clamp(0.0, 1.0));
        let p = cui.painter().clone();
        menu_surface(&p, &th, panel, 20.0);
        cui.allocate_rect(panel, Sense::click());
        let pad = 14.0;
        // toggle tiles
        let tiles: [(&'static str, &str, String, bool, u8); 4] = [
            ("wi-fi", "Wi-Fi", if self.shell.wifi { "Smart-5G".into() } else { "Off".into() }, self.shell.wifi, 0),
            ("bluetooth", "Bluetooth", if self.shell.bt { "On".into() } else { "Off".into() }, self.shell.bt, 1),
            ("moon-stars", "Dark mode", if th.dark { "On".into() } else { "Off".into() }, th.dark, 2),
            ("bell", "Focus", if self.shell.focus { "Do not disturb".into() } else { "Off".into() }, self.shell.focus, 3),
        ];
        let tw_ = (w - pad * 2.0 - 10.0) / 2.0;
        for (i, (icon, label, sub, on, id)) in tiles.iter().enumerate() {
            let r = Rect::from_min_size(panel.min + vec2(pad + (i % 2) as f32 * (tw_ + 10.0), pad + (i / 2) as f32 * 66.0), vec2(tw_, 58.0));
            let resp = self.hit(&cui, r, ("qs-tile", i), Sense::click());
            let a = self.anim(&cui, ("qs-t", i), resp.hovered());
            if *on {
                raised_accent(&p, &th, r, 16.0, resp.hovered(), resp.is_pointer_button_down_on());
            } else {
                raised(&p, &th, r, 16.0, resp.hovered(), resp.is_pointer_button_down_on());
            }
            let fg = if *on { th.on_accent() } else { th.ink1 };
            let orb = Rect::from_center_size(pos2(r.left() + 26.0, r.center().y), vec2(30.0, 30.0));
            grad_fill(&p, orb, 15.0, alpha(fg, if *on { 0.22 } else { 0.07 }), alpha(fg, if *on { 0.10 } else { 0.03 }));
            self.ico(&p, icon, if *on || a > 0.5 { Ic::Duo } else { Ic::Outline }, orb.center(), 17.0, fg);
            p.text(pos2(r.left() + 48.0, r.center().y - 8.0), Align2::LEFT_CENTER, *label, f_semi(T_BODY), fg);
            p.text(pos2(r.left() + 48.0, r.center().y + 9.0), Align2::LEFT_CENTER, self.fit(&p, sub, &f_medium(T_META), r.width() - 56.0), f_medium(T_META), alpha(fg, 0.72));
            if resp.clicked() {
                match id {
                    0 => self.shell.wifi = !self.shell.wifi,
                    1 => self.shell.bt = !self.shell.bt,
                    2 => self.acts.push(Act::ToggleTheme),
                    _ => self.shell.focus = !self.shell.focus,
                }
            }
        }
        // sliders
        let mut y = panel.top() + pad + 2.0 * 66.0 + 6.0;
        for k in 0..2 {
            let (icon, label) = if k == 0 { ("sun-2", "Display") } else { ("volume-loud", "Sound") };
            p.text(pos2(panel.left() + pad + 2.0, y + 2.0), Align2::LEFT_CENTER, label, f_semi(T_META), th.ink3);
            let track = Rect::from_min_size(pos2(panel.left() + pad + 30.0, y + 16.0), vec2(w - pad * 2.0 - 30.0, 12.0));
            let resp = self.hit(&cui, track.expand2(vec2(6.0, 8.0)), ("qs-slider", k), Sense::click_and_drag());
            let v = if k == 0 { self.shell.brightness } else { self.shell.volume };
            let mut nv = v;
            if resp.dragged() || resp.clicked() {
                if let Some(pp) = resp.interact_pointer_pos() {
                    nv = ((pp.x - track.left()) / track.width()).clamp(0.0, 1.0);
                }
            }
            if k == 0 {
                self.shell.brightness = nv.max(0.25);
            } else {
                self.shell.volume = nv;
            }
            let shown = spring_to(&ctx, ("qs-sl", k), nv, 400.0, 30.0).clamp(0.0, 1.0);
            well(&p, &th, track, 6.0);
            let fill = Rect::from_min_size(track.min, vec2((track.width() * shown).max(12.0), track.height()));
            let (c0, c1) = th.accent_grad();
            grad_fill(&p, fill, 6.0, c0, c1);
            rim(&p, fill, 6.0, 1.0, Color32::from_white_alpha(120), true, 6.0);
            let knob = Rect::from_center_size(pos2(track.left() + track.width() * shown, track.center().y), vec2(20.0, 20.0) * if resp.dragged() { 1.12 } else { 1.0 });
            raised(&p, &th, knob, 10.0, resp.hovered(), resp.dragged());
            self.ico(&p, icon, Ic::Outline, pos2(panel.left() + pad + 11.0, track.center().y), 16.0, th.ink2);
            y += 44.0;
        }
        // battery + actions
        let bat = Rect::from_min_size(pos2(panel.left() + pad, panel.bottom() - pad - 44.0), vec2(w - pad * 2.0 - 96.0, 44.0));
        self.ico(&p, "battery-charge-minimalistic", Ic::Duo, pos2(bat.left() + 12.0, bat.center().y), 20.0, th.good);
        p.text(pos2(bat.left() + 32.0, bat.center().y - 8.0), Align2::LEFT_CENTER, "87%  \u{b7}  charging", f_semi(T_BODY), th.ink1);
        p.text(pos2(bat.left() + 32.0, bat.center().y + 9.0), Align2::LEFT_CENTER, "About 2 h 10 min to full", f_medium(T_META), th.ink3);
        let sr = Rect::from_min_size(pos2(panel.right() - pad - 88.0, panel.bottom() - pad - 36.0), vec2(88.0, 32.0));
        if self.label_button(&cui, sr, Some("settings-minimalistic"), "Settings", "qs-settings", Btn::Raised).clicked() {
            self.shell.qs = false;
            self.acts.push(Act::OpenApp(AppId::Files));
            self.acts.push(Act::Settings);
        }
        if ctx.input(|i| i.pointer.primary_pressed()) && ctx.pointer_interact_pos().is_some_and(|pp| !panel.contains(pp) && !bar.contains(pp)) {
            self.shell.qs = false;
        }
    }

    // ───────────── dock ─────────────
    fn draw_dock(&mut self, ui: &mut Ui, screen: Rect) {
        #[derive(Clone, Copy)]
        enum Di {
            App(AppId),
            Sep,
            Palette,
            Settings,
        }
        let ctx = ui.ctx().clone();
        let th = self.th.clone();
        let p = ui.painter().clone();
        let items = [Di::App(AppId::Files), Di::App(AppId::Photos), Di::App(AppId::Terminal), Di::Sep, Di::Settings, Di::Palette];
        let (base, gap, pad) = (46.0f32, 8.0f32, 12.0f32);
        let n = items.len();
        let w0: f32 = items.iter().map(|i| if matches!(i, Di::Sep) { 10.0 } else { base }).sum::<f32>() + gap * (n as f32 - 1.0) + pad * 2.0;
        let left0 = screen.center().x - w0 / 2.0;
        let ptr = ctx.input(|i| i.pointer.hover_pos());
        let zone = Rect::from_min_max(pos2(left0 - 30.0, screen.bottom() - DOCK_H), pos2(left0 + w0 + 30.0, screen.bottom()));
        let in_zone = ptr.is_some_and(|pp| zone.contains(pp));
        // magnification targets from the un-magnified centres
        let mut cx0 = left0 + pad;
        let mut scales = vec![];
        for (i, it) in items.iter().enumerate() {
            let w = if matches!(it, Di::Sep) { 10.0 } else { base };
            let c0 = cx0 + w / 2.0;
            let tgt = match (it, ptr) {
                (Di::Sep, _) => 1.0,
                (_, Some(pp)) if in_zone => 1.0 + 0.62 * (-((pp.x - c0) / 62.0).powi(2)).exp(),
                _ => 1.0,
            };
            scales.push(spring_to(&ctx, ("dock-s", i), tgt, 320.0, 25.0));
            cx0 += w + gap;
        }
        let total: f32 = items.iter().zip(&scales).map(|(it, s)| if matches!(it, Di::Sep) { 10.0 } else { base * s }).sum::<f32>() + gap * (n as f32 - 1.0) + pad * 2.0;
        let left = screen.center().x - total / 2.0;
        let pill = Rect::from_min_size(pos2(left, screen.bottom() - 14.0 - 62.0), vec2(total, 62.0));
        elevate(&p, &th, pill, 26.0, 4);
        let (g0, g1) = if th.dark { (c(26, 34, 52, 0.70), c(12, 18, 30, 0.78)) } else { (c(255, 255, 255, 0.62), c(236, 243, 255, 0.50)) };
        grad_fill(&p, pill, 26.0, g0, g1);
        rim(&p, pill, 26.0, 1.4, if th.dark { c(255, 255, 255, 0.22) } else { c(255, 255, 255, 0.95) }, true, 60.0);
        rim(&p, pill, 26.0, 1.0, if th.dark { c(255, 255, 255, 0.07) } else { c(255, 255, 255, 0.6) }, false, 14.0);
        p.rect_stroke(pill.shrink(0.5), cr(26.0), egui::Stroke::new(1.0, if th.dark { c(255, 255, 255, 0.08) } else { c(16, 26, 44, 0.07) }), egui::StrokeKind::Inside);

        let mut x = left + pad;
        let mut hovered_label: Option<(String, f32, f32)> = None;
        let mut clicked: Option<Di> = None;
        for (i, it) in items.iter().enumerate() {
            if let Di::Sep = it {
                p.line_segment([pos2(x + 5.0, pill.top() + 14.0), pos2(x + 5.0, pill.bottom() - 14.0)], egui::Stroke::new(1.0, alpha(th.ink4, 0.4)));
                x += 10.0 + gap;
                continue;
            }
            let sz = base * scales[i];
            let (name, icon, c0, c1, running, id) = match it {
                Di::App(AppId::Files) => ("Files", "folder", th.accent_grad().0, th.accent_grad().1, !(self.win_closed), Some(AppId::Files)),
                Di::App(AppId::Photos) => ("Photos", "gallery", hex("#ff9fb8"), hex("#ff7a59"), self.shell.photos_w.open, Some(AppId::Photos)),
                Di::App(AppId::Terminal) => ("Terminal", "terminal", hex("#2a3550"), hex("#0d1220"), self.shell.term_w.open, Some(AppId::Terminal)),
                Di::Settings => ("Settings", "settings-minimalistic", hex("#9aa6bd"), hex("#6b778f"), false, None),
                _ => ("Command palette", "command", hex("#8f7cf5"), hex("#5b4fd6"), false, None),
            };
            let bounce = match id.and_then(|a| self.shell.launched.get(&a)) {
                Some(t0) if self.time - t0 < 1.4 => {
                    ctx.request_repaint();
                    let a = (self.time - t0) as f32;
                    -26.0 * (-a * 3.4).exp() * (a * 12.0).sin().abs()
                }
                _ => 0.0,
            };
            let r = Rect::from_min_size(pos2(x, pill.bottom() - 8.0 - sz + bounce), vec2(sz, sz));
            let resp = ui.interact(Rect::from_min_size(pos2(x - gap / 2.0, pill.top() - 24.0), vec2(sz + gap, pill.height() + 24.0)), Id::new(("dock-i", i)), Sense::click());
            let down = resp.is_pointer_button_down_on();
            let rr = if down { r.shrink(sz * 0.04) } else { r };
            let rad = sz * 0.27;
            shadow(&p, rr, rad, 5.0, 12.0, -3.0, alpha(th.shadow, if th.dark { 0.55 } else { 0.30 }));
            grad_fill(&p, rr, rad, c0, c1);
            rim(&p, rr, rad, 1.4, Color32::from_white_alpha(if th.dark { 120 } else { 170 }), true, sz * 0.7);
            rim(&p, rr, rad, 1.0, Color32::from_black_alpha(60), false, 8.0);
            // gloss
            let gl = Rect::from_min_max(rr.min + vec2(sz * 0.06, sz * 0.05), pos2(rr.max.x - sz * 0.06, rr.top() + sz * 0.46));
            grad_fill(&p, gl, rad * 0.8, Color32::from_white_alpha(46), Color32::from_white_alpha(0));
            if id == Some(AppId::Terminal) {
                p.text(rr.center() + vec2(0.0, -sz * 0.02), Align2::CENTER_CENTER, ">_", f_bold(sz * 0.36), hex("#7dffb2"));
            } else {
                self.ico(&p, icon, if resp.hovered() { Ic::Filled } else { Ic::Duo }, rr.center(), sz * 0.52, Color32::WHITE);
            }
            if running {
                p.circle_filled(pos2(r.center().x, pill.bottom() - 4.0), 2.2, alpha(th.ink2, 0.9));
            }
            if resp.hovered() {
                hovered_label = Some((name.to_string(), r.center().x, r.top()));
            }
            if resp.clicked() {
                clicked = Some(*it);
            }
            x += sz + gap;
        }
        if let Some((label, cx, top)) = hovered_label {
            let f = f_semi(11.5);
            let w = self.tw(&p, &label, &f) + 20.0;
            let r = Rect::from_center_size(pos2(cx, top - 20.0), vec2(w, 24.0));
            shadow(&p, r, 12.0, 4.0, 10.0, -2.0, alpha(th.shadow, 0.4));
            grad_fill(&p, r, 12.0, th.tooltip_bg, alpha(th.tooltip_bg, 0.92));
            rim(&p, r, 12.0, 1.0, Color32::from_white_alpha(40), true, 12.0);
            p.text(r.center(), Align2::CENTER_CENTER, label, f, if th.dark { hex("#10151f") } else { Color32::WHITE });
        }
        match clicked {
            Some(Di::App(id)) => self.open_app(id),
            Some(Di::Settings) => {
                self.open_app(AppId::Files);
                self.acts.push(Act::Settings);
            }
            Some(Di::Palette) => {
                self.open_app(AppId::Files);
                self.acts.push(Act::Palette);
            }
            _ => {}
        }
    }
}
