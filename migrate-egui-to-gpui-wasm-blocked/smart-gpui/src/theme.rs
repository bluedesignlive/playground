//! Smart design contract v0.1 — tokens (mirrors assets/tokens.toml of the egui build).
use gpui_kit::*;

pub fn c(r: u8, g: u8, b: u8, a: f32) -> Hsla {
    Hsla::from(Rgba { r: r as f32 / 255., g: g as f32 / 255., b: b as f32 / 255., a })
}
pub fn hexc(h: u32) -> Hsla {
    Hsla::from(rgb(h))
}
pub fn alpha(col: Hsla, a: f32) -> Hsla {
    Hsla { a: col.a * a, ..col }
}
pub fn mix(a: Hsla, b: Hsla, t: f32) -> Hsla {
    let (a, b) = (Rgba::from(a), Rgba::from(b));
    let l = |x: f32, y: f32| x + (y - x) * t;
    Hsla::from(Rgba { r: l(a.r, b.r), g: l(a.g, b.g), b: l(a.b, b.b), a: l(a.a, b.a) })
}
pub fn bg_grad(top: Hsla, bottom: Hsla) -> Background {
    linear_gradient(180., linear_color_stop(top, 0.), linear_color_stop(bottom, 1.))
}

#[derive(Clone, Copy)]
pub struct Theme {
    pub dark: bool,
    pub ink1: Hsla,
    pub ink2: Hsla,
    pub ink3: Hsla,
    pub ink4: Hsla,
    pub accent: Hsla,
    pub accent_hover: Hsla,
    pub accent_ink: Hsla,
    pub accent_tint: Hsla,
    pub accent_tint2: Hsla,
    pub danger: Hsla,
    pub danger_tint: Hsla,
    pub good: Hsla,
    pub warn: Hsla,
    pub window_top: Hsla,
    pub window_bottom: Hsla,
    pub sidebar_top: Hsla,
    pub sidebar_bottom: Hsla,
    pub content_top: Hsla,
    pub control_top: Hsla,
    pub control_bottom: Hsla,
    pub control_hover_top: Hsla,
    pub control_hover_bottom: Hsla,
    pub field: Hsla,
    pub row_hover: Hsla,
    pub row_press: Hsla,
    pub card_top: Hsla,
    pub card_bottom: Hsla,
    pub card_hover_top: Hsla,
    pub seam: Hsla,
    pub scrim: Hsla,
    pub menu: Hsla,
    pub panel_top: Hsla,
    pub panel_bottom: Hsla,
    pub rim: Hsla,
    pub shadow: Hsla,
    pub tooltip_bg: Hsla,
    pub well: Hsla,
    pub grad_a: Hsla,
    pub grad_b: Hsla,
    pub on_accent: Hsla,
}

pub struct ThemeG(pub Theme);
impl Global for ThemeG {}

pub fn theme(cx: &App) -> Theme {
    cx.global::<ThemeG>().0
}

impl Theme {
    pub fn light() -> Self {
        Self {
            dark: false,
            ink1: hexc(0x141d30),
            ink2: hexc(0x3f4b60),
            ink3: hexc(0x667286),
            ink4: hexc(0x8b95a6),
            accent: hexc(0x2f6feb),
            accent_hover: hexc(0x3d7bf7),
            accent_ink: hexc(0x1c56c4),
            accent_tint: c(47, 111, 235, 0.105),
            accent_tint2: c(47, 111, 235, 0.055),
            danger: hexc(0xc53b48),
            danger_tint: c(197, 59, 72, 0.10),
            good: hexc(0x2b9b6d),
            warn: hexc(0xc9861a),
            window_top: c(255, 255, 255, 0.86),
            window_bottom: c(241, 246, 253, 0.82),
            sidebar_top: c(222, 233, 248, 0.78),
            sidebar_bottom: c(230, 239, 250, 0.66),
            content_top: c(255, 255, 255, 0.34),
            control_top: c(255, 255, 255, 0.96),
            control_bottom: c(255, 255, 255, 0.62),
            control_hover_top: c(255, 255, 255, 1.0),
            control_hover_bottom: c(255, 255, 255, 0.80),
            field: c(255, 255, 255, 0.62),
            row_hover: c(24, 35, 56, 0.055),
            row_press: c(24, 35, 56, 0.082),
            card_top: c(255, 255, 255, 0.92),
            card_bottom: c(255, 255, 255, 0.70),
            card_hover_top: c(255, 255, 255, 1.0),
            seam: c(255, 255, 255, 0.56),
            scrim: c(20, 30, 48, 0.32),
            menu: c(251, 253, 255, 0.995),
            panel_top: c(252, 253, 255, 0.995),
            panel_bottom: c(246, 249, 255, 0.99),
            rim: c(255, 255, 255, 0.95),
            shadow: c(16, 26, 44, 1.0),
            tooltip_bg: c(20, 29, 48, 0.96),
            well: c(24, 35, 56, 0.07),
            grad_a: hexc(0x4a86f5),
            grad_b: hexc(0x2760d8),
            on_accent: hexc(0xffffff),
        }
    }
    pub fn dark() -> Self {
        Self {
            dark: true,
            ink1: hexc(0xeef3fb),
            ink2: hexc(0xc6cfdf),
            ink3: hexc(0x96a1b5),
            ink4: hexc(0x707b90),
            accent: hexc(0x6f9bff),
            accent_hover: hexc(0x86acff),
            accent_ink: hexc(0xa7c1ff),
            accent_tint: c(111, 155, 255, 0.155),
            accent_tint2: c(111, 155, 255, 0.07),
            danger: hexc(0xff7a84),
            danger_tint: c(255, 122, 132, 0.14),
            good: hexc(0x59c79a),
            warn: hexc(0xe5b05a),
            window_top: c(20, 27, 40, 0.88),
            window_bottom: c(13, 19, 30, 0.90),
            sidebar_top: c(13, 19, 30, 0.72),
            sidebar_bottom: c(10, 15, 25, 0.62),
            content_top: c(255, 255, 255, 0.022),
            control_top: c(255, 255, 255, 0.135),
            control_bottom: c(255, 255, 255, 0.05),
            control_hover_top: c(255, 255, 255, 0.19),
            control_hover_bottom: c(255, 255, 255, 0.085),
            field: c(255, 255, 255, 0.06),
            row_hover: c(255, 255, 255, 0.055),
            row_press: c(255, 255, 255, 0.085),
            card_top: c(255, 255, 255, 0.085),
            card_bottom: c(255, 255, 255, 0.04),
            card_hover_top: c(255, 255, 255, 0.125),
            seam: c(255, 255, 255, 0.07),
            scrim: c(3, 7, 14, 0.56),
            menu: c(27, 35, 50, 0.985),
            panel_top: c(24, 32, 47, 0.985),
            panel_bottom: c(15, 22, 34, 0.99),
            rim: c(255, 255, 255, 0.26),
            shadow: c(0, 0, 0, 1.0),
            tooltip_bg: c(238, 243, 251, 0.96),
            well: c(0, 0, 0, 0.28),
            grad_a: hexc(0x86acff),
            grad_b: hexc(0x5b87f0),
            on_accent: hexc(0x0b1a3a),
        }
    }
}

fn bs(color: Hsla, y: f32, blur: f32, spread: f32) -> BoxShadow {
    BoxShadow {
        color,
        offset: point(px(0.), px(y)),
        blur_radius: px(blur),
        spread_radius: px(spread),
        inset: false,
    }
}

/// Elevation ladder e1..e5 — tonal + shadow, with the inset top reflection of the contract.
pub fn elev(t: &Theme, level: u8) -> Vec<BoxShadow> {
    let k = |a: f32| alpha(t.shadow, a);
    let mut v = match (t.dark, level) {
        (false, 1) => vec![bs(k(0.07), 1., 2., 0.)],
        (false, 2) => vec![bs(k(0.13), 3., 10., -3.), bs(k(0.05), 1., 2., 0.)],
        (false, 3) => vec![bs(k(0.24), 14., 34., -12.), bs(k(0.08), 3., 10., -2.)],
        (false, 4) => vec![bs(k(0.40), 28., 72., -16.), bs(k(0.16), 6., 18., -6.)],
        (false, _) => vec![bs(k(0.55), 54., 140., -28.), bs(k(0.30), 18., 48., -18.)],
        (true, 1) => vec![bs(k(0.28), 1., 2., 0.)],
        (true, 2) => vec![bs(k(0.40), 3., 10., -3.), bs(k(0.24), 1., 2., 0.)],
        (true, 3) => vec![bs(k(0.58), 16., 38., -12.), bs(k(0.30), 3., 10., -2.)],
        (true, 4) => vec![bs(k(0.70), 32., 80., -16.), bs(k(0.44), 8., 22., -6.)],
        (true, _) => vec![bs(k(0.86), 60., 150., -30.), bs(k(0.60), 20., 54., -18.)],
    };
    v.push(BoxShadow {
        color: alpha(t.rim, if t.dark { 0.30 } else { 0.68 }),
        offset: point(px(0.), px(1.)),
        blur_radius: px(0.),
        spread_radius: px(0.),
        inset: true,
    });
    v
}

pub fn inset_well(t: &Theme) -> Vec<BoxShadow> {
    vec![BoxShadow {
        color: alpha(t.shadow, if t.dark { 0.45 } else { 0.10 }),
        offset: point(px(0.), px(1.)),
        blur_radius: px(2.),
        spread_radius: px(0.),
        inset: true,
    }]
}

pub fn accent_glow(t: &Theme) -> Vec<BoxShadow> {
    vec![
        bs(alpha(t.accent, if t.dark { 0.30 } else { 0.38 }), 5., 14., -4.),
        BoxShadow {
            color: c(255, 255, 255, 0.28),
            offset: point(px(0.), px(1.)),
            blur_radius: px(0.),
            spread_radius: px(0.),
            inset: true,
        },
    ]
}
