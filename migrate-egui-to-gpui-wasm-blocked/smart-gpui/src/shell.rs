//! Smart Shell — wallpaper, menu bar, window manager, dock, toasts.
use crate::assets::*;
use crate::files::FilesView;
use crate::kit::KitView;
use crate::photos::PhotosView;
use crate::sig::*;
use crate::terminal::TermView;
use crate::theme::*;
use crate::ui::*;
use gpui_kit::prelude::*;
use gpui_kit::*;
use std::time::Duration;

const TOPBAR_H: f32 = 30.0;
const DOCK_H: f32 = 104.0;

struct Win {
    app: AppId,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    open: bool,
    min: bool,
    max: bool,
    z: u32,
    epoch: u32,
    prev: (f32, f32, f32, f32),
}

#[derive(Clone, Copy)]
enum Drag {
    Move { i: usize, ox: f32, oy: f32 },
    Resize { i: usize, mx: f32, my: f32, w: f32, h: f32 },
}

pub struct Shell {
    dark: bool,
    wins: Vec<Win>,
    views: Vec<AnyView>,
    drag: Option<Drag>,
    z: u32,
    toasts: Vec<(String, f64)>,
    inited: bool,
    term: Entity<TermView>,
    auto_fit: bool,
}

fn idx(app: AppId) -> usize {
    match app {
        AppId::Files => 0,
        AppId::Photos => 1,
        AppId::Terminal => 2,
        AppId::Kit => 3,
    }
}

impl Shell {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let files = cx.new(FilesView::new);
        let photos = cx.new(PhotosView::new);
        let term = cx.new(TermView::new);
        let term_handle = term.clone();
        let kit = cx.new(KitView::new);
        let mk = |app, w, h| Win { app, x: 0., y: 0., w, h, open: false, min: false, max: false, z: 0, epoch: 0, prev: (0., 0., w, h) };
        // clock tick
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(Duration::from_secs(5)).await;
                if this.update(cx, |_, cx| cx.notify()).is_err() {
                    break;
                }
            }
        })
        .detach();
        Self {
            dark: false,
            wins: vec![mk(AppId::Files, 1080., 690.), mk(AppId::Photos, 940., 620.), mk(AppId::Terminal, 780., 470.), mk(AppId::Kit, 1020., 680.)],
            views: vec![files.into(), photos.into(), term.into(), kit.into()],
            term: term_handle,
            drag: None,
            z: 1,
            toasts: vec![("Welcome to Smart Shell \u{b7} GPUI on WebAssembly".into(), now_ms())],
            inited: false,
            auto_fit: true,
        }
    }

    fn raise(&mut self, i: usize) {
        self.z += 1;
        self.wins[i].z = self.z;
    }

    fn launch(&mut self, app: AppId, vw: f32, vh: f32) {
        let i = idx(app);
        let n_open = self.wins.iter().filter(|w| w.open).count() as f32;
        let w = &mut self.wins[i];
        if !w.open {
            w.open = true;
            w.epoch += 1;
            w.w = w.w.min(vw - 40.).max(480.);
            w.h = w.h.min(vh - TOPBAR_H - DOCK_H + 30.).max(340.);
            w.x = ((vw - w.w) / 2. + n_open * 26. - 26.).max(8.);
            w.y = (TOPBAR_H + 14. + n_open * 22.).min((vh - DOCK_H - w.h).max(TOPBAR_H + 6.));
            w.max = false;
        }
        w.min = false;
        self.raise(i);
    }

    fn active_app(&self) -> Option<AppId> {
        self.wins.iter().filter(|w| w.open && !w.min).max_by_key(|w| w.z).map(|w| w.app)
    }

    fn set_dark(&mut self, dark: bool, cx: &mut App) {
        self.dark = dark;
        cx.set_global(ThemeG(if dark { Theme::dark() } else { Theme::light() }));
    }

    fn hm() -> String {
        let d = js_sys::Date::new_0();
        format!("{:02}:{:02}", d.get_hours(), d.get_minutes())
    }
    fn clock() -> String {
        let d = js_sys::Date::new_0();
        let days = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
        let months = ["January","February","March","April","May","June","July","August","September","October","November","December"];
        let (m, rest) = (months[d.get_month() as usize % 12], format!("{:02}:{:02}", d.get_hours(), d.get_minutes()));
        let _ = rest;
        format!("{} {} {}", days[d.get_day() as usize % 7], d.get_date(), m)
    }

    fn topbar(&self, t: &Theme) -> Div {
        let t = *t;
        let active = self.active_app().map(|a| a.name()).unwrap_or("Smart Shell");
        let menu = |s: &str| div().px(px(9.)).h(px(22.)).flex().items_center().rounded(px(7.)).text_size(px(12.5)).font_weight(FontWeight::MEDIUM).text_color(t.ink2).hover(move |d| d.bg(t.row_hover)).child(s.to_string());
        let sys = |ic: &str| icon(ic, IV::Lin, 16., t.ink2);
        div()
            .absolute()
            .top_0()
            .left_0()
            .right_0()
            .h(px(TOPBAR_H))
            .flex()
            .items_center()
            .px(px(12.))
            .gap(px(2.))
            .bg(if t.dark { c(8, 12, 20, 0.58) } else { c(255, 255, 255, 0.52) })
            .child(div().ml(px(6.)).mr(px(2.)).flex().items_center().justify_center().size(px(20.)).rounded(px(6.)).bg(bg_grad(t.grad_a, t.grad_b)).text_size(px(12.)).font_weight(FontWeight::BOLD).text_color(t.on_accent).child("S"))
            .child(div().px(px(8.)).text_size(px(13.)).font_weight(FontWeight::BOLD).text_color(t.ink1).child(active))
            .children(["File", "Edit", "View", "Go", "Window", "Help"].map(menu))
            .child(div().flex_1())
            .child(sys("wi-fi"))
            .child(div().w(px(10.)))
            .child(sys("bluetooth"))
            .child(div().w(px(10.)))
            .child(div().flex().items_center().gap(px(4.)).child(sys("battery-full-minimalistic")).child(div().text_size(px(12.)).font_weight(FontWeight::MEDIUM).text_color(t.ink2).child("87%")))
            .child(div().w(px(10.)))
            .child(sys("volume-small"))
            .child(div().w(px(14.)))
            .child(div().text_size(px(12.5)).font_weight(FontWeight::MEDIUM).text_color(t.ink2).child(Self::clock()))
            .child(div().w(px(8.)))
            .child(div().text_size(px(12.5)).font_weight(FontWeight::BOLD).text_color(t.ink1).child(Self::hm()))
            .child(div().w(px(6.)))
    }

    fn dock(&self, t: &Theme, cx: &mut Context<Self>) -> Div {
        let t = *t;
        enum Tile { App(AppId, Hsla, Hsla), Theme, Expand }
        let tiles = [
            Tile::App(AppId::Files, hexc(0x5b9bff), hexc(0x2457d6)),
            Tile::App(AppId::Photos, hexc(0xff8aa8), hexc(0xe8553f)),
            Tile::App(AppId::Terminal, hexc(0x4a5568), hexc(0x151b28)),
            Tile::Theme,
            Tile::Expand,
        ];
        let dot = |on: bool| div().size(px(5.)).rounded(px(999.)).bg(if on { t.ink2 } else { c(0, 0, 0, 0.) });
        let wrap = |g: &'static str, tip: &str, tile: Stateful<Div>| {
            div()
                .group(g)
                .relative()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(4.))
                .child(
                    div()
                        .absolute()
                        .top(px(-34.))
                        .px(px(10.))
                        .h(px(24.))
                        .flex()
                        .items_center()
                        .rounded(px(8.))
                        .bg(t.tooltip_bg)
                        .text_size(px(12.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(if t.dark { hexc(0x141d30) } else { hexc(0xffffff) })
                        .opacity(0.)
                        .group_hover(g, |s| s.opacity(1.))
                        .child(tip.to_string()),
                )
                .child(tile)
        };
        let mut pill = div()
            .flex()
            .items_end()
            .gap(px(14.))
            .p(px(10.))
            .rounded(px(30.))
            .bg(if t.dark { c(20, 28, 42, 0.62) } else { c(255, 255, 255, 0.56) })
            .shadow(elev(&t, 4));
        for tile in tiles {
            match tile {
                Tile::App(app, a, b) => {
                    let i = idx(app);
                    let open = self.wins[i].open;
                    let inner = if app == AppId::Terminal { div().font_family("JetBrains Mono").text_size(px(22.)).font_weight(FontWeight::BOLD).text_color(hexc(0xffffff)).child(">_") } else { div().child(icon(app.icon(), IV::Duo, 34., hexc(0xffffff))) };
                    let tile = div()
                        .id(("dock", i))
                        .flex().items_center().justify_center().size(px(62.)).rounded(px(18.))
                        .bg(bg_grad(a, b)).shadow(elev(&Theme { dark: true, ..t }, 3)).cursor_pointer()
                        .hover(|s| s.mb(px(8.)).size(px(66.)))
                        .child(inner)
                        .on_click(cx.listener(move |this, _, window, cx| {
                        let vs = window.viewport_size();
                        this.launch(app, f32::from(vs.width), f32::from(vs.height));
                        cx.notify();
                    }));
                    pill = pill.child(wrap(match app { AppId::Files => "d-files", AppId::Photos => "d-photos", _ => "d-term" }, app.name(), tile).child(dot(open)));
                }
                Tile::Theme => {
                    let g1 = if t.dark { hexc(0x3a4356) } else { hexc(0xeef2f8) };
                    let g2 = if t.dark { hexc(0x161c2a) } else { hexc(0xb9c4d6) };
                    let ic = icon("settings-minimalistic", IV::Duo, 32., if t.dark { hexc(0xdfe6f2) } else { t.ink2 });
                    let tile = div()
                        .id("dock-theme").flex().items_center().justify_center().size(px(62.)).rounded(px(18.))
                        .bg(bg_grad(g1, g2)).shadow(elev(&Theme { dark: true, ..t }, 3)).cursor_pointer()
                        .hover(|s| s.mb(px(8.)).size(px(66.))).child(ic)
                        .on_click(cx.listener(|this, _, _, cx| {
                        this.set_dark(!this.dark, cx);
                        cx.notify();
                    }));
                    pill = pill.child(wrap("d-theme", "Appearance", tile).child(dot(false)));
                }
                Tile::Expand => {
                    let ic = icon("maximize-square-minimalistic", IV::Duo, 34., hexc(0xffffff));
                    let tile = div()
                        .id("dock-max").flex().items_center().justify_center().size(px(62.)).rounded(px(18.))
                        .bg(bg_grad(hexc(0xa78bfa), hexc(0x4f46e5))).shadow(elev(&Theme { dark: true, ..t }, 3)).cursor_pointer()
                        .hover(|s| s.mb(px(8.)).size(px(66.))).child(ic)
                        .on_click(cx.listener(|this, _, window, cx| {
                        let vs = window.viewport_size();
                        let (vw, vh) = (f32::from(vs.width), f32::from(vs.height));
                        if let Some(i) = this.wins.iter().enumerate().filter(|(_, w)| w.open && !w.min).max_by_key(|(_, w)| w.z).map(|(i, _)| i) {
                            let w = &mut this.wins[i];
                            if w.max {
                                (w.x, w.y, w.w, w.h) = w.prev;
                                w.max = false;
                            } else {
                                w.prev = (w.x, w.y, w.w, w.h);
                                (w.x, w.y, w.w, w.h) = (8., TOPBAR_H + 6., vw - 16., vh - TOPBAR_H - 14.);
                                w.max = true;
                            }
                            this.raise(i);
                        }
                        cx.notify();
                    }));
                    pill = pill.child(wrap("d-max", "Fill Screen", tile).child(dot(false)));
                }
            }
        }
        div().absolute().bottom(px(10.)).left_0().right_0().flex().justify_center().child(pill)
    }

    fn window(&self, t: &Theme, i: usize, active: bool, cx: &mut Context<Self>) -> impl IntoElement {
        let t = *t;
        let w = &self.wins[i];
        let epoch = w.epoch;
        let lights = div()
            .absolute()
            .top(px(17.))
            .left(px(16.))
            .flex()
            .gap(px(8.))
            .children([(0u8, 0xff5f57u32), (1, 0xfebc2e), (2, 0x28c840)].map(|(k, col)| {
                div()
                    .id(("tl", i * 3 + k as usize))
                    .size(px(13.))
                    .rounded(px(999.))
                    .bg(if active { hexc(col) } else { alpha(t.ink4, 0.5) })
                    .shadow(vec![BoxShadow { color: c(0, 0, 0, 0.18), offset: point(px(0.), px(1.)), blur_radius: px(1.), spread_radius: px(0.), inset: false }])
                    .cursor_pointer()
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .on_click(cx.listener(move |this, _, window, cx| {
                        let vs = window.viewport_size();
                        let (vw, vh) = (f32::from(vs.width), f32::from(vs.height));
                        this.auto_fit = false;
                        let w = &mut this.wins[i];
                        match k {
                            0 => w.open = false,
                            1 => w.min = true,
                            _ => {
                                if w.max {
                                    (w.x, w.y, w.w, w.h) = w.prev;
                                    w.max = false;
                                } else {
                                    w.prev = (w.x, w.y, w.w, w.h);
                                    (w.x, w.y, w.w, w.h) = (8., TOPBAR_H + 6., vw - 16., vh - TOPBAR_H - 14.);
                                    w.max = true;
                                }
                            }
                        }
                        cx.notify();
                    }))
            }));
        div()
            .absolute()
            .left(px(w.x))
            .top(px(w.y))
            .w(px(w.w))
            .h(px(w.h))
            .rounded(px(22.))
            .overflow_hidden()
            .bg(if t.dark { bg_grad(c(19, 26, 39, 1.0), c(12, 17, 27, 1.0)) } else { bg_grad(c(247, 250, 254, 1.0), c(236, 242, 251, 1.0)) })
            .shadow(elev(&t, if active { 5 } else { 3 }))
            .capture_any_mouse_down(cx.listener(move |this, _, _, cx| {
                if this.wins[i].z != this.z {
                    this.raise(i);
                    cx.notify();
                }
            }))
            .child(self.views[i].clone())
            .child(lights)
            .child(
                div()
                    .id(("rz", i))
                    .absolute()
                    .right_0()
                    .bottom_0()
                    .size(px(22.))
                    .cursor_nwse_resize()
                    .on_mouse_down(MouseButton::Left, cx.listener(move |this, ev: &MouseDownEvent, _, cx| {
                        cx.stop_propagation();
                        let w = &this.wins[i];
                        this.auto_fit = false;
                        this.drag = Some(Drag::Resize { i, mx: f32::from(ev.position.x), my: f32::from(ev.position.y), w: w.w, h: w.h });
                    })),
            )
            .with_animation(("win", i * 1000 + epoch as usize), Animation::new(Duration::from_millis(260)).with_easing(ease_out_quint()), |el, d| el.opacity(0.2 + 0.8 * d))
    }
}

impl Render for Shell {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let vs = window.viewport_size();
        let (vw, vh) = (f32::from(vs.width), f32::from(vs.height));
        if !self.inited {
            self.inited = true;
            self.launch(AppId::Files, vw, vh);
        }
        if self.auto_fit && vw > 200. && vh > 200. {
            let w = &mut self.wins[0];
            w.w = (vw - 104.).clamp(520., 1360.);
            w.h = (vh - 56. - 118.).clamp(340., 760.);
            w.x = ((vw - w.w) / 2.).max(8.);
            w.y = 56.;
        }
        // drain the signal bus
        let sg = sig(cx);
        let (new_toasts, launches, dark) = {
            let mut s = sg.borrow_mut();
            (std::mem::take(&mut s.toasts), std::mem::take(&mut s.launch), s.dark.take())
        };
        if let Some(d) = dark {
            if d != self.dark {
                self.set_dark(d, cx);
            }
        }
        for a in launches {
            self.launch(a, vw, vh);
        }
        let now = now_ms();
        for m in new_toasts {
            self.toasts.push((m, now));
            cx.spawn(async move |this, cx| {
                cx.background_executor().timer(Duration::from_millis(3600)).await;
                let _ = this.update(cx, |_, cx| cx.notify());
            })
            .detach();
        }
        self.toasts.retain(|(_, at)| now - *at < 3500.);

        let t = theme(cx);
        let active = self.active_app();
        let mut order: Vec<usize> = (0..self.wins.len()).filter(|&i| self.wins[i].open && !self.wins[i].min).collect();
        order.sort_by_key(|&i| self.wins[i].z);

        let mut root = div()
            .id("shell")
            .key_context("Shell")
            .on_key_down(cx.listener(|this, ev: &KeyDownEvent, _, cx| {
                if this.active_app() == Some(AppId::Terminal) {
                    cx.stop_propagation();
                    this.term.update(cx, |t, cx| t.handle_key(ev, cx));
                }
            }))
            .relative()
            .size_full()
            .overflow_hidden()
            .bg(if t.dark { hexc(0x0b0f18) } else { hexc(0xe8effa) })
            .font_family("Inter")
            .text_size(px(BODY))
            .text_color(t.ink1)
            .child(img(if t.dark { "wallpapers/dark.jpg" } else { "wallpapers/light.jpg" }).absolute().inset_0().size_full().object_fit(ObjectFit::Cover))
            .on_mouse_down(MouseButton::Left, cx.listener(|this, ev: &MouseDownEvent, _, cx| {
                let req = sig(cx).borrow_mut().drag.take();
                if let Some((app, pos)) = req {
                    let i = idx(app);
                    let w = &this.wins[i];
                    if !w.max {
                        this.auto_fit = false;
                        this.drag = Some(Drag::Move { i, ox: f32::from(pos.x) - w.x, oy: f32::from(pos.y) - w.y });
                    }
                }
                let _ = ev;
            }))
            .on_mouse_move(cx.listener(|this, ev: &MouseMoveEvent, window, cx| {
                if ev.pressed_button != Some(MouseButton::Left) {
                    this.drag = None;
                    return;
                }
                let vs = window.viewport_size();
                let (vw, vh) = (f32::from(vs.width), f32::from(vs.height));
                match this.drag {
                    Some(Drag::Move { i, ox, oy }) => {
                        let w = &mut this.wins[i];
                        w.x = (f32::from(ev.position.x) - ox).clamp(-w.w + 120., vw - 120.);
                        w.y = (f32::from(ev.position.y) - oy).clamp(TOPBAR_H, vh - 60.);
                    }
                    Some(Drag::Resize { i, mx, my, w, h }) => {
                        let win = &mut this.wins[i];
                        win.w = (w + f32::from(ev.position.x) - mx).max(520.);
                        win.h = (h + f32::from(ev.position.y) - my).max(360.);
                    }
                    None => return,
                }
                cx.notify();
            }))
            .on_mouse_up(MouseButton::Left, cx.listener(|this, _, _, cx| {
                this.drag = None;
                cx.notify();
            }))
            .child(self.topbar(&t));
        for i in order {
            root = root.child(self.window(&t, i, active == Some(self.wins[i].app), cx));
        }
        root = root.child(self.dock(&t, cx));

        // toasts
        if !self.toasts.is_empty() {
            let mut col = div().absolute().bottom(px(DOCK_H + 8.)).left_0().right_0().flex().flex_col().items_center().gap(px(8.));
            for (k, (m, _)) in self.toasts.iter().enumerate() {
                col = col.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(10.))
                        .h(px(40.))
                        .pl(px(14.))
                        .pr(px(18.))
                        .rounded(px(999.))
                        .bg(t.tooltip_bg)
                        .shadow(elev(&t, 4))
                        .text_size(px(BODY))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(if t.dark { hexc(0x141d30) } else { hexc(0xf4f7fd) })
                        .child(icon("check-circle", IV::Duo, 18., if t.dark { hexc(0x2b9b6d) } else { hexc(0x59c79a) }))
                        .child(m.clone())
                        .with_animation(("toast", k * 1000 + m.len()), Animation::new(Duration::from_millis(260)).with_easing(ease_out_quint()), |el, d| el.opacity(d)),
                );
            }
            root = root.child(col);
        }
        root
    }
}
