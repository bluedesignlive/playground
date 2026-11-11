//! Smart Files — the file manager, rebuilt on GPUI to match the Smart Shell contract.
use crate::assets::*;
use crate::data::*;
use crate::sig::*;
use crate::theme::*;
use crate::ui::*;
use gpui_kit::prelude::*;
use std::cell::Cell;
use std::rc::Rc;
use gpui_kit::*;

#[derive(Clone, PartialEq)]
pub enum Route {
    Recents,
    Trash,
    Dir(Vec<String>),
}

#[derive(Clone, Copy, PartialEq)]
enum Sort {
    Name,
    Modified,
    Size,
    Kind,
}
impl Sort {
    fn label(self) -> &'static str {
        match self {
            Sort::Name => "Name",
            Sort::Modified => "Modified",
            Sort::Size => "Size",
            Sort::Kind => "Kind",
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum ViewMode {
    Grid,
    List,
    Tree,
}

#[derive(Clone, Copy)]
enum Act {
    Open,
    Look,
    Copy,
    Cut,
    Dup,
    Trash,
    Restore,
    Purge,
    NewFolder,
    NewText,
    Paste,
    SelectAll,
    EmptyTrash,
}

#[derive(Clone)]
struct Item {
    key: String,
    parent: Vec<String>,
    node: Node,
}

struct Menu {
    pos: Point<Pixels>,
    on_item: bool,
}

pub struct FilesView {
    focus: FocusHandle,
    route: Route,
    back: Vec<Route>,
    fwd: Vec<Route>,
    view: ViewMode,
    sort: Sort,
    asc: bool,
    sort_menu: bool,
    hover: Option<String>,
    anchor: Option<usize>,
    tree_open: Vec<Vec<String>>,
    content_w: Rc<Cell<f32>>,
    selected: Vec<String>,
    search: String,
    searching: bool,
    menu: Option<Menu>,
    details: bool,
    sidebar: bool,
    ask: bool,
    clip: Option<(Vec<Node>, bool, Vec<String>)>,
    look: Option<Item>,
}

fn key_of(parent: &[String], name: &str) -> String {
    format!("{}/{}", parent.join("/"), name)
}

fn tag_dot(t: &Theme, tag: u8) -> Option<Hsla> {
    match tag {
        1 => Some(t.warn),
        2 => Some(t.accent),
        3 => Some(t.good),
        _ => None,
    }
}

impl FilesView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus: cx.focus_handle(),
            route: Route::Dir(vec!["Home".into(), "Pictures".into()]),
            back: vec![],
            fwd: vec![],
            view: ViewMode::Grid,
            sort: Sort::Name,
            asc: true,
            sort_menu: false,
            hover: None,
            anchor: None,
            tree_open: vec![vec!["Home".into(), "Pictures".into()]],
            content_w: Rc::new(Cell::new(1032.)),
            selected: vec![],
            search: String::new(),
            searching: false,
            menu: None,
            details: false,
            sidebar: true,
            ask: false,
            clip: None,
            look: None,
        }
    }

    fn fs(&self, cx: &App) -> std::rc::Rc<std::cell::RefCell<Fs>> {
        cx.global::<FsG>().0.clone()
    }

    fn go(&mut self, r: Route) {
        if r == self.route {
            return;
        }
        let old = std::mem::replace(&mut self.route, r);
        self.back.push(old);
        self.fwd.clear();
        self.selected.clear();
        self.search.clear();
        self.searching = false;
        self.menu = None;
        self.ask = false;
        self.sort_menu = false;
    }

    fn items(&self, fs: &Fs) -> Vec<Item> {
        let mut out: Vec<Item> = match &self.route {
            Route::Recents => fs.recents().into_iter().map(|(p, n)| Item { key: key_of(&p, &n.name), parent: p, node: n }).collect(),
            Route::Trash => fs.trash.iter().map(|(p, n)| Item { key: key_of(p, &n.name), parent: p.clone(), node: n.clone() }).collect(),
            Route::Dir(p) => {
                let Some(dir) = fs.dir(p) else { return vec![] };
                if self.search.trim().is_empty() {
                    dir.children.iter().map(|n| Item { key: key_of(p, &n.name), parent: p.clone(), node: n.clone() }).collect()
                } else {
                    let q = self.search.to_lowercase();
                    let mut res = vec![];
                    fn walk(n: &Node, path: &mut Vec<String>, q: &str, res: &mut Vec<Item>) {
                        for c in &n.children {
                            if c.name.to_lowercase().contains(q) {
                                res.push(Item { key: key_of(path, &c.name), parent: path.clone(), node: c.clone() });
                            }
                            if c.is_dir() {
                                path.push(c.name.clone());
                                walk(c, path, q, res);
                                path.pop();
                            }
                        }
                    }
                    walk(dir, &mut p.clone(), &q, &mut res);
                    res
                }
            }
        };
        let (sort, asc) = (self.sort, self.asc);
        out.sort_by(|a, b| {
            let base = match sort {
                Sort::Name => a.node.name.to_lowercase().cmp(&b.node.name.to_lowercase()),
                Sort::Modified => a.node.hours.partial_cmp(&b.node.hours).unwrap(),
                Sort::Size => a.node.total().cmp(&b.node.total()),
                Sort::Kind => a.node.kind.label().cmp(b.node.kind.label()).then(a.node.name.cmp(&b.node.name)),
            };
            let base = if asc { base } else { base.reverse() };
            b.node.is_dir().cmp(&a.node.is_dir()).then(base)
        });
        out
    }

    fn act(&mut self, a: Act, cx: &mut Context<Self>) {
        self.menu = None;
        let fsr = self.fs(cx);
        let items = self.items(&fsr.borrow());
        let sel: Vec<Item> = items.iter().filter(|i| self.selected.contains(&i.key)).cloned().collect();
        match a {
            Act::Open => {
                if let Some(i) = sel.first() {
                    self.open(i.clone());
                }
            }
            Act::Look => {
                if let Some(i) = sel.first() {
                    self.look = Some(i.clone());
                }
            }
            Act::SelectAll => self.selected = items.iter().map(|i| i.key.clone()).collect(),
            Act::Copy | Act::Cut => {
                if let Route::Dir(p) = &self.route {
                    if !sel.is_empty() {
                        let n = sel.len();
                        self.clip = Some((sel.iter().map(|i| i.node.clone()).collect(), matches!(a, Act::Cut), p.clone()));
                        toast(cx, format!("{} {} {}", if matches!(a, Act::Cut) { "Cut" } else { "Copied" }, n, if n == 1 { "item" } else { "items" }));
                    }
                }
            }
            Act::Paste => {
                if let (Route::Dir(p), Some((nodes, cut, from))) = (self.route.clone(), self.clip.clone()) {
                    let mut fs = fsr.borrow_mut();
                    if cut {
                        if let Some(src) = fs.dir_mut(&from) {
                            src.children.retain(|c| !nodes.iter().any(|n| n.name == c.name));
                        }
                        self.clip = None;
                    }
                    let n = nodes.len();
                    for mut node in nodes {
                        node.name = fs.unique_name(&p, &node.name);
                        node.hours = 0.0;
                        if let Some(dir) = fs.dir_mut(&p) {
                            dir.children.push(node);
                        }
                    }
                    toast(cx, format!("Pasted {n} {}", if n == 1 { "item" } else { "items" }));
                }
            }
            Act::Dup => {
                if let Route::Dir(p) = self.route.clone() {
                    let mut fs = fsr.borrow_mut();
                    for i in &sel {
                        let mut node = i.node.clone();
                        node.name = fs.unique_name(&p, &node.name);
                        node.hours = 0.0;
                        if let Some(dir) = fs.dir_mut(&p) {
                            dir.children.push(node);
                        }
                    }
                    toast(cx, "Duplicated");
                }
            }
            Act::NewFolder | Act::NewText => {
                if let Route::Dir(p) = self.route.clone() {
                    let mut fs = fsr.borrow_mut();
                    let folder = matches!(a, Act::NewFolder);
                    let name = fs.unique_name(&p, if folder { "Untitled folder" } else { "Untitled.txt" });
                    let node = Node {
                        name: name.clone(),
                        kind: if folder { Kind::Folder } else { Kind::Text },
                        size: 0,
                        hours: 0.0,
                        hue: 0.6,
                        children: vec![],
                        text: if folder { None } else { Some(String::new()) },
                        tag: 0,
                    };
                    if let Some(dir) = fs.dir_mut(&p) {
                        dir.children.push(node);
                    }
                    self.selected = vec![key_of(&p, &name)];
                    toast(cx, format!("Created \u{201c}{name}\u{201d}"));
                }
            }
            Act::Trash => {
                if matches!(self.route, Route::Dir(_)) && !sel.is_empty() {
                    let mut fs = fsr.borrow_mut();
                    for i in &sel {
                        if let Some(dir) = fs.dir_mut(&i.parent) {
                            dir.children.retain(|c| c.name != i.node.name);
                        }
                        fs.trash.push((i.parent.clone(), i.node.clone()));
                    }
                    self.selected.clear();
                    toast(cx, format!("Moved {} to Trash", if sel.len() == 1 { format!("\u{201c}{}\u{201d}", sel[0].node.name) } else { format!("{} items", sel.len()) }));
                }
            }
            Act::Restore => {
                let mut fs = fsr.borrow_mut();
                for i in &sel {
                    fs.trash.retain(|(_, n)| n.name != i.node.name);
                    let parent = if fs.dir(&i.parent).is_some() { i.parent.clone() } else { vec!["Home".to_string()] };
                    if let Some(dir) = fs.dir_mut(&parent) {
                        dir.children.push(i.node.clone());
                    }
                }
                self.selected.clear();
                toast(cx, "Restored");
            }
            Act::Purge => {
                let mut fs = fsr.borrow_mut();
                for i in &sel {
                    fs.trash.retain(|(_, n)| n.name != i.node.name);
                }
                self.selected.clear();
                toast(cx, "Deleted permanently");
            }
            Act::EmptyTrash => {
                fsr.borrow_mut().trash.clear();
                self.selected.clear();
                toast(cx, "Trash emptied");
            }
        }
        cx.notify();
    }

    fn open(&mut self, i: Item) {
        if i.node.is_dir() && !matches!(self.route, Route::Trash) {
            let mut p = i.parent.clone();
            p.push(i.node.name.clone());
            self.go(Route::Dir(p));
        } else {
            self.look = Some(i);
        }
    }

    fn on_key(&mut self, ev: &KeyDownEvent, cx: &mut Context<Self>) {
        let ks = &ev.keystroke;
        let k = ks.key.as_str();
        if self.look.is_some() {
            if k == "escape" || k == "space" {
                self.look = None;
                cx.notify();
            }
            return;
        }
        if self.searching {
            if k == "escape" {
                self.search.clear();
                self.searching = false;
            } else if let Some(true) = edit_text(&mut self.search, ev) {
                self.searching = false;
            }
            cx.notify();
            return;
        }
        let cmd = ks.modifiers.platform || ks.modifiers.control;
        match k {
            "escape" => {
                self.menu = None;
                self.ask = false;
                self.sort_menu = false;
                self.selected.clear();
            }
            "a" if cmd => return self.act(Act::SelectAll, cx),
            "c" if cmd => return self.act(Act::Copy, cx),
            "x" if cmd => return self.act(Act::Cut, cx),
            "v" if cmd => return self.act(Act::Paste, cx),
            "d" if cmd => return self.act(Act::Dup, cx),
            "k" if cmd => {
                self.ask = !self.ask;
                self.sort_menu = false;
            }
            "d" => {
                sig(cx).borrow_mut().dark = Some(!theme(cx).dark);
            }
            "g" => self.view = match self.view { ViewMode::Grid => ViewMode::List, ViewMode::List => ViewMode::Tree, ViewMode::Tree => ViewMode::Grid },
            "backspace" | "delete" => return self.act(if matches!(self.route, Route::Trash) { Act::Purge } else { Act::Trash }, cx),
            "enter" => return self.act(Act::Open, cx),
            "space" => return self.act(Act::Look, cx),
            "/" => self.searching = true,
            "left" if ks.modifiers.alt || cmd => {
                if let Some(r) = self.back.pop() {
                    self.fwd.push(std::mem::replace(&mut self.route, r));
                    self.selected.clear();
                }
            }
            _ => {}
        }
        cx.notify();
    }

    // ───────────────────────────── sidebar ─────────────────────────────
    fn nav_row(&self, t: &Theme, id: &'static str, ic: &'static str, label: &str, route: Route, right: Option<(String, Hsla)>, cx: &mut Context<Self>) -> Stateful<Div> {
        let t = *t;
        let on = self.route == route;
        let mut d = div()
            .id(id)
            .flex()
            .items_center()
            .gap(px(10.))
            .h(px(30.))
            .px(px(10.))
            .rounded(px(10.))
            .cursor_pointer()
            .text_size(px(BODY))
            .font_weight(if on { FontWeight::SEMIBOLD } else { FontWeight::MEDIUM })
            .text_color(if on { t.accent_ink } else { t.ink2 })
            .when(on, |d| d.bg(t.accent_tint))
            .when(!on, |d| d.hover(move |s| s.bg(t.row_hover)))
            .child(icon(ic, if on { IV::Duo } else { IV::Lin }, 18., if on { t.accent } else { t.ink3 }))
            .child(div().flex_1().truncate().child(label.to_string()));
        if let Some((txt, col)) = right {
            d = d.child(div().text_size(px(META)).font_weight(FontWeight::MEDIUM).text_color(col).child(txt));
        }
        d.on_click(cx.listener(move |this, _, window, cx| {
            window.focus(&this.focus, cx);
            this.go(route.clone());
            cx.notify();
        }))
    }

    fn sidebar(&self, t: &Theme, fs: &Fs, cx: &mut Context<Self>) -> Div { // takes whole fs
        let t = *t;
        let cnt = |p: &[&str]| fs.dir(&p.iter().map(|s| s.to_string()).collect::<Vec<_>>()).map(|d| d.children.len()).unwrap_or(0);
        let home = |s: &str| Route::Dir(vec!["Home".into(), s.into()]);
        let c4 = t.ink4;
        let n_rec = fs.recents().len();
        div()
            .flex()
            .flex_col()
            .flex_none()
            .w(px(252.))
            .h_full()
            .bg(bg_grad(t.sidebar_top, t.sidebar_bottom))
            // drag strip + settings circle
            .child(
                div()
                    .h(px(46.))
                    .flex_none()
                    .flex()
                    .items_center()
                    .justify_end()
                    .pr(px(14.))
                    .on_mouse_down(MouseButton::Left, |ev, _, cx| start_drag(cx, AppId::Files, ev.position))
                    .child(
                        button("side-set", &t, Btn::Tonal, Some("settings-minimalistic"), None, 30.).on_click(cx.listener(|_, _, _, cx| {
                            sig(cx).borrow_mut().launch.push(AppId::Kit);
                        })),
                    ),
            )
            // workspace header
            .child(
                div()
                    .id("ws")
                    .flex()
                    .flex_none()
                    .items_center()
                    .gap(px(10.))
                    .px(px(14.))
                    .pb(px(8.))
                    .cursor_pointer()
                    .hover(move |s| s.bg(t.row_hover))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .size(px(36.))
                            .rounded(px(11.))
                            .bg(bg_grad(t.grad_a, t.grad_b))
                            .shadow(accent_glow(&t))
                            .text_size(px(15.))
                            .font_weight(FontWeight::BOLD)
                            .text_color(t.on_accent)
                            .child("N"),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .child(div().text_size(px(BODY)).font_weight(FontWeight::SEMIBOLD).text_color(t.ink1).child("Naledzi"))
                            .child(div().text_size(px(META)).text_color(t.ink3).child("Personal workspace")),
                    )
                    .child(icon("alt-arrow-down", IV::Lin, 14., t.ink4)),
            )
            .child(
                div()
                    .id("side-scroll")
                    .flex_1()
                    .min_h(px(0.))
                    .overflow_y_scroll()
                    .px(px(10.))
                    .child(section_label(&t, "Favorites"))
                    .child(self.nav_row(&t, "n-home", "home", "Home", home("Home"), Some((cnt(&["Home"]).to_string(), c4)), cx))
                    .child(self.nav_row(&t, "n-rec", "clock-circle", "Recents", Route::Recents, None, cx))
                    .child(self.nav_row(&t, "n-desk", "monitor", "Desktop", home("Desktop"), Some((cnt(&["Home", "Desktop"]).to_string(), c4)), cx))
                    .child(self.nav_row(&t, "n-docs", "document-text", "Documents", home("Documents"), Some((cnt(&["Home", "Documents"]).to_string(), c4)), cx))
                    .child(self.nav_row(&t, "n-down", "download-minimalistic", "Downloads", home("Downloads"), Some((cnt(&["Home", "Downloads"]).to_string(), c4)), cx))
                    .child(self.nav_row(&t, "n-pics", "gallery", "Pictures", home("Pictures"), Some((cnt(&["Home", "Pictures"]).to_string(), c4)), cx))
                    .child(self.nav_row(&t, "n-mus", "music-notes", "Music", home("Music"), Some((cnt(&["Home", "Music"]).to_string(), c4)), cx))
                    .child(self.nav_row(&t, "n-mov", "video-library", "Movies", home("Movies"), Some((cnt(&["Home", "Movies"]).to_string(), c4)), cx))
                    .child(section_label(&t, "Locations"))
                    .child(self.nav_row(&t, "n-drv", "ssd-square", "Smart Drive", Route::Dir(vec![]), None, cx))
                    .child(self.nav_row(&t, "n-usb", "flash-drive", "SanDisk Ultra", Route::Dir(vec!["SanDisk".into()]), Some(("41 GB free".to_string(), c4)), cx))
                    .child(self.nav_row(&t, "n-nas", "global", "Studio NAS", Route::Dir(vec!["Studio NAS".into()]), None, cx))
                    .child(self.nav_row(&t, "n-pxl", "smartphone", "Naledzi\u{2019}s Pixel", Route::Dir(vec![]), Some(("Mount".to_string(), t.accent)), cx).on_click(cx.listener(|_, _, _, cx| toast(cx, "Naledzi\u{2019}s Pixel \u{2014} pairing code 482 190"))))
                    .child(self.nav_row(&t, "n-cld", "cloud", "Smart Cloud", Route::Dir(vec![]), None, cx).on_click(cx.listener(|_, _, _, cx| toast(cx, "Smart Cloud is in sync"))))
                    .child(
                        div()
                            .id("n-mnt")
                            .flex()
                            .items_center()
                            .gap(px(10.))
                            .h(px(32.))
                            .px(px(10.))
                            .rounded(px(10.))
                            .cursor_pointer()
                            .text_size(px(BODY))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(t.ink3)
                            .hover(move |s| s.bg(t.row_hover))
                            .child(icon("add", IV::Lin, 18., t.ink4))
                            .child("Mount a device\u{2026}")
                            .on_click(cx.listener(|_, _, _, cx| toast(cx, "No new devices found"))),
                    ),
            )
            // storage card
            .child(
                div().flex_none().p(px(12.)).child(
                    div()
                        .relative()
                        .flex()
                        .flex_col()
                        .gap(px(8.))
                        .p(px(14.))
                        .rounded(px(16.))
                        .bg(bg_grad(t.card_top, t.card_bottom))
                        .shadow(elev(&t, 1))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.))
                                .child(icon("ssd-square", IV::Duo, 18., t.accent))
                                .child(div().flex_1().text_size(px(BODY)).font_weight(FontWeight::SEMIBOLD).text_color(t.ink1).child("Smart Drive"))
                                .child(div().text_size(px(META)).font_weight(FontWeight::SEMIBOLD).text_color(t.ink2).child("62%")),
                        )
                        .child(progress(&t, 0.62, 6.))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .child(div().text_size(px(META)).text_color(t.ink3).child("194 GB free of 512 GB"))
                                .child(div().flex_1())
                                .child(button("drv-set", &t, Btn::Ghost, Some("settings-minimalistic"), None, 26.).on_click(cx.listener(|_, _, _, cx| toast(cx, "Drive settings")))),
                        ),
                ),
            )
    }

    // ───────────────────────────── app bar ─────────────────────────────
    fn sidebar_glyph(&self, t: &Theme) -> Div {
        div()
            .size(px(15.))
            .rounded(px(4.))
            .border_1()
            .border_color(alpha(t.ink1, 0.55))
            .flex()
            .child(div().w(px(5.)).h_full().rounded_l(px(3.)).bg(alpha(t.ink1, 0.55)))
    }

    fn appbar(&self, t: &Theme, title_parts: Vec<(String, Route)>, cx: &mut Context<Self>) -> Div {
        let t = *t;
        let can_back = !self.back.is_empty();
        let can_fwd = !self.fwd.is_empty();
        // breadcrumbs as plain text on the bar
        let mut crumbs = div().flex().flex_1().min_w(px(0.)).items_center().gap(px(4.)).overflow_hidden();
        let n = title_parts.len();
        for (i, (label, route)) in title_parts.into_iter().enumerate() {
            let last = i + 1 == n;
            if i > 0 {
                crumbs = crumbs.child(icon("alt-arrow-right", IV::Lin, 12., t.ink4));
            }
            crumbs = crumbs.child(
                div()
                    .id(("crumb", i))
                    .flex_none()
                    .h(px(28.))
                    .flex()
                    .items_center()
                    .px(px(6.))
                    .rounded(px(8.))
                    .cursor_pointer()
                    .text_size(px(14.))
                    .font_weight(if last { FontWeight::SEMIBOLD } else { FontWeight::MEDIUM })
                    .text_color(if last { t.ink1 } else { t.ink3 })
                    .when(!last, |d| d.hover(move |s| s.bg(t.row_hover)))
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.go(route.clone());
                        cx.notify();
                    }))
                    .child(label),
            );
        }

        let nav_capsule = div()
            .flex()
            .items_center()
            .h(px(34.))
            .px(px(3.))
            .rounded(px(999.))
            .bg(bg_grad(t.control_top, t.control_bottom))
            .shadow(elev(&t, 1))
            .child(
                div()
                    .id("back")
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(px(28.))
                    .rounded(px(999.))
                    .cursor_pointer()
                    .opacity(if can_back { 1. } else { 0.35 })
                    .hover(move |s| s.bg(t.row_hover))
                    .child(icon("alt-arrow-left", IV::Lin, 15., t.ink2))
                    .on_click(cx.listener(|this, _, _, cx| {
                        if let Some(r) = this.back.pop() {
                            let cur = std::mem::replace(&mut this.route, r);
                            this.fwd.push(cur);
                            this.selected.clear();
                            this.search.clear();
                            cx.notify();
                        }
                    })),
            )
            .child(div().w(px(1.)).h(px(16.)).bg(alpha(t.ink1, 0.08)))
            .child(
                div()
                    .id("fwd")
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(px(28.))
                    .rounded(px(999.))
                    .cursor_pointer()
                    .opacity(if can_fwd { 1. } else { 0.35 })
                    .hover(move |s| s.bg(t.row_hover))
                    .child(icon("alt-arrow-right", IV::Lin, 15., t.ink2))
                    .on_click(cx.listener(|this, _, _, cx| {
                        if let Some(r) = this.fwd.pop() {
                            let cur = std::mem::replace(&mut this.route, r);
                            this.back.push(cur);
                            this.selected.clear();
                            cx.notify();
                        }
                    })),
            );

        let ask_on = self.ask;
        let mut bar = div()
            .flex()
            .flex_none()
            .items_center()
            .gap(px(10.))
            .h(px(60.))
            .pr(px(16.))
            .pl(px(if self.sidebar { 16. } else { 84. }))
            .on_mouse_down(MouseButton::Left, |ev, _, cx| start_drag(cx, AppId::Files, ev.position));
        bar = bar
            .child(
                button("sb", &t, Btn::Tonal, None, None, 34.)
                    .child(self.sidebar_glyph(&t))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.sidebar = !this.sidebar;
                        cx.notify();
                    })),
            )
            .child(nav_capsule)
            .child(crumbs)
            .child(
                button("sort", &t, Btn::Tonal, Some("sort-from-top-to-bottom"), Some(self.sort.label()), 34.).on_click(cx.listener(|this, _, _, cx| {
                    this.sort_menu = !this.sort_menu;
                    this.ask = false;
                    cx.notify();
                })),
            )
            .child(self.view_segmented(&t, cx))
            .child(
                div()
                    .id("ask")
                    .flex()
                    .flex_none()
                    .items_center()
                    .gap(px(7.))
                    .h(px(34.))
                    .px(px(14.))
                    .rounded(px(999.))
                    .cursor_pointer()
                    .text_size(px(BODY))
                    .font_weight(FontWeight::SEMIBOLD)
                    .when(ask_on, |d| d.bg(bg_grad(t.grad_a, t.grad_b)).shadow(accent_glow(&t)).text_color(t.on_accent))
                    .when(!ask_on, |d| d.bg(t.accent_tint).text_color(t.accent_ink).hover(move |s| s.bg(alpha(t.accent, 0.18))))
                    .child(icon("stars-minimalistic", if ask_on { IV::Bold } else { IV::Duo }, 17., if ask_on { t.on_accent } else { t.accent }))
                    .child("Ask Smart")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.ask = !this.ask;
                        this.sort_menu = false;
                        cx.notify();
                    })),
            )
            .child(button("term", &t, Btn::Tonal, Some("code-square"), None, 34.).on_click(cx.listener(|_, _, _, cx| {
                sig(cx).borrow_mut().launch.push(AppId::Terminal);
            })))
            .child(
                div()
                    .id("search")
                    .flex()
                    .flex_none()
                    .items_center()
                    .gap(px(8.))
                    .h(px(34.))
                    .w(px(200.))
                    .px(px(12.))
                    .rounded(px(999.))
                    .bg(t.field)
                    .shadow(if self.searching {
                        let mut s = inset_well(&t);
                        s.push(BoxShadow { color: alpha(t.accent, 0.45), offset: point(px(0.), px(0.)), blur_radius: px(0.), spread_radius: px(2.), inset: false });
                        s
                    } else {
                        inset_well(&t)
                    })
                    .cursor_text()
                    .text_size(px(BODY))
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, window, cx| {
                        cx.stop_propagation();
                        this.searching = true;
                        window.focus(&this.focus, cx);
                        cx.notify();
                    }))
                    .child(icon("magnifer", IV::Lin, 15., if self.searching { t.accent } else { t.ink4 }))
                    .child(div().flex_1().truncate().text_color(if self.search.is_empty() { t.ink4 } else { t.ink1 }).child(if self.search.is_empty() { "Search Pictures".to_string() } else { self.search.clone() }))
                    .when(self.searching && self.search.is_empty(), |d| d.child(caret(&t)))
                    .child(kbd(&t, "\u{2318}K")),
            )
            .child(
                button("info", &t, Btn::Tonal, Some("menu-dots"), None, 34.).on_click(cx.listener(|this, _, _, cx| {
                    this.details = !this.details;
                    cx.notify();
                })),
            );
        bar
    }

    // ───────────────────────────── cards ─────────────────────────────
    fn thumb(&self, t: &Theme, n: &Node, h: f32) -> Div {
        thumb(t, n, h)
    }

    fn select_click(&mut self, idx: usize, ev: &ClickEvent, order: &[String]) {
        let mods = ev.modifiers();
        let key = order[idx].clone();
        if mods.shift {
            if let Some(a) = self.anchor {
                let (lo, hi) = (a.min(idx), a.max(idx));
                self.selected = order[lo..=hi].to_vec();
            } else {
                self.selected = vec![key];
                self.anchor = Some(idx);
            }
        } else if mods.platform || mods.control {
            if self.selected.contains(&key) {
                self.selected.retain(|k| *k != key);
            } else {
                self.selected.push(key);
            }
            self.anchor = Some(idx);
        } else {
            self.selected = vec![key];
            self.anchor = Some(idx);
        }
    }

    fn tree_glyph(t: &Theme, col: Hsla) -> Div {
        div()
            .relative()
            .size(px(14.))
            .flex_none()
            .child(div().absolute().left(px(1.)).top(px(1.)).bottom(px(1.)).w(px(1.5)).rounded(px(1.)).bg(col))
            .child(div().absolute().left(px(5.)).top(px(1.)).w(px(8.)).h(px(1.5)).rounded(px(1.)).bg(col))
            .child(div().absolute().left(px(5.)).top(px(6.)).w(px(8.)).h(px(1.5)).rounded(px(1.)).bg(col))
            .child(div().absolute().left(px(5.)).top(px(11.)).w(px(8.)).h(px(1.5)).rounded(px(1.)).bg(col))
    }

    fn view_segmented(&self, t: &Theme, cx: &mut Context<Self>) -> Div {
        let t = *t;
        let tile = |id: &'static str, on: bool, child: Div, h: Click| {
            let mut d = div()
                .id(id)
                .flex()
                .items_center()
                .justify_center()
                .h(px(30.))
                .w(px(36.))
                .rounded(px(999.))
                .cursor_pointer()
                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                .on_click(move |e, w, cx| h(e, w, cx));
            if on {
                d = d.bg(bg_grad(t.card_hover_top, if t.dark { t.control_hover_bottom } else { t.control_hover_top })).shadow(elev(&t, 2));
            } else {
                d = d.hover(move |s| s.bg(t.row_hover));
            }
            d.child(child)
        };
        let sel = self.view;
        div()
            .flex()
            .flex_none()
            .items_center()
            .gap(px(2.))
            .p(px(3.))
            .h(px(36.))
            .rounded(px(999.))
            .bg(t.well)
            .shadow(inset_well(&t))
            .child(tile("v-grid", sel == ViewMode::Grid, div().child(icon("widget", if sel == ViewMode::Grid { IV::Duo } else { IV::Lin }, 16., if sel == ViewMode::Grid { t.accent } else { t.ink3 })), Box::new(cx.listener(|this, _, _, cx| { this.view = ViewMode::Grid; cx.notify(); }))))
            .child(tile("v-list", sel == ViewMode::List, div().child(icon("list-check", if sel == ViewMode::List { IV::Duo } else { IV::Lin }, 16., if sel == ViewMode::List { t.accent } else { t.ink3 })), Box::new(cx.listener(|this, _, _, cx| { this.view = ViewMode::List; cx.notify(); }))))
            .child(tile("v-tree", sel == ViewMode::Tree, Self::tree_glyph(&t, if sel == ViewMode::Tree { t.accent } else { t.ink3 }), Box::new(cx.listener(|this, _, _, cx| { this.view = ViewMode::Tree; cx.notify(); }))))
    }

    fn folder_card(&self, t: &Theme, idx: usize, it: &Item, cw: f32, order: &[String], cx: &mut Context<Self>) -> Stateful<Div> {
        let t = *t;
        let on = self.selected.contains(&it.key);
        let hov = self.hover.as_deref() == Some(it.key.as_str());
        let n = &it.node;
        let meta = format!("{} item{} \u{b7} {}", n.children.len(), if n.children.len() == 1 { "" } else { "s" }, fmt_size(n.total()));
        let (k1, k2, it2, idx2) = (it.key.clone(), it.key.clone(), it.clone(), idx);
        let ord = order.to_vec();
        let dot = tag_dot(&t, n.tag);
        div()
            .id(("fcard", idx))
            .relative()
            .flex()
            .flex_none()
            .items_center()
            .gap(px(12.))
            .w(px(cw))
            .h(px(70.))
            .p(px(12.))
            .rounded(px(16.))
            .cursor_pointer()
            .bg(bg_grad(if on { alpha(t.accent, if t.dark { 0.20 } else { 0.12 }) } else { mix(t.card_top, t.card_hover_top, if hov { 1. } else { 0. }) }, t.card_bottom))
            .shadow(elev(&t, if hov || on { 3 } else { 2 }))
            .when(on, |d| d.shadow(vec![BoxShadow { color: alpha(t.accent, 0.6), offset: point(px(0.), px(0.)), blur_radius: px(0.), spread_radius: px(1.5), inset: false }]))
            .on_hover(cx.listener(move |this, &h: &bool, _, cx| { this.hover = h.then(|| k1.clone()); cx.notify(); }))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(px(46.))
                    .rounded(px(13.))
                    .bg(t.accent_tint)
                    .shadow(vec![BoxShadow { color: alpha(t.rim, if t.dark { 0.14 } else { 0.9 }), offset: point(px(0.), px(1.)), blur_radius: px(0.), spread_radius: px(0.), inset: true }])
                    .child(icon(if hov { "folder-open" } else { "folder" }, IV::Duo, 24., t.accent)),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w(px(0.))
                    .gap(px(2.))
                    .child(div().truncate().text_size(px(13.5)).font_weight(FontWeight::SEMIBOLD).text_color(t.ink1).child(n.name.clone()))
                    .child(div().truncate().text_size(px(META)).text_color(t.ink3).child(meta)),
            )
            .when_some(dot, |d, c| d.child(div().size(px(7.)).rounded(px(999.)).bg(c).flex_none()))
            .on_mouse_down(MouseButton::Right, cx.listener(move |this, ev: &MouseDownEvent, window, cx| {
                cx.stop_propagation();
                window.focus(&this.focus, cx);
                if !this.selected.contains(&k2) { this.selected = vec![k2.clone()]; }
                this.menu = Some(Menu { pos: ev.position, on_item: true });
                cx.notify();
            }))
            .on_click(cx.listener(move |this, ev: &ClickEvent, window, cx| {
                window.focus(&this.focus, cx);
                this.searching = false; this.menu = None; this.ask = false; this.sort_menu = false;
                if ev.click_count() >= 2 { this.open(it2.clone()); } else { this.select_click(idx2, ev, &ord); }
                cx.notify();
            }))
    }

    fn image_card(&self, t: &Theme, idx: usize, it: &Item, cw: f32, mh: f32, order: &[String], cx: &mut Context<Self>) -> Stateful<Div> {
        let t = *t;
        let on = self.selected.contains(&it.key);
        let hov = self.hover.as_deref() == Some(it.key.as_str());
        let n = &it.node;
        let meta = format!("{} \u{b7} {}", fmt_size(n.size), fmt_when(n.hours));
        let (k1, k2, k3, it2, idx2) = (it.key.clone(), it.key.clone(), it.key.clone(), it.clone(), idx);
        let ord = order.to_vec();
        let dot = tag_dot(&t, n.tag);
        let previewable = matches!(n.kind, Kind::Image | Kind::Video | Kind::Doc | Kind::Text | Kind::Code);
        let it3 = it2.clone();
        div()
            .id(("icard", idx))
            .relative()
            .flex()
            .flex_col()
            .flex_none()
            .gap(px(8.))
            .w(px(cw))
            .p(px(0.))
            .rounded(px(16.))
            .cursor_pointer()
            .on_hover(cx.listener(move |this, &h: &bool, _, cx| { this.hover = h.then(|| k1.clone()); cx.notify(); }))
            .child(
                div()
                    .relative()
                    .w_full()
                    .h(px(mh))
                    .rounded(px(14.))
                    .overflow_hidden()
                    .child(thumb(&t, n, mh).rounded(px(14.)))
                    .when(on, |d| d.child(div().absolute().inset_0().bg(alpha(t.accent, if t.dark { 0.16 } else { 0.10 }))))
                    .when(on, |d| d.shadow(vec![BoxShadow { color: alpha(t.accent, 0.7), offset: point(px(0.), px(0.)), blur_radius: px(0.), spread_radius: px(2.), inset: false }]))
                    .when(!on && hov, |d| d.shadow(elev(&t, 3)))
                    .when(hov && previewable, |d| {
                        d.child(
                            div()
                                .id(("qa-eye", idx2))
                                .absolute()
                                .top(px(8.))
                                .right(px(44.))
                                .flex().items_center().justify_center()
                                .size(px(30.)).rounded(px(999.))
                                .bg(bg_grad(t.card_hover_top, t.control_hover_bottom))
                                .shadow(elev(&t, 2)).cursor_pointer()
                                .child(icon("eye", IV::Duo, 15., t.ink2))
                                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                                .on_click(cx.listener(move |this, _, _, cx| { cx.stop_propagation(); this.look = Some(it3.clone()); cx.notify(); })),
                        )
                    })
                    .when(hov, |d| {
                        d.child(
                            div()
                                .id(("qa-dot", idx2))
                                .absolute()
                                .top(px(8.))
                                .right(px(8.))
                                .flex().items_center().justify_center()
                                .size(px(30.)).rounded(px(999.))
                                .bg(bg_grad(t.card_hover_top, t.control_hover_bottom))
                                .shadow(elev(&t, 2)).cursor_pointer()
                                .child(icon("menu-dots", IV::Duo, 15., t.ink2))
                                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                                .on_click(cx.listener(move |this, ev: &ClickEvent, _, cx| {
                                    cx.stop_propagation();
                                    this.selected = vec![k3.clone()];
                                    this.menu = Some(Menu { pos: ev.position(), on_item: true });
                                    cx.notify();
                                })),
                        )
                    }),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .px(px(4.))
                    .child(div().flex_1().truncate().text_size(px(BODY)).font_weight(FontWeight::MEDIUM).text_color(t.ink1).child(n.name.clone()))
                    .when_some(dot, |d, c| d.child(div().size(px(7.)).rounded(px(999.)).bg(c).flex_none())),
            )
            .child(div().px(px(4.)).text_size(px(META)).text_color(t.ink3).child(meta))
            .on_mouse_down(MouseButton::Right, cx.listener(move |this, ev: &MouseDownEvent, window, cx| {
                cx.stop_propagation();
                window.focus(&this.focus, cx);
                if !this.selected.contains(&k2) { this.selected = vec![k2.clone()]; }
                this.menu = Some(Menu { pos: ev.position, on_item: true });
                cx.notify();
            }))
            .on_click(cx.listener(move |this, ev: &ClickEvent, window, cx| {
                window.focus(&this.focus, cx);
                this.searching = false; this.menu = None; this.ask = false; this.sort_menu = false;
                if ev.click_count() >= 2 { this.open(it2.clone()); } else { this.select_click(idx2, ev, &ord); }
                cx.notify();
            }))
    }

    fn list_row(&self, t: &Theme, idx: usize, it: &Item, order: &[String], cx: &mut Context<Self>) -> Stateful<Div> {
        let t = *t;
        let on = self.selected.contains(&it.key);
        let n = &it.node;
        let (k1, k2, it2, idx2) = (it.key.clone(), it.key.clone(), it.clone(), idx);
        let ord = order.to_vec();
        let ic_col = if n.is_dir() { t.accent } else { t.ink3 };
        div()
            .id(("row", idx))
            .flex()
            .flex_none()
            .items_center()
            .gap(px(12.))
            .h(px(40.))
            .px(px(12.))
            .rounded(px(11.))
            .cursor_pointer()
            .text_size(px(BODY))
            .text_color(t.ink2)
            .when(on, |d| d.bg(t.accent_tint))
            .when(!on, |d| d.hover(move |s| s.bg(t.row_hover)))
            .child(icon(n.kind.icon(), IV::Duo, 22., ic_col))
            .child(div().flex_1().truncate().font_weight(FontWeight::SEMIBOLD).text_color(t.ink1).child(n.name.clone()))
            .child(div().w(px(150.)).truncate().text_color(t.ink3).child(fmt_when(n.hours)))
            .child(div().w(px(90.)).text_color(t.ink3).child(if n.is_dir() { format!("{} items", n.children.len()) } else { fmt_size(n.size) }))
            .child(div().w(px(100.)).truncate().text_color(t.ink3).child(n.kind.label()))
            .on_mouse_down(MouseButton::Right, cx.listener(move |this, ev: &MouseDownEvent, window, cx| {
                cx.stop_propagation();
                window.focus(&this.focus, cx);
                if !this.selected.contains(&k1) { this.selected = vec![k1.clone()]; }
                this.menu = Some(Menu { pos: ev.position, on_item: true });
                cx.notify();
            }))
            .on_click(cx.listener(move |this, ev: &ClickEvent, window, cx| {
                window.focus(&this.focus, cx);
                this.searching = false; this.menu = None;
                if ev.click_count() >= 2 { this.open(it2.clone()); } else { this.select_click(idx2, ev, &ord); }
                cx.notify();
            }))
    }

    fn section_label(&self, t: &Theme, name: &str, count: usize) -> Div {
        div()
            .flex()
            .items_baseline()
            .gap(px(8.))
            .pt(px(18.))
            .pb(px(10.))
            .child(div().text_size(px(12.5)).font_weight(FontWeight::SEMIBOLD).text_color(t.ink2).child(name.to_string()))
            .child(div().text_size(px(META)).text_color(t.ink4).child(count.to_string()))
    }

    fn details_panel(&self, t: &Theme, items: &[Item]) -> Div {
        let t = *t;
        let sel: Vec<&Item> = items.iter().filter(|i| self.selected.contains(&i.key)).collect();
        let mut p = div().flex().flex_col().flex_none().w(px(272.)).h_full().p(px(16.)).gap(px(14.)).bg(bg_grad(alpha(t.panel_top, 0.55), alpha(t.panel_bottom, 0.35)));
        let row = |k: &str, v: String| {
            div()
                .flex()
                .justify_between()
                .gap(px(10.))
                .text_size(px(12.5))
                .child(div().text_color(t.ink3).child(k.to_string()))
                .child(div().text_color(t.ink1).font_weight(FontWeight::MEDIUM).truncate().child(v))
        };
        if sel.len() == 1 {
            let it = sel[0];
            let n = &it.node;
            p = p
                .child(self.thumb(&t, n, 170.))
                .child(div().text_size(px(TITLE)).font_weight(FontWeight::BOLD).text_color(t.ink1).child(n.name.clone()))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(9.))
                        .child(row("Kind", n.kind.label().to_string()))
                        .child(row("Size", if n.is_dir() { fmt_size(n.total()) } else { fmt_size(n.size) }))
                        .child(row("Modified", fmt_when(n.hours)))
                        .child(row("Where", if it.parent.is_empty() { "Computer".into() } else { it.parent.join(" \u{203a} ") })),
                )
                .child(div().flex().gap(px(6.)).child(chip(&t, n.kind.label(), t.accent)).child(chip(&t, "On this device", t.good)));
        } else if sel.len() > 1 {
            let total: u64 = sel.iter().map(|i| i.node.total()).sum();
            p = p
                .child(div().flex().items_center().justify_center().h(px(120.)).rounded(px(16.)).bg(t.accent_tint2).child(icon("layers-minimalistic", IV::Duo, 54., t.accent)))
                .child(div().text_size(px(TITLE)).font_weight(FontWeight::BOLD).text_color(t.ink1).child(format!("{} items selected", sel.len())))
                .child(row("Total size", fmt_size(total)));
        } else {
            let total: u64 = items.iter().map(|i| i.node.total()).sum();
            p = p
                .child(div().flex().items_center().justify_center().h(px(120.)).rounded(px(16.)).bg(t.accent_tint2).child(icon("folder-open", IV::Duo, 54., t.accent)))
                .child(div().text_size(px(TITLE)).font_weight(FontWeight::BOLD).text_color(t.ink1).child("This folder"))
                .child(row("Items", items.len().to_string()))
                .child(row("Total size", fmt_size(total)))
                .child(div().text_size(px(META)).text_color(t.ink4).child("Select a file to see its details."));
        }
        p
    }

    fn ctx_menu(&self, t: &Theme, m: &Menu, cx: &mut Context<Self>) -> impl IntoElement {
        let t = *t;
        let mut panel = menu_panel(&t, 252.);
        let in_trash = matches!(self.route, Route::Trash);
        let it = |id: &'static str, ic: &str, label: &str, sc: Option<&str>, danger: bool, a: Act, cx: &mut Context<Self>| menu_item(id, &t, ic, label, sc, danger).on_click(cx.listener(move |this, _, _, cx| this.act(a, cx)));
        if m.on_item {
            if in_trash {
                panel = panel
                    .child(it("m-restore", "undo-left", "Put Back", None, false, Act::Restore, cx))
                    .child(menu_sep(&t))
                    .child(it("m-purge", "trash-bin-trash", "Delete Permanently", None, true, Act::Purge, cx));
            } else {
                panel = panel
                    .child(it("m-open", "folder-open", "Open", Some("\u{21b5}"), false, Act::Open, cx))
                    .child(it("m-look", "eye", "Quick Look", Some("Space"), false, Act::Look, cx))
                    .child(menu_sep(&t))
                    .child(it("m-copy", "copy", "Copy", Some("\u{2318}C"), false, Act::Copy, cx))
                    .child(it("m-cut", "scissors", "Cut", Some("\u{2318}X"), false, Act::Cut, cx))
                    .child(it("m-dup", "layers-minimalistic", "Duplicate", Some("\u{2318}D"), false, Act::Dup, cx))
                    .child(menu_sep(&t))
                    .child(it("m-trash", "trash-bin-minimalistic", "Move to Trash", Some("\u{232b}"), true, Act::Trash, cx));
            }
        } else if in_trash {
            panel = panel.child(it("m-empty", "trash-bin-trash", "Empty Trash", None, true, Act::EmptyTrash, cx));
        } else {
            panel = panel
                .child(it("m-nf", "add-folder", "New Folder", None, false, Act::NewFolder, cx))
                .child(it("m-nt", "document-text", "New Text File", None, false, Act::NewText, cx))
                .child(menu_sep(&t))
                .child(it("m-paste", "clipboard", "Paste", Some("\u{2318}V"), false, Act::Paste, cx))
                .child(it("m-all", "checklist-minimalistic", "Select All", Some("\u{2318}A"), false, Act::SelectAll, cx));
        }
        deferred(
            anchored().position(m.pos).snap_to_window_with_margin(px(10.)).child(
                panel.on_mouse_down_out(cx.listener(|this, _, _, cx| {
                    this.menu = None;
                    cx.notify();
                })),
            ),
        )
        .with_priority(10)
    }

    fn quick_look(&self, t: &Theme, it: &Item, cx: &mut Context<Self>) -> Div {
        let t = *t;
        let n = &it.node;
        let body = if let Some(text) = &n.text {
            div()
                .flex_1()
                .p(px(18.))
                .rounded(px(14.))
                .bg(t.well)
                .font_family("JetBrains Mono")
                .text_size(px(12.5))
                .text_color(t.ink2)
                .line_height(px(20.))
                .overflow_hidden()
                .child(if text.is_empty() { "(empty)".to_string() } else { text.clone() })
        } else {
            div().flex_1().child(thumb(&t, n, 280.).h_full())
        };
        div()
            .absolute()
            .inset_0()
            .flex()
            .items_center()
            .justify_center()
            .bg(t.scrim)
            .occlude()
            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                this.look = None;
                cx.notify();
            }))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(14.))
                    .w(px(520.))
                    .h(px(400.))
                    .p(px(18.))
                    .rounded(px(20.))
                    .bg(bg_grad(t.panel_top, t.panel_bottom))
                    .shadow(elev(&t, 5))
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .with_animation("ql", Animation::new(std::time::Duration::from_millis(200)).with_easing(ease_out_quint()), |el, d| el.opacity(d))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(10.))
                            .child(icon(n.kind.icon(), IV::Duo, 22., t.accent))
                            .child(div().flex_1().truncate().text_size(px(TITLE)).font_weight(FontWeight::BOLD).text_color(t.ink1).child(n.name.clone()))
                            .child(chip(&t, &fmt_size(n.size), t.ink3))
                            .child(button("ql-x", &t, Btn::Tonal, Some("close-circle"), None, 32.).on_click(cx.listener(|this, _, _, cx| {
                                this.look = None;
                                cx.notify();
                            }))),
                    )
                    .child(body),
            )
    }
}

impl Render for FilesView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = theme(cx);
        let fsr = self.fs(cx);
        let fsb = fsr.borrow();
        let items = self.items(&fsb);
        let trash_n = fsb.trash.len();
        self.selected.retain(|k| items.iter().any(|i| &i.key == k));

        let crumbs: Vec<(String, Route)> = match &self.route {
            Route::Recents => vec![("Recents".into(), Route::Recents)],
            Route::Trash => vec![("Trash".into(), Route::Trash)],
            Route::Dir(p) => {
                if p.is_empty() {
                    vec![("Smart".to_string(), Route::Dir(vec![]))]
                } else {
                    let mut v = vec![("Smart".to_string(), Route::Dir(vec![]))];
                    for i in 0..p.len() {
                        v.push((p[i].clone(), Route::Dir(p[..=i].to_vec())));
                    }
                    v
                }
            }
        };

        // header meta
        let n_folders = items.iter().filter(|i| i.node.is_dir()).count();
        let n_files = items.len() - n_folders;
        let total: u64 = items.iter().map(|i| i.node.total()).sum();
        let title = match &self.route {
            Route::Recents => "Recents".to_string(),
            Route::Trash => "Trash".to_string(),
            Route::Dir(p) => p.last().cloned().unwrap_or_else(|| "Smart Drive".to_string()),
        };
        let meta = format!("{} items \u{b7} {} \u{b7} {} folders", items.len(), fmt_size(total), n_folders);
        // kind-grouped sections (mirrors the original: Folders, Images, Audio, Video, Documents, Files)
        let groups: Vec<(&'static str, Vec<usize>, bool)> = {
            let mut g: Vec<(&'static str, Vec<usize>, bool)> = vec![];
            let label = |k: Kind| -> &'static str {
                match k {
                    Kind::Image => "Images",
                    Kind::Audio => "Audio",
                    Kind::Video => "Video",
                    Kind::Doc | Kind::Text | Kind::Code => "Documents",
                    _ => "Files",
                }
            };
            let mut folders = vec![];
            for (i, it) in items.iter().enumerate() {
                if it.node.is_dir() {
                    folders.push(i);
                } else {
                    let l = label(it.node.kind);
                    if let Some(e) = g.iter_mut().find(|(n, _, _)| *n == l) {
                        e.1.push(i);
                    } else {
                        g.push((l, vec![i], false));
                    }
                }
            }
            if !folders.is_empty() {
                g.insert(0, ("Folders", folders, true));
            }
            g
        };

        // responsive, width-filling grid (4 / 3 / 2 columns like the contract)
        let vp = window.viewport_size();
        let avail = f32::from(vp.width) - 104. - (if self.sidebar { 252. } else { 0. }) - 52.;
        let cols = if avail > 900. { 4. } else if avail > 620. { 3. } else { 2. };
        let cw = ((avail - 16. * (cols - 1.)) / cols).floor();
        let mh = (cw * 10. / 16.).max(120.);
        let order: Vec<String> = items.iter().map(|i| i.key.clone()).collect();

        let mut content = div().id("files-scroll").relative().flex_1().min_h(px(0.)).overflow_y_scroll().px(px(26.)).pb(px(20.));
        content = content.child(
            div()
                .flex()
                .flex_col()
                .gap(px(2.))
                .pt(px(20.))
                .pb(px(4.))
                .child(div().text_size(px(26.)).font_weight(FontWeight::BOLD).text_color(t.ink1).child(title))
                .child(div().text_size(px(META)).text_color(t.ink3).child(meta)),
        );
        if items.is_empty() {
            let msg = if !self.search.is_empty() { format!("No results for \u{201c}{}\u{201d}", self.search) } else if matches!(self.route, Route::Trash) { "Trash is empty".to_string() } else { "This folder is empty".to_string() };
            content = content.child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap(px(10.))
                    .h(px(300.))
                    .child(icon("folder-open", IV::Duo, 64., alpha(t.accent, 0.8)))
                    .child(div().text_size(px(TITLE)).font_weight(FontWeight::SEMIBOLD).text_color(t.ink2).child(msg))
                    .child(div().text_size(px(META)).text_color(t.ink4).child("Right-click to create something new.")),
            );
        } else if self.view == ViewMode::Grid {
            for (name, idxs, _folders) in &groups {
                let mut row = div().flex().flex_wrap().gap(px(16.));
                for &i in idxs {
                    let it = &items[i];
                    row = row.child(if it.node.is_dir() {
                        self.folder_card(&t, i, it, cw, &order, cx)
                    } else {
                        self.image_card(&t, i, it, cw, mh, &order, cx)
                    });
                }
                content = content.child(self.section_label(&t, name, idxs.len())).child(row);
            }
        } else if self.view == ViewMode::List {
            let head = |label: &'static str, w: f32, s: Sort, cx: &mut Context<Self>| {
                let active = self.sort == s;
                let mut h = div()
                    .id(label)
                    .flex()
                    .items_center()
                    .gap(px(4.))
                    .when(w > 0., |d| d.w(px(w)).flex_none())
                    .h_full()
                    .cursor_pointer()
                    .text_size(px(LABEL))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(if active { t.ink2 } else { t.ink4 })
                    .hover(move |d| d.text_color(t.ink2))
                    .child(label)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        if this.sort == s {
                            this.asc = !this.asc;
                        } else {
                            this.sort = s;
                            this.asc = true;
                        }
                        cx.notify();
                    }));
                if active {
                    h = h.child(icon(if self.asc { "alt-arrow-up" } else { "alt-arrow-down" }, IV::Lin, 11., t.ink3));
                }
                h
            };
            let mut l = div().flex().flex_col().gap(px(2.)).pt(px(12.)).child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(12.))
                    .h(px(30.))
                    .px(px(12.))
                    .rounded(px(9.))
                    .bg(t.well)
                    .child(div().w(px(22.)))
                    .child(div().flex_1().min_w(px(0.)).child(head("NAME", 0., Sort::Name, cx)))
                    .child(head("MODIFIED", 150., Sort::Modified, cx))
                    .child(head("SIZE", 90., Sort::Size, cx))
                    .child(head("KIND", 100., Sort::Kind, cx)),
            );
            for (i, it) in items.iter().enumerate() {
                l = l.child(self.list_row(&t, i, it, &order, cx));
            }
            content = content.child(l);
        } else {
            // ── tree view ──
            let base: Vec<String> = match &self.route {
                Route::Dir(p) => p.clone(),
                _ => vec![],
            };
            let mut rows: Vec<(Vec<String>, Node, usize, bool)> = vec![];
            if let Some(dir) = fsb.dir(&base) {
                fn walk(dir: &Node, path: Vec<String>, depth: usize, open: &Vec<Vec<String>>, out: &mut Vec<(Vec<String>, Node, usize, bool)>) {
                    let mut ch: Vec<&Node> = dir.children.iter().collect();
                    ch.sort_by(|a, b| b.is_dir().cmp(&a.is_dir()).then(a.name.to_lowercase().cmp(&b.name.to_lowercase())));
                    for c in ch {
                        let mut p = path.clone();
                        p.push(c.name.clone());
                        let is_open = c.is_dir() && open.contains(&p);
                        out.push((p.clone(), c.clone(), depth, is_open));
                        if is_open {
                            walk(c, p, depth + 1, open, out);
                        }
                    }
                }
                walk(dir, base.clone(), 0, &self.tree_open, &mut rows);
            }
            let mut tree = div().flex().flex_col().gap(px(1.)).pt(px(14.));
            for (path, node, depth, is_open) in &rows {
                let key = key_of(&path[..path.len() - 1], &node.name);
                let on = self.selected.contains(&key);
                let (pk, node2, pathc) = (key.clone(), node.clone(), path.clone());
                let pkc = pk.clone();
                let ord = order.clone();
                let idx = order.iter().position(|k| k == &key).unwrap_or(0);
                let mut row = div()
                    .id(("tr", idx))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .h(px(32.))
                    .pr(px(12.))
                    .rounded(px(9.))
                    .cursor_pointer()
                    .pl(px(10. + 18. * *depth as f32))
                    .text_size(px(BODY))
                    .font_weight(if node.is_dir() { FontWeight::SEMIBOLD } else { FontWeight::MEDIUM })
                    .text_color(if on { t.accent_ink } else if node.is_dir() { t.ink1 } else { t.ink2 })
                    .when(on, |d| d.bg(t.accent_tint))
                    .when(!on, |d| d.hover(move |s| s.bg(t.row_hover)));
                if node.is_dir() {
                    row = row
                        .child(icon(if *is_open { "alt-arrow-down" } else { "alt-arrow-right" }, IV::Lin, 12., t.ink4))
                        .child(icon("folder", if *is_open { IV::Duo } else { IV::Lin }, 18., t.accent));
                } else {
                    row = row.child(div().w(px(12.))).child(icon(node.kind.icon(), IV::Lin, 17., t.ink3));
                }
                row = row.child(div().flex_1().truncate().child(node.name.clone()));
                if node.is_dir() {
                    row = row.child(div().text_size(px(META)).text_color(t.ink4).child(format!("{} items", node.children.len())));
                } else {
                    row = row.child(div().text_size(px(META)).text_color(t.ink4).child(fmt_size(node.size)));
                }
                row = row
                    .on_click(cx.listener(move |this, ev: &ClickEvent, window, cx| {
                        window.focus(&this.focus, cx);
                        this.menu = None;
                        let mods = ev.modifiers();
                        if node2.is_dir() && !mods.platform && !mods.control && !mods.shift && ev.click_count() < 2 {
                            if this.tree_open.contains(&pathc) {
                                this.tree_open.retain(|p| p != &pathc);
                            } else {
                                this.tree_open.push(pathc.clone());
                            }
                        }
                        if ev.click_count() >= 2 && node2.is_dir() {
                            this.open(Item { key: pkc.clone(), parent: pathc[..pathc.len() - 1].to_vec(), node: node2.clone() });
                        } else {
                            this.select_click(idx, ev, &ord);
                        }
                        cx.notify();
                    }))
                    .on_mouse_down(MouseButton::Right, cx.listener(move |this, ev: &MouseDownEvent, window, cx| {
                        cx.stop_propagation();
                        window.focus(&this.focus, cx);
                        this.selected = vec![pk.clone()];
                        this.menu = Some(Menu { pos: ev.position, on_item: true });
                        cx.notify();
                    }));
                tree = tree.child(row);
            }
            content = content.child(tree);
        }
        content = content
            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, window, cx| {
                window.focus(&this.focus, cx);
                this.selected.clear();
                this.menu = None;
                this.ask = false;
                this.sort_menu = false;
                this.searching = false;
                cx.notify();
            }))
            .on_mouse_down(MouseButton::Right, cx.listener(|this, ev: &MouseDownEvent, window, cx| {
                window.focus(&this.focus, cx);
                this.selected.clear();
                this.menu = Some(Menu { pos: ev.position, on_item: false });
                cx.notify();
            }));

        // status bar with kbd hints
        let sel_n = self.selected.len();
        let sel_size: u64 = items.iter().filter(|i| self.selected.contains(&i.key)).map(|i| i.node.total()).sum();
        let left = if sel_n > 0 { format!("{} of {} selected \u{b7} {}", sel_n, items.len(), fmt_size(sel_size)) } else { format!("{} items \u{b7} {}", items.len(), fmt_size(total)) };
        let hint = |k: &str, l: &str| div().flex().items_center().gap(px(5.)).child(kbd(&t, k)).child(div().text_size(px(META)).text_color(t.ink4).child(l.to_string()));
        let mut footer = div()
            .flex()
            .flex_none()
            .items_center()
            .gap(px(10.))
            .h(px(34.))
            .px(px(20.))
            .text_size(px(META))
            .text_color(t.ink3)
            .font_weight(FontWeight::MEDIUM)
            .child(div().flex_1().child(left));
        if let Some((nodes, cut, _)) = &self.clip {
            footer = footer.child(chip(&t, &format!("{} {}", nodes.len(), if *cut { "cut" } else { "copied" }), t.warn));
        }
        if matches!(self.route, Route::Trash) && trash_n > 0 {
            footer = footer.child(button("empty", &t, Btn::Danger, Some("trash-bin-trash"), Some("Empty Trash"), 26.).on_click(cx.listener(|this, _, _, cx| this.act(Act::EmptyTrash, cx))));
        }
        footer = footer
            .child(div().size(px(7.)).rounded(px(999.)).bg(t.good))
            .child("Smart Drive \u{b7} 194 GB free")
            .child(div().w(px(14.)))
            .child(hint("D", "theme"))
            .child(div().w(px(6.)))
            .child(hint("G", "grid"))
            .child(div().w(px(6.)))
            .child(hint("\u{2318}K", "commands"));

        let mut main = div()
            .relative()
            .flex()
            .flex_col()
            .flex_1()
            .min_w(px(0.))
            .h_full()
            .bg(bg_grad(t.content_top, alpha(t.content_top, 0.0)))
            .child(self.appbar(&t, crumbs, cx))
            .child(div().flex().flex_1().min_h(px(0.)).child(content).when(self.details, |d| d.child(self.details_panel(&t, &items))))
            .child(footer);

        if self.sort_menu {
            let mk = |id: &'static str, label: &'static str, s: Sort, cx: &mut Context<Self>, cur: Sort| {
                let on = cur == s;
                menu_item(id, &t, if on { "check-circle" } else { "sort-vertical" }, label, None, false).on_click(cx.listener(move |this, _, _, cx| {
                    this.sort = s;
                    this.sort_menu = false;
                    cx.notify();
                }))
            };
            main = main.child(
                div().absolute().top(px(58.)).right(px(268.)).child(
                    menu_panel(&t, 200.)
                        .child(mk("s-name", "Name", Sort::Name, cx, self.sort))
                        .child(mk("s-mod", "Modified", Sort::Modified, cx, self.sort))
                        .child(mk("s-size", "Size", Sort::Size, cx, self.sort))
                        .child(mk("s-kind", "Kind", Sort::Kind, cx, self.sort)),
                ),
            );
        }
        if self.ask {
            let sug = |id: &'static str, ic: &'static str, label: &'static str, hintt: &'static str, cx: &mut Context<Self>, f: fn(&mut FilesView, &mut Context<FilesView>)| {
                div()
                    .id(id)
                    .flex()
                    .items_center()
                    .gap(px(12.))
                    .p(px(10.))
                    .rounded(px(12.))
                    .cursor_pointer()
                    .hover(move |s| s.bg(t.accent_tint))
                    .child(div().flex().items_center().justify_center().size(px(34.)).rounded(px(10.)).bg(t.accent_tint).child(icon(ic, IV::Duo, 18., t.accent)))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .child(div().text_size(px(BODY)).font_weight(FontWeight::SEMIBOLD).text_color(t.ink1).child(label))
                            .child(div().text_size(px(META)).text_color(t.ink3).child(hintt)),
                    )
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.ask = false;
                        f(this, cx);
                        cx.notify();
                    }))
            };
            main = main.child(
                div().absolute().top(px(58.)).right(px(16.)).child(
                    div()
                        .flex()
                        .flex_col()
                        .w(px(320.))
                        .p(px(8.))
                        .gap(px(2.))
                        .rounded(px(20.))
                        .bg(bg_grad(t.panel_top, t.panel_bottom))
                        .shadow(elev(&t, 4))
                        .occlude()
                        .child(div().px(px(10.)).pt(px(6.)).pb(px(4.)).text_size(px(META)).font_weight(FontWeight::SEMIBOLD).text_color(t.ink4).child("ON-DEVICE \u{b7} PRIVATE"))
                        .child(sug("a1", "gallery", "Find images", "Filter this folder to pictures only", cx, |this, _| {
                            this.search = ".".into();
                        }))
                        .child(sug("a2", "database", "Largest files first", "Sort this folder by size", cx, |this, cx| {
                            this.sort = Sort::Size;
                            toast(cx, "Sorted by size");
                        }))
                        .child(sug("a3", "clock-circle", "What changed recently?", "Open Recents", cx, |this, _| this.go(Route::Recents)))
                        .child(sug("a4", "trash-bin-minimalistic", "Review Trash", "See what you can delete forever", cx, |this, _| this.go(Route::Trash))),
                ),
            );
        }
        if let Some(l) = self.look.clone() {
            main = main.child(self.quick_look(&t, &l, cx));
        }

        let mut root = div()
            .id("files-root")
            .track_focus(&self.focus)
            .key_context("Files")
            .on_key_down(cx.listener(|this, ev: &KeyDownEvent, _, cx| this.on_key(ev, cx)))
            .flex()
            .size_full()
            .text_color(t.ink1);
        if self.sidebar {
            root = root.child(self.sidebar(&t, &fsb, cx));
        }
        root = root.child(main);
        if let Some(m) = &self.menu {
            root = root.child(self.ctx_menu(&t, m, cx));
        }
        root
    }
}

// ───────────────────────── thumbnails ─────────────────────────
pub fn thumb(t: &Theme, n: &Node, h: f32) -> Div {
    let t = *t;
    let base = div().relative().flex().items_center().justify_center().w_full().h(px(h)).rounded(px(13.)).overflow_hidden().flex_none();
    if let Some(p) = thumb_path(&n.name) {
        return base.child(img(p).size_full().object_fit(ObjectFit::Cover));
    }
    let hue = n.hue;
    match n.kind {
        Kind::Folder => base.bg(t.accent_tint).child(icon("folder", IV::Duo, h * 0.55, t.accent)),
        Kind::Image => base
            .bg(bg_grad(hsla(hue, 0.62, if t.dark { 0.55 } else { 0.74 }, 1.0), hsla((hue + 0.09).fract(), 0.58, if t.dark { 0.38 } else { 0.56 }, 1.0)))
            .child(div().absolute().top(px(-h * 0.3)).left(px(-h * 0.2)).size(px(h * 0.9)).rounded(px(999.)).bg(c(255, 255, 255, 0.18)))
            .child(icon("gallery", IV::Duo, h * 0.36, c(255, 255, 255, 0.85))),
        Kind::Audio => {
            let mut bars = div().flex().items_center().gap(px(3.)).h(px(h * 0.5));
            for i in 0..16 {
                let v = (((i * 37 + (hue * 100.0) as i32 * 7) % 9) + 3) as f32;
                bars = bars.child(div().w(px(3.)).h(px(v * h * 0.04)).rounded(px(2.)).bg(c(255, 255, 255, 0.85)));
            }
            base.bg(bg_grad(hsla(hue, 0.55, 0.52, 1.0), hsla((hue + 0.12).fract(), 0.60, 0.36, 1.0))).child(bars)
        }
        Kind::Video => base
            .bg(bg_grad(hexc(0x24304d), hexc(0x0e1524)))
            .child(div().flex().items_center().justify_center().size(px(h * 0.4)).rounded(px(999.)).bg(c(255, 255, 255, 0.18)).child(icon("play", IV::Bold, h * 0.2, c(255, 255, 255, 0.95)))),
        Kind::Doc | Kind::Text | Kind::Code => {
            let mut sheet = div()
                .flex()
                .flex_col()
                .gap(px(h * 0.045))
                .w(px(h * 0.62))
                .h(px(h * 0.8))
                .p(px(h * 0.07))
                .rounded(px(8.))
                .bg(if t.dark { hexc(0xe8eef9) } else { hexc(0xffffff) })
                .shadow(elev(&Theme { dark: false, ..t }, 2));
            for i in 0..5 {
                sheet = sheet.child(div().h(px(h * 0.03)).w(relative(if i % 3 == 2 { 0.55 } else { 0.95 })).rounded(px(2.)).bg(c(40, 60, 100, 0.20)));
            }
            base.bg(t.well).child(sheet)
        }
        _ => base.bg(t.well).child(icon(n.kind.icon(), IV::Duo, h * 0.46, t.ink3)),
    }
}
