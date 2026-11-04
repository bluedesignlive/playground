//! egui frontend for Smart Files. All behaviour lives in `smart-core`; this crate only draws and routes input.
pub mod app;
pub mod chrome;
pub mod content;
pub mod icons;
pub mod icons_gen;
pub mod modals;
pub mod overlays;
pub mod paint;
pub mod theme;

use app::*;
use eframe::egui;
use smart_core::*;

impl eframe::App for SmartApp {
    fn clear_color(&self, _v: &egui::Visuals) -> [f32; 4] {
        if IS_WEB { [0.56, 0.62, 0.72, 1.0] } else { [0.0, 0.0, 0.0, 0.0] }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = &ui.ctx().clone();
        self.time = ctx.input(|i| i.time);
        self.dt = ctx.input(|i| i.stable_dt).min(0.1);
        self.pump(ctx);
        // debounce live search
        if self.search_active() && self.search.query.trim() != self.search.last_sent && self.time - self.search.dirty_at > 0.10 {
            self.run_search();
        }
        if !self.search_active() && !self.search.last_sent.is_empty() {
            self.clear_search();
        }
        // sidebar slide
        let target = if self.st().sidebar_collapsed || (IS_WEB && ctx.content_rect().width() < 760.0) { 0.0 } else { 1.0 };
        if (self.sidebar_open - target).abs() > 0.002 {
            self.sidebar_open += (target - self.sidebar_open) * (1.0 - (-self.dt * 16.0).exp());
            ctx.request_repaint();
        } else {
            self.sidebar_open = target;
        }
        self.handle_keys(ctx);
        let vis = if self.th.dark { egui::Visuals::dark() } else { egui::Visuals::light() };
        if ctx.global_style().visuals.dark_mode != vis.dark_mode {
            ctx.set_visuals(vis);
            ctx.global_style_mut(|s| {
                s.visuals.selection.bg_fill = theme::alpha(self.th.accent, 0.3);
                s.visuals.text_cursor.stroke = egui::Stroke::new(1.5, self.th.accent);
            });
        }
        overlays::WIN_RECT.with(|w| w.set(self.window_rect(ui.max_rect())));
        self.draw_window(ui);
        self.native_resize_grips(ctx);
        if !(self.win_closed || self.win_min) {
            self.draw_menu(ctx);
            self.draw_modal(ctx);
            self.draw_palette(ctx);
            self.draw_ai(ctx);
            self.draw_search_sheet(ctx);
            self.draw_jobs_card(ctx);
            self.draw_toasts(ctx);
            self.draw_drag_ghost(ctx);
        }
        // resolve a drag exactly once, against whatever was under the pointer
        if ctx.input(|i| i.pointer.any_released()) {
            if let Some(drag) = self.drag.take() {
                let modifiers = ctx.input(|i| i.modifiers);
                if let Some(pos) = ctx.pointer_latest_pos() {
                    if let Some(target) = chrome::drop_target_at(pos) {
                        match target {
                            chrome::DropTarget::Tag(id) => {
                                let n = drag.items.len();
                                for l in &drag.items {
                                    if !self.lib(|lib| lib.has_tag(l, &id)) {
                                        self.be.library.write().unwrap().toggle_tag(l, &id);
                                    }
                                }
                                let name = self.lib(|l| l.tags.iter().find(|t| t.id == id).map(|t| t.name.clone())).unwrap_or(id);
                                self.toast(format!("Tagged {} item{} as {}", n, if n == 1 { "" } else { "s" }, name), true);
                            }
                            chrome::DropTarget::Folder(dest) | chrome::DropTarget::Place(dest) => {
                                let srcs: Vec<_> = drag.items.into_iter().filter(|s| s != &dest && !dest.starts_with(s) && s.parent().as_ref() != Some(&dest)).collect();
                                if srcs.is_empty() {
                                    self.toast("That is already the destination", false);
                                } else {
                                    let cross = srcs.iter().any(|s| s.fs != dest.fs);
                                    let copy = modifiers.alt || modifiers.ctrl || modifiers.command || cross;
                                    let n = srcs.len();
                                    self.submit_job(if copy { smart_core::JobKind::Copy { srcs, dest: dest.clone() } } else { smart_core::JobKind::Move { srcs, dest: dest.clone() } });
                                    self.toast(format!("{} {} item{} to {}", if copy { "Copying" } else { "Moving" }, n, if n == 1 { "" } else { "s" }, dest.name()), true);
                                }
                            }
                            chrome::DropTarget::Trash => {
                                self.submit_job(smart_core::JobKind::Trash { srcs: drag.items });
                                self.toast("Moved to Trash", true);
                            }
                        }
                    }
                }
            }
        }
        // drain queued actions (widgets push, we apply after the frame)
        for _ in 0..4 {
            let mut v = std::mem::take(&mut self.acts);
            v.extend(chrome::ACTS.with(|q| std::mem::take(&mut *q.borrow_mut())));
            if v.is_empty() {
                break;
            }
            for a in v {
                match &a {
                    Act::Toast(t) if t == "\u{1}" => self.drag = None,
                    Act::Toast(t) if t.is_empty() || t == "\u{0}" => {}
                    _ => self.act(a),
                }
            }
        }
        if let Some(t) = self.pending_copy.take() {
            ctx.copy_text(t);
        }
        // big thumbnails for the preview
        if self.be.audio.is_loaded() {
            ctx.request_repaint_after(std::time::Duration::from_millis(60));
        }
        if self.search_active() && !self.search.done {
            ctx.request_repaint_after(std::time::Duration::from_millis(30));
        }
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        storage.set_string("smart-library", self.be.library.read().unwrap().to_json());
        self.be.save_library();
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.be.save_library();
    }
}

impl SmartApp {
    fn native_resize_grips(&mut self, ctx: &egui::Context) {
        if IS_WEB {
            return;
        }
        use egui::viewport::ResizeDirection as D;
        let r = ctx.content_rect();
        let Some(p) = ctx.pointer_latest_pos() else { return };
        let m = 6.0;
        let (l, rr, t, b) = (p.x - r.left() < m, r.right() - p.x < m, p.y - r.top() < m, r.bottom() - p.y < m);
        let dir = match (l, rr, t, b) {
            (true, _, true, _) => Some(D::NorthWest),
            (_, true, true, _) => Some(D::NorthEast),
            (true, _, _, true) => Some(D::SouthWest),
            (_, true, _, true) => Some(D::SouthEast),
            (true, ..) => Some(D::West),
            (_, true, ..) => Some(D::East),
            (_, _, true, _) => Some(D::North),
            (_, _, _, true) => Some(D::South),
            _ => None,
        };
        if let Some(d) = dir {
            ctx.set_cursor_icon(match d {
                D::North | D::South => egui::CursorIcon::ResizeVertical,
                D::East | D::West => egui::CursorIcon::ResizeHorizontal,
                D::NorthWest | D::SouthEast => egui::CursorIcon::ResizeNwSe,
                _ => egui::CursorIcon::ResizeNeSw,
            });
            if ctx.input(|i| i.pointer.primary_pressed()) {
                ctx.send_viewport_cmd(egui::ViewportCommand::BeginResize(d));
            }
        }
    }

    fn handle_keys(&mut self, ctx: &egui::Context) {
        use egui::Key;
        let typing = ctx.egui_wants_keyboard_input();
        let (cmd, shift, alt) = ctx.input(|i| (i.modifiers.command, i.modifiers.shift, i.modifiers.alt));
        let pressed = |k: Key| ctx.input(|i| i.key_pressed(k));
        let events: Vec<egui::Event> = ctx.input(|i| i.events.clone());
        if cmd && pressed(Key::K) {
            self.act(Act::Palette);
            return;
        }
        if cmd && pressed(Key::Comma) {
            self.act(Act::Settings);
            return;
        }
        if pressed(Key::Escape) && self.search.sheet {
            self.act(Act::SearchSheet(false));
            return;
        }
        if pressed(Key::Escape) {
            if self.palette.open {
                // palette handles its own Esc
            } else if self.menu.is_some() {
                self.menu = None;
            } else if !matches!(self.modal, Modal::None) {
                if let Modal::Preview(_) = self.modal {
                    self.be.audio.stop();
                }
                self.act(Act::CloseModal);
            } else if self.rename.is_some() {
                self.rename = None;
            } else if self.ai.open && !typing {
                self.ai.open = false;
            } else if self.search_active() {
                self.clear_search();
            } else if !typing {
                self.sel.clear();
            }
            return;
        }
        if self.palette.open {
            return;
        }
        // preview navigation
        if let Modal::Preview(l) = self.modal.clone() {
            let list: Vec<Location> = self.items.iter().filter(|x| x.kind.previewable()).map(|x| x.loc.clone()).collect();
            if let Some(i) = list.iter().position(|x| *x == l) {
                if pressed(Key::ArrowRight) && i + 1 < list.len() {
                    self.act(Act::Preview(list[i + 1].clone()));
                }
                if pressed(Key::ArrowLeft) && i > 0 {
                    self.act(Act::Preview(list[i - 1].clone()));
                }
            }
            if pressed(Key::Space) && !typing {
                self.be.audio.stop();
                self.act(Act::CloseModal);
            }
            return;
        }
        if !matches!(self.modal, Modal::None) {
            return;
        }
        if alt && pressed(Key::ArrowLeft) {
            self.act(Act::Back);
        }
        if alt && pressed(Key::ArrowRight) {
            self.act(Act::Forward);
        }
        if cmd && pressed(Key::Backslash) {
            self.act(Act::ToggleSidebar);
        }
        if cmd && pressed(Key::F) {
            self.act(Act::FocusSearch);
        }
        if pressed(Key::F5) {
            self.act(Act::Refresh);
        }
        if typing {
            return;
        }
        for e in &events {
            match e {
                egui::Event::Copy => self.act(Act::Copy),
                egui::Event::Cut => self.act(Act::Cut),
                egui::Event::Paste(_) => self.act(Act::Paste),
                _ => {}
            }
        }
        if cmd && pressed(Key::A) {
            self.act(Act::SelectAll);
        }
        if cmd && pressed(Key::I) {
            self.act(Act::Info);
        }
        if cmd && pressed(Key::D) {
            self.act(Act::Duplicate);
        }
        if cmd && pressed(Key::H) {
            self.act(Act::ToggleHidden);
        }
        if cmd && shift && pressed(Key::N) {
            self.act(Act::NewFolder);
        }
        if pressed(Key::F2) {
            if let Some(e) = self.selected().first() {
                self.act(Act::Rename(e.loc.clone()));
            }
        }
        if pressed(Key::Delete) || (cmd && pressed(Key::Backspace)) {
            self.act(if shift { Act::DeleteForever } else { Act::Trash });
        }
        if pressed(Key::Enter) {
            self.act(Act::OpenSel);
        }
        if pressed(Key::Space) {
            let s = self.selected();
            if s.len() == 1 && s[0].kind.previewable() {
                self.act(Act::Preview(s[0].loc.clone()));
            } else if s.len() == 1 && s[0].kind == smart_core::Kind::Audio {
                self.act(Act::TogglePlayPause(s[0].loc.clone()));
            }
        }
        if (pressed(Key::Backspace) && !cmd) || (alt && pressed(Key::ArrowUp)) {
            self.act(Act::Up);
        }
        // arrow navigation
        if !self.search_active() && !self.items.is_empty() {
            let grid = self.st().grid;
            let cols = if grid { self.cols as i32 } else { 1 };
            let mut d = 0i32;
            if pressed(Key::ArrowRight) && grid { d = 1 }
            if pressed(Key::ArrowLeft) && grid { d = -1 }
            if pressed(Key::ArrowDown) { d = cols }
            if pressed(Key::ArrowUp) { d = -cols }
            if !alt && d != 0 {
                let n = self.items.len() as i32;
                let cur = self.focus.map(|f| f as i32).unwrap_or(if d > 0 { -cols.min(1) } else { n });
                let next = (cur + d).clamp(0, n - 1) as usize;
                self.focus = Some(next);
                let loc = self.items[next].loc.clone();
                if shift {
                    let a = self.anchor.unwrap_or(next);
                    let (lo, hi) = (a.min(next), a.max(next));
                    self.sel = self.items[lo..=hi].iter().map(|e| e.loc.clone()).collect();
                } else {
                    self.sel = [loc].into_iter().collect();
                    self.anchor = Some(next);
                }
                self.scroll_to = Some(next);
            }
        }
        // single-key shortcuts (match the status-bar hints) and type-to-search
        if !cmd && !alt {
            if pressed(Key::G) { return self.act(Act::SetGrid(true)); }
            if pressed(Key::L) { return self.act(Act::SetGrid(false)); }
            if pressed(Key::D) { return self.act(Act::ToggleTheme); }
            if pressed(Key::T) { return self.act(Act::Modal(Modal::Tokens)); }
            if pressed(Key::Questionmark) { return self.act(Act::Modal(Modal::Shortcuts)); }
            if pressed(Key::Slash) { return self.act(Act::FocusSearch); }
            for e in &events {
                if let egui::Event::Text(t) = e {
                    if t.chars().all(|c| !c.is_control()) && !t.trim().is_empty() && self.rename.is_none() {
                        self.search.query = t.clone();
                        self.search.dirty_at = self.time;
                        self.search.focus_next = true;
                        self.run_search();
                        return;
                    }
                }
            }
        }
    }
}

pub fn make_app(cc: &eframe::CreationContext<'_>) -> SmartApp {
    let mut a = SmartApp::new(cc, None);
    a.seed_demo_tags();
    a
}

#[cfg(not(target_arch = "wasm32"))]
pub fn run_native() -> eframe::Result {
    let vp = egui::ViewportBuilder::default()
        .with_title("Smart Files")
        .with_app_id("smart-files")
        .with_inner_size([1440.0, 860.0])
        .with_min_inner_size([820.0, 540.0])
        .with_decorations(false)
        .with_transparent(true);
    eframe::run_native("Smart Files", eframe::NativeOptions { viewport: vp, ..Default::default() }, Box::new(|cc| Ok(Box::new(make_app(cc)))))
}

#[cfg(target_arch = "wasm32")]
mod web {
    use super::*;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;

    #[wasm_bindgen]
    pub async fn start(canvas_id: String) -> Result<(), JsValue> {
        console_error_panic_hook::set_once();
        let doc = web_sys::window().and_then(|w| w.document()).ok_or("no document")?;
        let canvas = doc.get_element_by_id(&canvas_id).ok_or("no canvas")?.dyn_into::<web_sys::HtmlCanvasElement>()?;
        eframe::WebRunner::new().start(canvas, eframe::WebOptions::default(), Box::new(|cc| Ok(Box::new(make_app(cc))))).await
    }
}
