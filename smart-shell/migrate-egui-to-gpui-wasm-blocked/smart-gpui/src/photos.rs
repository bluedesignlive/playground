//! Smart Photos — library grid + viewer with favourites and a live exposure slider.
use crate::assets::*;
use crate::sig::*;
use crate::theme::*;
use crate::ui::*;
use gpui_kit::prelude::*;
use gpui_kit::*;
use std::cell::Cell;
use std::rc::Rc;

#[derive(Clone)]
struct Photo {
    name: &'static str,
    wall: Option<&'static str>,
    h1: f32,
    h2: f32,
    fav: bool,
}

pub struct PhotosView {
    photos: Vec<Photo>,
    only_fav: bool,
    open: Option<usize>,
    expo: f32,
    bounds: Rc<Cell<Bounds<Pixels>>>,
}

impl PhotosView {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        let p = |name, wall, h1, h2, fav| Photo { name, wall, h1, h2, fav };
        Self {
            photos: vec![
                p("Frosted Glass", Some("wallpapers/light.jpg"), 0.6, 0.6, true),
                p("Midnight Silk", Some("wallpapers/dark.jpg"), 0.65, 0.7, false),
                p("Blossom Study", None, 0.93, 0.02, false),
                p("Green Fold", None, 0.38, 0.5, true),
                p("Orbit Poster", None, 0.72, 0.85, false),
                p("Window Study", None, 0.55, 0.62, false),
                p("Soweto Ride", None, 0.08, 0.13, true),
                p("Layout Study", None, 0.48, 0.58, false),
                p("Reference 01", None, 0.78, 0.9, false),
                p("Package B", None, 0.12, 0.95, false),
                p("Landing Draft", None, 0.28, 0.4, false),
                p("Brand Board", None, 0.62, 0.75, true),
            ],
            only_fav: false,
            open: None,
            expo: 0.5,
            bounds: Rc::new(Cell::new(Bounds::default())),
        }
    }
}

fn visual(t: &Theme, p: &Photo, rounded: f32) -> Div {
    let base = div().relative().size_full().rounded(px(rounded)).overflow_hidden();
    match p.wall {
        Some(w) => base.child(img(w).size_full().object_fit(ObjectFit::Cover)),
        None => base
            .bg(linear_gradient(145., linear_color_stop(hsla(p.h1, 0.62, if t.dark { 0.55 } else { 0.72 }, 1.), 0.), linear_color_stop(hsla(p.h2, 0.6, if t.dark { 0.36 } else { 0.5 }, 1.), 1.)))
            .child(div().absolute().top(px(-30.)).right(px(-20.)).size(px(150.)).rounded(px(999.)).bg(c(255, 255, 255, 0.16)))
            .child(div().absolute().bottom(px(-50.)).left(px(-30.)).size(px(190.)).rounded(px(999.)).bg(c(0, 0, 0, 0.10))),
    }
}

impl Render for PhotosView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = theme(cx);
        let visible: Vec<(usize, Photo)> = self.photos.iter().cloned().enumerate().filter(|(_, p)| !self.only_fav || p.fav).collect();
        let mut grid = div().flex().flex_wrap().gap(px(14.));
        for (idx, p) in &visible {
            let idx = *idx;
            grid = grid.child(
                div()
                    .id(("ph", idx))
                    .relative()
                    .flex_none()
                    .w(px(176.))
                    .h(px(176.))
                    .rounded(px(16.))
                    .cursor_pointer()
                    .shadow(elev(&t, 2))
                    .hover(move |s| s.shadow(elev(&t, 3)))
                    .child(visual(&t, p, 16.))
                    .child(
                        div().absolute().left(px(10.)).bottom(px(8.)).text_size(px(META)).font_weight(FontWeight::SEMIBOLD).text_color(hexc(0xffffff)).child(p.name),
                    )
                    .when(p.fav, |d| d.child(div().absolute().top(px(8.)).right(px(8.)).child(icon("heart", IV::Bold, 16., hexc(0xff6b7d)))))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.open = Some(idx);
                        cx.notify();
                    })),
            );
        }
        let header = div()
            .flex()
            .flex_none()
            .items_center()
            .gap(px(14.))
            .h(px(60.))
            .pl(px(84.))
            .pr(px(20.))
            .on_mouse_down(MouseButton::Left, |ev, _, cx| start_drag(cx, AppId::Photos, ev.position))
            .child(div().flex_1().text_size(px(TITLE)).font_weight(FontWeight::BOLD).text_color(t.ink1).child("Library"))
            .child(chip(&t, &format!("{} photos", visible.len()), t.accent))
            .child(segmented(
                "phseg",
                &t,
                if self.only_fav { 1 } else { 0 },
                vec![
                    (None, Some("All"), Box::new(cx.listener(|this, _, _, cx| { this.only_fav = false; cx.notify(); }))),
                    (Some("heart"), Some("Favorites"), Box::new(cx.listener(|this, _, _, cx| { this.only_fav = true; cx.notify(); }))),
                ],
            ));
        let mut root = div()
            .relative()
            .flex()
            .flex_col()
            .size_full()
            .bg(bg_grad(alpha(t.sidebar_top, 0.6), t.content_top))
            .child(header)
            .child(div().id("ph-scroll").flex_1().min_h(px(0.)).overflow_y_scroll().px(px(20.)).pb(px(20.)).child(grid));

        if let Some(i) = self.open {
            let p = self.photos[i].clone();
            let n = self.photos.len();
            let e = cx.entity();
            let overlay = if self.expo < 0.5 { c(0, 0, 0, (0.5 - self.expo) * 1.3) } else { c(255, 255, 255, (self.expo - 0.5) * 0.9) };
            root = root.child(
                div()
                    .absolute()
                    .inset_0()
                    .flex()
                    .flex_col()
                    .bg(if t.dark { c(5, 8, 14, 0.96) } else { c(240, 244, 251, 0.97) })
                    .occlude()
                    .child(
                        div()
                            .flex()
                            .flex_none()
                            .items_center()
                            .gap(px(10.))
                            .h(px(60.))
                            .pl(px(84.))
                            .pr(px(20.))
                            .on_mouse_down(MouseButton::Left, |ev, _, cx| start_drag(cx, AppId::Photos, ev.position))
                            .child(button("pv-back", &t, Btn::Tonal, Some("alt-arrow-left"), Some("Library"), 34.).on_click(cx.listener(|this, _, _, cx| { this.open = None; this.expo = 0.5; cx.notify(); })))
                            .child(div().flex_1().text_size(px(TITLE)).font_weight(FontWeight::BOLD).text_color(t.ink1).child(p.name))
                            .child(chip(&t, &format!("{} / {}", i + 1, n), t.ink3))
                            .child(button("pv-fav", &t, if p.fav { Btn::Accent } else { Btn::Tonal }, Some("heart"), None, 34.).on_click(cx.listener(move |this, _, _, cx| {
                                this.photos[i].fav = !this.photos[i].fav;
                                toast(cx, if this.photos[i].fav { "Added to Favorites" } else { "Removed from Favorites" });
                                cx.notify();
                            }))),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_1()
                            .min_h(px(0.))
                            .items_center()
                            .justify_center()
                            .gap(px(14.))
                            .px(px(20.))
                            .child(button("pv-prev", &t, Btn::Tonal, Some("alt-arrow-left"), None, 40.).on_click(cx.listener(move |this, _, _, cx| { this.open = Some((i + n - 1) % n); cx.notify(); })))
                            .child(div().relative().flex_1().h_full().max_w(px(760.)).rounded(px(18.)).shadow(elev(&t, 4)).child(visual(&t, &p, 18.)).child(div().absolute().inset_0().rounded(px(18.)).bg(overlay)))
                            .child(button("pv-next", &t, Btn::Tonal, Some("alt-arrow-right"), None, 40.).on_click(cx.listener(move |this, _, _, cx| { this.open = Some((i + 1) % n); cx.notify(); }))),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_none()
                            .items_center()
                            .justify_center()
                            .gap(px(14.))
                            .h(px(70.))
                            .child(icon("sun-2", IV::Lin, 18., t.ink3))
                            .child(div().w(px(260.)).child(slider("expo", &t, self.expo, self.bounds.clone(), Rc::new(move |v, _, cx| { e.update(cx, |this, cx| { this.expo = v; cx.notify(); }); }))))
                            .child(div().w(px(60.)).text_size(px(META)).text_color(t.ink3).child(format!("Exposure {:+.0}", (self.expo - 0.5) * 100.0)))
                            .child(button("pv-reset", &t, Btn::Ghost, Some("restart"), Some("Reset"), 30.).on_click(cx.listener(|this, _, _, cx| { this.expo = 0.5; cx.notify(); }))),
                    ),
            );
        }
        root
    }
}
