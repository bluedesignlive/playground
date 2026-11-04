//! Context menus (state aware), command palette (premium rail), Ask Smart panel, toasts, drag ghost, jobs.
use crate::app::*;
use crate::chrome::*;
use crate::content::kind_color;
use crate::icons::Ic;
use crate::paint::*;
use crate::theme::*;
use egui::{pos2, vec2, Align2, Color32, Context, Id, Order, Pos2, Rect, Sense, Ui};
use smart_core::*;

thread_local! {
    pub static WIN_RECT: std::cell::Cell<Rect> = const { std::cell::Cell::new(Rect::NOTHING) };
}

pub fn win_rect() -> Rect {
    WIN_RECT.with(|c| c.get())
}

#[derive(Clone)]
pub struct MI {
    pub icon: &'static str,
    pub label: String,
    pub kbd: &'static str,
    pub act: Option<Act>,
    pub danger: bool,
    pub disabled: bool,
    pub checked: bool,
    pub sub: Vec<MI>,
    pub gap: bool,
    pub dot: Option<Color32>,
    pub tog: bool,
    pub quick: bool,
    pub qrow: bool,
}

pub(crate) fn mi(icon: &'static str, label: impl Into<String>, act: Act) -> MI {
    MI { icon, label: label.into(), kbd: "", act: Some(act), danger: false, disabled: false, checked: false, sub: vec![], gap: false, dot: None, tog: false, quick: false, qrow: false }
}
impl MI {
    pub(crate) fn k(mut self, k: &'static str) -> Self {
        self.kbd = k;
        self
    }
    pub(crate) fn q(mut self) -> Self {
        self.quick = true;
        self
    }
    pub(crate) fn gap(mut self) -> Self {
        self.gap = true;
        self
    }
    pub(crate) fn danger(mut self) -> Self {
        self.danger = true;
        self
    }
    pub(crate) fn off(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
    /// Marks the row as a toggle: it always shows a tick slot, bright when on and dimmed when off.
    pub(crate) fn on(mut self, c: bool) -> Self {
        self.checked = c;
        self.tog = true;
        self
    }
    pub(crate) fn sub(mut self, s: Vec<MI>) -> Self {
        self.act = None;
        self.sub = s;
        self
    }
}

const ITEM_H: f32 = 33.0;
const PAD_Y: f32 = 7.0;
const GAP: f32 = 9.0;
const QROW_H: f32 = 40.0;

impl SmartApp {
    pub(crate) fn menu_width(&self, p: &egui::Painter, items: &[MI], min: f32) -> f32 {
        let mut w = min;
        for i in items {
            if i.qrow {
                continue;
            }
            let f = f_medium(T_BODY);
            let mut n = 46.0 + self.tw(p, &i.label, &f) + 18.0;
            if !i.kbd.is_empty() {
                n += self.tw(p, i.kbd, &f_medium(T_META)) + 18.0;
            }
            if !i.sub.is_empty() {
                n += 18.0;
            }
            w = w.max(n);
        }
        w.max(items.iter().filter(|i| i.qrow).count() as f32 * 46.0 + 16.0).min(340.0)
    }

    fn menu_items(&self, m: &Menu) -> Vec<MI> {
        let st = self.st();
        let sel = self.selected();
        let n = sel.len();
        match m.kind {
            MenuKind::Item => {
                let Some(t) = m.target.clone() else { return vec![] };
                let e = self.be.stat(&t).ok();
                let is_dir = e.as_ref().is_some_and(|e| e.is_dir);
                let kinds: Vec<Kind> = sel.iter().map(|e| e.kind).collect();
                let all_images = self.sel_all_images();
                let all_audio = self.sel_all_audio();
                let has_archive = self.sel_has_archive();
                let multi = n > 1;
                let paced = sel.iter().any(|e| e.paced);
                let mut apps: Vec<MI> = if is_dir || multi { vec![] } else { self.be.apps_for(&t).into_iter().take(7).map(|a| MI { icon: a.icon, label: format!("{}{}", a.name, if a.is_default { "   default" } else { "" }), kbd: "", act: Some(Act::OpenWith(t.clone(), a)), danger: false, disabled: false, checked: false, sub: vec![], gap: false, dot: None, tog: false, quick: false, qrow: false }).collect() };
                apps.push(mi("menu-dots", "Other\u{2026}", Act::Modal(Modal::OpenWith(t.clone()))).gap());
                let tags: Vec<MI> = self
                    .lib(|l| l.tags.clone())
                    .into_iter()
                    .map(|tg| {
                        let on = !sel.is_empty() && sel.iter().all(|s| self.lib(|l| l.has_tag(&s.loc, &tg.id)));
                        let mut x = mi("tag", tg.name.clone(), Act::ToggleTag(t.clone(), tg.id.clone())).on(on);
                        x.dot = Some(hex(&tg.color));
                        x
                    })
                    .chain([mi("add", "New tag\u{2026}", Act::Modal(Modal::NewTag)).gap()])
                    .collect();
                let mut v: Vec<MI> = vec![];
                if multi {
                    v.push(mi("arrow-right-up", format!("Open {} items", n), Act::OpenSel).k("Enter"));
                    v.push(mi("info-circle", format!("Get Info on {} items", n), Act::Info).k("Ctrl+I"));
                } else {
                    v.push(mi(if is_dir { "folder-open" } else { "arrow-right-up" }, "Open", Act::OpenSel).k("Enter"));
                    v.push(mi("widget", "Open With", Act::Toast(String::new())).sub(apps).off(is_dir));
                    v.push(mi("eye", "Quick Look", Act::Preview(t.clone())).k("Space").off(!e.as_ref().is_some_and(|e| e.kind.previewable())));
                    v.push(mi("info-circle", "Get Info", Act::Info).k("Ctrl+I"));
                }
                // kind-specific, state-aware commands
                if !multi {
                    match e.as_ref().map(|e| e.kind) {
                        Some(Kind::Image) | Some(Kind::Vector) => {
                            v.push(mi("gallery", "Open in Photos", Act::OpenInPhotos(t.clone())).gap());
                            v.push(mi("gallery-wide", "Set as desktop wallpaper", Act::WallpaperFrom(t.clone())));
                        }
                        Some(Kind::Audio) => {
                            let playing = self.audio_loc.as_ref() == Some(&t) && self.be.audio.is_playing();
                            v.push(mi(if playing { "pause" } else { "play" }, if playing { "Pause playback" } else { "Play" }, Act::TogglePlayPause(t.clone())).gap().k("Space"));
                            v.push(mi("music-notes", "Open in Smart Player", Act::Preview(t.clone())));
                        }
                        Some(Kind::Archive) => {
                            v.push(mi("folder-open", "Extract here", Act::ExtractHere(t.clone())).gap());
                            v.push(mi("copy", "Extract to\u{2026}", Act::ArchiveBrowse(t.clone())));
                            v.push(mi("archive", "Show archive contents", Act::ArchiveBrowse(t.clone())));
                        }
                        Some(Kind::Text) | Some(Kind::Code) | Some(Kind::Document) | Some(Kind::Pdf) => {
                            v.push(mi("magnifer-zoom-in", "Search inside this file", Act::RunSearch(format!("content:{}", t.name()))).gap());
                        }
                        Some(Kind::App) => {
                            v.push(mi("play", "Launch", Act::Open(t.clone())).gap());
                        }
                        _ => {}
                    }
                    if is_dir {
                        v.push(mi("maximize", "Open in new window", Act::OpenNewWindow).gap());
                        let pinned = self.lib(|l| l.is_pinned(&t));
                        v.push(mi("bookmark", if pinned { "Remove from sidebar" } else { "Pin to sidebar" }, Act::TogglePin(t.clone())));
                        v.push(mi("add", "Open in new tab", Act::NewTab(Some(Route::Loc(t.clone())))).k("Ctrl+T"));
                        v.push(mi("terminal", "Open in terminal", Act::OpenInTerminal(t.clone())));
                    }
                }
                if has_archive && multi {
                    v.push(mi("folder-open", "Extract all here", Act::ExtractHere(sel.iter().find(|e| e.kind == Kind::Archive).map(|e| e.loc.clone()).unwrap_or(t.clone()))).gap());
                }
                if paced {
                    v.push(mi("copy", "Copy\u{2026}  (5 GB transfer)", Act::Copy).gap());
                }
                v.push(mi("share", if multi { format!("Send {} to\u{2026}", n) } else { "Send to\u{2026}".into() }, Act::Modal(Modal::Transfer)).gap().off(is_dir || multi));
                v.push(mi("music-notes", "Play in Smart DAW", Act::OpenWith(t.clone(), smart_core::apps::AppInfo { id: "daw".into(), name: "Smart DAW".into(), exec: "daw".into(), icon: "music-notes", is_default: false })).off(!matches!(kinds.first(), Some(Kind::Audio | Kind::Project | Kind::App))));
                v.push(mi("copy", "Copy", Act::Copy).k("Ctrl+C").gap().q());
                v.push(mi("scissors", "Cut", Act::Cut).k("Ctrl+X").q());
                v.push(mi("copy", "Copy to\u{2026}", Act::CopyTo(false)));
                v.push(mi("folder", "Move to\u{2026}", Act::CopyTo(true)));
                v.push(mi("posts-carousel-horizontal", "Duplicate", Act::Duplicate).k("Ctrl+D").q());
                v.push(mi("archive", if multi { format!("Compress {} items to ZIP\u{2026}", n) } else { "Compress to ZIP\u{2026}".into() }, Act::ArchiveCompress).gap());
                v.push(mi("pen", "Rename", Act::Rename(t.clone())).k("F2").off(multi));
                v.push(mi("tag", "Tags", Act::Toast(String::new())).sub(tags));
                v.push(mi("link", "Copy path", Act::CopyPath));
                if !cfg!(target_arch = "wasm32") && !self.be.demo {
                    v.push(mi("folder-open", "Show in file manager", Act::Reveal));
                }
                v.push(mi("trash-bin-minimalistic", if multi { format!("Move {} items to Trash", n) } else { "Move to Trash".into() }, Act::Trash).k("Del").gap().danger().q());
                v
            }
            MenuKind::Background => vec![
                mi("add-folder", "New folder", Act::NewFolder).k("Ctrl+Shift+N"),
                mi("file", "New text file", Act::NewFile),
                mi("archive", "Compress this folder\u{2026}", Act::ArchiveCompress).off(self.sel.is_empty()),
                mi("clipboard", "Paste", Act::Paste).k("Ctrl+V").off(self.clipboard.is_none() || self.current_loc().is_none()),
                mi("sort-from-top-to-bottom", "Sort by", Act::Toast(String::new())).sub(self.sort_items()).gap(),
                mi("widget", "View", Act::Toast(String::new())).sub(vec![mi("widget", "As grid", Act::SetGrid(true)).on(st.grid).k("G"), mi("list-check", "As list", Act::SetGrid(false)).on(!st.grid).k("L")]),
                mi("eye", "Show hidden files", Act::ToggleHidden).on(st.show_hidden).k("Ctrl+H"),
                mi("checklist-minimalistic", "Select all", Act::SelectAll).k("Ctrl+A").gap(),
                mi("refresh", "Refresh", Act::Refresh).k("F5"),
                mi("settings-minimalistic", "Settings\u{2026}", Act::Settings).gap().k("Ctrl+,"),
                mi("terminal", "Command palette", Act::Palette).k("Ctrl+K"),
            ],
            MenuKind::Place => {
                let t = m.target.clone().unwrap_or_else(|| self.be.home());
                let pinned = self.lib(|l| l.is_pinned(&t));
                let mut v = vec![mi("folder-open", "Open", Act::Nav(Route::Loc(t.clone())))];
                v.push(mi("add", "Open in new tab", Act::NewTab(Some(Route::Loc(t.clone())))));
                v.push(mi("terminal", "Open in terminal", Act::OpenInTerminal(t.clone())));
                v.push(mi("maximize", "Open in new window", Act::OpenNewWindow));
                v.push(mi("link", "Copy path", Act::CopyPath));
                v.push(mi("bookmark", if pinned { "Unpin from sidebar" } else { "Pin to sidebar" }, Act::TogglePin(t.clone())).gap());
                v.push(mi("refresh", "Refresh", Act::Refresh));
                v
            }
            MenuKind::Device => {
                let id = m.device.clone().unwrap_or_default();
                let d = self.devices.iter().find(|d| d.id == id).cloned();
                let mut v = vec![];
                if let Some(d) = d {
                    if d.mounted {
                        if let Some(l) = &d.loc {
                            v.push(mi("folder-open", "Open", Act::Nav(Route::Loc(l.clone()))));
                        }
                        if d.kind == DeviceKind::Phone {
                            v.push(mi("smartphone", "Phone continuity\u{2026}", Act::Modal(Modal::Phone)));
                        }
                        if d.can_unmount {
                            v.push(mi("upload-minimalistic", "Unmount", Act::Unmount(id.clone())).gap());
                        }
                        if d.can_eject {
                            v.push(mi("eject", "Eject safely", Act::Eject(id.clone())));
                        }
                    } else {
                        v.push(mi("plug-circle", "Mount", Act::Mount(id.clone())));
                    }
                    v.push(mi("info-circle", "Devices & locations\u{2026}", Act::Modal(Modal::Devices)).gap());
                }
                v
            }
            MenuKind::Crumb => {
                let t = m.target.clone().unwrap_or_else(|| self.be.home());
                let here = self.current_loc().as_ref() == Some(&t);
                let pinned = self.lib(|l| l.is_pinned(&t));
                let mut v = vec![];
                if !here {
                    v.push(mi("folder-open", "Open", Act::Nav(Route::Loc(t.clone()))));
                }
                v.push(mi("add", "Open in new tab", Act::NewTab(Some(Route::Loc(t.clone())))).k("Ctrl+T"));
                v.push(mi("terminal", "Open in terminal here", Act::OpenInTerminal(t.clone())));
                v.push(mi("link", "Copy path", Act::CopyText(t.path.clone())).gap());
                v.push(mi("bookmark", if pinned { "Unpin from sidebar" } else { "Pin to sidebar" }, Act::TogglePin(t.clone())));
                if here {
                    v.push(mi("add-folder", "New folder here", Act::NewFolder).gap().k("Ctrl+Shift+N"));
                    v.push(mi("refresh", "Refresh", Act::Refresh).k("F5"));
                } else {
                    v.push(mi("maximize", "Open in new window", Act::OpenNewWindow).gap());
                }
                v
            }
            MenuKind::Sort => self.sort_items(),
            MenuKind::Workspace => vec![
                mi("check-circle", format!("{} \u{b7} Personal", self.user_name()), Act::Toast("Personal workspace".into())).on(true),
                mi("settings-minimalistic", "Settings\u{2026}", Act::Settings).gap().k("Ctrl+,"),
                mi("cloud", "Add cloud account\u{2026}", Act::Modal(Modal::Connect)),
                mi("ssd-square", "Devices & locations\u{2026}", Act::Modal(Modal::Devices)),
                mi("smartphone", "Phone continuity\u{2026}", Act::Modal(Modal::Phone)),
                mi("stars-minimalistic", "About Smart Files", Act::Modal(Modal::About)).gap(),
            ],
            MenuKind::More => vec![
                mi("command", "Command palette", Act::Palette).k("Ctrl+K"),
                mi("add", "New tab", Act::NewTab(None)).k("Ctrl+T"),
                mi("terminal", "Open terminal here", Act::OpenTerminal),
                mi("gallery", "Open Photos", Act::OpenApp(crate::shell::AppId::Photos)),
                mi("settings-minimalistic", "Settings\u{2026}", Act::Settings).k("Ctrl+,").gap(),
                mi("sidebar-minimalistic", "Toggle sidebar", Act::ToggleSidebar).k("Ctrl+\\"),
                mi("stars-minimalistic", if st.dark { "Switch to light" } else { "Switch to dark" }, Act::ToggleTheme).k("D"),
                mi("checklist-minimalistic", "Selection mode", Act::ToggleSelectMode).on(self.select_mode),
                mi("eye", "Show hidden files", Act::ToggleHidden).on(st.show_hidden).k("Ctrl+H"),
                mi("folder", "Folders first", Act::ToggleFoldersFirst).on(st.folders_first),
                mi("gallery-wide", "Desktop wallpaper\u{2026}", Act::Modal(Modal::Wallpaper)).gap(),
                mi("widget", "Design tokens", Act::Modal(Modal::Tokens)).k("T"),
                mi("magnifer-zoom-in", "8 px grid overlay", Act::ToggleGridOverlay).on(self.grid_overlay),
                mi("ssd-square", "Devices & locations\u{2026}", Act::Modal(Modal::Devices)).gap(),
                mi("smartphone", "Phone continuity\u{2026}", Act::Modal(Modal::Phone)),
                mi("cloud", "Connect to server\u{2026}", Act::Modal(Modal::Connect)),
                mi("maximize", "New window", Act::OpenNewWindow),
                mi("list-check", "Keyboard shortcuts", Act::Modal(Modal::Shortcuts)).gap().k("?"),
                mi("info-circle", "About Smart Files", Act::Modal(Modal::About)),
            ],
        }
    }

    fn sort_items(&self) -> Vec<MI> {
        let st = self.st();
        let mut v: Vec<MI> = SortKey::ALL.iter().map(|k| mi(match k { SortKey::Name => "text", SortKey::Modified => "clock-circle", SortKey::Size => "database", SortKey::Kind => "tag" }, k.label(), Act::SetSort(*k)).on(st.sort == *k).k(if st.sort == *k && st.ascending { "\u{2191}" } else if st.sort == *k { "\u{2193}" } else { "" })).collect();
        v.push(mi("sort-vertical", if st.ascending { "Descending order" } else { "Ascending order" }, Act::ToggleAsc).gap());
        v.push(mi("folder", "Folders first", Act::ToggleFoldersFirst).on(st.folders_first));
        v
    }

    /// Context menus only show what applies: disabled rows disappear (their group gap moves on), and dense menus
    /// lift the quick actions into a row of icon buttons at the top.
    pub(crate) fn tidy_menu(items: Vec<MI>) -> Vec<MI> {
        let mut out: Vec<MI> = vec![];
        let mut carry = false;
        for mut it in items {
            if it.disabled {
                carry |= it.gap;
                continue;
            }
            if !it.sub.is_empty() {
                it.sub = Self::tidy_menu(std::mem::take(&mut it.sub));
                if it.sub.is_empty() {
                    carry |= it.gap;
                    continue;
                }
            }
            if carry {
                it.gap = true;
                carry = false;
            }
            out.push(it);
        }
        if let Some(f) = out.first_mut() {
            f.gap = false;
        }
        let nq = out.iter().filter(|i| i.quick).count();
        if out.len() >= 11 && nq >= 2 {
            let (mut q, mut rest): (Vec<MI>, Vec<MI>) = out.into_iter().partition(|i| i.quick);
            q.truncate(6);
            for i in &mut q {
                i.qrow = true;
                i.gap = false;
            }
            if let Some(f) = rest.first_mut() {
                f.gap = false;
            }
            q.extend(rest);
            return q;
        }
        out
    }

    fn qrow_h(items: &[MI]) -> f32 {
        if items.iter().any(|i| i.qrow) {
            QROW_H + 8.0
        } else {
            0.0
        }
    }

    pub(crate) fn menu_height(items: &[MI]) -> f32 {
        PAD_Y * 2.0 + Self::qrow_h(items) + items.iter().filter(|i| !i.qrow).map(|i| ITEM_H + if i.gap { GAP } else { 0.0 }).sum::<f32>()
    }

    /// y offset (from the menu top) of row `idx`.
    pub(crate) fn menu_row_y(items: &[MI], idx: usize) -> f32 {
        PAD_Y + Self::qrow_h(items) + items.iter().take(idx).filter(|i| !i.qrow).map(|i| ITEM_H + if i.gap { GAP } else { 0.0 }).sum::<f32>() + if items[idx].gap { GAP } else { 0.0 }
    }

    pub(crate) fn paint_menu(&mut self, ui: &mut Ui, pos: Pos2, items: &[MI], width: f32, id: &str, open_sub: Option<usize>) -> (Rect, Option<usize>, Option<Act>) {
        let th = self.th.clone();
        let p = ui.painter().clone();
        let h = Self::menu_height(items);
        let r = Rect::from_min_size(pos, vec2(width, h));
        menu_surface(&p, &th, r, 18.0);
        ui.allocate_rect(r, Sense::hover());
        let mut y = r.top() + PAD_Y;
        let mut chosen = None;
        let mut hover_sub = None;
        // ── quick-action row (dense menus)
        let qn = items.iter().filter(|i| i.qrow).count();
        if qn > 0 {
            let cw = (width - 12.0) / qn as f32;
            for (k, (i, it)) in items.iter().enumerate().filter(|(_, i)| i.qrow).enumerate() {
                let cell = Rect::from_min_size(pos2(r.left() + 6.0 + cw * k as f32 + 2.0, y), vec2(cw - 4.0, QROW_H));
                let resp = ui.interact(cell, Id::unique((id, i, "q")), Sense::click());
                let hv = self.anim(ui, (id, i, "qa"), resp.hovered());
                let down = resp.is_pointer_button_down_on();
                if down {
                    grad_fill(&p, cell, 11.0, alpha(th.shadow, if th.dark { 0.34 } else { 0.10 }), alpha(th.shadow, if th.dark { 0.2 } else { 0.05 }));
                    rim(&p, cell, 11.0, 3.0, alpha(th.shadow, if th.dark { 0.35 } else { 0.10 }), true, 12.0);
                } else {
                    raised(&p, &th, cell, 11.0, resp.hovered(), false);
                }
                let col = if it.danger { th.danger } else { mix(th.ink2, if th.dark { th.ink1 } else { th.accent_ink }, hv) };
                self.ico(&p, it.icon, if hv > 0.5 { Ic::Duo } else { Ic::Outline }, cell.center() + if down { vec2(0.0, 0.6) } else { vec2(0.0, 0.0) }, 18.0, col);
                let tip = if it.kbd.is_empty() { it.label.clone() } else { format!("{}   {}", it.label, it.kbd) };
                resp.clone().on_hover_text(tip);
                if resp.clicked() && it.act.is_some() {
                    chosen = it.act.clone();
                }
            }
            y += QROW_H + 8.0;
        }
        for (i, it) in items.iter().enumerate() {
            if it.qrow {
                continue;
            }
            if it.gap {
                y += GAP;
            }
            let row = Rect::from_min_size(pos2(r.left() + 6.0, y), vec2(width - 12.0, ITEM_H));
            let resp = ui.interact(row, Id::unique((id, i)), if it.disabled { Sense::hover() } else { Sense::click() });
            let hov = !it.disabled && (resp.hovered() || open_sub == Some(i));
            let hv = self.anim(ui, (id, i, "mi"), hov);
            let down = hov && resp.is_pointer_button_down_on();
            if down {
                // pressed: the row sinks into the surface
                grad_fill(&p, row, 9.0, alpha(th.shadow, if th.dark { 0.32 } else { 0.10 }), alpha(th.shadow, if th.dark { 0.18 } else { 0.05 }));
                rim(&p, row, 9.0, 3.0, alpha(th.shadow, if th.dark { 0.35 } else { 0.11 }), true, 12.0);
            } else if hv > 0.01 {
                // hover / open: a softly raised tile lit from above
                let (t0, t1) = if it.danger {
                    (alpha(th.danger, 0.18), alpha(th.danger, 0.08))
                } else if th.dark {
                    (c(255, 255, 255, 0.15), c(255, 255, 255, 0.06))
                } else {
                    (alpha(th.accent, 0.14), alpha(th.accent, 0.07))
                };
                shadow(&p, row, 9.0, 1.5, 4.0, -1.0, alpha(th.shadow, (if th.dark { 0.30 } else { 0.09 }) * hv));
                grad_fill(&p, row, 9.0, alpha(t0, hv), alpha(t1, hv));
                rim(&p, row, 9.0, 1.0, alpha(Color32::WHITE, (if th.dark { 0.18 } else { 0.95 }) * hv), true, 14.0);
                rim(&p, row, 9.0, 1.0, alpha(th.shadow, (if th.dark { 0.30 } else { 0.05 }) * hv), false, 5.0);
            } else if it.tog && it.checked {
                // on-state rows keep a faint recessed accent wash
                grad_fill(&p, row, 9.0, alpha(th.accent, 0.07), alpha(th.accent, 0.03));
            }
            let col = if it.disabled { alpha(th.ink2, 0.38) } else if it.danger { th.danger } else { th.ink1 };
            let ic = pos2(row.left() + 19.0, row.center().y);
            if let Some(dot) = it.dot {
                grad_fill(&p, Rect::from_center_size(ic, vec2(9.0, 9.0)), 4.5, mix(dot, Color32::WHITE, 0.25), dot);
            } else {
                self.ico(&p, it.icon, if hv > 0.5 { Ic::Duo } else { Ic::Outline }, ic, 16.0, if it.disabled { alpha(th.ink3, 0.4) } else if it.danger { th.danger } else { mix(th.ink3, if th.dark { th.ink1 } else { th.accent_ink }, hv) });
            }
            let right_reserve = if !it.sub.is_empty() { 24.0 } else if !it.kbd.is_empty() { self.tw(&p, it.kbd, &f_medium(T_META)) + 22.0 } else if it.tog { 26.0 } else { 12.0 };
            let label = self.fit(&p, &it.label, &f_medium(T_BODY), (row.width() - 40.0 - right_reserve).max(20.0));
            p.text(pos2(row.left() + 36.0, row.center().y + if down { 0.6 } else { 0.0 }), Align2::LEFT_CENTER, label, f_medium(T_BODY), col);
            if !it.sub.is_empty() {
                let c = pos2(row.right() - 13.0, row.center().y);
                let s = egui::Stroke::new(1.5, th.ink4);
                p.line_segment([c + vec2(-1.5, -3.5), c + vec2(2.0, 0.0)], s);
                p.line_segment([c + vec2(2.0, 0.0), c + vec2(-1.5, 3.5)], s);
                if hov {
                    hover_sub = Some(i);
                }
            } else if it.tog {
                let c = pos2(row.right() - 17.0, row.center().y);
                if it.checked {
                    self.ico(&p, "check-circle", Ic::Filled, c, 15.0, if it.disabled { alpha(th.accent, 0.4) } else { th.accent });
                } else {
                    // switched off: the tick stays, greyed, so the row still reads as a toggle
                    self.ico(&p, "check-circle", Ic::Outline, c, 15.0, alpha(th.ink4, if hov { 0.7 } else { 0.38 }));
                }
            } else if !it.kbd.is_empty() {
                p.text(pos2(row.right() - 12.0, row.center().y), Align2::RIGHT_CENTER, it.kbd, f_medium(T_META), th.ink4);
            }
            if resp.clicked() && it.act.is_some() && !it.disabled {
                chosen = it.act.clone();
            }
            y += ITEM_H;
        }
        (r, hover_sub, chosen)
    }

    pub fn draw_menu(&mut self, ctx: &Context) {
        let Some(mut m) = self.menu.clone() else { return };
        let screen = ctx.content_rect().intersect(win_rect().expand(0.0));
        let items = Self::tidy_menu(self.menu_items(&m));
        if items.is_empty() {
            self.menu = None;
            return;
        }
        let measure = ctx.debug_painter();
        let w = self.menu_width(&measure, &items, 238.0);
        let h = Self::menu_height(&items);
        let mut pos = m.pos;
        if matches!(m.kind, MenuKind::More | MenuKind::Sort) {
            pos.x -= w;
        }
        pos.x = pos.x.min(screen.right() - w - 10.0).max(screen.left() + 8.0);
        pos.y = if pos.y + h > screen.bottom() - 10.0 { (screen.bottom() - h - 10.0).max(8.0) } else { pos.y };
        // spring pop-in: slides a few px with a touch of overshoot while fading up
        let pop = if self.lib(|l| l.settings.reduce_motion) { 1.0 } else { spring_out(m.age) };
        pos.y += (1.0 - pop) * -9.0;
        if m.age < 0.5 {
            ctx.request_repaint();
        }
        let mut chosen: Option<Act> = None;
        let mut inside_any = false;
        let mut next_sub = m.sub;
        egui::Area::new(Id::unique("ctx-menu")).order(Order::Foreground).fixed_pos(screen.min).interactable(true).show(ctx, |ui| {
            ui.set_clip_rect(screen);
            ui.set_opacity((m.age * 9.0).clamp(0.0, 1.0));
            let bg = ui.allocate_rect(screen, Sense::click());
            let (r, hs, act) = self.paint_menu(ui, pos, &items, w, "m", next_sub);
            inside_any = ui.rect_contains_pointer(r);
            chosen = act;
            // submenu (kept open while the pointer is over the parent row or the submenu itself)
            if let Some(si) = hs.or(next_sub) {
                if let Some(row) = items.get(si) {
                    if !row.sub.is_empty() {
                        let row_y = pos.y + Self::menu_row_y(&items, si);
                        let sw = self.menu_width(&ui.painter().clone(), &row.sub, 220.0);
                        let sh = Self::menu_height(&row.sub);
                        let mut sp = pos2(r.right() + 4.0, row_y);
                        if sp.x + sw > screen.right() - 8.0 {
                            sp.x = r.left() - sw - 4.0;
                        }
                        sp.y = sp.y.min(screen.bottom() - sh - 10.0).max(8.0);
                        let (sr, _, sact) = self.paint_menu(ui, sp, &row.sub, sw, "ms", None);
                        if sact.is_some() {
                            chosen = sact;
                        }
                        let over_row = ui.rect_contains_pointer(Rect::from_min_size(pos2(r.left(), row_y), vec2(w, ITEM_H)));
                        let over_sub = ui.rect_contains_pointer(sr);
                        inside_any = inside_any || over_sub;
                        next_sub = if hs.is_some() || over_sub || (over_row && next_sub.is_some()) { Some(si) } else { None };
                    }
                }
            } else {
                next_sub = None;
            }
            if bg.clicked() && !inside_any {
                self.menu = None;
            }
        });
        if let Some(m) = self.menu.as_mut() {
            m.sub = next_sub;
            m.age += 0.016;
        }
        if let Some(a) = chosen {
            self.menu = None;
            self.acts.push(a);
        }
    }

    // ───────────── toasts + drag ghost + jobs ─────────────
    pub fn draw_toasts(&mut self, ctx: &Context) {
        let dt = ctx.input(|i| i.stable_dt).min(0.1);
        for t in &mut self.toasts {
            t.age += dt;
        }
        self.toasts.retain(|t| t.age < 3.8);
        for j in &mut self.jobs {
            j.age += dt;
        }
        self.jobs.retain(|j| j.state == JobState::Running || j.age < 0.1);
        if self.toasts.is_empty() {
            return;
        }
        ctx.request_repaint();
        let win = win_rect();
        let th = self.th.clone();
        egui::Area::new(Id::unique("toasts")).order(Order::Tooltip).fixed_pos(win.min).interactable(false).show(ctx, |ui| {
            let p = ui.painter().clone();
            let mut y = win.bottom() - STATUS_H - 18.0;
            for t in self.toasts.iter().rev() {
                let a = (1.0 - ((t.age - 3.2) / 0.6).clamp(0.0, 1.0)).min((t.age / 0.18).clamp(0.0, 1.0));
                let f = f_semi(T_BODY);
                let w = (self.tw(&p, &t.text, &f) + 62.0).min(win.width() - 40.0);
                let r = Rect::from_center_size(pos2(win.center().x + 126.0 * self.sidebar_open, y - 20.0 + (1.0 - a) * 10.0), vec2(w, 40.0));
                let sh = |c: Color32| alpha(c, a);
                shadow(&p, r, 20.0, 12.0, 30.0, -8.0, sh(alpha(th.shadow, if th.dark { 0.7 } else { 0.35 })));
                grad_fill(&p, r, 20.0, sh(if th.dark { c(40, 50, 70, 0.98) } else { c(28, 38, 58, 0.97) }), sh(if th.dark { c(26, 34, 50, 0.98) } else { c(20, 28, 44, 0.97) }));
                rim(&p, r, 20.0, 1.2, sh(c(255, 255, 255, 0.28)), true, 24.0);
                self.ico(&p, if t.ok { "check-circle" } else { "info-circle" }, Ic::Filled, pos2(r.left() + 22.0, r.center().y), 16.0, sh(if t.ok { hex("#59c79a") } else { hex("#ff7a84") }));
                let s = self.fit(&p, &t.text, &f, w - 62.0);
                p.text(pos2(r.left() + 40.0, r.center().y), Align2::LEFT_CENTER, s, f, sh(hex("#f4f7fd")));
                y -= 48.0;
            }
        });
    }

    pub fn draw_drag_ghost(&mut self, ctx: &Context) {
        let Some(d) = &self.drag else { return };
        let Some(pos) = ctx.pointer_latest_pos() else { return };
        ctx.set_cursor_icon(egui::CursorIcon::Grabbing);
        let th = self.th.clone();
        let label = d.label.clone();
        let n = d.items.len();
        let target = drop_target_at(pos);
        let hint = match &target {
            Some(DropTarget::Folder(l)) | Some(DropTarget::Place(l)) => Some(format!("Drop into {}", l.name())),
            Some(DropTarget::Tag(t)) => Some(format!("Tag {}", t)),
            Some(DropTarget::Trash) => Some("Move to Trash".into()),
            None => None,
        };
        let ctx_clone = ctx.clone();
        egui::Area::new(Id::unique("drag-ghost")).order(Order::Tooltip).fixed_pos(pos + vec2(16.0, 12.0)).interactable(false).show(&ctx_clone, |ui| {
            let p = ui.painter().clone();
            let f = f_semi(T_BODY);
            let w = self.tw(&p, &label, &f).min(230.0) + 48.0;
            let h = if hint.is_some() { 52.0 } else { 36.0 };
            let r = Rect::from_min_size(pos + vec2(16.0, 12.0), vec2(w, h));
            elevate(&p, &th, r, 12.0, 3);
            grad_fill(&p, r, 12.0, th.panel_top, th.panel_bottom);
            rim(&p, r, 12.0, 1.2, th.rim, true, 24.0);
            self.ico(&p, if hint.is_some() { "folder-open" } else { "copy" }, Ic::Duo, pos2(r.left() + 20.0, r.top() + 18.0), 15.0, th.accent);
            p.text(pos2(r.left() + 36.0, r.top() + 18.0), Align2::LEFT_CENTER, self.fit(&p, &label, &f, 220.0), f, th.ink1);
            if n > 1 {
                let b = Rect::from_center_size(r.right_top() + vec2(-12.0, 12.0), vec2(20.0, 20.0));
                grad_fill(&p, b, 10.0, th.accent, th.accent_press);
                p.text(b.center(), Align2::CENTER_CENTER, n.to_string(), f_bold(10.5), th.on_accent());
            }
            if let Some(h) = hint {
                p.text(pos2(r.left() + 12.0, r.bottom() - 12.0), Align2::LEFT_CENTER, self.fit(&p, &h, &f_medium(10.5), w - 24.0), f_medium(10.5), th.accent_ink);
            }
        });
    }

    pub fn draw_jobs_card(&mut self, ctx: &Context) {
        let running: Vec<(u64, String, f32, String)> = self.jobs.iter().filter(|j| j.state == JobState::Running).map(|j| (j.id, j.title.clone(), j.done as f32 / j.total.max(1) as f32, j.current.clone())).collect();
        if running.is_empty() {
            return;
        }
        let win = win_rect();
        let th = self.th.clone();
        let mut cancel = None;
        egui::Area::new(Id::unique("jobs-card")).order(Order::Foreground).fixed_pos(win.min).show(ctx, |ui| {
            let p = ui.painter().clone();
            let mut y = win.bottom() - STATUS_H - 16.0;
            for (id, title, frac, cur) in running.iter().rev() {
                let r = Rect::from_min_size(pos2(win.right() - 16.0 - 330.0, y - 74.0), vec2(330.0, 74.0));
                elevate(&p, &th, r, 18.0, 4);
                grad_fill(&p, r, 18.0, th.panel_top, th.panel_bottom);
                rim(&p, r, 18.0, 1.3, th.rim, true, 40.0);
                self.ico(&p, "archive", Ic::Filled, pos2(r.left() + 26.0, r.top() + 24.0), 18.0, th.accent);
                p.text(pos2(r.left() + 46.0, r.top() + 24.0), Align2::LEFT_CENTER, self.fit(&p, title, &f_semi(T_BODY), 230.0), f_semi(T_BODY), th.ink1);
                let c = Rect::from_center_size(pos2(r.right() - 24.0, r.top() + 24.0), vec2(26.0, 26.0));
                let cr_ = ui.interact(c, Id::unique(("cancel-job", id)), Sense::click());
                self.ico(&p, "close-circle", if cr_.hovered() { Ic::Filled } else { Ic::Duo }, c.center(), 18.0, th.ink3);
                if cr_.clicked() {
                    cancel = Some(*id);
                }
                let bar = Rect::from_min_size(pos2(r.left() + 18.0, r.top() + 46.0), vec2(r.width() - 36.0, 6.0));
                well(&p, &th, bar, 3.0);
                let (a, b) = th.accent_grad();
                grad_fill(&p, Rect::from_min_size(bar.min, vec2((bar.width() * frac).max(6.0), 6.0)), 3.0, a, b);
                p.text(pos2(r.left() + 18.0, r.top() + 62.0), Align2::LEFT_CENTER, self.fit(&p, cur, &f_medium(T_META), 200.0), f_medium(T_META), th.ink3);
                p.text(pos2(r.right() - 18.0, r.top() + 62.0), Align2::RIGHT_CENTER, format!("{:.0}%", frac * 100.0), f_semi(T_META), th.ink2);
                y -= 84.0;
            }
        });
        if let Some(id) = cancel {
            self.be.cancel_job(id);
        }
    }

    // ───────────── command palette ─────────────
    fn palette_groups(&self) -> Vec<(usize, String, Vec<(&'static str, String, String, Act, &'static str)>)> {
        if let Some(app) = self.palette.app {
            if app != crate::shell::AppId::Files {
                return self.app_palette_groups(app);
            }
        }
        let st = self.st();
        let q = self.palette.query.trim().to_lowercase();
        let cat = self.palette.cat;
        let mut commands: Vec<(&'static str, String, String, Act, &'static str)> = vec![
            ("add-folder", "New folder".into(), "Create an untitled folder here".into(), Act::NewFolder, "Ctrl Shift N"),
            ("terminal", "Open Terminal".into(), "A friendly shell on the same file system".into(), Act::OpenApp(crate::shell::AppId::Terminal), ""),
            ("gallery", "Open Photos".into(), "Browse, edit and favourite pictures".into(), Act::OpenApp(crate::shell::AppId::Photos), ""),
            ("add", "New tab".into(), "Open this folder in another tab".into(), Act::NewTab(None), "Ctrl T"),
            ("checklist-minimalistic", "Selection mode".into(), "Tick items to act on several at once".into(), Act::ToggleSelectMode, ""),
            ("stars-minimalistic", "Ask Smart".into(), "Search and tidy with on-device help".into(), Act::Ask, ""),
            ("sidebar-minimalistic", if self.sidebar_open > 0.5 { "Hide sidebar" } else { "Show sidebar" }.into(), "Layout".into(), Act::ToggleSidebar, "Ctrl \\"),
            ("widget", "Grid view".into(), "Large media, adaptive columns".into(), Act::SetGrid(true), "G"),
            ("list-check", "List view".into(), "Dense rows with aligned columns".into(), Act::SetGrid(false), "L"),
            ("stars-minimalistic", if st.dark { "Switch to light" } else { "Switch to dark" }.into(), "Appearance".into(), Act::ToggleTheme, "D"),
            ("eye", "Toggle hidden files".into(), "Show or hide dotfiles".into(), Act::ToggleHidden, "Ctrl H"),
            ("ssd-square", "Devices & locations".into(), "Mount, unmount, eject".into(), Act::Modal(Modal::Devices), ""),
            ("smartphone", "Phone continuity".into(), "Browse and send to your phone".into(), Act::Modal(Modal::Phone), ""),
            ("cloud", "Connect to server".into(), "SMB, SFTP, WebDAV, MTP".into(), Act::Modal(Modal::Connect), ""),
            ("gallery-wide", "Change wallpaper".into(), "Surfaces behind the window".into(), Act::Modal(Modal::Wallpaper), ""),
            ("widget", "Open design tokens".into(), "smart.toml \u{b7} draft 0.1".into(), Act::Modal(Modal::Tokens), "T"),
            ("copy", "Copy design tokens".into(), "Put smart.toml on the clipboard".into(), Act::CopyTokens, ""),
            ("settings-minimalistic", "Open settings".into(), "Appearance, browsing, transfers".into(), Act::Settings, "Ctrl ,"),
            ("magnifer-zoom-in", "8 px grid overlay".into(), "Craft".into(), Act::ToggleGridOverlay, ""),
            ("list-check", "Keyboard shortcuts".into(), "Every binding in one sheet".into(), Act::Modal(Modal::Shortcuts), "?"),
            ("refresh", "Refresh".into(), "Reload this folder".into(), Act::Refresh, "F5"),
            ("pen", "Rename selection".into(), "Edit the name in place".into(), Act::Rename(self.selected().first().map(|e| e.loc.clone()).unwrap_or_else(|| self.target_dir())), "F2"),
        ];
        let mut places: Vec<(&'static str, String, String, Act, &'static str)> = vec![];
        for p in &self.places {
            places.push((p.icon, format!("Open {}", p.title), self.pretty_path(&p.loc), Act::Nav(Route::Loc(p.loc.clone())), ""));
        }
        places.push(("clock-circle", "Open Recents".into(), "Everything you touched lately".into(), Act::Nav(Route::Recents), ""));
        for d in &self.devices {
            if let (Some(l), true) = (&d.loc, d.mounted) {
                places.push((d.kind.icon(), format!("Open {}", d.name), d.capacity_text(), Act::Nav(Route::Loc(l.clone())), ""));
            }
        }
        for t in self.lib(|l| l.tags.clone()) {
            places.push(("tag", format!("Open {} tag", t.name), format!("{} items", self.lib(|l| l.tag_count(&t.id))), Act::Nav(Route::Tag(t.id.clone())), ""));
        }
        let sel = self.selected();
        let mut selection: Vec<(&'static str, String, String, Act, &'static str)> = vec![];
        if !sel.is_empty() {
            let n = sel.len();
            selection.push(("info-circle", format!("Get Info on {} item{}", n, if n == 1 { "" } else { "s" }), format_size(self.sel_total()), Act::Info, "Ctrl I"));
            selection.push(("pen", "Rename the selection".into(), sel.first().map(|e| e.name.clone()).unwrap_or_default(), Act::Rename(sel[0].loc.clone()), "F2"));
            selection.push(("copy", "Copy to\u{2026}".into(), "Pick a destination".into(), Act::CopyTo(false), ""));
            selection.push(("folder", "Move to\u{2026}".into(), "Relocate without copy-paste".into(), Act::CopyTo(true), ""));
            selection.push(("archive", "Compress to ZIP\u{2026}".into(), "Native archive engine, optional password".into(), Act::ArchiveCompress, ""));
            selection.push(("trash-bin-minimalistic", "Move to Trash".into(), "Recoverable".into(), Act::Trash, "Del"));
            if self.sel_all_images() {
                selection.push(("gallery-wide", "Set as desktop wallpaper".into(), sel[0].name.clone(), Act::WallpaperFrom(sel[0].loc.clone()), ""));
            }
            if self.sel_has_archive() {
                selection.push(("folder-open", "Extract archive here".into(), "Unzip into a new folder".into(), Act::ExtractHere(sel[0].loc.clone()), ""));
            }
        }
        let mut settings: Vec<(&'static str, String, String, Act, &'static str)> = vec![
            ("settings-minimalistic", "Settings".into(), "Open the full settings page".into(), Act::Settings, "Ctrl ,"),
            ("stars-minimalistic", if st.dark { "Light appearance" } else { "Dark appearance" }.into(), "Theme".into(), Act::ToggleTheme, "D"),
            ("eye", "Hidden files".into(), if st.show_hidden { "Currently shown" } else { "Currently hidden" }.into(), Act::ToggleHidden, "Ctrl H"),
            ("archive", "Transfers".into(), format!("Demo transfer animation: {:.0}s", st.copy_animation_secs), Act::SettingsTab(3), ""),
            ("text", "Show extensions".into(), if st.show_extensions { "On" } else { "Off" }.into(), Act::ToggleExtensions, ""),
            ("widget", "Wallpaper".into(), "Desktop surface".into(), Act::Modal(Modal::Wallpaper), ""),
        ];
        let mut files: Vec<(&'static str, String, String, Act, &'static str)> = vec![];
        if q.len() >= 2 {
            for h in self.palette.hits.iter().take(8) {
                let act = if h.entry.is_dir { Act::Nav(Route::Loc(h.entry.loc.clone())) } else { Act::Open(h.entry.loc.clone()) };
                files.push((h.entry.kind.icon(), h.entry.name.clone(), h.entry.loc.parent().map(|p| self.pretty_path(&p)).unwrap_or_default(), act, if h.entry.is_dir { "" } else { "Open" }));
            }
        }
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
        let filt = |v: Vec<(&'static str, String, String, Act, &'static str)>, limit: usize| -> Vec<(&'static str, String, String, Act, &'static str)> {
            let mut s: Vec<_> = v.into_iter().filter_map(|x| score(&format!("{} {}", x.1, x.2)).map(|sc| (sc, x))).collect();
            s.sort_by_key(|x| -x.0);
            s.into_iter().take(limit).map(|x| x.1).collect()
        };
        let mut out: Vec<(usize, String, Vec<(&'static str, String, String, Act, &'static str)>)> = vec![];
        let want = |i: usize| cat == 0 || cat == i;
        if want(1) {
            let v = filt(selection, if q.is_empty() { 4 } else { 6 });
            if !v.is_empty() {
                out.push((1, "Selection".into(), v));
            }
        }
        if want(2) {
            let v = filt(commands.drain(..).collect(), 7);
            if !v.is_empty() {
                out.push((2, if q.is_empty() { "Suggested" } else { "Commands" }.to_string(), v));
            }
        }
        if want(3) {
            let v = filt(places, 6);
            if !v.is_empty() {
                out.push((3, "Navigate".into(), v));
            }
        }
        if want(4) && !files.is_empty() {
            out.push((4, "Files & folders".into(), files));
        }
        if want(5) {
            let v = filt(settings.drain(..).collect(), 6);
            if !v.is_empty() {
                out.push((5, "Preferences".into(), v));
            }
        }
        if !q.is_empty() {
            out.push((2, "Search".into(), vec![("magnifer", format!("Search everywhere for \u{201c}{}\u{201d}", self.palette.query.trim()), "Fuzzy name search \u{b7} kind: size: modified: tag:".into(), Act::RunSearch(self.palette.query.trim().to_string()), "Enter")]));
        }
        out
    }

    pub fn draw_palette(&mut self, ctx: &Context) {
        if !self.palette.open {
            return;
        }
        let win = self.palette_win();
        let th = self.th.clone();
        let q = self.palette.query.trim().to_string();
        if q != self.palette.last_sent {
            self.palette.last_sent = q.clone();
            self.palette.hits.clear();
            self.palette.sel = 0;
            if q.len() >= 2 {
                let home = self.be.home();
                self.palette.req = self.be.search(&q, &home, false);
            }
        }
        let groups = self.palette_groups();
        let flat: Vec<(usize, &'static str, String, String, Act, &'static str)> = groups.iter().flat_map(|(g, _, v)| v.iter().map(move |x| (*g, x.0, x.1.clone(), x.2.clone(), x.3.clone(), x.4))).collect();
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
            self.palette.sel = (self.palette.sel + 1).min(flat.len().saturating_sub(1));
        }
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
            self.palette.sel = self.palette.sel.saturating_sub(1);
        }
        self.palette.sel = self.palette.sel.min(flat.len().saturating_sub(1));
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.palette.open = false;
            return;
        }
        let enter = ctx.input(|i| i.key_pressed(egui::Key::Enter));
        let ctrl_n = ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::N));
        let mut run: Option<Act> = None;
        let mut run_idx: Option<usize> = None;
        let mut close = false;
        let w = 760.0f32.min(win.width() - 36.0);
        let h = (win.height() - 130.0).clamp(240.0, 600.0);
        let card = Rect::from_min_size(pos2(win.center().x - w / 2.0, win.top() + (win.height() - h) * 0.32), vec2(w, h));
        let rail_w = 78.0;
        egui::Area::new(Id::unique("palette")).order(Order::Foreground).fixed_pos(win.min).show(ctx, |ui| {
            let p = ui.painter().clone();
            ui.set_clip_rect(win);
            let scrim = ui.allocate_rect(win, Sense::click());
            grad_fill(&p, win, R_WIN, th.scrim, th.scrim);
            if scrim.clicked() {
                close = true;
            }
            elevate(&p, &th, card, 22.0, 5);
            grad_fill(&p, card, 22.0, th.panel_top, th.panel_bottom);
            rim(&p, card, 22.0, 1.4, th.rim, true, 50.0);
            ui.allocate_rect(card, Sense::click());
            // premium rail
            let rail = Rect::from_min_size(card.min, vec2(rail_w, card.height()));
            grad_fill(&p, rail, 22.0, th.well, alpha(th.well, 0.35));
            p.rect_filled(Rect::from_min_max(pos2(rail.right() - 1.0, rail.top() + 20.0), pos2(rail.right(), rail.bottom() - 20.0)), cr(0.5), alpha(th.shadow, if th.dark { 0.4 } else { 0.06 }));
            let cats: Vec<(usize, &'static str, &'static str)> = if self.palette.app.is_some_and(|a| a != crate::shell::AppId::Files) {
                vec![(0, "magnifer", "All"), (2, "terminal", "Commands"), (3, "folder", "Places"), (4, "file", "Items"), (5, "settings-minimalistic", "Settings")]
            } else {
                vec![(0, "magnifer", "All"), (1, "check-circle", "Select"), (2, "terminal", "Commands"), (3, "folder", "Places"), (4, "file", "Files"), (5, "settings-minimalistic", "Settings")]
            };
            let mut ry = card.top() + 58.0;
            for (i, icon, label) in cats {
                let b = Rect::from_min_size(pos2(rail.left() + 9.0, ry), vec2(rail_w - 18.0, 52.0));
                let resp = ui.interact(b, Id::unique(("pal-cat", i)), Sense::click());
                let on = self.palette.cat == i;
                if on {
                    grad_fill(&p, b, 13.0, alpha(th.accent, 0.22), alpha(th.accent, 0.10));
                    rim(&p, b, 13.0, 1.0, alpha(Color32::WHITE, if th.dark { 0.12 } else { 0.6 }), true, 16.0);
                } else if resp.hovered() {
                    p.rect_filled(b, cr(13.0), th.row_hover);
                }
                self.ico(&p, icon, if on { Ic::Filled } else { Ic::Duo }, pos2(b.center().x, b.top() + 19.0), 18.0, if on { th.accent_ink } else { th.ink3 });
                p.text(pos2(b.center().x, b.top() + 38.0), Align2::CENTER_CENTER, label, f_semi(9.5), if on { th.accent_ink } else { th.ink4 });
                if resp.clicked() {
                    self.palette.cat = i;
                    self.palette.sel = 0;
                }
                ry += 56.0;
            }
            let badge = Rect::from_center_size(pos2(rail.center().x, card.bottom() - 30.0), vec2(52.0, 20.0));
            grad_fill(&p, badge, 10.0, alpha(th.good, 0.18), alpha(th.good, 0.10));
            p.text(badge.center(), Align2::CENTER_CENTER, "ON DEVICE", f_semi(8.5), th.good);
            // input
            let inp = Rect::from_min_size(pos2(rail.right() + 14.0, card.top() + 14.0), vec2(card.width() - rail_w - 28.0, 46.0));
            well(&p, &th, inp, 23.0);
            self.ico(&p, "magnifer", Ic::Duo, pos2(inp.left() + 24.0, inp.center().y), 18.0, th.accent_ink);
            let mut qq = std::mem::take(&mut self.palette.query);
            let te = ui.put(Rect::from_min_max(inp.min + vec2(46.0, 8.0), inp.max - vec2(58.0, 8.0)), egui::TextEdit::singleline(&mut qq).id(Id::unique("palette-input")).frame(egui::Frame::NONE).hint_text(egui::RichText::new("Search commands, places and files\u{2026}").color(th.ink4)).font(f_medium(T_TITLE)).text_color(th.ink1).vertical_align(egui::Align::Center).margin(egui::Margin::ZERO));
            if self.palette.focus {
                te.request_focus();
                self.palette.focus = false;
            }
            self.palette.query = qq;
            self.keycap(&p, pos2(inp.right() - 12.0, inp.center().y), "Esc", false);
            // results
            let list = Rect::from_min_max(pos2(inp.left(), inp.bottom() + 10.0), pos2(card.right() - 12.0, card.bottom() - 46.0));
            let mut cui = ui.new_child(egui::UiBuilder::new().max_rect(list).layout(egui::Layout::top_down(egui::Align::Min)));
            cui.set_clip_rect(list.intersect(ui.clip_rect()));
            let mut idx = 0usize;
            let sel_i = self.palette.sel;
            let inner = egui::ScrollArea::vertical().id_salt("palette-scroll").auto_shrink([false, false]).show(&mut cui, |ui| {
                let p = ui.painter().with_clip_rect(list.intersect(ui.clip_rect()));
                let top = ui.max_rect().top();
                let total_h = groups.iter().map(|(_, _, v)| 26.0 + v.len() as f32 * 46.0).sum::<f32>() + 10.0;
                ui.allocate_rect(Rect::from_min_size(pos2(list.left(), top), vec2(list.width(), total_h)), Sense::hover());
                let mut yy = top;
                for (_, gname, items) in &groups {
                    p.text(pos2(list.left() + 6.0, yy + 13.0), Align2::LEFT_CENTER, gname.to_uppercase(), f_semi(T_LABEL), th.ink4);
                    yy += 26.0;
                    for (icon, title, sub, act, kbd) in items {
                        let row = Rect::from_min_size(pos2(list.left(), yy), vec2(list.width(), 44.0));
                        let r = ui.interact(row, Id::unique(("pal", idx)), Sense::click());
                        let on = idx == sel_i;
                        if on {
                            grad_fill(&p, row, 12.0, th.accent_tint, th.accent_tint2);
                            if ui.input(|i| i.key_pressed(egui::Key::ArrowDown) || i.key_pressed(egui::Key::ArrowUp)) {
                                ui.scroll_to_rect(row, None);
                            }
                        } else if r.hovered() {
                            p.rect_filled(row, cr(12.0), th.row_hover);
                        }
                        let ic = Rect::from_center_size(pos2(row.left() + 26.0, row.center().y), vec2(30.0, 30.0));
                        grad_fill(&p, ic, 9.0, alpha(th.accent, if on { 0.22 } else { 0.10 }), alpha(th.accent, if on { 0.14 } else { 0.05 }));
                        self.ico(&p, icon, if on { Ic::Filled } else { Ic::Duo }, ic.center(), 16.0, if on { th.accent_ink } else { th.ink2 });
                        let text_w = row.width() - 104.0;
                        p.text(pos2(row.left() + 54.0, row.center().y - 8.0), Align2::LEFT_CENTER, self.fit(&p, title, &f_semi(T_BODY), text_w), f_semi(T_BODY), th.ink1);
                        p.text(pos2(row.left() + 54.0, row.center().y + 9.0), Align2::LEFT_CENTER, self.fit(&p, sub, &f_medium(T_META), text_w), f_medium(T_META), th.ink3);
                        if !kbd.is_empty() {
                            self.keycap(&p, pos2(row.right() - 12.0, row.center().y), kbd, false);
                        }
                        if r.clicked() {
                            run = Some(act.clone());
                            run_idx = Some(idx);
                        }
                        if r.hovered() && ui.input(|i| i.pointer.delta().length() > 0.5) {
                            self.palette.sel = idx;
                        }
                        yy += 46.0;
                        idx += 1;
                    }
                }
                if flat.is_empty() {
                    p.text(pos2(list.center().x, top + 70.0), Align2::CENTER_CENTER, "No matching commands or places", f_medium(T_BODY), th.ink3);
                    p.text(pos2(list.center().x, top + 94.0), Align2::CENTER_CENTER, "Try \u{201c}archive\u{201d}, \u{201c}settings\u{201d} or \u{201c}documents\u{201d}", f_medium(T_META), th.ink4);
                }
            });
            // enforce the card bounds after the scroll area
            p.rect_filled(Rect::from_min_max(pos2(card.left(), card.top()), pos2(card.left() + rail_w, card.bottom())), cr(0.0), alpha(th.well, 0.0));
            let _ = inner;
            // footer (inside the card, never overlapping results)
            let foot_y = card.bottom() - 24.0;
            for (i, (k, t)) in [("\u{2191}\u{2193}", "navigate"), ("Enter", "run"), ("Esc", "close")].into_iter().enumerate() {
                let x = card.left() + rail_w + 18.0 + i as f32 * 96.0;
                self.keycap(&p, pos2(x + 26.0, foot_y), k, false);
                p.text(pos2(x + 32.0, foot_y), Align2::LEFT_CENTER, t, f_medium(T_META), th.ink4);
            }
            p.text(pos2(card.right() - 16.0, foot_y), Align2::RIGHT_CENTER, format!("{} indexed \u{b7} on-device", self.be.indexed_count()), f_medium(T_META), th.ink4);
        });
        let _ = run_idx;
        if ctrl_n {
            run = Some(Act::NewFolder);
        }
        if close {
            self.palette.open = false;
        }
        if let Some(a) = run {
            self.palette.open = false;
            self.acts.push(a);
        } else if enter {
            if let Some((_, _, _, _, act, _)) = flat.get(self.palette.sel) {
                let a = act.clone();
                self.palette.open = false;
                self.acts.push(a);
            }
        }
    }


    /// Compact search opened from the collapsed app-bar button.
    pub fn draw_search_sheet(&mut self, ctx: &Context) {
        if !self.search.sheet || self.win_closed || self.win_min {
            return;
        }
        let win = win_rect();
        let th = self.th.clone();
        let w = 560.0f32.min(win.width() - 32.0);
        let card = Rect::from_min_size(pos2(win.right() - 16.0 - w, win.top() + 66.0), vec2(w, 330.0));
        let mut close = false;
        let mut run: Option<Act> = None;
        egui::Area::new(Id::unique("search-sheet")).order(Order::Foreground).fixed_pos(win.min).show(ctx, |ui| {
            let p = ui.painter().clone();
            ui.set_clip_rect(win);
            let bg = ui.allocate_rect(win, Sense::click());
            if bg.clicked() {
                close = true;
            }
            elevate(&p, &th, card, 20.0, 5);
            grad_fill(&p, card, 20.0, th.panel_top, th.panel_bottom);
            rim(&p, card, 20.0, 1.3, th.rim, true, 50.0);
            ui.allocate_rect(card, Sense::click());
            let inp = Rect::from_min_size(card.min + vec2(12.0, 12.0), vec2(card.width() - 24.0, 44.0));
            well(&p, &th, inp, 22.0);
            self.ico(&p, "magnifer", Ic::Duo, pos2(inp.left() + 23.0, inp.center().y), 17.0, th.accent_ink);
            let mut q = std::mem::take(&mut self.search.query);
            let id = Id::unique("sheet-input");
            let te = ui.put(Rect::from_min_max(inp.min + vec2(42.0, 6.0), inp.max - vec2(44.0, 6.0)), egui::TextEdit::singleline(&mut q).id(id).frame(egui::Frame::NONE).hint_text(egui::RichText::new(format!("Search {}", self.title())).color(th.ink4)).font(f_medium(T_TITLE)).text_color(th.ink1).vertical_align(egui::Align::Center).margin(egui::Margin::ZERO));
            if self.search.focus_next {
                te.request_focus();
                self.search.focus_next = false;
            }
            self.search.query = q;
            if self.search.dirty_at < self.time - 0.001 {
                // typing is handled by the debounce in update(); mark activity
            }
            self.keycap(&p, pos2(inp.right() - 12.0, inp.center().y), "Esc", false);
            let chips: Vec<(String, bool, Act)> = vec![
                (if self.search.everywhere { "Everywhere".into() } else { format!("In {}", self.title()) }, self.search.everywhere, Act::ToggleEverywhere),
                ("Inside files".into(), self.st().search_contents, Act::ToggleContents),
                ("Images".into(), self.search.query.contains("kind:image"), Act::RunSearch(format!("{} kind:image", self.search.query.trim()))),
                ("Archives".into(), self.search.query.contains("kind:archive"), Act::RunSearch(format!("{} kind:archive", self.search.query.trim()))),
            ];
            let mut x = card.left() + 14.0;
            for (i, (label, on, act)) in chips.into_iter().enumerate() {
                let f = f_semi(11.0);
                let wd = self.tw(&p, &label, &f) + 26.0;
                let r = Rect::from_min_size(pos2(x, inp.bottom() + 10.0), vec2(wd, 26.0));
                x += wd + 6.0;
                let resp = ui.interact(r, Id::unique(("sheet-chip", i)), Sense::click());
                if on {
                    grad_fill(&p, r, 13.0, alpha(th.accent, 0.20), alpha(th.accent, 0.12));
                } else {
                    raised(&p, &th, r, 13.0, resp.hovered(), false);
                }
                p.text(r.center(), Align2::CENTER_CENTER, label, f, if on { th.accent_ink } else { th.ink2 });
                if resp.clicked() {
                    run = Some(act);
                }
            }
            // preview of top results
            let hits = self.search.hits.iter().take(4).cloned().collect::<Vec<_>>();
            let mut y = inp.bottom() + 44.0;
            if hits.is_empty() {
                p.text(pos2(card.left() + 18.0, y + 16.0), Align2::LEFT_CENTER, if self.search_active() { "No matches yet \u{2014} keep typing" } else { "Type to search names, or use kind:, tag:, modified:" }, f_medium(T_BODY), th.ink3);
            }
            for (i, h) in hits.iter().enumerate() {
                let r = Rect::from_min_size(pos2(card.left() + 10.0, y), vec2(card.width() - 20.0, 40.0));
                let resp = ui.interact(r, Id::unique(("sheet-hit", i)), Sense::click());
                if resp.hovered() {
                    p.rect_filled(r, cr(10.0), th.row_hover);
                }
                self.ico(&p, h.entry.kind.icon(), Ic::Duo, pos2(r.left() + 18.0, r.center().y), 16.0, kind_color(h.entry.kind, &th));
                p.text(pos2(r.left() + 36.0, r.center().y), Align2::LEFT_CENTER, self.fit(&p, &h.entry.name, &f_medium(T_BODY), r.width() - 120.0), f_medium(T_BODY), th.ink1);
                p.text(pos2(r.right() - 12.0, r.center().y), Align2::RIGHT_CENTER, format_size(h.entry.size), f_medium(T_META), th.ink3);
                if resp.clicked() {
                    run = Some(if h.entry.is_dir { Act::Nav(Route::Loc(h.entry.loc.clone())) } else { Act::Open(h.entry.loc.clone()) });
                    close = true;
                }
                y += 42.0;
            }
            p.text(pos2(card.left() + 18.0, card.bottom() - 18.0), Align2::LEFT_CENTER, format!("{} results \u{b7} {} ms \u{b7} on-device", self.search.hits.len(), self.search.ms), f_medium(10.5), th.ink4);
            let all = Rect::from_min_size(pos2(card.right() - 130.0, card.bottom() - 32.0), vec2(116.0, 26.0));
            if self.label_button(ui, all, None, "Show all results", "sheet-all", Btn::Accent).clicked() {
                close = true;
                self.acts.push(Act::FocusSearch);
            }
        });
        if close {
            self.acts.push(Act::SearchSheet(false));
        }
        if let Some(a) = run {
            self.acts.push(a);
        }
    }

    // ───────────── Ask Smart panel ─────────────
    pub fn draw_ai(&mut self, ctx: &Context) {
        let dt = ctx.input(|i| i.stable_dt).min(0.05);
        let target = if self.ai.open { 1.0 } else { 0.0 };
        self.ai.anim += (target - self.ai.anim) * (1.0 - (-dt * 18.0).exp());
        if (target - self.ai.anim).abs() > 0.002 {
            ctx.request_repaint();
        }
        if self.ai.anim < 0.01 {
            return;
        }
        let win = win_rect();
        let th = self.th.clone();
        let a = self.ai.anim;
        let w = 394.0f32.min((win.width() - 40.0).max(280.0));
        let panel = Rect::from_min_max(pos2(win.right() - 10.0 - w + (1.0 - a) * 40.0, win.top() + 68.0), pos2(win.right() - 10.0 + (1.0 - a) * 40.0, win.bottom() - 44.0));
        let mut acts: Vec<Act> = vec![];
        let mut send = false;
        // geometry: header 62, composer 108, body scrollable rest
        let header = Rect::from_min_size(panel.min, vec2(panel.width(), 62.0));
        let composer = Rect::from_min_max(pos2(panel.left() + 14.0, panel.bottom() - 116.0), pos2(panel.right() - 14.0, panel.bottom() - 42.0));
        let body = Rect::from_min_max(pos2(panel.left() + 12.0, header.bottom()), pos2(panel.right() - 12.0, composer.top() - 8.0));
        // wheel scroll
        if ui_contains(ctx, body) {
            let d = ctx.input(|i| i.smooth_scroll_delta.y);
            if d != 0.0 {
                self.ai.scroll = (self.ai.scroll - d).max(0.0);
            }
        }
        let body_clip = body.intersect(win);
        egui::Area::new(Id::unique("ai-panel")).order(Order::Foreground).fixed_pos(win.min).show(ctx, |ui| {
            let p = ui.painter().clone();
            ui.set_clip_rect(win.expand(90.0));
            let fa = |c: Color32| alpha(c, a);
            elevate(&p, &th, panel, 20.0, 4);
            grad_fill(&p, panel, 20.0, fa(th.panel_top), fa(th.panel_bottom));
            rim(&p, panel, 20.0, 1.3, fa(th.rim), true, 40.0);
            ui.allocate_rect(panel, Sense::click());
            // header
            let orb = Rect::from_min_size(panel.min + vec2(14.0, 12.0), vec2(38.0, 38.0));
            let (c0, c1) = th.accent_grad();
            shadow(&p, orb, 12.0, 4.0, 12.0, -2.0, fa(alpha(th.accent, 0.4)));
            grad_fill(&p, orb, 12.0, fa(c0), fa(c1));
            rim(&p, orb, 12.0, 1.2, fa(Color32::from_white_alpha(140)), true, 20.0);
            self.ico(&p, "stars-minimalistic", Ic::Filled, orb.center(), 20.0, th.on_accent());
            p.text(pos2(orb.right() + 12.0, orb.top() + 12.0), Align2::LEFT_CENTER, "Ask Smart", f_bold(T_TITLE), fa(th.ink1));
            let loc = self.title();
            p.circle_filled(pos2(orb.right() + 16.0, orb.top() + 31.0), 3.0, fa(th.good));
            p.text(pos2(orb.right() + 25.0, orb.top() + 31.0), Align2::LEFT_CENTER, self.fit(&p, &format!("On-device \u{b7} aware of {}", loc), &f_medium(T_META), panel.width() - 130.0), f_medium(T_META), th.ink3);
            let close = Rect::from_center_size(pos2(panel.right() - 28.0, orb.center().y), vec2(30.0, 30.0));
            let cr_ = ui.interact(close, Id::unique("ai-close"), Sense::click());
            if cr_.hovered() {
                p.circle_filled(close.center(), 15.0, th.row_hover);
            }
            self.ico(&p, "close-circle", if cr_.hovered() { Ic::Filled } else { Ic::Duo }, close.center(), 18.0, th.ink3);
            if cr_.clicked() {
                acts.push(Act::Ask);
            }
            let pp = p.with_clip_rect(body_clip);
            let mut y = body.top() - self.ai.scroll;
            let x0 = body.left() + 2.0;
            let mw = body.width() - 4.0;
            // awareness card (always; this is the context the panel is reasoning over)
            let aw = Rect::from_min_size(pos2(x0, y), vec2(mw, 96.0));
            grad_fill(&pp, aw, 14.0, fa(th.card_top), fa(th.card_bottom));
            rim(&pp, aw, 14.0, 1.0, fa(th.rim), true, 30.0);
            self.ico(&pp, "folder-check", Ic::Duo, pos2(aw.left() + 20.0, aw.top() + 20.0), 14.0, th.accent_ink);
            pp.text(pos2(aw.left() + 34.0, aw.top() + 20.0), Align2::LEFT_CENTER, "Awareness \u{b7} current folder", f_semi(T_META), th.ink2);
            let imgs: Vec<Entry> = self.items.iter().filter(|e| e.kind == Kind::Image).take(4).cloned().collect();
            let mut tx = aw.left() + 14.0;
            for e in &imgs {
                let tr = Rect::from_min_size(pos2(tx, aw.top() + 32.0), vec2(52.0, 34.0));
                if let Some(t) = self.thumbs.get(&e.loc) {
                    let sz = t.tex.size_vec2();
                    textured_rrect(&pp, tr, 7.0, t.tex.id(), cover_uv(sz.x, sz.y, tr), fa(Color32::WHITE));
                } else {
                    grad_fill(&pp, tr, 7.0, alpha(th.accent, 0.2), alpha(th.accent, 0.1));
                }
                tx += 58.0;
            }
            let old = smart_core::tools::older_than(&self.items, 90).len();
            let stats = format!("{} items \u{b7} {} \u{b7} {} folders \u{b7} {} stale", self.items.len(), format_size(self.total_bytes()), self.items.iter().filter(|e| e.is_dir).count(), old);
            pp.text(pos2(aw.left() + 14.0, aw.top() + 76.0), Align2::LEFT_CENTER, self.fit(&pp, &stats, &f_medium(10.5), mw - 28.0), f_medium(10.5), th.ink3);
            y = aw.bottom() + 10.0;
            // suggestion chips (wrap, measured)
            let mut cx = x0;
            let mut cy = y;
            for (ic, label, a_) in [("folder-check", "Organise this folder", Act::Organise), ("copy", "Find duplicates", Act::FindDuplicates), ("database", "Largest files", Act::FindLarge), ("clock-circle", "Find old files", Act::FindOld), ("gallery-wide", "Wallpaper", Act::Modal(Modal::Wallpaper))] {
                let f = f_semi(11.0);
                let wd = self.tw(&pp, label, &f) + 42.0;
                if cx + wd > x0 + mw {
                    cx = x0;
                    cy += 34.0;
                }
                let r = Rect::from_min_size(pos2(cx, cy), vec2(wd, 28.0));
                let resp = ui.interact(r, Id::unique(("ai-chip", label)), Sense::click());
                raised(&pp, &th, r, 14.0, resp.hovered(), resp.is_pointer_button_down_on());
                self.ico(&pp, ic, Ic::Duo, pos2(r.left() + 17.0, r.center().y), 13.0, th.accent_ink);
                pp.text(pos2(r.left() + 29.0, r.center().y), Align2::LEFT_CENTER, label, f, th.ink1);
                if resp.clicked() {
                    acts.push(a_);
                }
                cx += wd + 6.0;
            }
            y = cy + 40.0;
            // conversation
            let msgs: Vec<(bool, String, Vec<Location>, Vec<(String, Act)>)> = self.ai.msgs.iter().map(|m| (m.user, m.text.clone(), m.files.clone(), m.actions.clone())).collect();
            for (mi_, (user, text, files, actions)) in msgs.iter().enumerate() {
                let h = self.draw_bubble(&pp, x0, y, mw, text, *user);
                y += h;
                if !files.is_empty() {
                    let mut fx = x0 + if *user { 0.0 } else { 34.0 };
                    for l in files.iter().take(6) {
                        let r = Rect::from_min_size(pos2(fx, y), vec2(46.0, 32.0));
                        if let Some(t) = self.thumbs.get(l) {
                            let sz = t.tex.size_vec2();
                            textured_rrect(&pp, r, 7.0, t.tex.id(), cover_uv(sz.x, sz.y, r), Color32::WHITE);
                        } else {
                            let k = Kind::from_ext(&ext_of(&l.name()));
                            grad_fill(&pp, r, 7.0, alpha(kind_color(k, &th), 0.2), alpha(kind_color(k, &th), 0.1));
                            self.ico(&pp, k.icon(), Ic::Duo, r.center(), 15.0, kind_color(k, &th));
                        }
                        fx += 52.0;
                    }
                    y += 40.0;
                }
                if !actions.is_empty() {
                    let mut bx = x0 + 34.0;
                    let mut by = y;
                    for (label, a_) in actions {
                        let wd = self.tw(&pp, label, &f_semi(11.5)) + 26.0;
                        if bx + wd > x0 + mw {
                            bx = x0 + 34.0;
                            by += 36.0;
                        }
                        let r = Rect::from_min_size(pos2(bx, by), vec2(wd, 30.0));
                        let resp = ui.interact(r, Id::unique(("ai-act", mi_, label)), Sense::click());
                        raised_accent(&pp, &th, r, 15.0, resp.hovered(), resp.is_pointer_button_down_on());
                        pp.text(r.center(), Align2::CENTER_CENTER, label, f_semi(11.5), th.on_accent());
                        if resp.clicked() {
                            acts.push(a_.clone());
                        }
                        bx += wd + 6.0;
                    }
                    y = by + 38.0;
                }
                y += 6.0;
            }
            let content_h = (y - (body.top() - self.ai.scroll)).max(1.0);
            let max = (content_h - body.height()).max(0.0);
            self.ai.scroll = self.ai.scroll.clamp(0.0, max);
            if max > 0.5 {
                let track = Rect::from_min_size(pos2(body.right() - 3.0, body.top() + 3.0), vec2(3.0, body.height() - 6.0));
                p.rect_filled(track, cr(1.5), alpha(th.shadow, if th.dark { 0.25 } else { 0.05 }));
                let h = (track.height() * body.height() / content_h).max(24.0);
                let t = (self.ai.scroll / max).clamp(0.0, 1.0);
                p.rect_filled(Rect::from_min_size(pos2(track.left(), track.top() + (track.height() - h) * t), vec2(3.0, h)), cr(1.5), alpha(th.ink3, 0.45));
            }
            // composer
            well(&pp, &th, composer, 16.0);
            let mut inp = std::mem::take(&mut self.ai.input);
            let field = Rect::from_min_size(composer.min + vec2(14.0, 8.0), vec2(composer.width() - 28.0, 32.0));
            let te = ui.put(field, egui::TextEdit::singleline(&mut inp).id(Id::unique("ai-input")).frame(egui::Frame::NONE).hint_text(egui::RichText::new("Ask about these files\u{2026}").color(th.ink4)).font(f_medium(T_BODY)).text_color(th.ink1).vertical_align(egui::Align::Center).margin(egui::Margin::ZERO));
            self.ai.input = inp;
            if te.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                send = true;
                te.request_focus();
            }
            let chip = Rect::from_min_size(composer.left_bottom() + vec2(12.0, -28.0), vec2(116.0, 22.0));
            grad_fill(&pp, chip, 8.0, alpha(th.well, 0.7), alpha(th.well, 0.4));
            self.ico(&pp, "widget", Ic::Duo, pos2(chip.left() + 13.0, chip.center().y), 12.0, th.ink3);
            pp.text(pos2(chip.left() + 24.0, chip.center().y), Align2::LEFT_CENTER, "Smart Neura \u{b7} local", f_medium(10.5), th.ink3);
            let sb = Rect::from_center_size(pos2(composer.right() - 24.0, composer.bottom() - 19.0), vec2(32.0, 32.0));
            let sr = ui.interact(sb, Id::unique("ai-send"), Sense::click());
            raised_accent(&pp, &th, sb, 16.0, sr.hovered(), sr.is_pointer_button_down_on());
            self.ico(&pp, "arrow-right-up", Ic::Duo, sb.center(), 16.0, th.on_accent());
            if sr.clicked() {
                send = true;
            }
            p.text(pos2(panel.center().x, panel.bottom() - 20.0), Align2::CENTER_CENTER, "AI actions are previews \u{b7} nothing leaves this device", f_medium(10.5), th.ink4);
        });
        for a in acts {
            self.acts.push(a);
        }
        if send {
            self.ai_send();
        }
    }

    fn draw_bubble(&self, p: &egui::Painter, x: f32, y: f32, w: f32, text: &str, user: bool) -> f32 {
        let th = &self.th;
        let f = f_medium(T_BODY);
        let maxw = if user { (w - 80.0).max(120.0) } else { (w - 40.0).max(120.0) };
        let galley = p.layout(text.to_string(), f.clone(), if user { th.on_accent() } else { th.ink1 }, maxw - 24.0);
        let h = galley.size().y + 20.0;
        if user {
            let bw = (galley.size().x + 24.0).min(maxw);
            let r = Rect::from_min_size(pos2(x + w - bw, y), vec2(bw, h));
            raised_accent(p, th, r, 15.0, false, false);
            p.galley(r.min + vec2(12.0, 10.0), galley, th.on_accent());
        } else {
            let ic = Rect::from_min_size(pos2(x, y + 2.0), vec2(26.0, 26.0));
            grad_fill(p, ic, 9.0, alpha(th.accent, 0.16), alpha(th.accent, 0.08));
            self.icons.paint_at(p, "stars-minimalistic", Ic::Duo, ic.center(), 14.0, th.accent_ink);
            let r = Rect::from_min_size(pos2(x + 34.0, y), vec2(w - 34.0, h));
            p.rect_filled(r, cr(12.0), alpha(th.card_top, 0.7));
            p.galley(r.min + vec2(12.0, 10.0), galley, th.ink1);
        }
        h + 8.0
    }
}

fn ui_contains(ctx: &Context, r: Rect) -> bool {
    ctx.pointer_latest_pos().is_some_and(|p| r.contains(p))
}
