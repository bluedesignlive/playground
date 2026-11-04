//! Page header, grid + list views (aligned columns), search results, empty states.
use crate::app::*;
use crate::chrome::*;
use crate::icons::Ic;
use crate::paint::*;
use crate::theme::*;
use egui::{pos2, vec2, Align2, Color32, Id, Painter, Pos2, Rect, Sense, Ui};
use smart_core::*;

pub fn kind_color(k: Kind, th: &Theme) -> Color32 {
    match k {
        Kind::Folder => th.accent,
        Kind::Image | Kind::Vector => hex("#7a5af8"),
        Kind::Audio => th.warn,
        Kind::Video => hex("#e0457b"),
        Kind::Pdf => th.danger,
        Kind::Document | Kind::Text => hex("#3b82c4"),
        Kind::Code => th.good,
        Kind::Archive => hex("#a0709a"),
        Kind::App => hex("#0ea5a4"),
        _ => th.ink3,
    }
}

enum Row {
    Header(usize),
    FieldHeader,
    Cards(usize, usize, bool),
}

#[derive(Clone, Copy)]
struct Cols {
    name: f32,
    kind: f32,
    modified: f32,
    size_right: f32,
    compact: bool,
}

impl SmartApp {
    pub fn seed_demo_tags(&mut self) {
        if !self.be.demo || self.lib(|l| !l.assignments.is_empty()) {
            return;
        }
        let h = self.be.home();
        let map = [
            ("Pictures/4K Frosted Glass.jpg", "design"),
            ("Pictures/Blossom Study.jpg", "archive"),
            ("Pictures/Cycling in Soweto.png", "archive"),
            ("Pictures/Green Fold.png", "design"),
            ("Pictures/Nixio Dashboard.png", "design"),
            ("Pictures/Orbit Poster.png", "amapiano"),
            ("Pictures/Soweto Moto Ad.png", "amapiano"),
            ("Pictures/Window Study.png", "design"),
            ("Pictures/Amapiano Pack Vol. 1", "amapiano"),
            ("Pictures/Brand \u{2014} Naledzi", "design"),
            ("Pictures/Log Drum Sessions", "amapiano"),
            ("Pictures/SMART-DAW Beta", "design"),
            ("Pictures/Amapiano Pack Vol. 1/01 Log Drum 126 BPM.wav", "amapiano"),
            ("Pictures/Log Drum Sessions/Session Notes.txt", "archive"),
        ];
        let mut lib = self.be.library.write().unwrap();
        for (p, t) in map {
            lib.toggle_tag(&h.join(p), t);
        }
    }

    pub fn request_thumb_for(&mut self, e: &Entry) {
        if e.kind == Kind::Image && !self.thumbs.contains_key(&e.loc) && !self.thumb_failed.contains(&e.loc) {
            self.be.request_thumb(&e.loc, 360);
        }
    }

    pub fn draw_content(&mut self, ui: &mut Ui, rect: Rect) {
        let mut cui = ui.new_child(egui::UiBuilder::new().max_rect(rect).layout(egui::Layout::top_down(egui::Align::Min)));
        cui.set_clip_rect(rect.intersect(ui.clip_rect()));
        let inner_w = (rect.width() - 2.0 * PAD_X).max(200.0);
        let grid = self.st().grid;
        let zoom = self.st().thumb_scale.clamp(0.75, 1.4);
        let base = 246.0 * zoom;
        self.cols = if grid { (((inner_w + 16.0) / (base + 16.0)).floor() as usize).clamp(2, 5) } else { 1 };
        let mut sa = egui::ScrollArea::vertical().id_salt(("content-scroll", format!("{:?}", self.route))).auto_shrink([false, false]).scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded);
        if let Some(i) = self.scroll_to.take() {
            if i == 0 {
                sa = sa.vertical_scroll_offset(0.0);
            } else if let Some(y) = self.item_y(i, inner_w) {
                sa = sa.vertical_scroll_offset((y - 150.0).max(0.0));
            }
        }
        let search = self.search_active();
        sa.show_viewport(&mut cui, |ui, vp| {
            let o = pos2(rect.left(), ui.max_rect().top());
            let h = if search { self.search_height() } else { self.content_height(inner_w) };
            let total = Rect::from_min_size(o, vec2(rect.width(), h.max(rect.height() + 1.0)));
            ui.allocate_rect(total, Sense::hover());
            let bg = ui.interact(total, Id::unique("content-bg"), Sense::click_and_drag());
            if search {
                self.draw_search_results(ui, o, rect.width(), vp);
            } else {
                self.draw_page_head(ui, o, rect.width());
                self.draw_items(ui, o, rect.width(), inner_w, vp);
            }
            self.background_interaction(ui, &bg);
        });
    }

    fn background_interaction(&mut self, ui: &Ui, bg: &egui::Response) {
        if bg.clicked() && !ui.input(|i| i.modifiers.command || i.modifiers.shift) {
            self.sel.clear();
            self.rename = None;
        }
        if bg.secondary_clicked() && !self.search_active() {
            self.sel.clear();
            self.menu = Some(Menu { kind: MenuKind::Background, pos: ui.ctx().pointer_latest_pos().unwrap_or_default(), target: None, device: None, sub: None, age: 0.0 });
        }
        let id = Id::unique("marquee");
        if bg.drag_started() {
            if let Some(p) = ui.ctx().pointer_interact_pos() {
                ui.data_mut(|d| d.insert_temp(id, p));
                if !ui.input(|i| i.modifiers.command) {
                    self.sel.clear();
                }
            }
        }
        if bg.dragged() {
            if let (Some(s), Some(c)) = (ui.data(|d| d.get_temp::<Pos2>(id)), ui.ctx().pointer_interact_pos()) {
                let r = Rect::from_two_pos(s, c);
                let p = ui.painter();
                p.rect_filled(r, cr(4.0), alpha(self.th.accent, 0.10));
                p.rect_stroke(r, cr(4.0), egui::Stroke::new(1.0, alpha(self.th.accent, 0.5)), egui::StrokeKind::Inside);
                let hits: Vec<Location> = ITEM_RECTS.with(|v| v.borrow().iter().filter(|(_, rr)| rr.intersects(r)).map(|(l, _)| l.clone()).collect());
                for l in hits {
                    self.sel.insert(l);
                }
            }
        }
    }

    // ───────────── geometry ─────────────
    fn card_dims(&self, inner_w: f32) -> (f32, f32, f32) {
        let gap = 16.0;
        let cols = self.cols as f32;
        let cw = ((inner_w - gap * (cols - 1.0)) / cols).floor();
        let mh = (cw * 10.0 / 16.0).round().max(120.0);
        (cw, mh, mh + 48.0 + gap)
    }

    fn cols_for(&self, o: Pos2, w: f32) -> Cols {
        let right = o.x + w - PAD_X;
        let compact = w < 760.0;
        let kind = if compact { right - 120.0 } else { right - 348.0 };
        let modified = if compact { right - 120.0 } else { right - 186.0 };
        Cols { name: o.x + PAD_X + 42.0, kind, modified, size_right: right, compact }
    }

    fn layout_rows(&self, inner_w: f32) -> (Vec<(Row, f32, f32)>, f32) {
        let grid = self.st().grid;
        let (_, _, pitch) = self.card_dims(inner_w);
        let (_, mh, _) = self.card_dims(inner_w);
        let mut rows = vec![];
        let mut y = if grid { 92.0 } else { 118.0 };
        if !grid {
            rows.push((Row::FieldHeader, 84.0, 28.0));
        }
        for (si, s) in self.sections.iter().enumerate() {
            rows.push((Row::Header(si), y, 24.0));
            y += 26.0;
            if grid {
                let mut i = s.start;
                while i < s.end {
                    let e = (i + self.cols).min(s.end);
                    let h = if s.folders { 70.0 + 16.0 } else { pitch };
                    rows.push((Row::Cards(i, e, s.folders), y, h));
                    y += h;
                    i = e;
                }
                y += 26.0;
            } else {
                let mut i = s.start;
                while i < s.end {
                    rows.push((Row::Cards(i, i + 1, s.folders), y, 42.0));
                    y += 42.0;
                    i += 1;
                }
                y += 18.0;
            }
        }
        let _ = mh;
        (rows, y + 26.0)
    }

    fn content_height(&self, inner_w: f32) -> f32 {
        if self.items.is_empty() {
            return 400.0;
        }
        let (_, h) = self.layout_rows(inner_w);
        h
    }

    fn item_y(&self, i: usize, inner_w: f32) -> Option<f32> {
        let (rows, _) = self.layout_rows(inner_w);
        rows.iter().find_map(|(r, y, _)| if let Row::Cards(a, b, _) = r {
            if i >= *a && i < *b { Some(*y) } else { None }
        } else { None })
    }

    fn search_height(&self) -> f32 {
        170.0 + self.search.hits.len() as f32 * 56.0 + 50.0
    }

    // ───────────── header ─────────────
    fn draw_page_head(&mut self, ui: &mut Ui, o: Pos2, w: f32) {
        let p = ui.painter().clone();
        let th = self.th.clone();
        p.text(pos2(o.x + PAD_X, o.y + 28.0), Align2::LEFT_CENTER, self.title(), f_bold(T_PAGE), th.ink1);
        let n = self.items.len();
        let folders = self.items.iter().filter(|e| e.is_dir).count();
        let mut x = o.x + PAD_X;
        let cy = o.y + 57.0;
        let r = p.text(pos2(x, cy), Align2::LEFT_CENTER, format!("{} item{}", n, if n == 1 { "" } else { "s" }), f_semi(12.0), th.ink2);
        x = r.right() + 8.0;
        let mut parts = vec![format_size(self.total_bytes())];
        if folders > 0 && w > 560.0 {
            parts.push(format!("{} folder{}", folders, if folders == 1 { "" } else { "s" }));
        }
        if self.items.iter().any(|e| e.paced) && w > 700.0 {
            parts.push("includes a demo 5 GB file".into());
        }
        for t in parts {
            p.circle_filled(pos2(x, cy), 1.5, th.ink4);
            let r = p.text(pos2(x + 8.0, cy), Align2::LEFT_CENTER, t, f_medium(12.0), th.ink3);
            x = r.right() + 8.0;
        }
        if self.loading {
            p.text(pos2(x + 4.0, cy), Align2::LEFT_CENTER, "Loading\u{2026}", f_medium(12.0), th.ink4);
        }
        // quick "selection" hint under the header is handled by the app bar + status bar
    }

    fn section_label(&self, p: &Painter, o: Pos2, y: f32, w: f32, title: &str, count: usize) {
        let th = &self.th;
        let r = p.text(pos2(o.x + PAD_X, o.y + y), Align2::LEFT_CENTER, title, f_semi(12.0), th.ink2);
        let r2 = p.text(pos2(r.right() + 7.0, o.y + y + 0.5), Align2::LEFT_CENTER, count.to_string(), f_medium(11.0), th.ink4);
        let x0 = r2.right() + 12.0;
        let x1 = o.x + w - PAD_X;
        if x1 > x0 {
            let mut m = egui::epaint::Mesh::default();
            let c0 = alpha(th.shadow, if th.dark { 0.45 } else { 0.07 });
            for (x, col) in [(x0, c0), (x1, Color32::TRANSPARENT)] {
                m.vertices.push(egui::epaint::Vertex { pos: pos2(x, o.y + y), uv: egui::epaint::WHITE_UV, color: col });
                m.vertices.push(egui::epaint::Vertex { pos: pos2(x, o.y + y + 1.0), uv: egui::epaint::WHITE_UV, color: col });
            }
            m.indices.extend([0, 1, 2, 1, 3, 2]);
            p.add(egui::Shape::mesh(m));
        }
    }

    // ───────────── items ─────────────
    fn draw_items(&mut self, ui: &mut Ui, o: Pos2, w: f32, inner_w: f32, vp: Rect) {
        ITEM_RECTS.with(|v| v.borrow_mut().clear());
        let p = ui.painter().clone();
        if self.items.is_empty() {
            self.draw_empty(ui, o, w);
            return;
        }
        let grid = self.st().grid;
        let (rows, _) = self.layout_rows(inner_w);
        let (cw, mh, _) = self.card_dims(inner_w);
        let cols = self.cols_for(o, w);
        for (row, y, h) in rows {
            if y + h.max(24.0) < vp.top() - 60.0 || y > vp.bottom() + 60.0 {
                continue;
            }
            match row {
                Row::FieldHeader => self.draw_list_head(ui, o, w, cols),
                Row::Header(si) => {
                    let (title, n) = (self.sections[si].title.clone(), self.sections[si].end - self.sections[si].start);
                    self.section_label(&p, o, y, w, &title, n);
                }
                Row::Cards(a, b, folders) => {
                    for i in a..b {
                        if grid {
                            let col = i - a;
                            let x = o.x + PAD_X + col as f32 * (cw + 16.0);
                            if folders {
                                let r = Rect::from_min_size(pos2(x, o.y + y), vec2(cw, 70.0));
                                self.folder_card(ui, i, r);
                            } else {
                                let r = Rect::from_min_size(pos2(x, o.y + y), vec2(cw, mh + 48.0));
                                self.file_card(ui, i, r, mh);
                            }
                        } else {
                            let r = Rect::from_min_size(pos2(o.x + PAD_X - 10.0, o.y + y), vec2(w - 2.0 * PAD_X + 20.0, 42.0));
                            self.list_row(ui, i, r, cols);
                        }
                    }
                }
            }
        }
    }

    fn item_interaction(&mut self, ui: &Ui, i: usize, hit: Rect, e: &Entry) -> egui::Response {
        let resp = ui.interact(hit, Id::unique(("item", &e.loc)), Sense::click_and_drag());
        ITEM_RECTS.with(|v| v.borrow_mut().push((e.loc.clone(), hit)));
        if resp.clicked() {
            let m = ui.input(|i| i.modifiers);
            if m.command {
                if !self.sel.remove(&e.loc) {
                    self.sel.insert(e.loc.clone());
                }
                self.anchor = Some(i);
            } else if m.shift && self.anchor.is_some() {
                let a = self.anchor.unwrap();
                let (lo, hi) = (a.min(i), a.max(i));
                self.sel = self.items[lo..=hi].iter().map(|e| e.loc.clone()).collect();
            } else {
                self.sel.clear();
                self.sel.insert(e.loc.clone());
                self.anchor = Some(i);
            }
            self.focus = Some(i);
        }
        if resp.double_clicked() {
            self.acts.push(Act::Open(e.loc.clone()));
        }
        if resp.secondary_clicked() {
            if !self.sel.contains(&e.loc) {
                self.sel.clear();
                self.sel.insert(e.loc.clone());
            }
            self.menu = Some(Menu { kind: MenuKind::Item, pos: ui.ctx().pointer_latest_pos().unwrap_or(hit.center()), target: Some(e.loc.clone()), device: None, sub: None, age: 0.0 });
        }
        if resp.drag_started() {
            if !self.sel.contains(&e.loc) {
                self.sel.clear();
                self.sel.insert(e.loc.clone());
            }
            let items: Vec<Location> = self.selected().iter().map(|x| x.loc.clone()).collect();
            let label = if items.len() == 1 { e.name.clone() } else { format!("{} items", items.len()) };
            self.drag = Some(Drag { items, label });
        }
        resp
    }

    fn name_or_rename(&mut self, ui: &mut Ui, e: &Entry, pos: Pos2, max_w: f32, font: egui::FontId, col: Color32) -> f32 {
        let p = ui.painter().clone();
        if self.rename.as_ref().is_some_and(|r| r.loc == e.loc) {
            let rr = Rect::from_min_size(pos + vec2(-4.0, -11.0), vec2(max_w + 8.0, 22.0));
            grad_fill(&p, rr, 7.0, self.th.panel_top, self.th.panel_bottom);
            p.rect_stroke(rr.expand(1.0), cr(8.0), egui::Stroke::new(2.0, alpha(self.th.accent, 0.7)), egui::StrokeKind::Outside);
            let mut text = self.rename.as_ref().map(|r| r.text.clone()).unwrap_or_default();
            let first = self.rename.as_ref().is_some_and(|r| r.first);
            let id = Id::unique(("rename", &e.loc));
            let out = ui.put(rr.shrink2(vec2(4.0, 0.0)), egui::TextEdit::singleline(&mut text).id(id).frame(egui::Frame::NONE).font(font).text_color(self.th.ink1).vertical_align(egui::Align::Center).margin(egui::Margin::ZERO));
            if first {
                out.request_focus();
                if let Some(mut st) = egui::TextEdit::load_state(ui.ctx(), id) {
                    let stem = text.rfind('.').filter(|i| *i > 0 && !e.is_dir).map(|i| text[..i].chars().count()).unwrap_or(text.chars().count());
                    st.cursor.set_char_range(Some(egui::text::CCursorRange::two(egui::text::CCursor::new(0), egui::text::CCursor::new(stem))));
                    st.store(ui.ctx(), id);
                }
                if let Some(r) = self.rename.as_mut() {
                    r.first = false;
                }
            }
            let enter = ui.input(|i| i.key_pressed(egui::Key::Enter));
            let esc = ui.input(|i| i.key_pressed(egui::Key::Escape));
            if let Some(r) = self.rename.as_mut() {
                r.text = text.clone();
            }
            if enter || esc || (out.lost_focus() && !first) {
                let loc = e.loc.clone();
                self.rename = None;
                if !esc {
                    match self.be.rename(&loc, &text) {
                        Ok(n) => {
                            self.select_after_list = vec![n];
                            self.load();
                        }
                        Err(m) => self.toast(m, false),
                    }
                }
            }
            return max_w;
        }
        let display = if !self.st().show_extensions && !e.is_dir && !e.ext.is_empty() { e.name.trim_end_matches(&format!(".{}", e.ext)).to_string() } else { e.name.clone() };
        let t = self.fit(&p, &display, &font, max_w);
        let r = p.text(pos, Align2::LEFT_CENTER, t, font, col);
        r.width()
    }

    fn tag_dots(&self, p: &Painter, e: &Entry, x: f32, cy: f32, max_x: f32) {
        let mut x = x;
        for col in self.tag_colors(&e.loc) {
            if x + 8.0 > max_x {
                break;
            }
            let c0 = pos2(x + 4.0, cy);
            shadow(p, Rect::from_center_size(c0, vec2(6.0, 6.0)), 3.0, 1.0, 3.0, 0.0, alpha(col, 0.5));
            grad_fill(p, Rect::from_center_size(c0, vec2(7.0, 7.0)), 3.5, mix(col, Color32::WHITE, 0.3), col);
            x += 11.0;
        }
    }

    fn selection_ring(&self, p: &Painter, r: Rect, rad: f32) {
        p.rect_stroke(r.expand(3.0), cr(rad + 3.0), egui::Stroke::new(2.0, self.th.accent), egui::StrokeKind::Inside);
        shadow(p, r.expand(3.0), rad + 3.0, 0.0, 10.0, 0.0, alpha(self.th.accent, 0.22));
    }

    fn folder_card(&mut self, ui: &mut Ui, i: usize, r: Rect) {
        let e = self.items[i].clone();
        let th = self.th.clone();
        let resp = self.item_interaction(ui, i, r, &e);
        let p = ui.painter().clone();
        let dragging = self.drag.is_some();
        let dropping = dragging && ui.rect_contains_pointer(r) && self.drag.as_ref().is_some_and(|d| !d.items.contains(&e.loc));
        let hov = resp.hovered();
        let a = self.anim(ui, ("fc", &e.loc), hov);
        let r = r.translate(vec2(0.0, -2.0 * a));
        let selected = self.sel.contains(&e.loc);
        DROP_TARGETS.with(|v| v.borrow_mut().push((DropTarget::Folder(e.loc.clone()), r)));
        elevate(&p, &th, r, 16.0, if a > 0.01 || dropping { 3 } else { 2 });
        grad_fill(&p, r, 16.0, mix(th.card_top, th.card_hover_top, a), th.card_bottom);
        rim(&p, r, 16.0, 1.3, th.rim, true, 36.0);
        rim(&p, r, 16.0, 1.0, th.rim_under, false, 6.0);
        if selected || dropping {
            grad_fill(&p, r, 16.0, th.accent_tint, th.accent_tint2);
            self.selection_ring(&p, r, 16.0);
        }
        let tile = Rect::from_min_size(r.min + vec2(12.0, 12.0), vec2(46.0, 46.0));
        grad_fill(&p, tile, 13.0, mix(th.accent_tint, Color32::WHITE, if th.dark { 0.0 } else { 0.55 }), alpha(th.accent, if th.dark { 0.20 } else { 0.16 }));
        rim(&p, tile, 13.0, 1.0, alpha(Color32::WHITE, if th.dark { 0.14 } else { 0.9 }), true, 18.0);
        self.ico(&p, if dropping { "folder-open" } else { "folder" }, Ic::Filled, tile.center(), 24.0, th.accent);
        let tx = tile.right() + 12.0;
        let max_w = r.right() - tx - 30.0;
        self.name_or_rename(ui, &e, pos2(tx, r.top() + 27.0), max_w, f_semi(T_BODY), th.ink1);
        let meta = match self.summaries.get(&e.loc) {
            Some((n, b)) => format!("{} item{} \u{b7} {}", n, if *n == 1 { "" } else { "s" }, format_size(*b)),
            None => "\u{2014}".into(),
        };
        p.text(pos2(tx, r.top() + 45.0), Align2::LEFT_CENTER, self.fit(&p, if dropping { "Drop to move here" } else { &meta }, &f_medium(T_META), max_w), f_medium(T_META), if dropping { th.accent_ink } else { th.ink3 });
        let n = self.tag_colors(&e.loc).len();
        self.tag_dots(&p, &e, r.right() - 14.0 - 11.0 * n as f32 + 3.0, r.center().y, r.right());
    }

    fn file_card(&mut self, ui: &mut Ui, i: usize, r: Rect, mh: f32) {
        let e = self.items[i].clone();
        let th = self.th.clone();
        self.request_thumb_for(&e);
        let resp = self.item_interaction(ui, i, r, &e);
        let p = ui.painter().clone();
        let hov = resp.hovered();
        let a = self.anim(ui, ("fl", &e.loc), hov);
        let lift = -2.0 * a;
        let media = Rect::from_min_size(r.min + vec2(0.0, lift), vec2(r.width(), mh));
        let selected = self.sel.contains(&e.loc);
        elevate(&p, &th, media, 14.0, if a > 0.01 { 3 } else { 2 });
        if let Some(t) = self.thumbs.get(&e.loc) {
            let sz = t.tex.size_vec2();
            textured_rrect(&p, media, 14.0, t.tex.id(), cover_uv(sz.x, sz.y, media), Color32::WHITE);
        } else {
            let kc = kind_color(e.kind, &th);
            grad_fill(&p, media, 14.0, mix(th.card_top, kc, if th.dark { 0.22 } else { 0.10 }), mix(th.card_bottom, kc, if th.dark { 0.12 } else { 0.18 }));
            let c = media.center();
            let tile = Rect::from_center_size(c, vec2(64.0, 64.0));
            grad_fill(&p, tile, 20.0, alpha(kc, 0.20), alpha(kc, 0.10));
            self.ico(&p, e.kind.icon(), Ic::Duo, c, 32.0, kc);
            if !e.ext.is_empty() {
                p.text(pos2(c.x, media.bottom() - 18.0), Align2::CENTER_CENTER, e.ext.to_uppercase(), f_bold(10.5), alpha(kc, 0.9));
            }
        }
        rim(&p, media, 14.0, 1.2, alpha(Color32::WHITE, if th.dark { 0.14 } else { 0.55 }), true, 24.0);
        if selected {
            grad_fill(&p, media, 14.0, alpha(th.accent, 0.10), alpha(th.accent, 0.04));
            self.selection_ring(&p, media, 14.0);
        }
        // 5 GB demo badge
        if e.paced {
            let f = f_semi(9.5);
            let w = self.tw(&p, "5 GB DEMO", &f) + 16.0;
            let b = Rect::from_min_size(media.left_bottom() + vec2(9.0, -25.0), vec2(w, 18.0));
            grad_fill(&p, b, 9.0, alpha(th.accent, 0.9), alpha(th.accent_press, 0.9));
            p.text(b.center(), Align2::CENTER_CENTER, "5 GB DEMO", f, th.on_accent());
        }
        if e.kind == Kind::Audio {
            let playing = self.audio_loc.as_ref() == Some(&e.loc) && self.be.audio.is_playing();
            if hov || playing {
                let b = Rect::from_center_size(media.center(), vec2(44.0, 44.0));
                let br = self.hit(ui, b, ("play", &e.loc), Sense::click());
                raised_accent(&p, &th, b, 22.0, br.hovered(), br.is_pointer_button_down_on());
                self.ico(&p, if playing { "pause" } else { "play" }, Ic::Filled, b.center() + vec2(if playing { 0.0 } else { 1.0 }, 0.0), 18.0, th.on_accent());
                if br.clicked() {
                    self.acts.push(Act::TogglePlayPause(e.loc.clone()));
                }
            }
        }
        if a > 0.05 || selected {
            let chk = Rect::from_center_size(media.left_top() + vec2(19.0, 19.0), vec2(26.0, 26.0));
            let cr_ = self.hit(ui, chk, ("chk", &e.loc), Sense::click());
            if selected {
                let (c0, c1) = th.accent_grad();
                grad_fill(&p, Rect::from_center_size(chk.center(), vec2(22.0, 22.0)), 11.0, c0, c1);
                self.ico(&p, "check-circle", Ic::Filled, chk.center(), 22.0, th.on_accent());
            } else {
                let g = Rect::from_center_size(chk.center(), vec2(22.0, 22.0));
                grad_fill(&p, g, 11.0, alpha(Color32::WHITE, 0.85), alpha(Color32::WHITE, 0.55));
                rim(&p, g, 11.0, 1.0, alpha(Color32::WHITE, 1.0), true, 10.0);
            }
            if cr_.clicked() {
                if !self.sel.remove(&e.loc) {
                    self.sel.insert(e.loc.clone());
                }
            }
            if a > 0.05 {
                let q = Rect::from_center_size(media.right_top() + vec2(-23.0, 23.0), vec2(30.0, 30.0));
                let qr = self.hit(ui, q, ("qa", &e.loc), Sense::click());
                raised(&p, &th, q, 15.0, qr.hovered(), qr.is_pointer_button_down_on());
                self.ico(&p, "menu-dots", Ic::Duo, q.center(), 15.0, th.ink2);
                if qr.clicked() {
                    self.sel.clear();
                    self.sel.insert(e.loc.clone());
                    self.menu = Some(Menu { kind: MenuKind::Item, pos: pos2(q.right(), q.bottom() + 6.0), target: Some(e.loc.clone()), device: None, sub: None, age: 0.0 });
                }
                if e.kind.previewable() && e.kind != Kind::Audio {
                    let q2 = Rect::from_center_size(media.right_top() + vec2(-59.0, 23.0), vec2(30.0, 30.0));
                    let qr2 = self.hit(ui, q2, ("qe", &e.loc), Sense::click());
                    raised(&p, &th, q2, 15.0, qr2.hovered(), qr2.is_pointer_button_down_on());
                    self.ico(&p, "eye", Ic::Duo, q2.center(), 15.0, th.ink3);
                    if qr2.clicked() {
                        self.acts.push(Act::Preview(e.loc.clone()));
                    }
                }
            }
        }
        let ty = media.bottom() + 16.0 - lift;
        let max_w = r.width() - 6.0 - 12.0 * self.tag_colors(&e.loc).len() as f32;
        let wname = self.name_or_rename(ui, &e, pos2(r.left() + 3.0, ty), max_w, f_semi(T_BODY), th.ink1);
        self.tag_dots(&p, &e, r.left() + 3.0 + wname + 7.0, ty + 0.5, r.right());
        let date = e.modified.map(format_date).unwrap_or_default();
        let meta = format!("{}  \u{b7}  {}", format_size(e.size), date);
        p.text(pos2(r.left() + 3.0, ty + 19.0), Align2::LEFT_CENTER, meta, f_medium(T_META), th.ink3);
    }

    fn draw_list_head(&mut self, ui: &mut Ui, o: Pos2, w: f32, cols: Cols) {
        let p = ui.painter().clone();
        let th = self.th.clone();
        let st = self.st();
        let y = o.y + 98.0;
        // tonal header band with the same column geometry as the rows
        let band = Rect::from_min_size(pos2(o.x + PAD_X - 10.0, y - 14.0), vec2(w - 2.0 * PAD_X + 20.0, 28.0));
        grad_fill(&p, band, 9.0, alpha(th.shadow, if th.dark { 0.18 } else { 0.035 }), Color32::TRANSPARENT);
        let heads: [(&str, SortKey, f32, Align2); 4] = [
            ("Name", SortKey::Name, cols.name, Align2::LEFT_CENTER),
            ("Kind", SortKey::Kind, cols.kind, Align2::LEFT_CENTER),
            ("Modified", SortKey::Modified, cols.modified, Align2::LEFT_CENTER),
            ("Size", SortKey::Size, cols.size_right, Align2::RIGHT_CENTER),
        ];
        for (i, (label, key, x, align)) in heads.into_iter().enumerate() {
            if cols.compact && i == 1 {
                continue;
            }
            let on = st.sort == key;
            let r = Rect::from_min_size(pos2(if i == 3 { x - 80.0 } else { x - 4.0 }, y - 12.0), vec2(if i == 3 { 84.0 } else { 120.0 }, 24.0));
            let rr = self.hit(ui, r, ("lh", i), Sense::click());
            let col = if on { th.accent_ink } else if rr.hovered() { th.ink2 } else { th.ink4 };
            let txt = format!("{}{}", label.to_uppercase(), if on { if st.ascending { "  \u{2191}" } else { "  \u{2193}" } } else { "" });
            p.text(pos2(x, y), align, txt, f_semi(T_LABEL), col);
            if rr.clicked() {
                self.acts.push(Act::SetSort(key));
            }
        }
    }

    fn list_row(&mut self, ui: &mut Ui, i: usize, r: Rect, cols: Cols) {
        let e = self.items[i].clone();
        let th = self.th.clone();
        self.request_thumb_for(&e);
        let resp = self.item_interaction(ui, i, r, &e);
        let p = ui.painter().clone();
        let selected = self.sel.contains(&e.loc);
        let dropping = e.is_dir && self.drag.is_some() && ui.rect_contains_pointer(r);
        if e.is_dir {
            DROP_TARGETS.with(|v| v.borrow_mut().push((DropTarget::Folder(e.loc.clone()), r)));
        }
        if selected || dropping {
            grad_fill(&p, r.shrink2(vec2(0.0, 1.0)), 11.0, th.accent_tint, th.accent_tint2);
            if dropping {
                p.rect_stroke(r.shrink(0.5), cr(11.0), egui::Stroke::new(1.5, alpha(th.accent, 0.7)), egui::StrokeKind::Inside);
            }
        } else if resp.hovered() {
            p.rect_filled(r.shrink2(vec2(0.0, 1.0)), cr(11.0), th.row_hover);
        }
        let cy = r.center().y;
        let icon = Rect::from_center_size(pos2(cols.name - 26.0, cy), vec2(30.0, 30.0));
        if let (Some(t), false) = (self.thumbs.get(&e.loc), e.is_dir) {
            let sz = t.tex.size_vec2();
            textured_rrect(&p, icon, 8.0, t.tex.id(), cover_uv(sz.x, sz.y, icon), Color32::WHITE);
        } else {
            let kc = kind_color(e.kind, &th);
            grad_fill(&p, icon, 9.0, alpha(kc, 0.20), alpha(kc, 0.10));
            self.ico(&p, e.kind.icon(), if e.is_dir { Ic::Filled } else { Ic::Duo }, icon.center(), 17.0, kc);
        }
        let name_w = (cols.kind - cols.name - 24.0).max(70.0);
        let wname = self.name_or_rename(ui, &e, pos2(cols.name, cy), name_w, if selected { f_semi(T_BODY) } else { f_medium(T_BODY) }, th.ink1);
        self.tag_dots(&p, &e, cols.name + wname + 8.0, cy, cols.kind - 10.0);
        if e.paced {
            let f = f_semi(9.0);
            let b = Rect::from_min_size(pos2(cols.name + wname + (self.tag_colors(&e.loc).len() as f32) * 11.0 + 8.0, cy - 8.0), vec2(52.0, 16.0));
            grad_fill(&p, b, 8.0, alpha(th.accent, 0.85), alpha(th.accent_press, 0.85));
            p.text(b.center(), Align2::CENTER_CENTER, "5 GB", f, th.on_accent());
        }
        if !cols.compact {
            p.text(pos2(cols.kind, cy), Align2::LEFT_CENTER, self.fit(&p, &e.kind_label(), &f_medium(T_META), 140.0), f_medium(T_META), th.ink3);
        }
        p.text(pos2(cols.modified, cy), Align2::LEFT_CENTER, e.modified.map(format_date).unwrap_or_default(), f_medium(T_META), th.ink3);
        let size = if e.is_dir {
            self.summaries.get(&e.loc).map(|(n, _)| format!("{} item{}", n, if *n == 1 { "" } else { "s" })).unwrap_or("\u{2014}".into())
        } else {
            format_size(e.size)
        };
        p.text(pos2(cols.size_right, cy), Align2::RIGHT_CENTER, size, f_medium(T_META), th.ink3);
        // hover affordances
        if resp.hovered() && !selected {
            let b = Rect::from_center_size(pos2(cols.size_right - 60.0, cy), vec2(28.0, 28.0));
            if self.icon_button(ui, b, "menu-dots", &format!("lm-{}", i), Btn::Quiet, true).clicked() {
                self.sel.clear();
                self.sel.insert(e.loc.clone());
                self.menu = Some(Menu { kind: MenuKind::Item, pos: ui.ctx().pointer_latest_pos().unwrap_or(b.center()), target: Some(e.loc.clone()), device: None, sub: None, age: 0.0 });
            }
        }
    }

    // ───────────── empty ─────────────
    fn draw_empty(&mut self, ui: &mut Ui, o: Pos2, w: f32) {
        let p = ui.painter().clone();
        let th = self.th.clone();
        let cx = o.x + w / 2.0;
        let cy = o.y + 210.0;
        let (icon, title, copy, cta): (&'static str, String, String, Option<(&str, Act)>) = if let Some(e) = &self.error {
            ("lock", "Can\u{2019}t open this folder".into(), e.clone(), Some(("Go Home", Act::Nav(Route::Loc(self.be.home())))))
        } else if self.loading {
            ("refresh", "Loading\u{2026}".into(), String::new(), None)
        } else {
            match &self.route {
                Route::Recents => ("clock-circle", "Nothing recent".into(), "Open a few files and they will appear in this view.".into(), None),
                Route::Tag(_) => ("tag", "Nothing tagged yet".into(), "Right-click a file and choose Tags to collect it here.".into(), None),
                _ => ("folder", "This folder is empty".into(), "Drop files here, or create something new.".into(), Some(("New folder", Act::NewFolder))),
            }
        };
        let orb = Rect::from_center_size(pos2(cx, cy), vec2(76.0, 76.0));
        elevate(&p, &th, orb, 38.0, 2);
        grad_fill(&p, orb, 38.0, th.card_top, th.card_bottom);
        rim(&p, orb, 38.0, 1.3, th.rim, true, 40.0);
        self.ico(&p, icon, Ic::Duo, orb.center(), 30.0, th.ink3);
        p.text(pos2(cx, cy + 58.0), Align2::CENTER_CENTER, title, f_semi(T_TITLE), th.ink1);
        p.text(pos2(cx, cy + 82.0), Align2::CENTER_CENTER, copy, f_medium(T_BODY), th.ink3);
        if let Some((label, act)) = cta {
            let b = Rect::from_center_size(pos2(cx, cy + 124.0), vec2(130.0, 36.0));
            if self.label_button(ui, b, Some("add-folder"), label, "empty-cta", Btn::Accent).clicked() {
                self.acts.push(act);
            }
        }
    }

    // ───────────── search results ─────────────
    fn highlight_name(&self, p: &Painter, pos: Pos2, name: &str, matched: &[u32], max_w: f32) -> f32 {
        let fs = f_semi(T_BODY);
        let fm = f_medium(T_BODY);
        let mut x = pos.x;
        let shown = self.fit(p, name, &fm, max_w);
        for (ci, ch) in shown.chars().enumerate() {
            let hit = matched.contains(&(ci as u32));
            let s = ch.to_string();
            let f = if hit { fs.clone() } else { fm.clone() };
            let r = p.text(pos2(x, pos.y), Align2::LEFT_CENTER, &s, f, if hit { self.th.accent_ink } else { self.th.ink1 });
            if hit {
                p.rect_filled(Rect::from_min_max(pos2(r.left() - 0.5, pos.y + 8.0), pos2(r.right() + 0.5, pos.y + 10.0)), cr(1.0), alpha(self.th.accent, 0.35));
            }
            x += r.width();
        }
        x - pos.x
    }

    fn draw_search_results(&mut self, ui: &mut Ui, o: Pos2, w: f32, vp: Rect) {
        let p = ui.painter().clone();
        let th = self.th.clone();
        ITEM_RECTS.with(|v| v.borrow_mut().clear());
        p.text(pos2(o.x + PAD_X, o.y + 28.0), Align2::LEFT_CENTER, "Search", f_bold(T_PAGE), th.ink1);
        let n = self.search.hits.len();
        let mut x = o.x + PAD_X;
        let cy = o.y + 57.0;
        let status = if self.search.done { format!("{} result{}", n, if n == 1 { "" } else { "s" }) } else { format!("{} found \u{b7} searching\u{2026}", n) };
        let r = p.text(pos2(x, cy), Align2::LEFT_CENTER, status, f_semi(12.0), th.ink2);
        x = r.right() + 8.0;
        if self.search.done {
            p.circle_filled(pos2(x, cy), 1.5, th.ink4);
            let t = format!("{} items {} in {} ms", self.search.scanned, if self.search.cached { "from index" } else { "scanned" }, self.search.ms);
            p.text(pos2(x + 8.0, cy), Align2::LEFT_CENTER, t, f_medium(12.0), th.ink3);
        }
        // scope + filter chips
        let st = self.st();
        let here = self.current_loc().map(|l| self.loc_title(&l)).unwrap_or_else(|| "Home".into());
        let mut cx = o.x + PAD_X;
        let cyy = o.y + 94.0;
        let mut chips: Vec<(String, Option<&'static str>, bool, Act)> = vec![
            (format!("In {}", here), Some("folder"), !self.search.everywhere, Act::SearchSheet(false)),
            ("Everywhere".into(), Some("global"), self.search.everywhere, Act::ToggleEverywhere),
            ("Inside files".into(), Some("text"), st.search_contents, Act::ToggleContents),
        ];
        for (label, f) in [("Images", "kind:image"), ("Documents", "kind:doc"), ("Archives", "kind:archive"), ("Audio", "kind:audio"), ("Folders", "kind:folder"), ("This week", "modified:week"), ("Large", "size:>50mb")] {
            let on = self.search.query.contains(f);
            chips.push((label.to_string(), None, on, Act::RunSearch(if on { self.search.query.replace(f, "").split_whitespace().collect::<Vec<_>>().join(" ") } else { format!("{} {}", self.search.query.trim(), f) })));
        }
        for (idx, (label, icon, on, act)) in chips.into_iter().enumerate() {
            let f = f_semi(11.5);
            let wd = self.tw(&p, &label, &f) + 28.0 + if icon.is_some() { 20.0 } else { 0.0 };
            if cx + wd > o.x + w - PAD_X {
                break;
            }
            let r = Rect::from_min_size(pos2(cx, cyy - 14.0), vec2(wd, 28.0));
            cx += wd + 8.0;
            let resp = self.hit(ui, r, ("chip", idx), Sense::click());
            if on {
                grad_fill(&p, r, 14.0, alpha(th.accent, 0.20), alpha(th.accent, 0.12));
                rim(&p, r, 14.0, 1.0, alpha(Color32::WHITE, if th.dark { 0.12 } else { 0.7 }), true, 12.0);
            } else {
                raised(&p, &th, r, 14.0, resp.hovered(), resp.is_pointer_button_down_on());
            }
            let col = if on { th.accent_ink } else { th.ink2 };
            let mut tx = r.left() + 14.0;
            if let Some(i) = icon {
                self.ico(&p, i, if on { Ic::Filled } else { Ic::Duo }, pos2(r.left() + 20.0, r.center().y), 14.0, col);
                tx += 20.0;
            }
            p.text(pos2(tx, r.center().y), Align2::LEFT_CENTER, &label, f, col);
            if resp.clicked() {
                match &act {
                    Act::SearchSheet(false) => {
                        self.search.everywhere = false;
                        self.run_search();
                    }
                    other => self.acts.push(other.clone()),
                }
            }
        }
        let hits = self.search.hits.clone();
        let y0 = 128.0;
        if hits.is_empty() && self.search.done {
            let cxm = o.x + w / 2.0;
            p.text(pos2(cxm, o.y + 230.0), Align2::CENTER_CENTER, format!("No matches for \u{201c}{}\u{201d}", self.search.query.trim()), f_semi(T_TITLE), th.ink1);
            p.text(pos2(cxm, o.y + 258.0), Align2::CENTER_CENTER, "Try fewer letters, or add filters like kind:image size:>5mb modified:week tag:design", f_medium(T_BODY), th.ink3);
            p.text(pos2(cxm, o.y + 280.0), Align2::CENTER_CENTER, "For password-protected archives, open the file to extract its contents.", f_medium(T_META), th.ink4);
        }
        let right_w = if w > 720.0 { 236.0 } else { 120.0 };
        for (i, h) in hits.iter().enumerate() {
            let y = y0 + i as f32 * 56.0;
            if y + 56.0 < vp.top() - 20.0 || y > vp.bottom() + 20.0 {
                continue;
            }
            let r = Rect::from_min_size(pos2(o.x + PAD_X - 10.0, o.y + y), vec2(w - 2.0 * PAD_X + 20.0, 52.0));
            let e = h.entry.clone();
            self.request_thumb_for(&e);
            let resp = ui.interact(r, Id::unique(("hit-row", &e.loc)), Sense::click());
            ITEM_RECTS.with(|v| v.borrow_mut().push((e.loc.clone(), r)));
            let selected = self.sel.contains(&e.loc);
            if selected {
                grad_fill(&p, r, 12.0, th.accent_tint, th.accent_tint2);
            } else if resp.hovered() {
                p.rect_filled(r, cr(12.0), th.row_hover);
            }
            if resp.clicked() {
                let m = ui.input(|i| i.modifiers);
                if m.command {
                    if !self.sel.remove(&e.loc) {
                        self.sel.insert(e.loc.clone());
                    }
                } else {
                    self.sel.clear();
                    self.sel.insert(e.loc.clone());
                }
                self.focus = Some(i);
            }
            if resp.double_clicked() {
                self.acts.push(Act::Open(e.loc.clone()));
            }
            if resp.secondary_clicked() {
                self.sel.clear();
                self.sel.insert(e.loc.clone());
                self.menu = Some(Menu { kind: MenuKind::Item, pos: ui.ctx().pointer_latest_pos().unwrap_or(r.center()), target: Some(e.loc.clone()), device: None, sub: None, age: 0.0 });
            }
            let icon = Rect::from_center_size(pos2(r.left() + 30.0, r.center().y), vec2(38.0, 38.0));
            if let (Some(t), false) = (self.thumbs.get(&e.loc), e.is_dir) {
                let sz = t.tex.size_vec2();
                elevate(&p, &th, icon, 10.0, 1);
                textured_rrect(&p, icon, 10.0, t.tex.id(), cover_uv(sz.x, sz.y, icon), Color32::WHITE);
            } else {
                let kc = kind_color(e.kind, &th);
                grad_fill(&p, icon, 11.0, alpha(kc, 0.22), alpha(kc, 0.10));
                rim(&p, icon, 11.0, 1.0, alpha(Color32::WHITE, if th.dark { 0.10 } else { 0.7 }), true, 14.0);
                self.ico(&p, e.kind.icon(), if e.is_dir { Ic::Filled } else { Ic::Duo }, icon.center(), 19.0, kc);
            }
            let tx = icon.right() + 14.0;
            let name_w = (r.right() - right_w - tx).max(80.0);
            let wname = self.highlight_name(&p, pos2(tx, r.center().y - 9.0), &e.name, &h.matched, name_w);
            self.tag_dots(&p, &e, tx + wname + 8.0, r.center().y - 9.0, r.right() - right_w);
            let sub = match &h.snippet {
                Some(s) => s.clone(),
                None => e.loc.parent().map(|pl| self.pretty_path(&pl)).unwrap_or_default(),
            };
            let sub = self.fit(&p, &sub, &f_medium(T_META), name_w);
            p.text(pos2(tx, r.center().y + 10.0), Align2::LEFT_CENTER, sub, f_medium(T_META), if h.snippet.is_some() { th.ink2 } else { th.ink3 });
            p.text(pos2(r.right() - 12.0, r.center().y - 9.0), Align2::RIGHT_CENTER, e.modified.map(format_date).unwrap_or_default(), f_medium(T_META), th.ink3);
            let meta = if e.is_dir { e.kind_label() } else { format!("{} \u{b7} {}", e.kind_label(), format_size(e.size)) };
            p.text(pos2(r.right() - 12.0, r.center().y + 10.0), Align2::RIGHT_CENTER, self.fit(&p, &meta, &f_medium(T_META), right_w - 16.0), f_medium(T_META), th.ink4);
            if resp.hovered() {
                let b = Rect::from_center_size(pos2(r.right() - right_w + 48.0, r.center().y), vec2(88.0, 28.0));
                if let Some(parent) = e.loc.parent() {
                    if self.label_button(ui, b, Some("folder-open"), "Reveal", &format!("rev-{}", i), Btn::Raised).clicked() {
                        self.acts.push(Act::Nav(Route::Loc(parent)));
                        self.select_after_list = vec![e.loc.clone()];
                    }
                }
            }
        }
    }

    pub fn pretty_path(&self, l: &Location) -> String {
        let home = self.be.home();
        if l.starts_with(&home) {
            let rest = l.path.strip_prefix(&home.path).unwrap_or("").trim_start_matches('/');
            if rest.is_empty() { "Home".into() } else { format!("Home / {}", rest.replace('/', " / ")) }
        } else {
            l.path.trim_start_matches('/').replace('/', " / ")
        }
    }

    pub fn display_name<'a>(&self, e: &'a Entry) -> &'a str {
        let _ = self;
        &e.name
    }
}

thread_local! {
    pub static ITEM_RECTS: std::cell::RefCell<Vec<(Location, Rect)>> = const { std::cell::RefCell::new(Vec::new()) };
}
