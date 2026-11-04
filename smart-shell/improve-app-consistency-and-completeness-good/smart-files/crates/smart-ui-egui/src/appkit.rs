//! Shared toolkit for the app suite (Files, Photos, Terminal): the unified window frame, draggable scrollbar,
//! switch / slider / segmented controls, the per-app settings sheet and the per-app context menu.
//! Every app composes these, so spacing, depth and behaviour stay identical everywhere.
use crate::app::*;
use crate::chrome::*;
use crate::icons::Ic;
use crate::overlays::MI;
use crate::paint::*;
use crate::shell::AppId;
use crate::theme::*;
use egui::{pos2, vec2, Align2, Color32, Id, Pos2, Rect, Response, Sense, Ui};

/// Geometry of a unified app window: full-height sidebar, toolbar that shares the surface, content stage.
#[derive(Clone, Copy)]
pub struct Frame {
    pub win: Rect,
    /// Full-height sidebar column (zero width when collapsed).
    pub side: Rect,
    /// Sidebar area below the traffic-light row.
    pub side_body: Rect,
    pub bar: Rect,
    pub stage: Rect,
    /// First free x in the toolbar (after traffic lights / sidebar toggle).
    pub lead_x: f32,
    pub right_x: f32,
    pub active: bool,
}

impl Frame {
    pub fn has_side(&self) -> bool {
        self.side.width() > 8.0
    }
}

pub struct AppMenu {
    pub pos: Pos2,
    pub items: Vec<MI>,
    pub born: f64,
}

#[derive(Clone, Copy)]
pub enum SVal {
    Bool(bool),
    Idx(usize),
    F(f32),
}

#[derive(Clone)]
pub enum SRow {
    Toggle { icon: &'static str, label: &'static str, sub: &'static str, on: bool },
    Choice { icon: &'static str, label: &'static str, sub: &'static str, opts: &'static [&'static str], sel: usize },
    Slider { icon: &'static str, label: &'static str, sub: &'static str, v: f32, lo: f32, hi: f32, unit: &'static str },
    Button { icon: &'static str, label: &'static str, sub: &'static str, cta: &'static str },
}

impl SmartApp {
    // ───────────── window controls ─────────────
    /// macOS-style traffic lights for suite windows. Returns (close, minimise, zoom).
    pub fn app_traffic(&self, ui: &Ui, origin: Pos2, id: AppId, active: bool) -> (bool, bool, bool) {
        let th = &self.th;
        let p = ui.painter();
        let cols = [("close", hex("#ff7c72"), hex("#f2554c")), ("min", hex("#ffd264"), hex("#f5b93c")), ("zoom", hex("#6fdc8c"), hex("#38bd62"))];
        let g_hover = ui.rect_contains_pointer(Rect::from_min_size(origin - vec2(2.0, 8.0), vec2(64.0, 16.0)));
        let mut out = (false, false, false);
        for (i, (n, a, b)) in cols.iter().enumerate() {
            let ctr = origin + vec2(6.0 + 20.0 * i as f32, 0.0);
            let resp = self.hit(ui, Rect::from_center_size(ctr, vec2(16.0, 16.0)), ("atl", id, *n), Sense::click());
            let dot = Rect::from_center_size(ctr, vec2(12.0, 12.0) * if resp.hovered() { 1.14 } else { 1.0 });
            let (a, b) = if active || g_hover { (*a, *b) } else { (alpha(th.ink4, 0.55), alpha(th.ink4, 0.42)) };
            let (a, b) = if resp.is_pointer_button_down_on() { (mix(a, Color32::BLACK, 0.12), mix(b, Color32::BLACK, 0.12)) } else { (a, b) };
            shadow(p, dot, 6.0, 0.5, 1.0, 0.0, c(20, 29, 48, 0.12));
            grad_fill(p, dot, 6.0, a, b);
            rim(p, dot, 6.0, 1.0, Color32::from_white_alpha(95), true, 7.0);
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
                        p.line_segment([ctr + vec2(2.6, -2.6), ctr + vec2(0.4, -2.6)], st);
                        p.line_segment([ctr + vec2(2.6, -2.6), ctr + vec2(2.6, -0.4)], st);
                    }
                }
            }
            if resp.clicked() {
                match *n {
                    "close" => out.0 = true,
                    "min" => out.1 = true,
                    _ => out.2 = true,
                }
            }
        }
        out
    }

    // ───────────── controls ─────────────
    /// Draggable overlay scrollbar + wheel scrolling for a hand-painted column. Returns the scroll offset.
    pub fn vscroll(&self, ui: &Ui, area: Rect, key: &str, content_h: f32) -> f32 {
        let ctx = ui.ctx();
        let id = Id::new(("vscroll", key));
        let mut off: f32 = ctx.data_mut(|d| d.get_temp(id)).unwrap_or(0.0);
        let max = (content_h - area.height()).max(0.0);
        let over = ui.rect_contains_pointer(area);
        if over {
            let d = ctx.input(|i| i.smooth_scroll_delta.y);
            if d != 0.0 {
                off -= d;
            }
        }
        if max > 0.5 {
            let track = Rect::from_min_max(pos2(area.right() - 9.0, area.top() + 4.0), pos2(area.right() - 3.0, area.bottom() - 4.0));
            let frac = (area.height() / content_h.max(1.0)).clamp(0.05, 1.0);
            let th_h = (track.height() * frac).max(30.0);
            let span = (track.height() - th_h).max(1.0);
            let hit = self.hit(ui, Rect::from_min_max(pos2(track.left() - 6.0, track.top()), pos2(track.right() + 3.0, track.bottom())), ("vs", key), Sense::click_and_drag());
            let t = (off / max).clamp(0.0, 1.0);
            let thumb0 = Rect::from_min_size(pos2(track.left(), track.top() + span * t), vec2(track.width(), th_h));
            if hit.dragged() {
                off += hit.drag_delta().y * max / span;
            } else if hit.clicked() {
                if let Some(pp) = hit.interact_pointer_pos() {
                    if !thumb0.contains(pp) {
                        off = ((pp.y - track.top() - th_h / 2.0) / span).clamp(0.0, 1.0) * max;
                    }
                }
            }
            off = off.clamp(0.0, max);
            let t = (off / max).clamp(0.0, 1.0);
            let a = self.anim(ui, ("vsa", key), hit.hovered() || hit.dragged());
            let vis = if over || hit.dragged() { 1.0 } else { 0.0 };
            let vis = spring_to(ctx, ("vsv", key), vis, 200.0, 24.0).clamp(0.0, 1.0);
            let w = 3.0 + 3.0 * a;
            let thumb = Rect::from_min_size(pos2(track.right() - w, track.top() + span * t), vec2(w, th_h));
            let p = ui.painter().with_clip_rect(area.expand(6.0));
            if a > 0.01 {
                p.rect_filled(Rect::from_min_max(pos2(track.right() - w, track.top()), track.max), cr(w / 2.0), alpha(self.th.shadow, (if self.th.dark { 0.25 } else { 0.06 }) * a));
            }
            grad_fill(&p, thumb, w / 2.0, alpha(self.th.ink3, (0.30 + 0.35 * a) * (0.35 + 0.65 * vis)), alpha(self.th.ink3, (0.22 + 0.3 * a) * (0.35 + 0.65 * vis)));
        }
        off = off.clamp(0.0, max);
        ctx.data_mut(|d| d.insert_temp(id, off));
        off
    }

    /// iOS-style switch with spring knob.
    pub fn switch(&self, ui: &Ui, r: Rect, on: bool, id: impl std::hash::Hash + std::fmt::Debug + Copy) -> bool {
        let th = &self.th;
        let p = ui.painter();
        let resp = self.hit(ui, r, ("sw", id), Sense::click());
        let t = spring_to(ui.ctx(), ("swk", id), if on { 1.0 } else { 0.0 }, 420.0, 26.0);
        let ct = t.clamp(0.0, 1.0);
        well(p, th, r, r.height() / 2.0);
        if ct > 0.01 {
            let (a, b) = th.accent_grad();
            let f = Rect::from_min_size(r.min, vec2(r.height() + (r.width() - r.height()) * ct, r.height()));
            grad_fill(p, f, r.height() / 2.0, alpha(a, ct), alpha(b, ct));
            rim(p, f, r.height() / 2.0, 1.0, Color32::from_white_alpha(100), true, 8.0);
        }
        let k = r.height() - 6.0 + if resp.is_pointer_button_down_on() { 3.0 } else { 0.0 };
        let cx = r.left() + 3.0 + k / 2.0 + (r.width() - r.height()) * t;
        let knob = Rect::from_center_size(pos2(cx, r.center().y), vec2(k, r.height() - 6.0));
        raised(p, th, knob, (r.height() - 6.0) / 2.0, resp.hovered(), false);
        resp.clicked() != false && true
    }

    /// Horizontal slider (0..1) with a recessed track and a raised knob. Returns the (possibly new) value.
    pub fn slider(&self, ui: &Ui, track: Rect, v: f32, id: impl std::hash::Hash + std::fmt::Debug + Copy) -> (f32, Response) {
        let th = &self.th;
        let p = ui.painter();
        let resp = self.hit(ui, track.expand2(vec2(9.0, 10.0)), ("sl", id), Sense::click_and_drag());
        let mut nv = v;
        if resp.dragged() || resp.clicked() {
            if let Some(pp) = resp.interact_pointer_pos() {
                nv = ((pp.x - track.left()) / track.width()).clamp(0.0, 1.0);
            }
        }
        let shown = spring_to(ui.ctx(), ("slk", id), nv, 420.0, 32.0).clamp(0.0, 1.0);
        let h = track.height();
        well(p, th, track, h / 2.0);
        let fill = Rect::from_min_size(track.min, vec2((track.width() * shown).max(h), h));
        let (c0, c1) = th.accent_grad();
        grad_fill(p, fill, h / 2.0, c0, c1);
        rim(p, fill, h / 2.0, 1.0, Color32::from_white_alpha(110), true, 6.0);
        let kd = h + 8.0;
        let knob = Rect::from_center_size(pos2(track.left() + h / 2.0 + (track.width() - h) * shown, track.center().y), vec2(kd, kd) * if resp.dragged() { 1.12 } else { 1.0 });
        raised(p, th, knob, kd / 2.0, resp.hovered(), resp.dragged());
        (nv, resp)
    }

    /// Segmented control: recessed track, spring-sliding raised pill. Icons when `icons` is true, else text.
    pub fn segment(&self, ui: &Ui, r: Rect, opts: &[(&'static str, &str)], icons: bool, sel: usize, id: &str) -> Option<usize> {
        let th = &self.th;
        let p = ui.painter();
        let n = opts.len().max(1);
        well(p, th, r, r.height() / 2.0);
        let cw = (r.width() - 6.0) / n as f32;
        let x = spring_to(ui.ctx(), ("seg", id), sel as f32, 380.0, 28.0);
        let pill = Rect::from_min_size(pos2(r.left() + 3.0 + cw * x, r.top() + 3.0), vec2(cw, r.height() - 6.0));
        raised(p, th, pill, pill.height() / 2.0, false, false);
        let mut out = None;
        for (i, (icon, label)) in opts.iter().enumerate() {
            let cell = Rect::from_min_size(pos2(r.left() + 3.0 + cw * i as f32, r.top() + 3.0), vec2(cw, r.height() - 6.0));
            let resp = self.hit(ui, cell, ("segc", id, i), Sense::click());
            let on = i == sel;
            let a = self.anim(ui, ("segh", id, i), resp.hovered());
            let col = if on { th.accent_ink } else { mix(th.ink3, th.ink1, a) };
            if icons {
                self.ico(p, icon, if on { Ic::Duo } else { Ic::Outline }, cell.center(), 16.0, col);
                if !label.is_empty() {
                    resp.clone().on_hover_text(*label);
                }
            } else {
                p.text(cell.center(), Align2::CENTER_CENTER, *label, if on { f_semi(T_META) } else { f_medium(T_META) }, col);
            }
            if resp.clicked() && !on {
                out = Some(i);
            }
        }
        out
    }

    // ───────────── per-app context menu ─────────────
    pub fn open_app_menu(&mut self, pos: Pos2, items: Vec<MI>) {
        self.shell.appmenu = Some(AppMenu { pos, items, born: self.time });
        self.menu = None;
    }

    pub fn draw_app_menu(&mut self, ui: &mut Ui, screen: Rect) {
        let Some(m) = self.shell.appmenu.as_ref() else { return };
        let items = Self::tidy_menu(m.items.clone());
        let (mut pos, age) = (m.pos, (self.time - m.born) as f32);
        if items.is_empty() {
            self.shell.appmenu = None;
            return;
        }
        let p = ui.painter().clone();
        let w = self.menu_width(&p, &items, 232.0);
        let h = Self::menu_height(&items);
        pos.x = pos.x.min(screen.right() - w - 8.0).max(screen.left() + 8.0);
        pos.y = if pos.y + h > screen.bottom() - 8.0 { (screen.bottom() - h - 8.0).max(screen.top() + TOP_PAD) } else { pos.y };
        let pop = if self.lib(|l| l.settings.reduce_motion) { 1.0 } else { spring_out(age) };
        pos.y += (1.0 - pop) * -9.0;
        if age < 0.5 {
            ui.ctx().request_repaint();
        }
        let mut mp = ui.new_child(egui::UiBuilder::new().max_rect(screen));
        mp.set_opacity((age * 9.0).clamp(0.0, 1.0));
        let open_sub = ui.ctx().data_mut(|d| d.get_temp::<Option<usize>>(Id::new("appmenu-sub"))).flatten();
        let (r, hs, act) = self.paint_menu(&mut mp, pos, &items, w, "am", open_sub);
        let mut inside = mp.rect_contains_pointer(r);
        let mut chosen = act;
        let mut next_sub = None;
        if let Some(si) = hs.or(open_sub) {
            if let Some(row) = items.get(si) {
                if !row.sub.is_empty() {
                    let row_y = pos.y + Self::menu_row_y(&items, si);
                    let sub = Self::tidy_menu(row.sub.clone());
                    let sw = self.menu_width(&p, &sub, 210.0);
                    let sh = Self::menu_height(&sub);
                    let mut sp = pos2(r.right() + 4.0, row_y);
                    if sp.x + sw > screen.right() - 8.0 {
                        sp.x = r.left() - sw - 4.0;
                    }
                    sp.y = sp.y.min(screen.bottom() - sh - 8.0).max(screen.top() + 4.0);
                    let (sr, _, sact) = self.paint_menu(&mut mp, sp, &sub, sw, "ams", None);
                    if sact.is_some() {
                        chosen = sact;
                    }
                    let over_row = mp.rect_contains_pointer(Rect::from_min_size(pos2(r.left(), row_y), vec2(w, 34.0)));
                    let over_sub = mp.rect_contains_pointer(sr);
                    inside = inside || over_sub;
                    next_sub = if hs.is_some() || over_sub || (over_row && open_sub.is_some()) { Some(si) } else { None };
                }
            }
        }
        ui.ctx().data_mut(|d| d.insert_temp(Id::new("appmenu-sub"), next_sub));
        let pressed = ui.ctx().input(|i| i.pointer.primary_pressed() || i.pointer.secondary_pressed());
        let esc = ui.ctx().input(|i| i.key_pressed(egui::Key::Escape));
        if let Some(a) = chosen {
            self.shell.appmenu = None;
            self.acts.push(a);
        } else if esc || (pressed && !inside && age > 0.05) {
            self.shell.appmenu = None;
        }
    }

    // ───────────── settings sheet ─────────────
    pub fn draw_app_settings(&mut self, ui: &mut Ui, win: Rect, app: AppId, pages: &[(&'static str, &'static str)], rows_for: &dyn Fn(&SmartApp, usize) -> Vec<SRow>) -> Option<(usize, usize, SVal)> {
        let th = self.th.clone();
        let ctx = ui.ctx().clone();
        let page = self.shell.settings.map(|s| s.1).unwrap_or(0).min(pages.len() - 1);
        let pop = spring_to(&ctx, ("setpop", app), 1.0, 260.0, 24.0).clamp(0.0, 1.2);
        let mut sui = ui.new_child(egui::UiBuilder::new().max_rect(win));
        sui.set_opacity((pop * 3.0).clamp(0.0, 1.0));
        let p = sui.painter().clone();
        let scrim = sui.allocate_rect(win, Sense::click());
        let sp = p.with_clip_rect(win);
        grad_fill(&sp, win, R_WIN, th.scrim, th.scrim);
        let w = (win.width() - 40.0).clamp(380.0, 660.0);
        let h = (win.height() - 56.0).clamp(300.0, 480.0);
        let card = Rect::from_center_size(win.center() + vec2(0.0, (1.0 - pop.min(1.0)) * 16.0), vec2(w, h));
        menu_surface(&p, &th, card, 22.0);
        sui.allocate_rect(card, Sense::click());
        // header
        let tile = Rect::from_min_size(card.min + vec2(20.0, 18.0), vec2(34.0, 34.0));
        let (a, b) = th.accent_grad();
        shadow(&p, tile, 11.0, 3.0, 8.0, -2.0, alpha(th.accent, 0.4));
        grad_fill(&p, tile, 11.0, a, b);
        rim(&p, tile, 11.0, 1.0, Color32::from_white_alpha(130), true, 16.0);
        self.ico(&p, app.icon(), Ic::Duo, tile.center(), 18.0, Color32::WHITE);
        p.text(pos2(tile.right() + 12.0, tile.center().y - 8.0), Align2::LEFT_CENTER, format!("{} Settings", app.name()), f_bold(T_TITLE), th.ink1);
        p.text(pos2(tile.right() + 12.0, tile.center().y + 9.0), Align2::LEFT_CENTER, "Changes apply instantly and are remembered", f_medium(T_META), th.ink3);
        let x = Rect::from_center_size(pos2(card.right() - 28.0, tile.center().y), vec2(28.0, 28.0));
        let mut close = self.icon_button(&sui, x, "close-circle", "aset-x", Btn::Quiet, true).clicked();
        // page rail
        let rail = Rect::from_min_max(pos2(card.left() + 12.0, card.top() + 68.0), pos2(card.left() + 168.0, card.bottom() - 62.0));
        let mut new_page = page;
        for (i, (icon, name)) in pages.iter().enumerate() {
            let r = Rect::from_min_size(rail.min + vec2(0.0, i as f32 * 38.0), vec2(rail.width(), 34.0));
            let resp = self.hit(&sui, r, ("aset-p", app, i), Sense::click());
            let on = i == page;
            if on {
                grad_fill(&p, r, 10.0, th.accent_tint, th.accent_tint2);
                rim(&p, r, 10.0, 1.0, alpha(Color32::WHITE, if th.dark { 0.10 } else { 0.7 }), true, 12.0);
            } else if resp.hovered() {
                p.rect_filled(r, cr(10.0), th.row_hover);
            }
            self.ico(&p, icon, if on { Ic::Duo } else { Ic::Outline }, pos2(r.left() + 19.0, r.center().y), 16.0, if on { th.accent_ink } else { th.ink3 });
            p.text(pos2(r.left() + 38.0, r.center().y), Align2::LEFT_CENTER, *name, if on { f_semi(T_BODY) } else { f_medium(T_BODY) }, if on { th.ink1 } else { th.ink2 });
            if resp.clicked() {
                new_page = i;
            }
        }
        // rows
        let body = Rect::from_min_max(pos2(card.left() + 180.0, card.top() + 62.0), pos2(card.right() - 16.0, card.bottom() - 62.0));
        let rows = rows_for(self, page);
        let mut change: Option<(usize, SVal)> = None;
        let mut y = body.top();
        for (ri, row) in rows.iter().enumerate() {
            let r = Rect::from_min_size(pos2(body.left(), y), vec2(body.width(), 52.0));
            let resp = self.hit(&sui, r, ("aset-r", app, page, ri), Sense::hover());
            if resp.hovered() {
                p.rect_filled(r, cr(12.0), th.row_hover);
            }
            let (icon, label, sub) = match row {
                SRow::Toggle { icon, label, sub, .. } | SRow::Choice { icon, label, sub, .. } | SRow::Slider { icon, label, sub, .. } | SRow::Button { icon, label, sub, .. } => (*icon, *label, *sub),
            };
            let ic = Rect::from_center_size(pos2(r.left() + 24.0, r.center().y), vec2(32.0, 32.0));
            grad_fill(&p, ic, 10.0, th.accent_tint, th.accent_tint2);
            self.ico(&p, icon, Ic::Duo, ic.center(), 17.0, th.accent_ink);
            let ctl_w = match row {
                SRow::Toggle { .. } => 44.0,
                SRow::Choice { opts, .. } => opts.iter().map(|o| self.tw(&p, o, &f_semi(T_META)) + 22.0).sum::<f32>() + 6.0,
                SRow::Slider { .. } => 176.0,
                SRow::Button { cta, .. } => self.tw(&p, cta, &f_semi(T_BODY)) + 36.0,
            };
            let text_w = r.width() - 52.0 - ctl_w - 18.0;
            p.text(pos2(r.left() + 48.0, r.center().y - 8.0), Align2::LEFT_CENTER, self.fit(&p, label, &f_semi(T_BODY), text_w), f_semi(T_BODY), th.ink1);
            p.text(pos2(r.left() + 48.0, r.center().y + 9.0), Align2::LEFT_CENTER, self.fit(&p, sub, &f_medium(T_META), text_w), f_medium(T_META), th.ink3);
            let right = r.right() - 10.0;
            match row {
                SRow::Toggle { on, .. } => {
                    if self.switch(&sui, Rect::from_center_size(pos2(right - 22.0, r.center().y), vec2(44.0, 26.0)), *on, (app, page, ri)) {
                        change = Some((ri, SVal::Bool(!*on)));
                    }
                }
                SRow::Choice { opts, sel, .. } => {
                    let cr_ = Rect::from_min_size(pos2(right - ctl_w, r.center().y - 15.0), vec2(ctl_w, 30.0));
                    let o: Vec<(&'static str, &str)> = opts.iter().map(|s| ("", *s)).collect();
                    // proportional cells keep long labels readable
                    let id = format!("aset-{:?}-{page}-{ri}", app);
                    let n = o.len();
                    let _ = n;
                    if let Some(i) = self.segment(&sui, cr_, &o, false, *sel, &id) {
                        change = Some((ri, SVal::Idx(i)));
                    }
                }
                SRow::Slider { v, lo, hi, unit, .. } => {
                    let track = Rect::from_min_size(pos2(right - ctl_w + 4.0, r.center().y - 4.0), vec2(ctl_w - 64.0, 8.0));
                    let t = ((*v - lo) / (hi - lo)).clamp(0.0, 1.0);
                    let (nt, resp) = self.slider(&sui, track, t, (app, page, ri));
                    if resp.dragged() || resp.clicked() {
                        change = Some((ri, SVal::F(lo + nt * (hi - lo))));
                    }
                    p.text(pos2(right - 2.0, r.center().y), Align2::RIGHT_CENTER, format!("{:.0}{}", v, unit), f_semi(T_META), th.ink2);
                }
                SRow::Button { cta, .. } => {
                    let b = Rect::from_min_size(pos2(right - ctl_w, r.center().y - 15.0), vec2(ctl_w, 30.0));
                    if self.label_button(&sui, b, None, cta, &format!("aset-b-{page}-{ri}"), Btn::Raised).clicked() {
                        change = Some((ri, SVal::Bool(true)));
                    }
                }
            }
            y += 54.0;
        }
        // footer
        let done = Rect::from_min_size(pos2(card.right() - 20.0 - 92.0, card.bottom() - 18.0 - 34.0), vec2(92.0, 34.0));
        p.text(pos2(card.left() + 22.0, done.center().y), Align2::LEFT_CENTER, format!("{} · stored on this device", app.name()), f_medium(T_META), th.ink4);
        if self.label_button(&sui, done, None, "Done", "aset-done", Btn::Accent).clicked() {
            close = true;
        }
        let esc = ctx.input(|i| i.key_pressed(egui::Key::Escape));
        if close || esc || (scrim.clicked() && ctx.pointer_interact_pos().is_some_and(|pp| !card.contains(pp))) {
            self.shell.settings = None;
            ctx.data_mut(|d| d.remove::<(f32, f32)>(Id::new(("spring", ("setpop", app)))));
        } else if new_page != page {
            self.shell.settings = Some((app, new_page));
        }
        change.map(|(r, v)| (page, r, v))
    }
}

pub const TOP_PAD: f32 = 4.0;

pub type PalItem = (&'static str, String, String, Act, &'static str);

impl SmartApp {
    /// Eight invisible resize handles around a window. Returns the drag direction and delta of the active one.
    pub fn resize_handles(&self, ui: &Ui, key: impl std::hash::Hash + std::fmt::Debug + Copy, r: Rect) -> Option<((f32, f32), egui::Vec2)> {
        let (ei, eo, k) = (3.0, 7.0, 18.0);
        use egui::CursorIcon as C;
        let zones: [(Rect, C, (f32, f32)); 8] = [
            (Rect::from_min_max(pos2(r.right() - ei, r.top() + k), pos2(r.right() + eo, r.bottom() - k)), C::ResizeHorizontal, (1.0, 0.0)),
            (Rect::from_min_max(pos2(r.left() - eo, r.top() + k), pos2(r.left() + ei, r.bottom() - k)), C::ResizeHorizontal, (-1.0, 0.0)),
            (Rect::from_min_max(pos2(r.left() + k, r.bottom() - ei), pos2(r.right() - k, r.bottom() + eo)), C::ResizeVertical, (0.0, 1.0)),
            (Rect::from_min_max(pos2(r.left() + k, r.top() - eo), pos2(r.right() - k, r.top() + 2.0)), C::ResizeVertical, (0.0, -1.0)),
            (Rect::from_min_max(pos2(r.right() - k, r.bottom() - k), pos2(r.right() + eo, r.bottom() + eo)), C::ResizeNwSe, (1.0, 1.0)),
            (Rect::from_min_max(pos2(r.left() - eo, r.bottom() - k), pos2(r.left() + k, r.bottom() + eo)), C::ResizeNeSw, (-1.0, 1.0)),
            (Rect::from_min_max(pos2(r.right() - k, r.top() - eo), pos2(r.right() + eo, r.top() + k)), C::ResizeNeSw, (1.0, -1.0)),
            (Rect::from_min_max(pos2(r.left() - eo, r.top() - eo), pos2(r.left() + k, r.top() + k)), C::ResizeNwSe, (-1.0, -1.0)),
        ];
        let mut out = None;
        for (zi, (zr, cur, dir)) in zones.iter().enumerate() {
            let resp = ui.interact(*zr, Id::new(("resize", key, zi)), Sense::drag());
            if resp.hovered() || resp.dragged() {
                ui.ctx().set_cursor_icon(*cur);
            }
            if resp.dragged() {
                out = Some((*dir, resp.drag_delta()));
            }
        }
        out
    }

    /// Command palette content for Terminal and Photos (same shell, same rail, same scoring as Files).
    pub(crate) fn app_palette_groups(&self, app: AppId) -> Vec<(usize, String, Vec<PalItem>)> {
        let q = self.palette.query.trim().to_lowercase();
        let cat = self.palette.cat;
        let st = self.st();
        let a = |s: String| Act::App(s);
        let theme: PalItem = ("stars-minimalistic", if st.dark { "Switch to light" } else { "Switch to dark" }.into(), "Appearance".into(), Act::ToggleTheme, "D");
        let mut commands: Vec<PalItem> = vec![];
        let mut places: Vec<PalItem> = vec![];
        let mut items: Vec<PalItem> = vec![];
        match app {
            AppId::Terminal => {
                commands.extend([
                    ("add", "New session".into(), "Another shell in this folder".into(), a("term.new".into()), "Ctrl T"),
                    ("trash-bin-trash", "Clear screen".into(), "Remove every block".into(), a("term.clear".into()), "Ctrl L"),
                    ("slider-vertical", "Run top".into(), "Live process monitor".into(), a("term.run:top".into()), ""),
                    ("palette", "Run colors".into(), "Colour test through the vt100 engine".into(), a("term.run:colors".into()), ""),
                    ("laptop", "Run neofetch".into(), "About this system".into(), a("term.run:neofetch".into()), ""),
                    ("text", "Bigger text".into(), "Increase the font".into(), a("term.bigger".into()), "Ctrl ="),
                    ("text", "Smaller text".into(), "Decrease the font".into(), a("term.smaller".into()), "Ctrl -"),
                    ("magic-stick-3", "Toggle ghost suggestions".into(), "History completion".into(), a("term.ghost".into()), ""),
                    ("clock-circle", "Toggle timings".into(), "Show how long commands took".into(), a("term.timing".into()), ""),
                    ("widget", "Toggle command cards".into(), "Group commands with their output".into(), a("term.cards".into()), ""),
                    ("copy", "Copy last output".into(), "Put it on the clipboard".into(), a("term.copy_last".into()), ""),
                    ("copy", "Duplicate session".into(), "Same folder, new shell".into(), a("term.dup".into()), ""),
                    ("folder", "Open Files".into(), "File manager".into(), Act::OpenApp(AppId::Files), ""),
                    ("gallery", "Open Photos".into(), "Picture library".into(), Act::OpenApp(AppId::Photos), ""),
                    theme.clone(),
                ]);
                for (c, d) in crate::terminal::COMMANDS {
                    commands.push(("terminal", format!("Insert {c}"), (*d).into(), a(format!("term.fill:{c} ")), ""));
                }
                for (i, pl) in self.places.iter().enumerate() {
                    places.push((pl.icon, format!("Go to {}", pl.title), self.pretty_path(&pl.loc), a(format!("term.go:{i}")), ""));
                }
                let mut seen: Vec<&String> = vec![];
                for h in self.shell.term.history.iter().rev() {
                    if !seen.contains(&h) {
                        seen.push(h);
                        items.push(("history", h.clone(), "Run again".into(), a(format!("term.run:{h}")), "Enter"));
                    }
                    if seen.len() >= 12 {
                        break;
                    }
                }
            }
            _ => {
                let ph = &self.shell.photos;
                commands.extend([
                    ("play", "Start slideshow".into(), "Crossfading full-screen show".into(), a("photos.slideshow".into()), "S"),
                    ("heart", "Show favorites".into(), "Photos you hearted".into(), a("photos.album:fav".into()), ""),
                    ("gallery", "Show all photos".into(), "The whole library".into(), a("photos.album:all".into()), ""),
                    ("tuning-2", "Edit current photo".into(), "Filters and adjustments".into(), a("photos.edit".into()), "E"),
                    ("info-circle", "Photo info".into(), "Details and histogram".into(), a("photos.info".into()), "I"),
                    ("gallery-wide", "Set as wallpaper".into(), "Use the open photo".into(), a("photos.wallpaper".into()), ""),
                    ("folder-open", "Show in Files".into(), "Reveal the folder".into(), a("photos.reveal".into()), ""),
                    ("magnifer", "Search photos".into(), "Filter by file name".into(), a("photos.search".into()), "Ctrl F"),
                    ("sort-vertical", "Sort by newest".into(), "Newest first".into(), a("photos.sort:0".into()), ""),
                    ("sort-vertical", "Sort by name".into(), "A to Z".into(), a("photos.sort:2".into()), ""),
                    ("sort-vertical", "Sort by size".into(), "Largest first".into(), a("photos.sort:3".into()), ""),
                    ("text", "Toggle captions".into(), "Names under thumbnails".into(), a("photos.captions".into()), ""),
                    ("crop-minimalistic", "Toggle fill / fit thumbnails".into(), "Crop or letterbox".into(), a("photos.fill".into()), ""),
                    ("gallery", "Smaller thumbnails".into(), "More per row".into(), a("photos.tile:140".into()), ""),
                    ("gallery", "Larger thumbnails".into(), "Fewer, bigger tiles".into(), a("photos.tile:300".into()), ""),
                    ("refresh", "Refresh library".into(), "Rescan folders".into(), a("photos.refresh".into()), ""),
                    ("folder", "Open Files".into(), "File manager".into(), Act::OpenApp(AppId::Files), ""),
                    ("terminal", "Open Terminal".into(), "Shell on the same files".into(), Act::OpenApp(AppId::Terminal), ""),
                    theme.clone(),
                ]);
                places.push(("gallery", "All photos".into(), format!("{} in library", ph.all.len()), a("photos.album:all".into()), ""));
                places.push(("heart", "Favorites".into(), format!("{} hearted", ph.favs.len()), a("photos.album:fav".into()), ""));
                for (i, (l, n)) in ph.folders.iter().enumerate() {
                    places.push(("folder", format!("Album {}", self.loc_title(l)), format!("{n} photos"), a(format!("photos.album:{i}")), ""));
                }
                for (i, e) in ph.all.iter().enumerate() {
                    if q.is_empty() && i >= 8 {
                        break;
                    }
                    items.push(("gallery", e.name.clone(), e.loc.parent().map(|p| self.pretty_path(&p)).unwrap_or_default(), a(format!("photos.view_idx:{i}")), "Open"));
                }
            }
        }
        let settings: Vec<PalItem> = vec![
            ("settings-minimalistic", format!("{} settings", app.name()), "Preferences for this app".into(), a("settings".into()), "Ctrl ,"),
            theme,
            ("widget", "Wallpaper".into(), "Desktop surface".into(), Act::Seq(vec![Act::OpenApp(AppId::Files), Act::Modal(Modal::Wallpaper)]), ""),
        ];
        let score = |s: &str| -> Option<i32> {
            let s = s.to_lowercase();
            if q.is_empty() {
                return Some(0);
            }
            if let Some(i) = s.find(&q) {
                return Some(1000 - i as i32);
            }
            let mut it = s.chars();
            let mut n = 0;
            for qc in q.chars() {
                n += 1;
                it.find(|c| *c == qc)?;
            }
            Some(100 - n)
        };
        let filt = |v: Vec<PalItem>, limit: usize| -> Vec<PalItem> {
            let mut s: Vec<_> = v.into_iter().filter_map(|x| score(&format!("{} {}", x.1, x.2)).map(|sc| (sc, x))).collect();
            s.sort_by_key(|x| -x.0);
            s.into_iter().take(limit).map(|x| x.1).collect()
        };
        let want = |i: usize| cat == 0 || cat == i;
        let mut out = vec![];
        let (cn, pn, in_) = if app == AppId::Terminal { ("Commands", "Go to", "History") } else { ("Commands", "Albums", "Photos") };
        if want(2) {
            let v = filt(commands, if q.is_empty() { 7 } else { 8 });
            if !v.is_empty() {
                out.push((2, if q.is_empty() { "Suggested".to_string() } else { cn.to_string() }, v));
            }
        }
        if want(3) {
            let v = filt(places, 6);
            if !v.is_empty() {
                out.push((3, pn.to_string(), v));
            }
        }
        if want(4) {
            let v = filt(items, 6);
            if !v.is_empty() {
                out.push((4, in_.to_string(), v));
            }
        }
        if want(5) {
            let v = filt(settings, 6);
            if !v.is_empty() {
                out.push((5, "Preferences".to_string(), v));
            }
        }
        out
    }
}
