//! Design tokens (mirrors assets/tokens.toml — “one contract, every host”).
use egui::{Color32, FontFamily, FontId};

pub fn c(r: u8, g: u8, b: u8, a: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(r, g, b, (a.clamp(0.0, 1.0) * 255.0).round() as u8)
}
pub fn hex(s: &str) -> Color32 {
    let s = s.trim_start_matches('#');
    let p = |i: usize| u8::from_str_radix(&s[i..i + 2], 16).unwrap_or(0);
    Color32::from_rgb(p(0), p(2), p(4))
}
pub fn alpha(col: Color32, a: f32) -> Color32 {
    let [r, g, b, a0] = col.to_srgba_unmultiplied();
    c(r, g, b, a0 as f32 / 255.0 * a)
}
pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let l = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color32::from_rgba_premultiplied(l(a.r(), b.r()), l(a.g(), b.g()), l(a.b(), b.b()), l(a.a(), b.a()))
}

#[derive(Clone)]
pub struct Theme {
    pub dark: bool,
    pub ink1: Color32,
    pub ink2: Color32,
    pub ink3: Color32,
    pub ink4: Color32,
    pub accent: Color32,
    pub accent_hover: Color32,
    pub accent_press: Color32,
    pub accent_ink: Color32,
    pub accent_tint: Color32,
    pub accent_tint2: Color32,
    pub danger: Color32,
    pub danger_tint: Color32,
    pub good: Color32,
    pub warn: Color32,
    pub window_top: Color32,
    pub window_bottom: Color32,
    pub sidebar_top: Color32,
    pub sidebar_bottom: Color32,
    pub content_top: Color32,
    pub content_bottom: Color32,
    pub control_top: Color32,
    pub control_bottom: Color32,
    pub control_hover_top: Color32,
    pub control_hover_bottom: Color32,
    pub field: Color32,
    pub row_hover: Color32,
    pub row_press: Color32,
    pub card_top: Color32,
    pub card_bottom: Color32,
    pub card_hover_top: Color32,
    pub seam: Color32,
    pub scrim: Color32,
    pub menu: Color32,
    pub panel_top: Color32,
    pub panel_bottom: Color32,
    pub veil_top: Color32,
    pub veil_bottom: Color32,
    pub rim: Color32,       // top reflection
    pub rim_under: Color32, // bottom inner shade
    pub shadow: Color32,    // base colour of drop shadows
    pub tooltip_bg: Color32,
    pub well: Color32,
}

impl Theme {
    pub fn light() -> Self {
        Self {
            dark: false,
            ink1: hex("#141d30"),
            ink2: hex("#3f4b60"),
            ink3: hex("#667286"),
            ink4: hex("#8b95a6"),
            accent: hex("#2f6feb"),
            accent_hover: hex("#3d7bf7"),
            accent_press: hex("#2560d4"),
            accent_ink: hex("#1c56c4"),
            accent_tint: c(47, 111, 235, 0.105),
            accent_tint2: c(47, 111, 235, 0.055),
            danger: hex("#c53b48"),
            danger_tint: c(197, 59, 72, 0.10),
            good: hex("#2b9b6d"),
            warn: hex("#c9861a"),
            window_top: c(255, 255, 255, 0.86),
            window_bottom: c(241, 246, 253, 0.82),
            sidebar_top: c(222, 233, 248, 0.78),
            sidebar_bottom: c(230, 239, 250, 0.66),
            content_top: c(255, 255, 255, 0.34),
            content_bottom: c(248, 251, 255, 0.18),
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
            veil_top: c(233, 239, 248, 0.04),
            veil_bottom: c(206, 217, 233, 0.26),
            rim: c(255, 255, 255, 0.95),
            rim_under: c(16, 26, 44, 0.06),
            shadow: c(16, 26, 44, 1.0),
            tooltip_bg: c(20, 29, 48, 0.96),
            well: c(24, 35, 56, 0.07),
        }
    }
    pub fn dark() -> Self {
        Self {
            dark: true,
            ink1: hex("#eef3fb"),
            ink2: hex("#c6cfdf"),
            ink3: hex("#96a1b5"),
            ink4: hex("#707b90"),
            accent: hex("#6f9bff"),
            accent_hover: hex("#86acff"),
            accent_press: hex("#5b87f0"),
            accent_ink: hex("#a7c1ff"),
            accent_tint: c(111, 155, 255, 0.155),
            accent_tint2: c(111, 155, 255, 0.07),
            danger: hex("#ff7a84"),
            danger_tint: c(255, 122, 132, 0.14),
            good: hex("#59c79a"),
            warn: hex("#e5b05a"),
            window_top: c(20, 27, 40, 0.86),
            window_bottom: c(13, 19, 30, 0.88),
            sidebar_top: c(13, 19, 30, 0.72),
            sidebar_bottom: c(10, 15, 25, 0.62),
            content_top: c(255, 255, 255, 0.022),
            content_bottom: c(255, 255, 255, 0.0),
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
            veil_top: c(10, 16, 28, 0.55),
            veil_bottom: c(8, 12, 22, 0.70),
            rim: c(255, 255, 255, 0.26),
            rim_under: c(0, 0, 0, 0.38),
            shadow: c(0, 0, 0, 1.0),
            tooltip_bg: c(238, 243, 251, 0.96),
            well: c(0, 0, 0, 0.28),
        }
    }
    pub fn accent_grad(&self) -> (Color32, Color32) {
        if self.dark {
            (hex("#86acff"), hex("#5b87f0"))
        } else {
            (hex("#4a86f5"), hex("#2760d8"))
        }
    }
    pub fn on_accent(&self) -> Color32 {
        if self.dark { hex("#0b1a3a") } else { Color32::WHITE }
    }
}

// ─────────── type ramp (px) ───────────
pub const T_LABEL: f32 = 10.5;
pub const T_META: f32 = 11.5;
pub const T_BODY: f32 = 13.0;
pub const T_TITLE: f32 = 15.0;
pub const T_PAGE: f32 = 26.0;

pub fn f_regular(s: f32) -> FontId {
    FontId::new(s, FontFamily::Proportional)
}
pub fn f_medium(s: f32) -> FontId {
    FontId::new(s, FontFamily::Name("medium".into()))
}
pub fn f_semi(s: f32) -> FontId {
    FontId::new(s, FontFamily::Name("semibold".into()))
}
pub fn f_bold(s: f32) -> FontId {
    FontId::new(s, FontFamily::Name("bold".into()))
}

pub fn install_fonts(ctx: &egui::Context) {
    use egui::{FontData, FontDefinitions};
    use std::sync::Arc;
    let mut fd = FontDefinitions::default();
    let faces: [(&str, &'static [u8]); 4] = [
        ("inter-regular", include_bytes!("../../../assets/fonts/Inter-regular.otf")),
        ("inter-medium", include_bytes!("../../../assets/fonts/Inter-medium.otf")),
        ("inter-semibold", include_bytes!("../../../assets/fonts/Inter-semibold.otf")),
        ("inter-bold", include_bytes!("../../../assets/fonts/Inter-bold.otf")),
    ];
    for (n, b) in faces {
        fd.font_data.insert(n.into(), Arc::new(FontData::from_static(b)));
    }
    let fallback: Vec<String> = fd.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
    for (fam, face) in [
        (FontFamily::Proportional, "inter-regular"),
        (FontFamily::Name("medium".into()), "inter-medium"),
        (FontFamily::Name("semibold".into()), "inter-semibold"),
        (FontFamily::Name("bold".into()), "inter-bold"),
    ] {
        let mut v = vec![face.to_string()];
        v.extend(fallback.iter().cloned());
        fd.families.insert(fam, v);
    }
    ctx.set_fonts(fd);
}
