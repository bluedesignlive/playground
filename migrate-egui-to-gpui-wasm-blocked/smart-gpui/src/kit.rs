//! Smart Kit — live gallery of the Smart design contract, every control is real and interactive.
use crate::assets::*;
use crate::sig::*;
use crate::theme::*;
use crate::ui::*;
use gpui_kit::prelude::*;
use gpui_kit::*;
use std::cell::Cell;
use std::rc::Rc;

const PAGES: [(&str, &str); 7] = [
    ("Overview", "stars-minimalistic"),
    ("Buttons", "cursor"),
    ("Controls", "tuning-2"),
    ("Surfaces", "layers-minimalistic"),
    ("Color", "palette"),
    ("Typography", "text"),
    ("Feedback", "bell"),
];

pub struct KitView {
    page: usize,
    sw: [bool; 3],
    chk: [bool; 3],
    radio: usize,
    seg: usize,
    chip: usize,
    vol: f32,
    bright: f32,
    b1: Rc<Cell<Bounds<Pixels>>>,
    b2: Rc<Cell<Bounds<Pixels>>>,
    text: String,
    text_focus: bool,
    focus: FocusHandle,
    dialog: bool,
}

impl KitView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            page: 0,
            sw: [true, false, true],
            chk: [true, false, true],
            radio: 1,
            seg: 0,
            chip: 0,
            vol: 0.62,
            bright: 0.4,
            b1: Rc::new(Cell::new(Bounds::default())),
            b2: Rc::new(Cell::new(Bounds::default())),
            text: String::new(),
            text_focus: false,
            focus: cx.focus_handle(),
            dialog: false,
        }
    }
}

fn panel(t: &Theme, title: &str, sub: &str, body: Div) -> Div {
    card(t)
        .flex()
        .flex_col()
        .gap(px(14.))
        .p(px(18.))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(2.))
                .child(div().text_size(px(TITLE)).font_weight(FontWeight::BOLD).text_color(t.ink1).child(title.to_string()))
                .child(div().text_size(px(META)).text_color(t.ink3).child(sub.to_string())),
        )
        .child(body)
}

fn labeled(t: &Theme, label: &str, ctl: impl IntoElement) -> Div {
    div()
        .flex()
        .items_center()
        .gap(px(12.))
        .h(px(32.))
        .child(div().flex_1().text_size(px(BODY)).font_weight(FontWeight::MEDIUM).text_color(t.ink2).child(label.to_string()))
        .child(ctl)
}

impl KitView {
    fn page_overview(&self, t: &Theme) -> Div {
        let t = *t;
        let stat = |n: &str, l: &str, ic: &str| {
            card(&t)
                .flex()
                .items_center()
                .gap(px(12.))
                .p(px(16.))
                .flex_1()
                .child(div().flex().items_center().justify_center().size(px(42.)).rounded(px(13.)).bg(t.accent_tint).child(icon(ic, IV::Duo, 22., t.accent)))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(div().text_size(px(20.)).font_weight(FontWeight::BOLD).text_color(t.ink1).child(n.to_string()))
                        .child(div().text_size(px(META)).text_color(t.ink3).child(l.to_string())),
                )
        };
        div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(10.))
                    .p(px(26.))
                    .rounded(px(20.))
                    .bg(linear_gradient(135., linear_color_stop(t.grad_a, 0.), linear_color_stop(mix(t.grad_b, hexc(0x7a5cf0), 0.55), 1.)))
                    .shadow(accent_glow(&t))
                    .child(div().text_size(px(LABEL)).font_weight(FontWeight::SEMIBOLD).text_color(c(255, 255, 255, 0.75)).child("SMART DESIGN CONTRACT v0.1"))
                    .child(div().text_size(px(30.)).font_weight(FontWeight::BOLD).text_color(hexc(0xffffff)).child("One contract, every host."))
                    .child(div().max_w(px(520.)).text_size(px(14.)).line_height(px(21.)).text_color(c(255, 255, 255, 0.86)).child("Every pixel on this page is drawn by GPUI \u{2014} Rust compiled to WebAssembly, GPU-rendered on a canvas. No JavaScript, no TypeScript, no DOM widgets."))
                    .child(div().flex().gap(px(8.)).pt(px(6.)).child(chip(&Theme { dark: true, ..t }, "GPUI", hexc(0xffffff))).child(chip(&Theme { dark: true, ..t }, "gpui-kit 0.9", hexc(0xffffff))).child(chip(&Theme { dark: true, ..t }, "wasm32", hexc(0xffffff)))),
            )
            .child(div().flex().gap(px(14.)).child(stat("5", "Elevation levels", "layers-minimalistic")).child(stat("2", "Themes: light & dark", "moon-stars")).child(stat("0", "Lines of JavaScript", "code-square")))
            .child(panel(
                &t,
                "Depth through tone",
                "Surfaces step up in tone and shadow \u{2014} never in strokes.",
                div().flex().gap(px(14.)).children((1..=5u8).map(|l| {
                    div()
                        .flex()
                        .flex_1()
                        .items_center()
                        .justify_center()
                        .h(px(74.))
                        .rounded(px(16.))
                        .bg(bg_grad(t.card_top, t.card_bottom))
                        .shadow(elev(&t, l))
                        .text_size(px(BODY))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(t.ink2)
                        .child(format!("e{l}"))
                })),
            ))
    }

    fn page_buttons(&self, t: &Theme, cx: &mut Context<Self>) -> Div {
        let t = *t;
        let t1 = |cx: &mut Context<Self>, id: &'static str, k: Btn, ic: Option<&str>, l: Option<&str>, msg: &'static str| {
            button(id, &t, k, ic, l, 36.).on_click(cx.listener(move |_, _, _, cx| {
                toast(cx, msg);
            }))
        };
        div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .child(panel(
                &t,
                "Buttons",
                "Pill-shaped, tonal-elevated. Accent is the single call to action.",
                div()
                    .flex()
                    .flex_col()
                    .gap(px(14.))
                    .child(
                        div()
                            .flex()
                            .gap(px(12.))
                            .items_center()
                            .child(t1(cx, "b1", Btn::Accent, Some("add"), Some("New project"), "New project created"))
                            .child(t1(cx, "b2", Btn::Tonal, Some("share"), Some("Share"), "Link copied"))
                            .child(t1(cx, "b3", Btn::Danger, Some("trash-bin-minimalistic"), Some("Delete"), "Deleted"))
                            .child(t1(cx, "b4", Btn::Ghost, None, Some("Cancel"), "Cancelled")),
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(12.))
                            .items_center()
                            .child(t1(cx, "i1", Btn::Tonal, Some("heart"), None, "Added to favourites"))
                            .child(t1(cx, "i2", Btn::Tonal, Some("bookmark"), None, "Bookmarked"))
                            .child(t1(cx, "i3", Btn::Accent, Some("play"), None, "Playing"))
                            .child(t1(cx, "i4", Btn::Ghost, Some("menu-dots"), None, "More\u{2026}"))
                            .child(button("sm", &t, Btn::Tonal, None, Some("Small"), 28.))
                            .child(button("lg", &t, Btn::Accent, Some("bolt"), Some("Large action"), 44.)),
                    ),
            ))
            .child(panel(
                &t,
                "Segmented control",
                "Recessed well, raised selection.",
                div().flex().gap(px(16.)).items_center().child(segmented(
                    "kseg",
                    &t,
                    self.seg,
                    vec![
                        (None, Some("Day"), Box::new(cx.listener(|this, _, _, cx| { this.seg = 0; cx.notify(); }))),
                        (None, Some("Week"), Box::new(cx.listener(|this, _, _, cx| { this.seg = 1; cx.notify(); }))),
                        (None, Some("Month"), Box::new(cx.listener(|this, _, _, cx| { this.seg = 2; cx.notify(); }))),
                    ],
                ))
                .child(segmented(
                    "kseg2",
                    &t,
                    self.chip,
                    vec![
                        (Some("widget"), None, Box::new(cx.listener(|this, _, _, cx| { this.chip = 0; cx.notify(); }))),
                        (Some("list-check"), None, Box::new(cx.listener(|this, _, _, cx| { this.chip = 1; cx.notify(); }))),
                        (Some("gallery-wide"), None, Box::new(cx.listener(|this, _, _, cx| { this.chip = 2; cx.notify(); }))),
                    ],
                )),
            ))
    }

    fn page_controls(&self, t: &Theme, cx: &mut Context<Self>) -> Div {
        let t = *t;
        let e = cx.entity();
        let e2 = e.clone();
        let sw = |i: usize, label: &str, cx: &mut Context<Self>| {
            labeled(&t, label, switch(("sw", i), &t, self.sw[i]).on_click(cx.listener(move |this, _, _, cx| {
                this.sw[i] = !this.sw[i];
                cx.notify();
            })))
        };
        let ck = |i: usize, label: &str, cx: &mut Context<Self>| {
            labeled(&t, label, checkbox(("ck", i), &t, self.chk[i]).on_click(cx.listener(move |this, _, _, cx| {
                this.chk[i] = !this.chk[i];
                cx.notify();
            })))
        };
        let rd = |i: usize, label: &str, cx: &mut Context<Self>| {
            labeled(&t, label, radio(("rd", i), &t, self.radio == i).on_click(cx.listener(move |this, _, _, cx| {
                this.radio = i;
                cx.notify();
            })))
        };
        div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .child(
                div()
                    .flex()
                    .gap(px(16.))
                    .child(div().flex_1().child(panel(&t, "Switches", "Spring-less, instant, 44\u{d7}26.", div().flex().flex_col().child(sw(0, "Wi-Fi", cx)).child(sw(1, "Bluetooth", cx)).child(sw(2, "Dark appearance hints", cx)))))
                    .child(div().flex_1().child(panel(&t, "Checkboxes", "Multi-select.", div().flex().flex_col().child(ck(0, "Photos", cx)).child(ck(1, "Documents", cx)).child(ck(2, "Music", cx)))))
                    .child(div().flex_1().child(panel(&t, "Radios", "Single choice.", div().flex().flex_col().child(rd(0, "Compact", cx)).child(rd(1, "Comfortable", cx)).child(rd(2, "Spacious", cx))))),
            )
            .child(panel(
                &t,
                "Sliders & progress",
                "Drag the thumbs. Track bounds are measured live with a canvas.",
                div()
                    .flex()
                    .flex_col()
                    .gap(px(10.))
                    .child(labeled(&t, &format!("Volume  {}%", (self.vol * 100.0) as u32), div().w(px(360.)).child(slider("vol", &t, self.vol, self.b1.clone(), Rc::new(move |v, _, cx| { e.update(cx, |this, cx| { this.vol = v; cx.notify(); }); })))))
                    .child(labeled(&t, &format!("Brightness  {}%", (self.bright * 100.0) as u32), div().w(px(360.)).child(slider("bri", &t, self.bright, self.b2.clone(), Rc::new(move |v, _, cx| { e2.update(cx, |this, cx| { this.bright = v; cx.notify(); }); })))))
                    .child(labeled(&t, "Upload", div().w(px(360.)).child(progress(&t, self.vol, 8.))))
                    .child(labeled(&t, "Indexing", spinner(&t, "kit-spin"))),
            ))
            .child(panel(
                &t,
                "Text field",
                "Click and type \u{2014} keyboard input is handled in Rust.",
                div().flex().items_center().gap(px(12.)).child(
                    div().id("kit-field").cursor_text().on_mouse_down(MouseButton::Left, cx.listener(|this, _, window, cx| {
                        this.text_focus = true;
                        window.focus(&this.focus, cx);
                        cx.notify();
                    })).child(search_field(&t, "pen", &self.text, "Type something\u{2026}", self.text_focus, 320.)),
                )
                .child(chip(&t, &format!("{} chars", self.text.chars().count()), t.accent)),
            ))
    }

    fn page_surfaces(&self, t: &Theme) -> Div {
        let t = *t;
        let sw = |id: &'static str, ic: &'static str, l: &'static str, d: bool| menu_item(id, &t, ic, l, None, d);
        div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .child(panel(
                &t,
                "Elevation ladder",
                "e1 \u{2192} e5. Tonal fill, layered drop shadow, inset top reflection.",
                div().flex().gap(px(18.)).pb(px(18.)).children((1..=5u8).map(|l| {
                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .items_center()
                        .justify_center()
                        .gap(px(4.))
                        .h(px(96.))
                        .rounded(px(16.))
                        .bg(bg_grad(t.panel_top, t.panel_bottom))
                        .shadow(elev(&t, l))
                        .child(div().text_size(px(20.)).font_weight(FontWeight::BOLD).text_color(t.ink1).child(format!("e{l}")))
                        .child(div().text_size(px(META)).text_color(t.ink3).child(["resting", "raised", "popover", "menu", "modal"][l as usize - 1]))
                })),
            ))
            .child(
                div()
                    .flex()
                    .gap(px(16.))
                    .child(div().flex_1().child(panel(&t, "Context menu", "252 wide, 18 radius, 33 rows.", div().pb(px(6.)).child(menu_panel(&t, 252.).child(sw("sm1", "folder-open", "Open", false)).child(sw("sm2", "copy", "Copy", false)).child(sw("sm3", "share", "Share\u{2026}", false)).child(menu_sep(&t)).child(sw("sm4", "trash-bin-minimalistic", "Move to Trash", true))))))
                    .child(div().flex_1().child(panel(
                        &t,
                        "Cards & chips",
                        "Tone-on-tone status colour.",
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(12.))
                            .child(div().flex().gap(px(8.)).child(chip(&t, "Synced", t.good)).child(chip(&t, "Pending", t.warn)).child(chip(&t, "Failed", t.danger)).child(chip(&t, "Brand", t.accent)))
                            .child(div().flex().gap(px(6.)).child(kbd(&t, "\u{2318}")).child(kbd(&t, "K")).child(div().text_size(px(META)).text_color(t.ink3).child("Command palette")))
                            .child(
                                card(&t).flex().items_center().gap(px(12.)).p(px(14.)).child(icon("cloud-storage", IV::Duo, 28., t.accent)).child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .flex_1()
                                        .child(div().text_size(px(BODY)).font_weight(FontWeight::SEMIBOLD).text_color(t.ink1).child("Studio NAS"))
                                        .child(div().text_size(px(META)).text_color(t.ink3).child("6.0 TB free of 8.0 TB")),
                                ),
                            ),
                    ))),
            )
    }

    fn page_color(&self, t: &Theme) -> Div {
        let t = *t;
        let sw = |name: &str, hex: &str, col: Hsla| {
            div()
                .flex()
                .flex_col()
                .gap(px(6.))
                .w(px(112.))
                .child(div().h(px(54.)).rounded(px(13.)).bg(col).shadow(elev(&t, 1)))
                .child(div().text_size(px(12.)).font_weight(FontWeight::SEMIBOLD).text_color(t.ink1).child(name.to_string()))
                .child(div().text_size(px(LABEL)).text_color(t.ink4).font_family("JetBrains Mono").child(hex.to_string()))
        };
        let mode = if t.dark { "dark" } else { "light" };
        div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .child(panel(&t, "Ink", &format!("Four text tiers \u{b7} {mode}"), div().flex().gap(px(14.)).flex_wrap().child(sw("ink 1", if t.dark { "#eef3fb" } else { "#141d30" }, t.ink1)).child(sw("ink 2", if t.dark { "#c6cfdf" } else { "#3f4b60" }, t.ink2)).child(sw("ink 3", if t.dark { "#96a1b5" } else { "#667286" }, t.ink3)).child(sw("ink 4", if t.dark { "#707b90" } else { "#8b95a6" }, t.ink4))))
            .child(panel(&t, "Accent & status", "Cobalt is the only accent.", div().flex().gap(px(14.)).flex_wrap().child(sw("accent", if t.dark { "#6f9bff" } else { "#2f6feb" }, t.accent)).child(sw("accent hover", if t.dark { "#86acff" } else { "#3d7bf7" }, t.accent_hover)).child(sw("accent ink", if t.dark { "#a7c1ff" } else { "#1c56c4" }, t.accent_ink)).child(sw("good", if t.dark { "#59c79a" } else { "#2b9b6d" }, t.good)).child(sw("warn", if t.dark { "#e5b05a" } else { "#c9861a" }, t.warn)).child(sw("danger", if t.dark { "#ff7a84" } else { "#c53b48" }, t.danger))))
            .child(panel(&t, "Gradient", "Accent fill used for primary actions.", div().h(px(54.)).rounded(px(16.)).bg(bg_grad(t.grad_a, t.grad_b)).shadow(accent_glow(&t))))
    }

    fn page_type(&self, t: &Theme) -> Div {
        let t = *t;
        let row = |size: f32, w: FontWeight, name: &str, sample: &str| {
            div()
                .flex()
                .items_baseline()
                .gap(px(16.))
                .child(div().w(px(110.)).text_size(px(META)).text_color(t.ink4).child(name.to_string()))
                .child(div().text_size(px(size)).font_weight(w).text_color(t.ink1).child(sample.to_string()))
        };
        div().flex().flex_col().gap(px(16.)).child(panel(
            &t,
            "Inter \u{b7} type ramp",
            "Label 10.5 \u{b7} Meta 11.5 \u{b7} Body 13 \u{b7} Title 15 \u{b7} Page 26",
            div()
                .flex()
                .flex_col()
                .gap(px(16.))
                .child(row(26., FontWeight::BOLD, "Page 26 / 700", "Quiet tools, loud craft"))
                .child(row(15., FontWeight::SEMIBOLD, "Title 15 / 600", "Studio Master Reel 2026.mov"))
                .child(row(13., FontWeight::MEDIUM, "Body 13 / 500", "The quick brown fox jumps over the lazy dog."))
                .child(row(11.5, FontWeight::NORMAL, "Meta 11.5 / 400", "842 MB \u{b7} Yesterday \u{b7} On this device"))
                .child(row(10.5, FontWeight::SEMIBOLD, "Label 10.5 / 600", "FAVORITES \u{b7} LOCATIONS \u{b7} TAGS"))
                .child(div().flex().items_baseline().gap(px(16.)).child(div().w(px(110.)).text_size(px(META)).text_color(t.ink4).child("Mono / JetBrains")).child(div().font_family("JetBrains Mono").text_size(px(13.)).text_color(t.accent_ink).child("fn main() { println!(\"smart\"); }"))),
        ))
    }

    fn page_feedback(&self, t: &Theme, cx: &mut Context<Self>) -> Div {
        let t = *t;
        div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .child(panel(
                &t,
                "Toasts",
                "Click to raise a toast from any view.",
                div()
                    .flex()
                    .gap(px(10.))
                    .child(button("t1", &t, Btn::Tonal, Some("check-circle"), Some("Success"), 36.).on_click(cx.listener(|_, _, _, cx| toast(cx, "Saved to Documents"))))
                    .child(button("t2", &t, Btn::Tonal, Some("info-circle"), Some("Info"), 36.).on_click(cx.listener(|_, _, _, cx| toast(cx, "Sync paused \u{2014} reconnecting"))))
                    .child(button("t3", &t, Btn::Tonal, Some("bell"), Some("Notify"), 36.).on_click(cx.listener(|_, _, _, cx| toast(cx, "3 new files from Phone")))),
            ))
            .child(panel(
                &t,
                "Dialog",
                "Scrim + e5 panel, destructive action in danger tone.",
                div().child(button("dlg", &t, Btn::Accent, None, Some("Open dialog"), 36.).on_click(cx.listener(|this, _, _, cx| {
                    this.dialog = true;
                    cx.notify();
                }))),
            ))
    }
}

impl Render for KitView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = theme(cx);
        let mut nav = div()
            .flex()
            .flex_col()
            .flex_none()
            .w(px(220.))
            .h_full()
            .px(px(10.))
            .bg(bg_grad(t.sidebar_top, t.sidebar_bottom))
            .child(div().h(px(52.)).flex_none().on_mouse_down(MouseButton::Left, |ev, _, cx| start_drag(cx, AppId::Kit, ev.position)))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(10.))
                    .px(px(10.))
                    .pb(px(8.))
                    .child(div().flex().items_center().justify_center().size(px(30.)).rounded(px(10.)).bg(bg_grad(t.grad_a, t.grad_b)).shadow(accent_glow(&t)).child(icon("widget", IV::Bold, 16., t.on_accent)))
                    .child(div().text_size(px(TITLE)).font_weight(FontWeight::BOLD).text_color(t.ink1).child("Smart Kit")),
            );
        for (i, (name, ic)) in PAGES.iter().enumerate() {
            let on = self.page == i;
            nav = nav.child(
                div()
                    .id(("kp", i))
                    .flex()
                    .items_center()
                    .gap(px(10.))
                    .h(px(34.))
                    .px(px(10.))
                    .rounded(px(10.))
                    .cursor_pointer()
                    .text_size(px(BODY))
                    .font_weight(if on { FontWeight::SEMIBOLD } else { FontWeight::MEDIUM })
                    .text_color(if on { t.accent_ink } else { t.ink2 })
                    .when(on, |d| d.bg(t.accent_tint))
                    .when(!on, |d| d.hover(move |s| s.bg(t.row_hover)))
                    .child(icon(ic, if on { IV::Duo } else { IV::Lin }, 18., if on { t.accent } else { t.ink3 }))
                    .child(name.to_string())
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.page = i;
                        cx.notify();
                    })),
            );
        }
        nav = nav.child(div().flex_1()).child(
            div().pb(px(14.)).child(segmented(
                "appearance",
                &t,
                if t.dark { 1 } else { 0 },
                vec![
                    (Some("sun-2"), Some("Light"), Box::new(|_, _, cx| sig(cx).borrow_mut().dark = Some(false))),
                    (Some("moon-stars"), Some("Dark"), Box::new(|_, _, cx| sig(cx).borrow_mut().dark = Some(true))),
                ],
            )),
        );

        let page = match self.page {
            0 => self.page_overview(&t),
            1 => self.page_buttons(&t, cx),
            2 => self.page_controls(&t, cx),
            3 => self.page_surfaces(&t),
            4 => self.page_color(&t),
            5 => self.page_type(&t),
            _ => self.page_feedback(&t, cx),
        };
        let title = PAGES[self.page].0;
        let mut main = div()
            .relative()
            .flex()
            .flex_col()
            .flex_1()
            .min_w(px(0.))
            .h_full()
            .bg(bg_grad(t.content_top, alpha(t.content_top, 0.)))
            .child(
                div()
                    .flex()
                    .flex_none()
                    .items_center()
                    .h(px(60.))
                    .px(px(26.))
                    .on_mouse_down(MouseButton::Left, |ev, _, cx| start_drag(cx, AppId::Kit, ev.position))
                    .child(div().text_size(px(26.)).font_weight(FontWeight::BOLD).text_color(t.ink1).child(title)),
            )
            .child(div().id("kit-scroll").flex_1().min_h(px(0.)).overflow_y_scroll().px(px(26.)).pb(px(26.)).child(page));
        if self.dialog {
            main = main.child(
                div()
                    .absolute()
                    .inset_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(t.scrim)
                    .occlude()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(14.))
                            .w(px(380.))
                            .p(px(22.))
                            .rounded(px(20.))
                            .bg(bg_grad(t.panel_top, t.panel_bottom))
                            .shadow(elev(&t, 5))
                            .with_animation("dlg-in", Animation::new(std::time::Duration::from_millis(220)).with_easing(ease_out_quint()), |el, d| el.opacity(d))
                            .child(div().flex().items_center().gap(px(12.)).child(div().flex().items_center().justify_center().size(px(40.)).rounded(px(12.)).bg(t.danger_tint).child(icon("trash-bin-trash", IV::Duo, 22., t.danger))).child(div().text_size(px(TITLE)).font_weight(FontWeight::BOLD).text_color(t.ink1).child("Delete 12 items?")))
                            .child(div().text_size(px(BODY)).line_height(px(19.)).text_color(t.ink3).child("They will be removed from this device. This can\u{2019}t be undone."))
                            .child(
                                div()
                                    .flex()
                                    .justify_end()
                                    .gap(px(10.))
                                    .child(button("d-no", &t, Btn::Tonal, None, Some("Cancel"), 36.).on_click(cx.listener(|this, _, _, cx| {
                                        this.dialog = false;
                                        cx.notify();
                                    })))
                                    .child(button("d-yes", &t, Btn::Danger, None, Some("Delete"), 36.).on_click(cx.listener(|this, _, _, cx| {
                                        this.dialog = false;
                                        toast(cx, "12 items deleted");
                                        cx.notify();
                                    }))),
                            ),
                    ),
            );
        }
        div()
            .id("kit-root")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|this, ev: &KeyDownEvent, _, cx| {
                if this.text_focus {
                    if ev.keystroke.key == "escape" {
                        this.text_focus = false;
                    } else {
                        edit_text(&mut this.text, ev);
                    }
                    cx.notify();
                }
            }))
            .flex()
            .size_full()
            .child(nav)
            .child(main)
    }
}
