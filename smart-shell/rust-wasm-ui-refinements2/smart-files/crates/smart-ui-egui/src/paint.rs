//! Depth primitives: anti-aliased gradient fills, top-light rims, layered elevation shadows.
use crate::theme::{alpha, c, mix, Theme};
use egui::epaint::{Mesh, RectShape, Shadow, Vertex, WHITE_UV};
use egui::{pos2, vec2, Color32, CornerRadius, Painter, Pos2, Rect, Shape, Vec2};
use std::f32::consts::{FRAC_PI_2, PI};

pub fn cr(r: f32) -> CornerRadius {
    CornerRadius::same(r.round().clamp(0.0, 255.0) as u8)
}

/// Outline of a rounded rect, clockwise from the top-left arc, with outward normals.
pub fn rr_points(rect: Rect, r: f32, segs: usize) -> Vec<(Pos2, Vec2)> {
    let r = r.max(0.0).min(rect.width() / 2.0).min(rect.height() / 2.0);
    let mut v = Vec::with_capacity(segs * 4 + 4);
    let corners = [
        (pos2(rect.min.x + r, rect.min.y + r), PI),
        (pos2(rect.max.x - r, rect.min.y + r), PI + FRAC_PI_2),
        (pos2(rect.max.x - r, rect.max.y - r), 0.0),
        (pos2(rect.min.x + r, rect.max.y - r), FRAC_PI_2),
    ];
    let n = if r < 0.5 { 1 } else { segs };
    for (c, a0) in corners {
        for i in 0..=n {
            let a = a0 + FRAC_PI_2 * (i as f32 / n as f32);
            let nrm = vec2(a.cos(), a.sin());
            v.push((c + nrm * r, nrm));
        }
    }
    v
}

fn lerp_y(top: Color32, bot: Color32, rect: Rect, y: f32) -> Color32 {
    mix(top, bot, (y - rect.min.y) / rect.height().max(1.0))
}

/// Anti-aliased rounded rect with a vertical colour gradient.
pub fn grad_fill(p: &Painter, rect: Rect, r: f32, top: Color32, bot: Color32) {
    if rect.width() < 1.0 || rect.height() < 1.0 {
        return;
    }
    let segs = (r / 2.0).ceil().clamp(3.0, 12.0) as usize;
    let inner = rr_points(rect.shrink(0.5), (r - 0.5).max(0.0), segs);
    let outer = rr_points(rect.expand(0.5), r + 0.5, segs);
    let mut m = Mesh::default();
    let center = rect.center();
    m.vertices.push(Vertex { pos: center, uv: WHITE_UV, color: lerp_y(top, bot, rect, center.y) });
    for (pt, _) in &inner {
        m.vertices.push(Vertex { pos: *pt, uv: WHITE_UV, color: lerp_y(top, bot, rect, pt.y) });
    }
    let n = inner.len() as u32;
    for i in 0..n {
        let j = (i + 1) % n;
        m.indices.extend([0, 1 + i, 1 + j]);
    }
    let base = m.vertices.len() as u32;
    for (pt, _) in &outer {
        m.vertices.push(Vertex { pos: *pt, uv: WHITE_UV, color: Color32::TRANSPARENT });
    }
    for i in 0..n {
        let j = (i + 1) % n;
        m.indices.extend([1 + i, base + i, base + j, 1 + i, base + j, 1 + j]);
    }
    p.add(Shape::mesh(m));
}

/// Light that catches the top edge (`up = true`) or shade that sits under the bottom edge (`up = false`).
/// `width` px thick, fading along the sides and down by `fade` px.
pub fn rim(p: &Painter, rect: Rect, r: f32, width: f32, col: Color32, up: bool, fade: f32) {
    if rect.width() < 2.0 || rect.height() < 2.0 {
        return;
    }
    let segs = (r / 2.0).ceil().clamp(3.0, 12.0) as usize;
    let outer = rr_points(rect, r, segs);
    let mut m = Mesh::default();
    let n = outer.len();
    let w = width;
    for (pt, nrm) in &outer {
        let facing = if up { -nrm.y } else { nrm.y }.max(0.0);
        let dist = if up { pt.y - rect.min.y } else { rect.max.y - pt.y };
        let f = (facing.powf(1.4) * (1.0 - dist / fade.max(1.0)).clamp(0.0, 1.0)).clamp(0.0, 1.0);
        let col_o = alpha(col, f);
        m.vertices.push(Vertex { pos: *pt, uv: WHITE_UV, color: col_o });
        m.vertices.push(Vertex { pos: *pt - *nrm * w, uv: WHITE_UV, color: alpha(col_o, 0.0) });
    }
    for i in 0..n as u32 {
        let j = (i + 1) % n as u32;
        let (a, b, c2, d) = (2 * i, 2 * i + 1, 2 * j, 2 * j + 1);
        m.indices.extend([a, b, d, a, d, c2]);
    }
    p.add(Shape::mesh(m));
}

/// Layered drop-shadows (blur, y-offset, spread — CSS semantics, negative spread supported).
pub fn shadow(p: &Painter, rect: Rect, r: f32, dy: f32, blur: f32, spread: f32, col: Color32) {
    let rr = if spread < 0.0 { rect.shrink(-spread) } else { rect };
    let s = Shadow { offset: [0, dy.round().clamp(-127.0, 127.0) as i8], blur: blur.clamp(0.0, 255.0) as u8, spread: spread.max(0.0) as u8, color: col };
    p.add(s.as_shape(rr, cr((r + spread).max(0.0))));
}

/// Elevation ladder e1…e5 from the design contract.
pub fn elevate(p: &Painter, t: &Theme, rect: Rect, r: f32, level: u8) {
    let base = t.shadow;
    let a = |l: f32, d: f32| alpha(base, if t.dark { d } else { l });
    match level {
        1 => shadow(p, rect, r, 1.0, 2.0, 0.0, a(0.07, 0.28)),
        2 => {
            shadow(p, rect, r, 3.0, 10.0, -3.0, a(0.13, 0.40));
            shadow(p, rect, r, 1.0, 2.0, 0.0, a(0.05, 0.24));
        }
        3 => {
            shadow(p, rect, r, 14.0, 34.0, -12.0, a(0.24, 0.58));
            shadow(p, rect, r, 3.0, 10.0, -2.0, a(0.08, 0.30));
        }
        4 => {
            shadow(p, rect, r, 28.0, 72.0, -16.0, a(0.40, 0.70));
            shadow(p, rect, r, 6.0, 18.0, -6.0, a(0.16, 0.44));
        }
        _ => {
            shadow(p, rect, r, 54.0, 140.0, -28.0, a(0.55, 0.86));
            shadow(p, rect, r, 18.0, 48.0, -18.0, a(0.30, 0.60));
        }
    }
}

pub fn textured_rrect(p: &Painter, rect: Rect, r: f32, tex: egui::TextureId, uv: Rect, tint: Color32) {
    p.add(Shape::Rect(RectShape::filled(rect, cr(r), tint).with_texture(tex, uv)));
}

/// UV rect that crops a texture of `aspect` (w/h) to cover `target`.
pub fn cover_uv(tex_w: f32, tex_h: f32, target: Rect) -> Rect {
    let ta = target.width() / target.height();
    let ia = tex_w / tex_h;
    if ia > ta {
        let w = ta / ia;
        Rect::from_min_max(pos2((1.0 - w) / 2.0, 0.0), pos2((1.0 + w) / 2.0, 1.0))
    } else {
        let h = ia / ta;
        Rect::from_min_max(pos2(0.0, (1.0 - h) / 2.0), pos2(1.0, (1.0 + h) / 2.0))
    }
}

/// Raised, skeuomorphic control surface: soft shadow → gradient body → top light → bottom shade.
pub fn raised(p: &Painter, t: &Theme, rect: Rect, r: f32, hover: bool, pressed: bool) {
    if pressed {
        grad_fill(p, rect, r, mix(t.control_bottom, t.well, 0.7), t.control_bottom);
        rim(p, rect, r, 3.0, alpha(t.shadow, if t.dark { 0.35 } else { 0.10 }), true, 12.0);
        return;
    }
    if hover {
        elevate(p, t, rect, r, 2);
    } else {
        elevate(p, t, rect, r, 1);
    }
    let (top, bot) = if hover { (t.control_hover_top, t.control_hover_bottom) } else { (t.control_top, t.control_bottom) };
    grad_fill(p, rect, r, top, bot);
    rim(p, rect, r, 1.2, t.rim, true, (rect.height() * 0.55).max(8.0));
    rim(p, rect, r, 1.0, t.rim_under, false, 6.0);
}

/// Accent (primary) control with the same light treatment.
pub fn raised_accent(p: &Painter, t: &Theme, rect: Rect, r: f32, hover: bool, pressed: bool) {
    let (mut a, mut b) = t.accent_grad();
    if hover {
        a = mix(a, Color32::WHITE, 0.10);
        b = mix(b, Color32::WHITE, 0.06);
    }
    if pressed {
        a = mix(a, Color32::BLACK, 0.10);
        b = mix(b, Color32::BLACK, 0.10);
    }
    if !pressed {
        shadow(p, rect, r, 5.0, 14.0, -4.0, alpha(t.accent, if t.dark { 0.45 } else { 0.42 }));
        shadow(p, rect, r, 1.0, 2.0, 0.0, alpha(t.shadow, if t.dark { 0.4 } else { 0.16 }));
    }
    grad_fill(p, rect, r, a, b);
    rim(p, rect, r, 1.3, Color32::from_white_alpha(if t.dark { 120 } else { 150 }), true, rect.height() * 0.6);
    rim(p, rect, r, 1.0, Color32::from_black_alpha(46), false, 6.0);
}

/// Tinted raised surface (accent / danger tonal buttons) with the same light treatment.
pub fn elevated_tonal(p: &Painter, t: &Theme, rect: Rect, r: f32, hover: bool, pressed: bool, tint: Color32, hue: Color32) {
    if !pressed {
        elevate(p, t, rect, r, if hover { 2 } else { 1 });
        shadow(p, rect, r, 3.0, 9.0, -3.0, alpha(hue, if t.dark { 0.18 } else { 0.14 }));
    }
    let a = if hover { 0.30 } else { 0.20 };
    grad_fill(p, rect, r, mix(tint, t.control_top, if t.dark { 0.0 } else { 0.4 }), alpha(tint, a * 0.8));
    grad_fill(p, rect, r, alpha(hue, a), alpha(hue, a * 0.55));
    if pressed {
        rim(p, rect, r, 3.0, alpha(t.shadow, if t.dark { 0.35 } else { 0.10 }), true, 12.0);
    } else {
        rim(p, rect, r, 1.2, t.rim, true, (rect.height() * 0.6).max(8.0));
        rim(p, rect, r, 1.0, t.rim_under, false, 6.0);
    }
}

/// Recessed well (search field, segmented track).
pub fn well(p: &Painter, t: &Theme, rect: Rect, r: f32) {
    grad_fill(p, rect, r, t.well, alpha(t.well, 0.55));
    rim(p, rect, r, 3.0, alpha(t.shadow, if t.dark { 0.55 } else { 0.10 }), true, 14.0);
    rim(p, rect, r, 1.0, alpha(Color32::WHITE, if t.dark { 0.06 } else { 0.8 }), false, 4.0);
}

/// Elevated glass surface for menus / popovers. Brighter than the window under it (so the bottom never melts into
/// the background), with a long top light, a lit lower lip and a full-perimeter hairline.
pub fn menu_surface(p: &Painter, t: &Theme, rect: Rect, r: f32) {
    elevate(p, t, rect, r, 4);
    // contact shadow tight to the edge keeps the silhouette readable on dark and light backdrops
    shadow(p, rect, r, 1.0, 3.0, 0.0, alpha(t.shadow, if t.dark { 0.55 } else { 0.14 }));
    let (top, bot) = if t.dark { (c(50, 61, 84, 0.99), c(37, 47, 66, 0.99)) } else { (c(255, 255, 255, 1.0), c(243, 247, 254, 1.0)) };
    grad_fill(p, rect, r, top, bot);
    // light that travels most of the way down the sides instead of dying after a few pixels
    rim(p, rect, r, 1.6, alpha(Color32::WHITE, if t.dark { 0.34 } else { 1.0 }), true, rect.height() * 0.9);
    // lit lower lip: glass catches a little light on its underside too
    rim(p, rect, r, 1.0, alpha(Color32::WHITE, if t.dark { 0.14 } else { 0.7 }), false, 22.0);
    p.rect_stroke(rect.shrink(0.5), cr(r), egui::Stroke::new(1.0, if t.dark { c(255, 255, 255, 0.10) } else { c(16, 26, 44, 0.08) }), egui::StrokeKind::Inside);
}

/// Closed-form damped spring step response (0 → 1 with a little overshoot). `t` in seconds.
pub fn spring_out(t: f32) -> f32 {
    if t <= 0.0 {
        return 0.0;
    }
    let (w, z) = (26.0f32, 0.62f32);
    let wd = w * (1.0 - z * z).sqrt();
    1.0 - (-z * w * t).exp() * ((wd * t).cos() + z / (1.0 - z * z).sqrt() * (wd * t).sin())
}

/// Stateful spring per `id`, stored in egui memory. Returns the current value; keeps repainting until at rest.
pub fn spring_to(ctx: &egui::Context, id: impl std::hash::Hash + std::fmt::Debug, target: f32, stiffness: f32, damping: f32) -> f32 {
    let key = egui::Id::new(("spring", id));
    let dt = ctx.input(|i| i.stable_dt).min(0.05);
    let (mut x, mut v) = ctx.data_mut(|d| d.get_temp::<(f32, f32)>(key)).unwrap_or((target, 0.0));
    // semi-implicit Euler, sub-stepped for stability
    let n = 3;
    for _ in 0..n {
        let h = dt / n as f32;
        v += (-stiffness * (x - target) - damping * v) * h;
        x += v * h;
    }
    if (x - target).abs() < 0.0006 && v.abs() < 0.004 {
        x = target;
        v = 0.0;
    } else {
        ctx.request_repaint();
    }
    ctx.data_mut(|d| d.insert_temp(key, (x, v)));
    x
}
