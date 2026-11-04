//! Window, sidebar, app bar, status bar + shared widget helpers.
//! The app bar is state-driven: normal → navigation; selection → contextual actions;
//! narrow widths progressively collapse search, Ask Smart, sorting and breadcrumbs.
use crate::app::*;
use crate::icons::Ic;
use crate::paint::*;
use crate::theme::*;
use egui::{pos2, vec2, Align2, Color32, FontId, Id, Painter, Pos2, Rect, Response, Sense, Ui};
use smart_core::*;

pub const R_WIN: f32 = 24.0;
pub const SIDEBAR_W: f32 = 252.0;
pub const APPBAR_H: f32 = 60.0;
pub const STATUS_H: f32 = 34.0;
pub const TAB_H: f32 = 40.0;
pub const PAD_X: f32 = 26.0;

#[derive(Clone, Copy, PartialEq)]
pub enum Btn {
    Raised,
    Quiet,
    Accent,
    Active,
    Tonal,
    Danger,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DropTarget {
    Folder(Location),
    Place(Location),
    Tag(String),
    Trash,
}

thread_local! {
    pub static ACTS: std::cell::RefCell<Vec<Act>> = const { std::cell::RefCell::new(Vec::new()) };
    pub static DROP_TARGETS: std::cell::RefCell<Vec<(DropTarget, Rect)>> = const { std::cell::RefCell::new(Vec::new()) };
}

pub fn clear_drop_targets() {
    DROP_TARGETS.with(|v| v.borrow_mut().clear());
}

pub fn drop_target_at(p: Pos2) -> Option<DropTarget> {
    DROP_TARGETS.with(|v| v.borrow().iter().rev().find(|(_, r)| r.contains(p)).map(|(t, _)| t.clone()))
}

impl SmartApp {
    // ───────────── text helpers ─────────────
    pub fn txt(&self, p: &Painter, pos: Pos2, a: Align2, s: &str, f: FontId, col: Color32) -> Rect {
        p.text(pos, a, s, f, col)
    }
    pub fn tw(&self, p: &Painter, s: &str, f: &FontId) -> f32 {
        p.layout_no_wrap(s.to_string(), f.clone(), Color32::WHITE).size().x
    }
    pub fn fit(&self, p: &Painter, s: &str, f: &FontId, w: f32) -> String {
        if w <= 0.0 {
            return String::new();
        }
        if self.tw(p, s, f) <= w {
            return s.to_string();
        }
        let chars: Vec<char> = s.chars().collect();
        let (mut lo, mut hi) = (0usize, chars.len());
        while lo < hi {
            let mid = (lo + hi).div_ceil(2);
            let t: String = chars[..mid].iter().collect::<String>() + "\u{2026}";
            if self.tw(p, &t, f) <= w {
                lo = mid
            } else {
                hi = mid - 1
            }
        }
        chars[..lo].iter().collect::<String>().trim_end().to_string() + "\u{2026}"
    }
    pub fn ico(&self, p: &Painter, name: &'static str, v: Ic, center: Pos2, size: f32, col: Color32) {
        self.icons.paint_at(p, name, v, center, size, col);
    }
    pub fn hit(&self, ui: &Ui, rect: Rect, id: impl std::hash::Hash + std::fmt::Debug, sense: Sense) -> Response {
        ui.interact(rect, Id::unique(("hit", id)), sense)
    }
    pub fn anim(&self, ui: &Ui, id: impl std::hash::Hash + std::fmt::Debug, on: bool) -> f32 {
        if self.lib(|l| l.settings.reduce_motion) {
            return if on { 1.0 } else { 0.0 };
        }
        ui.ctx().animate_bool_with_time(Id::unique(("anim", id)), on, 0.14)
    }
    pub fn keycap(&self, p: &Painter, right_center: Pos2, s: &str, strong: bool) -> f32 {
        let f = f_semi(10.0);
        let w = (self.tw(p, s, &f) + 9.0).max(18.0);
        let r = Rect::from_min_max(pos2(right_center.x - w, right_center.y - 9.0), pos2(right_center.x, right_center.y + 9.0));
        grad_fill(p, r, 5.0, alpha(self.th.well, if strong { 0.8 } else { 0.55 }), alpha(self.th.well, 0.35));
        rim(p, r, 5.0, 1.0, alpha(Color32::WHITE, if self.th.dark { 0.07 } else { 0.9 }), false, 4.0);
        p.text(r.center() + vec2(0.0, 0.5), Align2::CENTER_CENTER, s, f, self.th.ink3);
        w
    }

    /// Round icon button with the light-from-above reflection (every chrome control uses this).
    pub fn icon_button(&self, ui: &Ui, rect: Rect, name: &'static str, id: &str, kind: Btn, enabled: bool) -> Response {
        let sense = if enabled { Sense::click() } else { Sense::hover() };
        let resp = self.hit(ui, rect, id, sense);
        let p = ui.painter();
        let th = &self.th;
        let hov = enabled && resp.hovered();
        let down = enabled && resp.is_pointer_button_down_on();
        let r = rect.height() / 2.0;
        let a = self.anim(ui, id, hov);
        let mut col = if enabled { mix(th.ink2, th.ink1, a) } else { alpha(th.ink2, 0.38) };
        match kind {
            Btn::Raised => raised(p, th, rect, r, hov, down),
            Btn::Active => {
                raised(p, th, rect, r, false, true);
                col = th.accent_ink;
            }
            Btn::Accent => {
                raised_accent(p, th, rect, r, hov, down);
                col = th.on_accent();
            }
            Btn::Tonal => {
                elevated_tonal(p, th, rect, r, hov, down, th.accent_tint, th.accent);
                col = th.accent_ink;
            }
            Btn::Danger => {
                elevated_tonal(p, th, rect, r, hov, down, th.danger_tint, th.danger);
                col = th.danger;
            }
            Btn::Quiet => {
                if hov {
                    p.rect_filled(rect, cr(r), if down { th.row_press } else { th.row_hover });
                }
            }
        }
        let v = if kind == Btn::Active { Ic::Filled } else if hov || kind == Btn::Accent { Ic::Duo } else { Ic::Outline };
        let off = if down && kind != Btn::Quiet { vec2(0.0, 0.6) } else { vec2(0.0, 0.0) };
        self.ico(p, name, v, rect.center() + off, 17.0, col);
        resp
    }

    pub fn label_button(&self, ui: &Ui, rect: Rect, icon: Option<&'static str>, label: &str, id: &str, kind: Btn) -> Response {
        let resp = self.hit(ui, rect, id, Sense::click());
        let p = ui.painter();
        let th = &self.th;
        let (hov, down) = (resp.hovered(), resp.is_pointer_button_down_on());
        let r = rect.height() / 2.0;
        let mut col = th.ink1;
        match kind {
            Btn::Accent => {
                raised_accent(p, th, rect, r, hov, down);
                col = th.on_accent();
            }
            Btn::Danger => {
                elevated_tonal(p, th, rect, r, hov, down, th.danger_tint, th.danger);
                col = th.danger;
            }
            Btn::Tonal => {
                elevated_tonal(p, th, rect, r, hov, down, th.accent_tint, th.accent);
                col = th.accent_ink;
            }
            Btn::Quiet => {
                if hov {
                    p.rect_filled(rect, cr(r), th.row_hover);
                }
                col = th.ink2;
            }
            _ => raised(p, th, rect, r, hov, down),
        }
        let f = f_semi(T_BODY);
        let tw = self.tw(p, label, &f);
        let iw = if icon.is_some() { 18.0 + 7.0 } else { 0.0 };
        let mut x = rect.center().x - (tw + iw) / 2.0;
        if let Some(i) = icon {
            self.ico(p, i, if matches!(kind, Btn::Accent | Btn::Danger | Btn::Tonal) || hov { Ic::Duo } else { Ic::Outline }, pos2(x + 8.0, rect.center().y), 16.0, if matches!(kind, Btn::Accent) { col } else if matches!(kind, Btn::Danger | Btn::Tonal) { col } else { th.ink2 });
            x += iw;
        }
        p.text(pos2(x, rect.center().y + (if down { 0.8 } else { 0.0 })), Align2::LEFT_CENTER, label, f, col);
        resp
    }

    /// Collapsed (icon-only) or expanded control, measured from the label.
    pub fn adaptive_button(&self, ui: &Ui, right: f32, cy: f32, icon: &'static str, label: &str, id: &str, kind: Btn, expanded: bool) -> (Response, f32) {
        let w = if expanded { self.tw(ui.painter(), label, &f_semi(T_BODY)) + 54.0 } else { 34.0 };
        let r = Rect::from_min_size(pos2(right - w, cy - 17.0), vec2(w, 34.0));
        let resp = if expanded {
            self.label_button(ui, r, Some(icon), label, id, kind)
        } else {
            let resp = self.icon_button(ui, r, icon, id, kind, true);
            resp
        };
        (resp, r.left() - 8.0)
    }

    // ───────────── window ─────────────
    pub fn window_rect(&self, screen: Rect) -> Rect {
        if !IS_WEB {
            return screen;
        }
        let screen = Rect::from_min_max(pos2(screen.left(), screen.top() + crate::shell::TOPBAR_H), pos2(screen.right(), screen.bottom() - crate::shell::DOCK_H * 0.62));
        if self.win_zoom {
            return screen.shrink(6.0);
        }
        let padx = (screen.width() * 0.036).clamp(14.0, 58.0);
        let pady = (screen.height() * 0.034).clamp(14.0, 38.0);
        let w = (screen.width() - 2.0 * padx).min(1440.0);
        let h = (screen.height() - 2.0 * pady).min(860.0);
        let mut c = screen.center() + self.win_off;
        c.x = c.x.clamp(screen.left() + w / 2.0 - w * 0.6, screen.right() - w / 2.0 + w * 0.6);
        c.y = c.y.clamp(screen.top() + h / 2.0 - 8.0 - pady, screen.bottom() - h / 2.0 + h * 0.6);
        Rect::from_center_size(c, vec2(w, h))
    }

    pub fn active_wall(&self) -> (egui::TextureId, [f32; 2], Option<egui::TextureId>) {
        let wi = self.st().wallpaper;
        if wi == 4 {
            if let Some((tex, sz)) = &self.custom_wall {
                return (tex.id(), *sz, None);
            }
        }
        let i = wi.min(self.walls.len() - 1);
        let (sharp, blur, sz) = &self.walls[i];
        (sharp.id(), *sz, Some(blur.id()))
    }

    pub fn draw_desktop(&self, p: &Painter, screen: Rect) {
        let (tex, sz, _) = self.active_wall();
        let uv = cover_uv(sz[0], sz[1], screen);
        p.image(tex, screen, uv, Color32::WHITE);
        let th = &self.th;
        let mut m = egui::epaint::Mesh::default();
        for (y, col) in [(screen.top(), th.veil_top), (screen.bottom(), th.veil_bottom)] {
            m.vertices.push(egui::epaint::Vertex { pos: pos2(screen.left(), y), uv: egui::epaint::WHITE_UV, color: col });
            m.vertices.push(egui::epaint::Vertex { pos: pos2(screen.right(), y), uv: egui::epaint::WHITE_UV, color: col });
        }
        m.indices.extend([0, 1, 2, 1, 3, 2]);
        p.add(egui::Shape::mesh(m));
    }

    pub fn draw_files_window(&mut self, ui: &mut Ui) {
        let screen = ui.max_rect();
        let p = ui.painter().clone();
        if self.win_closed || self.win_min {
            return;
        }
        let win = self.window_rect(screen);
        let th = self.th.clone();
        if IS_WEB {
            elevate(&p, &th, win, R_WIN, 5);
        }
        // glass: blurred wallpaper under a tonal veil
        let (_, sz, blur) = self.active_wall();
        if let Some(b) = blur {
            let suv = cover_uv(sz[0], sz[1], screen);
            let f = |pt: Pos2| pos2(suv.min.x + (pt.x - screen.min.x) / screen.width() * suv.width(), suv.min.y + (pt.y - screen.min.y) / screen.height() * suv.height());
            let uv = Rect::from_min_max(f(win.min), f(win.max));
            textured_rrect(&p, win, R_WIN, b, uv, Color32::from_white_alpha(if th.dark { 210 } else { 255 }));
        } else {
            let (t, s2, _) = self.active_wall();
            let uv = cover_uv(s2[0], s2[1], win);
            textured_rrect(&p, win, R_WIN, t, uv, Color32::from_white_alpha(if th.dark { 120 } else { 190 }));
        }
        grad_fill(&p, win, R_WIN, th.window_top, th.window_bottom);

        let sb_w = SIDEBAR_W * self.sidebar_open;
        let sb = Rect::from_min_size(win.min, vec2(sb_w, win.height()));
        let stage = Rect::from_min_max(pos2(win.min.x + sb_w, win.min.y), win.max);
        if sb_w > 1.0 {
            let pp = p.with_clip_rect(sb);
            grad_fill(&pp, Rect::from_min_max(win.min, pos2(win.min.x + sb_w + R_WIN + 4.0, win.max.y)), R_WIN, th.sidebar_top, th.sidebar_bottom);
            let mut m = egui::epaint::Mesh::default();
            let x = sb.right() - 1.0;
            let (c0, c1) = if th.dark { (c(255, 255, 255, 0.09), c(0, 0, 0, 0.16)) } else { (c(255, 255, 255, 0.52), c(20, 29, 48, 0.04)) };
            for (y, col) in [(sb.top() + 14.0, c0), (sb.bottom() - 14.0, c1)] {
                m.vertices.push(egui::epaint::Vertex { pos: pos2(x, y), uv: egui::epaint::WHITE_UV, color: col });
                m.vertices.push(egui::epaint::Vertex { pos: pos2(x + 1.0, y), uv: egui::epaint::WHITE_UV, color: col });
            }
            m.indices.extend([0, 1, 2, 1, 3, 2]);
            p.add(egui::Shape::mesh(m));
        }
        {
            let pp = p.with_clip_rect(stage);
            grad_fill(&pp, Rect::from_min_max(pos2(stage.min.x - R_WIN - 4.0, win.min.y), win.max), R_WIN, th.content_top, th.content_bottom);
        }
        if sb_w > 8.0 {
            let mut sui = ui.new_child(egui::UiBuilder::new().max_rect(Rect::from_min_size(win.min, vec2(SIDEBAR_W, win.height()))).layout(egui::Layout::top_down(egui::Align::Min)));
            sui.set_clip_rect(sb.intersect(ui.clip_rect()));
            self.draw_sidebar(&mut sui, Rect::from_min_size(win.min, vec2(SIDEBAR_W, win.height())));
        }
        let appbar = Rect::from_min_size(stage.min, vec2(stage.width(), APPBAR_H));
        let status = Rect::from_min_max(pos2(stage.min.x, win.max.y - STATUS_H), win.max);
        let tab_h = TAB_H * spring_to(ui.ctx(), "tabstrip", if self.tabs.len() > 1 { 1.0 } else { 0.0 }, 220.0, 24.0).clamp(0.0, 1.0);
        let content = Rect::from_min_max(pos2(stage.min.x, appbar.max.y + tab_h), pos2(stage.max.x, status.min.y));
        self.draw_content(ui, content);
        self.draw_status(ui, status, win);
        self.draw_appbar(ui, appbar, win);
        if tab_h > 1.0 {
            self.draw_tabs(ui, Rect::from_min_size(pos2(stage.min.x, appbar.max.y), vec2(stage.width(), tab_h)));
        }
        if self.grid_overlay {
            self.draw_grid_overlay(&p, win);
        }
        rim(&p, win, R_WIN, 1.4, if th.dark { c(255, 255, 255, 0.12) } else { c(255, 255, 255, 0.9) }, true, 42.0);
        p.rect_stroke(win.shrink(0.5), cr(R_WIN), egui::Stroke::new(1.0, if th.dark { c(255, 255, 255, 0.035) } else { c(255, 255, 255, 0.12) }), egui::StrokeKind::Inside);
        crate::overlays::WIN_RECT.with(|c| c.set(win));
    }

    fn draw_grid_overlay(&self, p: &Painter, win: Rect) {
        let pp = p.with_clip_rect(win);
        let col = alpha(self.th.accent, 0.10);
        let major = alpha(self.th.accent, 0.20);
        let mut x = win.left();
        let mut i = 0;
        while x < win.right() {
            pp.line_segment([pos2(x, win.top()), pos2(x, win.bottom())], egui::Stroke::new(1.0, if i % 8 == 0 { major } else { col }));
            x += 8.0;
            i += 1;
        }
        let mut y = win.top();
        i = 0;
        while y < win.bottom() {
            pp.line_segment([pos2(win.left(), y), pos2(win.right(), y)], egui::Stroke::new(1.0, if i % 8 == 0 { major } else { col }));
            y += 8.0;
            i += 1;
        }
        let r = Rect::from_min_size(win.left_bottom() + vec2(16.0, -52.0), vec2(250.0, 28.0));
        grad_fill(&pp, r, 14.0, alpha(self.th.accent, 0.95), alpha(self.th.accent, 0.85));
        pp.text(r.center(), Align2::CENTER_CENTER, "8 px grid \u{b7} 64 px major \u{b7} 4 px baseline", f_semi(11.0), self.th.on_accent());
    }

    // ───────────── sidebar ─────────────
    fn traffic(&self, ui: &Ui, origin: Pos2, id: &str) {
        let p = ui.painter();
        let cols = [("close", hex("#ff7c72"), hex("#f2554c")), ("min", hex("#ffd264"), hex("#f5b93c")), ("zoom", hex("#6fdc8c"), hex("#38bd62"))];
        let group = Rect::from_min_size(origin - vec2(2.0, 8.0), vec2(64.0, 16.0));
        let g_hover = ui.rect_contains_pointer(group);
        for (i, (n, a, b)) in cols.iter().enumerate() {
            let ctr = origin + vec2(6.0 + 20.0 * i as f32, 0.0);
            let rect = Rect::from_center_size(ctr, vec2(16.0, 16.0));
            let resp = self.hit(ui, rect, (id, n), Sense::click());
            let s = if resp.hovered() { 1.14 } else { 1.0 };
            let dot = Rect::from_center_size(ctr, vec2(12.0 * s, 12.0 * s));
            shadow(p, dot, 6.0, 0.5, 1.0, 0.0, c(20, 29, 48, 0.12));
            let (a, b) = if resp.is_pointer_button_down_on() { (mix(*a, Color32::BLACK, 0.12), mix(*b, Color32::BLACK, 0.12)) } else { (*a, *b) };
            grad_fill(p, dot, 6.0 * s, a, b);
            rim(p, dot, 6.0, 1.0, Color32::from_white_alpha(95), true, 7.0);
            if g_hover {
                let k = c(40, 20, 18, 0.55);
                let st = egui::Stroke::new(1.35, k);
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
                        p.line_segment([ctr + vec2(2.6, -2.6), ctr + vec2(0.4, -2.6)], st);
                        p.line_segment([ctr + vec2(2.6, -2.6), ctr + vec2(2.6, -0.4)], st);
                    }
                }
            }
            if resp.clicked() {
                self.push_act(match *n {
                    "close" => Act::WinClose,
                    "min" => Act::WinMin,
                    _ => Act::WinZoom,
                });
            }
        }
    }

    pub fn push_act(&self, a: Act) {
        ACTS.with(|q| q.borrow_mut().push(a));
    }

    pub fn drag_region(&mut self, ui: &Ui, rect: Rect, id: &str) {
        let r = self.hit(ui, rect, id, Sense::click_and_drag());
        if r.drag_started() && !IS_WEB {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }
        if r.dragged() && IS_WEB && !self.win_zoom {
            self.win_off += r.drag_delta();
        }
        if r.double_clicked() {
            if IS_WEB {
                self.win_zoom = !self.win_zoom;
            } else {
                let m = ui.ctx().input(|i| i.viewport().maximized.unwrap_or(false));
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Maximized(!m));
            }
        }
    }

    fn draw_sidebar(&mut self, ui: &mut Ui, sb: Rect) {
        let th = self.th.clone();
        let p = ui.painter().clone();
        let top = Rect::from_min_size(sb.min, vec2(sb.width(), 54.0));
        self.drag_region(ui, top, "sb-drag");
        self.traffic(ui, pos2(sb.left() + 20.0, sb.top() + 27.0), "sb");
        // settings lives here (the app bar owns the sidebar toggle)
        let gear = Rect::from_center_size(pos2(sb.right() - 28.0, sb.top() + 27.0), vec2(30.0, 30.0));
        if self.icon_button(ui, gear, "settings-minimalistic", "sb-settings", Btn::Quiet, true).on_hover_text("Settings  Ctrl+,").clicked() {
            self.push_act(Act::Settings);
        }
        // workspace switcher
        let ws = Rect::from_min_size(pos2(sb.left() + 12.0, sb.top() + 54.0), vec2(SIDEBAR_W - 24.0, 42.0));
        let r = self.hit(ui, ws, "workspace", Sense::click());
        if r.hovered() {
            p.rect_filled(ws, cr(12.0), th.row_hover);
        }
        let av = Rect::from_min_size(ws.min + vec2(7.0, 7.0), vec2(28.0, 28.0));
        let (a, b) = th.accent_grad();
        shadow(&p, av, 9.0, 2.0, 6.0, -1.0, alpha(th.accent, 0.35));
        grad_fill(&p, av, 9.0, a, b);
        rim(&p, av, 9.0, 1.0, Color32::from_white_alpha(130), true, 14.0);
        let user = self.user_name();
        p.text(av.center(), Align2::CENTER_CENTER, user.chars().next().unwrap_or('N').to_uppercase().to_string(), f_bold(13.0), Color32::WHITE);
        p.text(pos2(av.right() + 9.0, ws.center().y - 7.5), Align2::LEFT_CENTER, &user, f_semi(13.0), th.ink1);
        p.text(pos2(av.right() + 9.0, ws.center().y + 8.0), Align2::LEFT_CENTER, "Personal workspace", f_medium(11.0), th.ink3);
        let ch = pos2(ws.right() - 16.0, ws.center().y);
        let st = egui::Stroke::new(1.5, th.ink4);
        p.line_segment([ch + vec2(-3.5, -1.5), ch + vec2(0.0, 2.0)], st);
        p.line_segment([ch + vec2(0.0, 2.0), ch + vec2(3.5, -1.5)], st);
        if r.clicked() {
            self.menu = Some(Menu { kind: MenuKind::Workspace, pos: pos2(ws.left(), ws.bottom() + 6.0), target: None, device: None, sub: None, age: 0.0 });
        }

        let foot_h = 73.0;
        let card = Rect::from_min_size(pos2(sb.left() + 12.0, sb.bottom() - 14.0 - foot_h), vec2(SIDEBAR_W - 24.0, foot_h));
        self.draw_storage_card(ui, card);

        let nav = Rect::from_min_max(pos2(sb.left(), ws.bottom() + 6.0), pos2(sb.right(), card.top() - 8.0));
        // manual scroll keeps absolute painting correct and never hides rows behind the storage card
        let content_h = SB_H.with(|c| c.get());
        let max = (content_h - nav.height()).max(0.0);
        if ui.rect_contains_pointer(nav) {
            let d = ui.ctx().input(|i| i.smooth_scroll_delta.y);
            if d != 0.0 {
                self.sidebar_scroll = (self.sidebar_scroll - d).clamp(0.0, max);
            }
        }
        self.sidebar_scroll = self.sidebar_scroll.clamp(0.0, max);
        let pp = p.with_clip_rect(nav.intersect(ui.clip_rect()));
        let dy = -self.sidebar_scroll;
        let saved_clip = ui.clip_rect();
        ui.set_clip_rect(nav.intersect(saved_clip));
        self.sidebar_rows(ui, &pp, nav, dy);
        ui.set_clip_rect(saved_clip);
        if max > 0.5 {
            let track = Rect::from_min_size(pos2(nav.right() - 4.0, nav.top() + 4.0), vec2(3.0, nav.height() - 8.0));
            pp.rect_filled(track, cr(1.5), alpha(th.shadow, if th.dark { 0.25 } else { 0.05 }));
            let frac = nav.height() / content_h.max(1.0);
            let h = (track.height() * frac).max(24.0);
            let t = (self.sidebar_scroll / max).clamp(0.0, 1.0);
            let thumb = Rect::from_min_size(pos2(track.left(), track.top() + (track.height() - h) * t), vec2(3.0, h));
            pp.rect_filled(thumb, cr(1.5), alpha(th.ink3, 0.45));
        }
    }

    fn sidebar_rows(&mut self, ui: &Ui, p: &Painter, nav: Rect, dy: f32) {
        let th = self.th.clone();
        let x0 = nav.left() + 12.0;
        let w = SIDEBAR_W - 24.0;
        let cur = self.route.clone();
        let mut y = nav.top() + dy + 12.0;
        let label = |y: f32, s: &str, n: Option<usize>| {
            p.text(pos2(nav.left() + 20.0, y + 7.0), Align2::LEFT_CENTER, s.to_uppercase(), f_semi(T_LABEL), th.ink4);
            if let Some(n) = n {
                p.text(pos2(nav.left() + 20.0 + 64.0, y + 7.0), Align2::LEFT_CENTER, n.to_string(), f_medium(10.0), alpha(th.ink4, 0.8));
            }
        };
        let base = nav.top() + dy;
        // ── Favorites
        label(y, "Favorites", None);
        y += 22.0;
        let mut rows: Vec<(String, &'static str, Route, Option<usize>, Option<Location>)> = vec![];
        let places = self.places.clone();
        for pl in &places {
            rows.push((pl.title.clone(), pl.icon, Route::Loc(pl.loc.clone()), self.place_counts.get(&pl.loc).copied(), Some(pl.loc.clone())));
            if pl.id == "home" {
                rows.push(("Recents".into(), "clock-circle", Route::Recents, None, None));
            }
        }
        for pin in self.lib(|l| l.pins.clone()) {
            rows.insert(1, (pin.name(), "folder", Route::Loc(pin.clone()), None, Some(pin)));
        }
        for (name, icon, route, count, loc) in rows {
            let rect = Rect::from_min_size(pos2(x0, y), vec2(w, 32.0));
            let active = cur == route;
            let visible = rect.bottom() > nav.top() - 4.0 && rect.top() < nav.bottom() + 4.0;
            let dropping = self.drag.is_some() && visible && ui.rect_contains_pointer(rect);
            if visible {
                if let Some(l) = &loc {
                    DROP_TARGETS.with(|v| v.borrow_mut().push((DropTarget::Place(l.clone()), rect)));
                }
                self.sb_row(ui, rect, icon, &name, active, count.map(|c| c.to_string()), dropping, &format!("place-{}", name));
            }
            if visible {
                let r = self.hit(ui, rect, ("placeclick", &name), Sense::click());
                if r.clicked() {
                    self.push_act(Act::Nav(route.clone()));
                }
                if r.secondary_clicked() {
                    if let Some(l) = loc.clone() {
                        self.menu = Some(Menu { kind: MenuKind::Place, pos: ui.ctx().pointer_latest_pos().unwrap_or(rect.center()), target: Some(l), device: None, sub: None, age: 0.0 });
                    }
                }
            }
            y += 33.0;
        }
        // ── Locations
        y += 14.0;
        label(y, "Locations", None);
        y += 23.0;
        for d in self.devices.clone() {
            let rect = Rect::from_min_size(pos2(x0, y), vec2(w, 32.0));
            let active = matches!(&cur, Route::Loc(l) if d.loc.as_ref().is_some_and(|dl| l == dl || (d.kind != DeviceKind::Internal && l.fs == dl.fs && l.starts_with(dl))));
            let visible = rect.bottom() > nav.top() - 4.0 && rect.top() < nav.bottom() + 4.0;
            let dropping = self.drag.is_some() && visible && d.mounted && ui.rect_contains_pointer(rect);
            if visible {
                if let (Some(l), true) = (&d.loc, d.mounted) {
                    DROP_TARGETS.with(|v| v.borrow_mut().push((DropTarget::Place(l.clone()), rect)));
                }
                self.sb_row_device(ui, rect, &d, active, dropping);
                let r = self.hit(ui, rect, ("devclick", &d.id), Sense::click());
                if r.clicked() {
                    match (&d.loc, d.mounted, d.kind) {
                        (_, _, DeviceKind::Phone) => self.push_act(Act::Modal(Modal::Phone)),
                        (Some(l), true, _) => self.push_act(Act::Nav(Route::Loc(l.clone()))),
                        _ => self.push_act(Act::Mount(d.id.clone())),
                    }
                }
                if r.secondary_clicked() {
                    self.menu = Some(Menu { kind: MenuKind::Device, pos: ui.ctx().pointer_latest_pos().unwrap_or(rect.center()), target: d.loc.clone(), device: Some(d.id.clone()), sub: None, age: 0.0 });
                }
            }
            y += 33.0;
        }
        {
            let rect = Rect::from_min_size(pos2(x0, y), vec2(w, 32.0));
            let visible = rect.bottom() > nav.top() - 4.0 && rect.top() < nav.bottom() + 4.0;
            if visible {
                if self.sb_row(ui, rect, "add", "Mount a device\u{2026}", false, None, false, "mount-dev").clicked() {
                    self.push_act(Act::Modal(Modal::Devices));
                }
            }
            y += 33.0;
        }
        // ── Tags
        y += 14.0;
        let tags = self.lib(|l| l.tags.clone());
        label(y, "Tags", None);
        y += 23.0;
        for t in tags {
            let rect = Rect::from_min_size(pos2(x0, y), vec2(w, 32.0));
            let active = cur == Route::Tag(t.id.clone());
            let n = self.lib(|l| l.tag_count(&t.id));
            let visible = rect.bottom() > nav.top() - 4.0 && rect.top() < nav.bottom() + 4.0;
            let dropping = self.drag.is_some() && visible && ui.rect_contains_pointer(rect);
            if visible {
                DROP_TARGETS.with(|v| v.borrow_mut().push((DropTarget::Tag(t.id.clone()), rect)));
                let r = self.sb_row_tag(ui, rect, &t.name, hex(&t.color), active, n, dropping);
                if r.clicked() {
                    self.push_act(Act::Nav(Route::Tag(t.id.clone())));
                }
            }
            y += 33.0;
        }
        {
            let rect = Rect::from_min_size(pos2(x0, y), vec2(w, 32.0));
            let visible = rect.bottom() > nav.top() - 4.0 && rect.top() < nav.bottom() + 4.0;
            if visible && self.sb_row(ui, rect, "add", "New tag\u{2026}", false, None, false, "new-tag").clicked() {
                self.push_act(Act::Modal(Modal::NewTag));
            }
            y += 33.0;
        }
        y += 12.0;
        // store content height for the scroll clamp
        let content = (y - base + self.sidebar_scroll + 6.0).max(1.0);
        SB_H.with(|c| c.set(content));
    }

    #[allow(clippy::too_many_arguments)]
    fn sb_row(&self, ui: &Ui, rect: Rect, icon: &'static str, name: &str, active: bool, tail: Option<String>, drop: bool, id: &str) -> Response {
        let r = self.hit(ui, rect, ("sbrow", id), Sense::click());
        let p = ui.painter();
        let th = &self.th;
        if active || drop {
            p.rect_filled(rect, cr(10.0), th.accent_tint);
            if drop && !active {
                p.rect_stroke(rect.shrink(0.5), cr(10.0), egui::Stroke::new(1.5, alpha(th.accent, 0.6)), egui::StrokeKind::Inside);
            }
        } else if r.is_pointer_button_down_on() {
            p.rect_filled(rect, cr(10.0), th.row_press);
        } else if r.hovered() {
            p.rect_filled(rect, cr(10.0), th.row_hover);
        }
        let col = if active { th.accent_ink } else { th.ink2 };
        self.ico(p, icon, if active { Ic::Filled } else { Ic::Duo }, pos2(rect.left() + 19.0, rect.center().y), 16.0, if active { th.accent_ink } else { th.ink3 });
        let f = if active { f_semi(T_BODY) } else { f_medium(T_BODY) };
        let max = rect.width() - 58.0;
        let label = self.fit(p, name, &f, max);
        p.text(pos2(rect.left() + 37.0, rect.center().y), Align2::LEFT_CENTER, label, f, col);
        if let Some(t) = tail {
            p.text(pos2(rect.right() - 12.0, rect.center().y), Align2::RIGHT_CENTER, t, f_medium(T_META), if active { alpha(th.accent_ink, 0.9) } else { th.ink4 });
        }
        r
    }

    fn sb_row_tag(&self, ui: &Ui, rect: Rect, name: &str, color: Color32, active: bool, n: usize, drop: bool) -> Response {
        let r = self.hit(ui, rect, ("tag", name), Sense::click());
        let p = ui.painter();
        let th = &self.th;
        if active || drop {
            p.rect_filled(rect, cr(10.0), th.accent_tint);
            if drop && !active {
                p.rect_stroke(rect.shrink(0.5), cr(10.0), egui::Stroke::new(1.5, alpha(th.accent, 0.6)), egui::StrokeKind::Inside);
            }
        } else if r.hovered() {
            p.rect_filled(rect, cr(10.0), th.row_hover);
        }
        let c0 = pos2(rect.left() + 19.0, rect.center().y);
        shadow(p, Rect::from_center_size(c0, vec2(8.0, 8.0)), 4.0, 1.0, 4.0, 0.0, alpha(color, 0.5));
        grad_fill(p, Rect::from_center_size(c0, vec2(9.0, 9.0)), 4.5, mix(color, Color32::WHITE, 0.25), color);
        let f = if active { f_semi(T_BODY) } else { f_medium(T_BODY) };
        p.text(pos2(rect.left() + 37.0, rect.center().y), Align2::LEFT_CENTER, self.fit(p, name, &f, rect.width() - 82.0), f, if active { th.accent_ink } else { th.ink2 });
        p.text(pos2(rect.right() - 12.0, rect.center().y), Align2::RIGHT_CENTER, n.to_string(), f_medium(T_META), th.ink4);
        r
    }

    fn sb_row_device(&self, ui: &Ui, rect: Rect, d: &Device, active: bool, dropping: bool) -> Response {
        let r = self.hit(ui, rect, ("dev", &d.id), Sense::click());
        let p = ui.painter();
        let th = &self.th;
        if active || dropping {
            p.rect_filled(rect, cr(10.0), th.accent_tint);
            if dropping && !active {
                p.rect_stroke(rect.shrink(0.5), cr(10.0), egui::Stroke::new(1.5, alpha(th.accent, 0.6)), egui::StrokeKind::Inside);
            }
        } else if r.hovered() {
            p.rect_filled(rect, cr(10.0), th.row_hover);
        }
        let dim = if d.mounted { 1.0 } else { 0.55 };
        let col = if active { th.accent_ink } else { th.ink2 };
        self.ico(p, d.kind.icon(), if active { Ic::Filled } else { Ic::Duo }, pos2(rect.left() + 19.0, rect.center().y), 16.0, alpha(if active { th.accent_ink } else { th.ink3 }, dim));
        let name = self.fit(p, &d.name, &f_medium(T_BODY), rect.width() - 84.0);
        p.text(pos2(rect.left() + 37.0, rect.center().y), Align2::LEFT_CENTER, name, if active { f_semi(T_BODY) } else { f_medium(T_BODY) }, alpha(col, dim));
        let btn = Rect::from_center_size(pos2(rect.right() - 20.0, rect.center().y), vec2(24.0, 24.0));
        if (d.can_unmount || d.can_eject) && d.mounted {
            let b = self.hit(ui, btn, ("eject", &d.id), Sense::click());
            if r.hovered() || b.hovered() {
                if b.hovered() {
                    p.rect_filled(btn, cr(12.0), th.row_press);
                }
                self.ico(p, "upload-minimalistic", Ic::Duo, btn.center(), 15.0, th.ink2);
                if b.clicked() {
                    self.push_act(if d.can_eject { Act::Eject(d.id.clone()) } else { Act::Unmount(d.id.clone()) });
                }
            }
        } else if !d.mounted {
            p.text(pos2(rect.right() - 12.0, rect.center().y), Align2::RIGHT_CENTER, "Mount", f_semi(T_LABEL), alpha(th.accent_ink, if r.hovered() { 1.0 } else { 0.55 }));
        } else if d.kind == DeviceKind::Phone {
            if let Some(b) = d.battery {
                p.text(pos2(rect.right() - 12.0, rect.center().y), Align2::RIGHT_CENTER, format!("{}%", b), f_medium(T_META), th.ink4);
            }
        }
        r
    }

    fn draw_storage_card(&mut self, ui: &Ui, card: Rect) {
        let p = ui.painter().clone();
        let th = self.th.clone();
        let dev = self.current_device();
        elevate(&p, &th, card, 14.0, 2);
        grad_fill(&p, card, 14.0, th.card_top, th.card_bottom);
        rim(&p, card, 14.0, 1.2, th.rim, true, 30.0);
        let (name, used, text) = match &dev {
            Some(d) => (d.name.clone(), d.used_fraction(), d.capacity_text()),
            None => ("Computer".to_string(), 0.0, String::new()),
        };
        self.ico(&p, "ssd-square", Ic::Duo, pos2(card.left() + 21.0, card.top() + 21.0), 16.0, th.ink3);
        p.text(pos2(card.left() + 37.0, card.top() + 21.0), Align2::LEFT_CENTER, self.fit(&p, &name, &f_semi(T_META), card.width() - 90.0), f_semi(T_META), th.ink1);
        p.text(pos2(card.right() - 14.0, card.top() + 21.0), Align2::RIGHT_CENTER, format!("{:.0}%", used * 100.0), f_medium(T_META), th.ink3);
        let bar = Rect::from_min_size(pos2(card.left() + 12.0, card.top() + 37.0), vec2(card.width() - 24.0, 5.0));
        well(&p, &th, bar, 2.5);
        let fill = Rect::from_min_size(bar.min, vec2((bar.width() * used).max(5.0), 5.0));
        let (a, b) = th.accent_grad();
        grad_fill(&p, fill, 2.5, a, b);
        p.text(pos2(card.left() + 14.0, card.top() + 56.0), Align2::LEFT_CENTER, self.fit(&p, &text, &f_medium(11.0), card.width() - 70.0), f_medium(11.0), th.ink3);
        let gear = Rect::from_center_size(pos2(card.right() - 20.0, card.top() + 55.0), vec2(24.0, 24.0));
        if self.icon_button(ui, gear, "settings-minimalistic", "card-settings", Btn::Quiet, true).clicked() {
            self.push_act(Act::Settings);
        }
    }

    pub fn user_name(&self) -> String {
        #[cfg(not(target_arch = "wasm32"))]
        if !self.be.demo {
            let u = std::env::var("USER").or_else(|_| std::env::var("USERNAME")).unwrap_or_else(|_| "You".into());
            let mut c = u.chars();
            return c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or(u);
        }
        "Naledzi".into()
    }

    pub fn current_device(&self) -> Option<Device> {
        let loc = self.current_loc().or_else(|| Some(self.be.home()))?;
        self.devices
            .iter()
            .filter(|d| d.loc.as_ref().is_some_and(|dl| loc.fs == dl.fs && (loc.starts_with(dl) || dl.is_root())))
            .max_by_key(|d| d.loc.as_ref().map(|l| l.path.len()).unwrap_or(0))
            .cloned()
            .or_else(|| self.devices.first().cloned())
    }

    // ───────────── app bar ─────────────
    fn draw_appbar(&mut self, ui: &mut Ui, bar: Rect, win: Rect) {
        let th = self.th.clone();
        let p = ui.painter().clone();
        let cy = bar.center().y;
        self.drag_region(ui, bar, "appbar-drag");
        let sel = self.selected();
        // contextual bar only in selection mode; a plain click merely focuses an item
        let selection = self.select_mode;
        let st = self.st();
        let mut x = bar.left() + 21.0;
        if self.sidebar_open < 0.5 {
            self.traffic(ui, pos2(win.left() + 20.0, cy), "ab");
            x = win.left() + 20.0 + 64.0 + 10.0;
        }
        // leading: the app bar OWNS the sidebar toggle
        let toggle = Rect::from_min_size(pos2(x, cy - 17.0), vec2(34.0, 34.0));
        if self.icon_button(ui, toggle, "sidebar-minimalistic", "sb-toggle", if self.sidebar_open > 0.5 { Btn::Raised } else { Btn::Tonal }, true).on_hover_text("Toggle sidebar  Ctrl+\\").clicked() {
            self.push_act(Act::ToggleSidebar);
        }
        x = toggle.right() + 10.0;
        let cap = Rect::from_min_size(pos2(x, cy - 17.0), vec2(70.0, 34.0));
        elevate(&p, &th, cap, 17.0, 1);
        grad_fill(&p, cap, 17.0, th.control_top, th.control_bottom);
        rim(&p, cap, 17.0, 1.2, th.rim, true, 18.0);
        rim(&p, cap, 17.0, 1.0, th.rim_under, false, 6.0);
        let b1 = Rect::from_center_size(pos2(cap.left() + 18.0, cy), vec2(30.0, 30.0));
        let b2 = Rect::from_center_size(pos2(cap.right() - 18.0, cy), vec2(30.0, 30.0));
        if self.icon_button(ui, b1, "alt-arrow-left", "back", Btn::Quiet, self.can_back()).on_hover_text("Back  Alt+\u{2190}").clicked() {
            self.push_act(Act::Back);
        }
        if self.icon_button(ui, b2, "alt-arrow-right", "fwd", Btn::Quiet, self.can_fwd()).on_hover_text("Forward  Alt+\u{2192}").clicked() {
            self.push_act(Act::Forward);
        }
        x = cap.right() + 14.0;

        // right cluster — measured first so the breadcrumb/section chip can never overlap it
        let mut rx = bar.right() - 14.0;
        let more = Rect::from_min_size(pos2(rx - 34.0, cy - 17.0), vec2(34.0, 34.0));
        if self.icon_button(ui, more, "menu-dots", "more", Btn::Raised, true).on_hover_text("More").clicked() {
            self.menu = Some(Menu { kind: MenuKind::More, pos: pos2(more.right(), more.bottom() + 8.0), target: None, device: None, sub: None, age: 0.0 });
        }
        rx = more.left() - 8.0;
        let space = rx - x;
        let mut crumb_left = x;
        if selection && sel.is_empty() {
            let (dr, nrx) = self.adaptive_button(ui, rx, cy, "check-circle", "Done", "sel-done", Btn::Accent, true);
            if dr.clicked() {
                self.push_act(Act::ClearSelection);
            }
            let (ar, nrx) = self.adaptive_button(ui, nrx, cy, "checklist-minimalistic", "Select all", "sel-all", Btn::Raised, space > 520.0);
            if ar.clicked() {
                self.push_act(Act::SelectAll);
            }
            rx = nrx;
            crumb_left = toggle.left().max(x);
        } else if selection {
            // contextual actions, right-to-left, degrading to icons and then into More
            let can_pack = sel.iter().any(|e| e.kind == Kind::Archive) || sel.iter().all(|e| !e.is_dir);
            let mut actions: Vec<(&'static str, String, &str, Btn, Act)> = vec![];
            actions.push(("trash-bin-minimalistic", "Trash".into(), "sel-del", Btn::Danger, Act::Trash));
            actions.push(("menu-dots", "More".into(), "sel-more", Btn::Raised, Act::Modal(Modal::None)));
            if can_pack {
                actions.push(("archive", "Compress".into(), "sel-zip", Btn::Raised, Act::ArchiveCompress));
            }
            actions.push(("folder", "Move".into(), "sel-move", Btn::Raised, Act::CopyTo(true)));
            actions.push(("copy", "Copy".into(), "sel-copy", Btn::Raised, Act::CopyTo(false)));
            actions.push(("info-circle", "Info".into(), "sel-info", Btn::Raised, Act::Info));
            actions.push(("arrow-right-up", "Open".into(), "sel-open", Btn::Accent, Act::OpenSel));
            let labelled = space > 860.0;
            for (icon, label, id, kind, act) in actions {
                let as_label = match id {
                    "sel-copy" | "sel-move" | "sel-open" => true,
                    "sel-zip" => labelled,
                    _ => false,
                };
                if space < 240.0 && matches!(id, "sel-zip" | "sel-info") {
                    continue;
                }
                let (r, new_rx) = self.adaptive_button(ui, rx, cy, icon, &label, id, kind, if space > 700.0 || matches!(id, "sel-open" | "sel-copy") { as_label } else { false });
                rx = new_rx;
                if id == "sel-more" {
                    if r.clicked() {
                        self.menu = Some(Menu { kind: MenuKind::Item, pos: pos2(r.rect.left(), r.rect.bottom() + 8.0), target: Some(sel[0].loc.clone()), device: None, sub: None, age: 0.0 });
                    }
                } else if r.clicked() {
                    self.push_act(act.clone());
                }
                if id == "sel-del" {
                    r.on_hover_text("Move to Trash  Del");
                }
            }
            crumb_left = toggle.left().max(x);
        } else {
            // search: full field on wide bars, a button when space is tight
            let active = self.search_active();
            let compact = space < 560.0;
            let sw = if compact { 34.0 } else { 214.0 };
            let search_rect = Rect::from_min_size(pos2(rx - sw, cy - 17.0), vec2(sw, 34.0));
            if compact {
                let resp = self.icon_button(ui, search_rect, "magnifer", "search-compact", if active { Btn::Tonal } else { Btn::Raised }, true);
                if active {
                    let n = self.search.hits.len().min(99);
                    let badge = Rect::from_center_size(search_rect.right_top() + vec2(-2.0, 2.0), vec2(18.0, 18.0));
                    grad_fill(&p, badge, 9.0, th.accent, th.accent_press);
                    p.text(badge.center(), Align2::CENTER_CENTER, n.to_string(), f_bold(9.5), th.on_accent());
                }
                if resp.clicked() {
                    self.push_act(Act::SearchSheet(true));
                }
            } else {
                self.draw_search(ui, search_rect);
            }
            rx = search_rect.left() - 8.0;
            // command palette keycap
            if space > 720.0 {
                let pal = Rect::from_min_size(pos2(rx - 40.0, cy - 17.0), vec2(40.0, 34.0));
                let resp = self.icon_button(ui, pal, "terminal", "palette-btn", Btn::Raised, true);
                if resp.clicked() {
                    self.push_act(Act::Palette);
                }
                resp.on_hover_text("Command palette  Ctrl K");
                rx = pal.left() - 8.0;
            }
            let ask_full = space > 470.0;
            let (ar, nrx) = self.adaptive_button(ui, rx, cy, "stars-minimalistic", "Ask Smart", "ask", if self.ai.open { Btn::Accent } else { Btn::Tonal }, ask_full);
            if ar.clicked() {
                self.push_act(Act::Ask);
            }
            ar.on_hover_text("Ask Smart \u{2014} on-device search & tidy");
            rx = nrx;
            let seg = Rect::from_min_size(pos2(rx - 62.0, cy - 17.0), vec2(62.0, 34.0));
            if space > 250.0 {
                well(&p, &th, seg, 17.0);
                for (i, (icon, grid)) in [("widget", true), ("list-check", false)].into_iter().enumerate() {
                    let b = Rect::from_center_size(pos2(seg.left() + 17.0 + 28.0 * i as f32, cy), vec2(28.0, 28.0));
                    let on = st.grid == grid;
                    let resp = self.hit(ui, b, ("seg", i), Sense::click());
                    if on {
                        raised(&p, &th, b, 14.0, false, false);
                    }
                    let col = if on { th.accent_ink } else if resp.hovered() { th.ink1 } else { th.ink3 };
                    self.ico(&p, icon, if on { Ic::Filled } else if resp.hovered() { Ic::Duo } else { Ic::Outline }, b.center(), 16.0, col);
                    if resp.clicked() {
                        self.push_act(Act::SetGrid(grid));
                    }
                }
                rx = seg.left() - 8.0;
            }
            let (sr, nrx) = self.adaptive_button(ui, rx, cy, "sort-from-top-to-bottom", st.sort.short(), "sort", Btn::Raised, space > 420.0);
            if sr.clicked() {
                self.menu = Some(Menu { kind: MenuKind::Sort, pos: pos2(sr.rect.right(), sr.rect.bottom() + 8.0), target: None, device: None, sub: None, age: 0.0 });
            }
            rx = nrx;
        }
        // middle: selection summary or breadcrumbs
        let max_x = (rx - 14.0).max(crumb_left + 40.0);
        if selection {
            let f = f_semi(T_BODY);
            let label = if sel.is_empty() { "Select items".to_string() } else { format!("{} selected", sel.len()) };
            let lw = self.tw(&p, &label, &f);
            let sub = if sel.is_empty() { "\u{b7} tick to choose".to_string() } else { format!("\u{b7} {}", format_size(self.sel_total())) };
            let sw = self.tw(&p, &sub, &f_medium(T_META));
            let w = (lw + sw + 66.0).min((max_x - crumb_left).max(140.0));
            let chip = Rect::from_min_size(pos2(crumb_left, cy - 16.0), vec2(w, 32.0));
            grad_fill(&p, chip, 16.0, alpha(th.accent, 0.16), alpha(th.accent, 0.08));
            rim(&p, chip, 16.0, 1.0, alpha(Color32::WHITE, if th.dark { 0.12 } else { 0.7 }), true, 18.0);
            self.ico(&p, "check-circle", Ic::Filled, pos2(chip.left() + 18.0, cy), 15.0, th.accent);
            p.text(pos2(chip.left() + 32.0, cy), Align2::LEFT_CENTER, &label, f.clone(), th.accent_ink);
            p.text(pos2(chip.left() + 36.0 + lw, cy), Align2::LEFT_CENTER, self.fit(&p, &sub, &f_medium(T_META), (w - 60.0 - lw).max(0.0)), f_medium(T_META), th.ink3);
            let clr = Rect::from_center_size(pos2(chip.right() - 14.0, cy), vec2(20.0, 20.0));
            let cr_ = self.hit(ui, clr, "sel-clear", Sense::click());
            if cr_.hovered() {
                p.circle_filled(clr.center(), 9.0, th.row_press);
            }
            self.ico(&p, "close-circle", Ic::Duo, clr.center(), 15.0, th.ink2);
            if cr_.clicked() {
                self.push_act(Act::ClearSelection);
            }
        } else {
            let crumbs = self.crumbs();
            self.draw_crumbs(ui, pos2(x, cy), crumbs, max_x);
        }
    }

    fn instant_act(&mut self, a: Act) {
        match a {
            Act::ToggleTag(l, id) => {
                let targets = if self.sel.contains(&l) { self.sel_locs() } else { vec![l] };
                let all = targets.iter().all(|t| self.lib(|lib| lib.has_tag(t, &id)));
                for t in targets {
                    if all == self.lib(|lib| lib.has_tag(&t, &id)) {
                        self.be.library.write().unwrap().toggle_tag(&t, &id);
                    }
                }
            }
            other => self.acts.push(other),
        }
    }

    fn draw_crumbs(&mut self, ui: &mut Ui, left: Pos2, crumbs: Vec<(String, Route)>, max_x: f32) {
        let p = ui.painter().clone();
        let th = self.th.clone();
        let n = crumbs.len();
        let fm = f_medium(T_BODY);
        let fs = f_semi(T_BODY);
        let sep = 20.0;
        let avail = (max_x - left.x).max(0.0);
        if avail < 110.0 {
            // compact path button — still reachable, never overlapping the right controls
            let r = Rect::from_min_size(pos2(left.x, left.y - 14.0), vec2(avail.min(160.0), 28.0));
            let cur = crumbs.last().map(|c| c.0.clone()).unwrap_or_default();
            let resp = self.hit(ui, r, "crumb-compact", Sense::click());
            if resp.hovered() {
                p.rect_filled(r, cr(9.0), th.row_hover);
            }
            self.ico(&p, "folder", Ic::Duo, pos2(r.left() + 12.0, r.center().y), 14.0, th.ink3);
            p.text(pos2(r.left() + 26.0, r.center().y), Align2::LEFT_CENTER, self.fit(&p, &cur, &fs, r.width() - 34.0), fs, th.ink1);
            if resp.clicked() {
                let home = self.be.home();
                self.push_act(Act::Nav(Route::Loc(home)));
            }
            return;
        }
        let widths: Vec<f32> = crumbs.iter().enumerate().map(|(i, (s, _))| self.tw(&p, s, if i + 1 == n { &fs } else { &fm }) + 16.0).collect();
        let total: f32 = widths.iter().sum::<f32>() + sep * (n.saturating_sub(1)) as f32;
        let collapse = total > avail && n > 3;
        let mut x = left.x;
        let mut ellipsis = false;
        for (i, (name, route)) in crumbs.iter().enumerate() {
            if collapse && i >= 1 && i + 2 < n {
                if !ellipsis {
                    ellipsis = true;
                    p.text(pos2(x + 2.0, left.y), Align2::LEFT_CENTER, "\u{2026}", fm.clone(), th.ink3);
                    x += 20.0;
                    p.text(pos2(x + 2.0, left.y), Align2::LEFT_CENTER, "\u{203a}", f_regular(T_BODY), th.ink4);
                    x += sep;
                }
                continue;
            }
            let last = i + 1 == n;
            let room = (max_x - x - if last { 0.0 } else { 24.0 }).max(40.0);
            let w = widths[i].min(room);
            let r = Rect::from_min_size(pos2(x, left.y - 14.0), vec2(w, 28.0));
            if let Route::Loc(l) = route {
                DROP_TARGETS.with(|v| v.borrow_mut().push((DropTarget::Place(l.clone()), r)));
            }
            let resp = self.hit(ui, r, ("crumb", i), Sense::click());
            if resp.secondary_clicked() {
                if let Route::Loc(l) = route {
                    self.menu = Some(Menu { kind: MenuKind::Crumb, pos: ui.ctx().pointer_latest_pos().unwrap_or(r.center()), target: Some(l.clone()), device: None, sub: None, age: 0.0 });
                }
            }
            if resp.middle_clicked() {
                self.push_act(Act::NewTab(Some(route.clone())));
            }
            if resp.hovered() && !last {
                p.rect_filled(r, cr(9.0), th.row_hover);
            }
            let f = if last { fs.clone() } else { fm.clone() };
            let t = self.fit(&p, name, &f, w - 14.0);
            p.text(pos2(x + 6.0, left.y), Align2::LEFT_CENTER, t, f, if last { th.ink1 } else if resp.hovered() { th.ink1 } else { th.ink3 });
            x += w;
            if !last {
                p.text(pos2(x + 4.0, left.y), Align2::LEFT_CENTER, "\u{203a}", f_regular(T_BODY), th.ink4);
                x += sep;
            }
            if resp.clicked() && !last {
                self.push_act(Act::Nav(route.clone()));
            }
        }
    }

    /// Browser-style tab strip for the file manager (shown from two tabs up; folders can be dropped onto a tab).
    fn draw_tabs(&mut self, ui: &mut Ui, strip: Rect) {
        let p = ui.painter().clone();
        let th = self.th.clone();
        let n = self.tabs.len();
        let add_w = 34.0;
        let inner = Rect::from_min_max(pos2(strip.left() + PAD_X - 6.0, strip.top() + 3.0), pos2(strip.right() - PAD_X + 6.0 - add_w - 6.0, strip.bottom() - 5.0));
        let tw = (inner.width() / n as f32).clamp(84.0, 200.0);
        let labels: Vec<String> = (0..n).map(|i| if i == self.tab_i { self.title() } else { self.route_label(&self.tabs[i].route) }).collect();
        for i in 0..n {
            let r = Rect::from_min_size(pos2(inner.left() + tw * i as f32, inner.top()), vec2(tw - 6.0, inner.height()));
            let active = i == self.tab_i;
            let resp = self.hit(ui, r, ("tab", i), Sense::click());
            let a = self.anim(ui, ("tabh", i), resp.hovered());
            let dropping = self.drag.is_some() && ui.rect_contains_pointer(r) && !active;
            if let Route::Loc(l) = if active { &self.route } else { &self.tabs[i].route } {
                DROP_TARGETS.with(|v| v.borrow_mut().push((DropTarget::Place(l.clone()), r)));
            }
            if active {
                raised(&p, &th, r, 13.0, false, false);
            } else if dropping {
                grad_fill(&p, r, 13.0, th.accent_tint, th.accent_tint2);
                p.rect_stroke(r, cr(13.0), egui::Stroke::new(1.5, alpha(th.accent, 0.7)), egui::StrokeKind::Inside);
            } else if a > 0.01 {
                p.rect_filled(r, cr(13.0), alpha(th.row_hover, a));
            }
            self.ico(&p, "folder", if active || a > 0.5 { Ic::Duo } else { Ic::Outline }, pos2(r.left() + 16.0, r.center().y), 15.0, if active { th.accent_ink } else { th.ink3 });
            let showx = n > 1 && (active || resp.hovered());
            let lab = self.fit(&p, &labels[i], &f_semi(T_BODY), r.width() - 32.0 - if showx { 22.0 } else { 0.0 });
            p.text(pos2(r.left() + 30.0, r.center().y), Align2::LEFT_CENTER, lab, if active { f_semi(T_BODY) } else { f_medium(T_BODY) }, if active { th.ink1 } else { th.ink2 });
            if showx {
                let xr = Rect::from_center_size(pos2(r.right() - 14.0, r.center().y), vec2(20.0, 20.0));
                let xh = self.hit(ui, xr, ("tabx", i), Sense::click());
                if xh.hovered() {
                    p.circle_filled(xr.center(), 9.0, th.row_press);
                }
                self.ico(&p, "close-circle", if xh.hovered() { Ic::Duo } else { Ic::Outline }, xr.center(), 14.0, th.ink3);
                if xh.clicked() {
                    self.push_act(Act::CloseTab(i));
                    continue;
                }
            }
            if resp.clicked() && !active {
                self.push_act(Act::SwitchTab(i));
            }
            if resp.middle_clicked() {
                self.push_act(Act::CloseTab(i));
            }
        }
        let plus = Rect::from_center_size(pos2((inner.left() + tw * n as f32 + 18.0).min(strip.right() - PAD_X - 4.0), inner.center().y), vec2(30.0, 30.0));
        if self.icon_button(ui, plus, "add", "tab-new", Btn::Quiet, true).on_hover_text("New tab  Ctrl+T").clicked() {
            self.push_act(Act::NewTab(None));
        }
    }

    fn draw_search(&mut self, ui: &mut Ui, r: Rect) {
        let p = ui.painter().clone();
        let th = self.th.clone();
        let focused = self.search.focused || self.search.sheet;
        // focus is spring-driven and keeps the recessed depth: the well stays inset, light only tints it
        let f = spring_to(ui.ctx(), "search-focus", if focused { 1.0 } else { 0.0 }, 240.0, 22.0).clamp(0.0, 1.15);
        if f > 0.01 {
            shadow(&p, r, 17.0, 3.0, 16.0, 0.0, alpha(th.accent, 0.22 * f));
        }
        well(&p, &th, r, 17.0);
        if f > 0.01 {
            grad_fill(&p, r, 17.0, alpha(th.accent, 0.12 * f), alpha(th.accent, 0.035 * f));
            rim(&p, r, 17.0, 3.0, alpha(th.shadow, (if th.dark { 0.45 } else { 0.10 }) * f), true, 14.0);
            rim(&p, r, 17.0, 1.3, alpha(th.accent, 0.75 * f), true, r.height());
            rim(&p, r, 17.0, 1.0, alpha(Color32::WHITE, (if th.dark { 0.16 } else { 0.9 }) * f), false, 6.0);
            p.rect_stroke(r.expand(1.5), cr(18.5), egui::Stroke::new(1.5, alpha(th.accent, 0.38 * f)), egui::StrokeKind::Outside);
        }
        self.ico(&p, "magnifer", if focused { Ic::Duo } else { Ic::Outline }, pos2(r.left() + 20.0, r.center().y), 16.0, if focused { th.accent_ink } else { th.ink3 });
        let active = self.search_active();
        let right_pad = if active { 34.0 } else { 46.0 };
        let te = Rect::from_min_max(pos2(r.left() + 36.0, r.top() + 5.0), pos2(r.right() - right_pad, r.bottom() - 5.0));
        let hint = format!("Search {}", self.title());
        let mut q = std::mem::take(&mut self.search.query);
        let id = Id::unique("search-input");
        let before = q.clone();
        let out = ui.put(te, egui::TextEdit::singleline(&mut q).id(id).frame(egui::Frame::NONE).hint_text(egui::RichText::new(hint).color(th.ink4)).font(f_medium(T_BODY)).text_color(th.ink1).vertical_align(egui::Align::Center).margin(egui::Margin::ZERO));
        if self.search.focus_next {
            out.request_focus();
            self.search.focus_next = false;
        }
        self.search.focused = out.has_focus();
        self.search.query = q;
        if before != self.search.query {
            self.search.dirty_at = self.time;
            if !self.search.query.trim().is_empty() {
                self.acts.push(Act::RunSearch(self.search.query.clone()));
            }
        }
        if out.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.clear_search();
            out.surrender_focus();
        }
        if active {
            let b = Rect::from_center_size(pos2(r.right() - 18.0, r.center().y), vec2(24.0, 24.0));
            let resp = self.hit(ui, b, "search-clear", Sense::click());
            self.ico(&p, "close-circle", if resp.hovered() { Ic::Filled } else { Ic::Duo }, b.center(), 16.0, th.ink3);
            if resp.clicked() {
                self.push_act(Act::ClearSearch);
            }
        } else {
            let kb = Rect::from_min_max(pos2(r.right() - 48.0, r.top()), r.right_bottom());
            self.keycap(&p, pos2(r.right() - 11.0, r.center().y), "\u{2318}K", false);
            if self.hit(ui, kb, "search-k", Sense::click()).clicked() {
                self.push_act(Act::Palette);
            }
        }
    }

    // ───────────── status bar + player ─────────────
    fn draw_status(&mut self, ui: &mut Ui, bar: Rect, _win: Rect) {
        let p = ui.painter().clone();
        let th = self.th.clone();
        let mut m = egui::epaint::Mesh::default();
        let seam = alpha(th.shadow, if th.dark { 0.35 } else { 0.06 });
        for (x, col) in [(bar.left(), Color32::TRANSPARENT), (bar.left() + 40.0, seam), (bar.right() - 40.0, seam), (bar.right(), Color32::TRANSPARENT)] {
            m.vertices.push(egui::epaint::Vertex { pos: pos2(x, bar.top()), uv: egui::epaint::WHITE_UV, color: col });
            m.vertices.push(egui::epaint::Vertex { pos: pos2(x, bar.top() + 1.0), uv: egui::epaint::WHITE_UV, color: col });
        }
        for i in 0..3u32 {
            m.indices.extend([2 * i, 2 * i + 1, 2 * i + 2, 2 * i + 1, 2 * i + 3, 2 * i + 2]);
        }
        p.add(egui::Shape::mesh(m));
        let cy = bar.center().y;
        let sel = self.selected();
        let mut x = bar.left() + 16.0;
        let f = f_semi(T_META);
        let fm = f_medium(T_META);
        let (head, tail): (String, String) = if self.search_active() {
            (format!("{} result{}", self.search.hits.len(), if self.search.hits.len() == 1 { "" } else { "s" }), if self.search.done { format!("{} scanned \u{b7} {} ms{}", self.search.scanned, self.search.ms, if self.search.cached { " \u{b7} indexed" } else { "" }) } else { "searching\u{2026}".into() })
        } else if !sel.is_empty() {
            (format!("{} selected", sel.len()), format!("{} of {} items", format_size(self.sel_total()), self.items.len()))
        } else {
            (format!("{} item{}", self.items.len(), if self.items.len() == 1 { "" } else { "s" }), format_size(self.total_bytes()))
        };
        let r = p.text(pos2(x, cy), Align2::LEFT_CENTER, head, f, th.ink2);
        x = r.right() + 8.0;
        let dot = p.circle_filled(pos2(x, cy), 1.5, th.ink4);
        let _ = dot;
        x += 8.0;
        p.text(pos2(x, cy), Align2::LEFT_CENTER, self.fit(&p, &tail, &fm, 190.0), fm.clone(), th.ink3);

        let mut rx = bar.right() - 16.0;
        if bar.width() > 900.0 {
            for (k, label) in [("D", "theme"), ("G", "grid"), ("\u{2318}K", "commands")].into_iter().rev() {
                let w = self.tw(&p, label, &fm);
                p.text(pos2(rx, cy), Align2::RIGHT_CENTER, label, fm.clone(), th.ink4);
                let kw = self.keycap(&p, pos2(rx - w - 6.0, cy), k, false);
                rx -= w + 6.0 + kw + 14.0;
            }
            rx -= 8.0;
        }
        // player
        if self.be.audio.is_loaded() {
            let title = self.audio_loc.as_ref().map(|l| l.name()).unwrap_or_else(|| "Audio".into());
            let playing = self.be.audio.is_playing();
            let dur = self.be.audio.duration().unwrap_or(0.0);
            let pos = self.be.audio.position();
            let w = 232.0;
            let pill = Rect::from_min_size(pos2(rx - w, cy - 12.0), vec2(w, 24.0));
            elevate(&p, &th, pill, 12.0, 1);
            grad_fill(&p, pill, 12.0, th.card_top, th.card_bottom);
            rim(&p, pill, 12.0, 1.0, th.rim, true, 14.0);
            let play = Rect::from_center_size(pos2(pill.left() + 15.0, cy), vec2(20.0, 20.0));
            let pr = self.hit(ui, play, "player-toggle", Sense::click());
            grad_fill(&p, play, 10.0, th.accent, th.accent_press);
            self.ico(&p, if playing { "pause" } else { "play" }, Ic::Filled, play.center(), 12.0, th.on_accent());
            if pr.clicked() {
                if let Some(l) = self.audio_loc.clone() {
                    self.push_act(Act::TogglePlayPause(l));
                }
            }
            let tw = self.tw(&p, &title, &f_semi(11.0));
            p.text(pos2(pill.left() + 30.0, cy - 1.0), Align2::LEFT_CENTER, self.fit(&p, &title, &f_semi(11.0), pill.width() - 74.0), f_semi(11.0), th.ink1);
            let frac = if dur > 0.0 { (pos / dur).clamp(0.0, 1.0) } else { 0.0 };
            let track = Rect::from_min_size(pos2(pill.left() + 30.0, cy + 7.0), vec2((pill.width() - 44.0).max(20.0), 2.5));
            well(&p, &th, track, 1.2);
            grad_fill(&p, Rect::from_min_size(track.min, vec2(track.width() * frac, 2.5)), 1.2, th.accent, th.accent_ink);
            let stop = Rect::from_center_size(pos2(pill.right() - 12.0, cy), vec2(18.0, 18.0));
            let sr = self.hit(ui, stop, "player-close", Sense::click());
            self.ico(&p, "close-circle", Ic::Duo, stop.center(), 14.0, if sr.hovered() { th.ink1 } else { th.ink3 });
            if sr.clicked() {
                self.be.audio.stop();
                self.audio_loc = None;
            }
            let _ = tw;
            rx = pill.left() - 10.0;
        }
        if let Some(d) = self.current_device() {
            let t = if d.total > 0 { format!("{} \u{b7} {} free", d.name, format_size(d.free)) } else { d.name.clone() };
            let w = self.tw(&p, &t, &fm).min(190.0);
            if rx - w > 380.0 {
                p.text(pos2(rx, cy), Align2::RIGHT_CENTER, self.fit(&p, &t, &fm, 190.0), fm.clone(), th.ink3);
                self.ico(&p, "check-circle", Ic::Duo, pos2(rx - w - 12.0, cy), 14.0, th.good);
                rx -= w + 30.0;
            }
        }
        if let Some(j) = self.jobs.iter().find(|j| j.state == JobState::Running) {
            let frac = j.done as f32 / j.total.max(1) as f32;
            let w = 190.0;
            let pill = Rect::from_min_size(pos2(rx - w, cy - 11.0), vec2(w, 22.0));
            well(&p, &th, pill, 11.0);
            let fill = Rect::from_min_size(pill.min, vec2((w * frac).max(14.0), 22.0));
            let (a, b) = th.accent_grad();
            grad_fill(&p, fill, 11.0, alpha(a, 0.9), alpha(b, 0.9));
            let t = self.fit(&p, &j.title, &f_semi(10.5), w - 18.0);
            p.text(pill.center(), Align2::CENTER_CENTER, t, f_semi(10.5), if frac > 0.5 { th.on_accent() } else { th.ink2 });
        }
    }

    pub fn scroll_sidebar(&mut self, delta: f32) {
        self.sidebar_scroll = (self.sidebar_scroll + delta).max(0.0);
    }
}

thread_local! {
    pub static SB_H: std::cell::Cell<f32> = const { std::cell::Cell::new(0.0) };
}
