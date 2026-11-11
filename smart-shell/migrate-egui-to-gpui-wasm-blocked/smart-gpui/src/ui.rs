//! Smart components built fresh on GPUI primitives (depth through tone + shadow, never strokes).
use crate::assets::{IV, icon};
use crate::theme::*;
use gpui_kit::prelude::*;
use gpui_kit::*;
use std::cell::Cell;
use std::rc::Rc;

pub type Click = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type Change = Rc<dyn Fn(f32, &mut Window, &mut App) + 'static>;

pub const BODY: f32 = 13.0;
pub const META: f32 = 11.5;
pub const LABEL: f32 = 10.5;
pub const TITLE: f32 = 15.0;

fn stop_down(_: &MouseDownEvent, _: &mut Window, cx: &mut App) {
    cx.stop_propagation();
}

// ───────────────────────── buttons ─────────────────────────
#[derive(Clone, Copy, PartialEq)]
pub enum Btn {
    Tonal,
    Accent,
    Danger,
    Ghost,
}

pub fn button(id: impl Into<ElementId>, t: &Theme, kind: Btn, ic: Option<&str>, label: Option<&str>, size: f32) -> Stateful<Div> {
    let t = *t;
    let (top, bot, htop, hbot, fg) = match kind {
        Btn::Tonal => (t.control_top, t.control_bottom, t.control_hover_top, t.control_hover_bottom, t.ink1),
        Btn::Accent => (t.grad_a, t.grad_b, mix(t.grad_a, t.on_accent, 0.0), t.accent_hover, t.on_accent),
        Btn::Danger => (t.danger_tint, t.danger_tint, alpha(t.danger, 0.2), alpha(t.danger, 0.2), t.danger),
        Btn::Ghost => (c(0, 0, 0, 0.0), c(0, 0, 0, 0.0), t.row_hover, t.row_hover, t.ink2),
    };
    let shadow = match kind {
        Btn::Tonal => elev(&t, 1),
        Btn::Accent => accent_glow(&t),
        _ => vec![],
    };
    let mut d = div()
        .id(id)
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .gap(px(7.))
        .h(px(size))
        .rounded(px(999.))
        .cursor_pointer()
        .text_size(px(BODY))
        .font_weight(FontWeight::MEDIUM)
        .text_color(fg)
        .bg(bg_grad(top, bot))
        .shadow(shadow)
        .hover(move |s| s.bg(bg_grad(htop, hbot)))
        .on_mouse_down(MouseButton::Left, stop_down);
    d = if label.is_none() { d.w(px(size)) } else { d.px(px(size * 0.44)) };
    if let Some(i) = ic {
        d = d.child(icon(i, IV::Lin, 17., fg));
    }
    if let Some(l) = label {
        d = d.child(l.to_string());
    }
    d
}

pub fn segmented(id: &'static str, t: &Theme, selected: usize, items: Vec<(Option<&'static str>, Option<&'static str>, Click)>) -> Div {
    let t = *t;
    let mut row = div()
        .flex()
        .flex_none()
        .items_center()
        .gap(px(2.))
        .p(px(3.))
        .h(px(36.))
        .rounded(px(999.))
        .bg(t.well)
        .shadow(inset_well(&t));
    for (i, (ic, label, h)) in items.into_iter().enumerate() {
        let on = i == selected;
        let fg = if on { t.ink1 } else { t.ink3 };
        let mut seg = div()
            .id((id, i))
            .flex()
            .items_center()
            .justify_center()
            .gap(px(6.))
            .h(px(30.))
            .min_w(px(34.))
            .px(px(10.))
            .rounded(px(999.))
            .cursor_pointer()
            .text_size(px(12.5))
            .font_weight(FontWeight::MEDIUM)
            .text_color(fg)
            .on_mouse_down(MouseButton::Left, stop_down)
            .on_click(move |e, w, cx| h(e, w, cx));
        if on {
            seg = seg.bg(bg_grad(t.card_hover_top, if t.dark { t.control_hover_bottom } else { t.control_hover_top })).shadow(elev(&t, 2));
        } else {
            seg = seg.hover(move |s| s.text_color(t.ink1));
        }
        if let Some(i) = ic {
            seg = seg.child(icon(i, if on { IV::Duo } else { IV::Lin }, 16., if on { t.accent } else { fg }));
        }
        if let Some(l) = label {
            seg = seg.child(l);
        }
        row = row.child(seg);
    }
    row
}

// ───────────────────────── toggles ─────────────────────────
pub fn switch(id: impl Into<ElementId>, t: &Theme, on: bool) -> Stateful<Div> {
    let t = *t;
    let track = if on { bg_grad(t.grad_a, t.grad_b) } else { bg_grad(alpha(t.ink4, 0.40), alpha(t.ink4, 0.32)) };
    div()
        .id(id)
        .flex()
        .flex_none()
        .items_center()
        .w(px(44.))
        .h(px(26.))
        .p(px(2.))
        .rounded(px(999.))
        .bg(track)
        .shadow(inset_well(&t))
        .cursor_pointer()
        .when(on, |d| d.justify_end())
        .on_mouse_down(MouseButton::Left, stop_down)
        .child(
            div()
                .size(px(22.))
                .rounded(px(999.))
                .bg(hexc(0xffffff))
                .shadow(vec![BoxShadow {
                    color: c(10, 20, 40, 0.28),
                    offset: point(px(0.), px(2.)),
                    blur_radius: px(5.),
                    spread_radius: px(0.),
                    inset: false,
                }]),
        )
}

pub fn checkbox(id: impl Into<ElementId>, t: &Theme, on: bool) -> Stateful<Div> {
    let t = *t;
    let base = div()
        .id(id)
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .size(px(20.))
        .rounded(px(6.))
        .cursor_pointer()
        .on_mouse_down(MouseButton::Left, stop_down);
    if on {
        base.bg(bg_grad(t.grad_a, t.grad_b)).shadow(accent_glow(&t)).child(icon("check-read", IV::Lin, 14., t.on_accent))
    } else {
        base.bg(t.field).shadow(inset_well(&t))
    }
}

pub fn radio(id: impl Into<ElementId>, t: &Theme, on: bool) -> Stateful<Div> {
    let t = *t;
    let base = div()
        .id(id)
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .size(px(20.))
        .rounded(px(999.))
        .cursor_pointer()
        .on_mouse_down(MouseButton::Left, stop_down);
    if on {
        base.bg(bg_grad(t.grad_a, t.grad_b)).shadow(accent_glow(&t)).child(div().size(px(8.)).rounded(px(999.)).bg(t.on_accent))
    } else {
        base.bg(t.field).shadow(inset_well(&t))
    }
}

pub fn progress(t: &Theme, frac: f32, h: f32) -> Div {
    div()
        .w_full()
        .h(px(h))
        .rounded(px(999.))
        .bg(t.well)
        .shadow(inset_well(t))
        .overflow_hidden()
        .child(div().h_full().w(relative(frac.clamp(0.0, 1.0))).rounded(px(999.)).bg(bg_grad(t.grad_a, t.grad_b)))
}

/// Draggable slider; the track bounds are measured by an invisible canvas.
pub fn slider(id: impl Into<ElementId>, t: &Theme, value: f32, bounds: Rc<Cell<Bounds<Pixels>>>, on_change: Change) -> Stateful<Div> {
    let t = *t;
    let v = value.clamp(0.0, 1.0);
    let b1 = bounds.clone();
    let b2 = bounds.clone();
    let c1 = on_change.clone();
    let measure = bounds.clone();
    let set = move |x: Pixels, b: &Rc<Cell<Bounds<Pixels>>>| {
        let bb = b.get();
        let w = f32::from(bb.size.width).max(1.0);
        ((f32::from(x) - f32::from(bb.origin.x)) / w).clamp(0.0, 1.0)
    };
    div()
        .id(id)
        .relative()
        .flex()
        .items_center()
        .w_full()
        .h(px(24.))
        .cursor_pointer()
        .on_mouse_down(MouseButton::Left, move |e, w, cx| {
            cx.stop_propagation();
            c1(set(e.position.x, &b1), w, cx);
        })
        .on_mouse_move(move |e, w, cx| {
            if e.pressed_button == Some(MouseButton::Left) {
                on_change(set(e.position.x, &b2), w, cx);
            }
        })
        .child(
            div()
                .relative()
                .w_full()
                .h(px(6.))
                .rounded(px(999.))
                .bg(t.well)
                .shadow(inset_well(&t))
                .child(canvas(move |b, _, _| measure.set(b), |_, _, _, _| {}).absolute().size_full())
                .child(div().h_full().w(relative(v)).rounded(px(999.)).bg(bg_grad(t.grad_a, t.grad_b)))
                .child(
                    div()
                        .absolute()
                        .top(px(-8.))
                        .left(relative(v))
                        .ml(px(-11.))
                        .size(px(22.))
                        .rounded(px(999.))
                        .bg(hexc(0xffffff))
                        .shadow(vec![BoxShadow {
                            color: c(10, 20, 40, 0.30),
                            offset: point(px(0.), px(2.)),
                            blur_radius: px(6.),
                            spread_radius: px(0.),
                            inset: false,
                        }]),
                ),
        )
}

// ───────────────────────── labels, chips, cards ─────────────────────────
pub fn section_label(t: &Theme, s: &str) -> Div {
    div()
        .px(px(12.))
        .pt(px(14.))
        .pb(px(6.))
        .text_size(px(LABEL))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(t.ink4)
        .child(s.to_uppercase())
}

pub fn chip(t: &Theme, label: &str, tone: Hsla) -> Div {
    div()
        .flex()
        .flex_none()
        .items_center()
        .h(px(22.))
        .px(px(9.))
        .rounded(px(999.))
        .bg(alpha(tone, if t.dark { 0.18 } else { 0.12 }))
        .text_size(px(META))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(tone)
        .child(label.to_string())
}

pub fn kbd(t: &Theme, s: &str) -> Div {
    div()
        .flex()
        .items_center()
        .h(px(20.))
        .px(px(6.))
        .rounded(px(6.))
        .bg(t.well)
        .text_size(px(LABEL))
        .font_weight(FontWeight::MEDIUM)
        .text_color(t.ink3)
        .child(s.to_string())
}

pub fn card(t: &Theme) -> Div {
    div().rounded(px(16.)).bg(bg_grad(t.card_top, t.card_bottom)).shadow(elev(t, 1))
}

pub fn spinner(t: &Theme, id: &'static str) -> impl IntoElement {
    icon("refresh", IV::Lin, 18., t.accent).with_animation(
        id,
        Animation::new(std::time::Duration::from_millis(1100)).repeat(),
        |el, d| el.with_transformation(Transformation::rotate(percentage(d))),
    )
}

// ───────────────────────── menus ─────────────────────────
pub fn menu_panel(t: &Theme, w: f32) -> Div {
    div()
        .flex()
        .flex_col()
        .w(px(w))
        .p(px(7.))
        .rounded(px(18.))
        .bg(t.menu)
        .shadow(elev(t, 4))
        .text_size(px(BODY))
        .text_color(t.ink1)
        .occlude()
}

pub fn menu_sep(t: &Theme) -> Div {
    div().h(px(1.)).mx(px(8.)).my(px(5.)).bg(alpha(t.ink1, 0.07))
}

pub fn menu_item(id: impl Into<ElementId>, t: &Theme, ic: &str, label: &str, shortcut: Option<&str>, danger: bool) -> Stateful<Div> {
    let t = *t;
    let fg = if danger { t.danger } else { t.ink1 };
    let hov = if danger { t.danger_tint } else { t.accent_tint };
    let mut d = div()
        .id(id)
        .flex()
        .items_center()
        .gap(px(10.))
        .h(px(33.))
        .px(px(10.))
        .rounded(px(9.))
        .cursor_pointer()
        .text_color(fg)
        .font_weight(FontWeight::MEDIUM)
        .hover(move |s| s.bg(hov))
        .child(icon(ic, IV::Lin, 17., if danger { t.danger } else { t.ink3 }))
        .child(div().flex_1().child(label.to_string()));
    if let Some(s) = shortcut {
        d = d.child(div().text_size(px(META)).text_color(t.ink4).child(s.to_string()));
    }
    d
}

// ───────────────────────── text input helper ─────────────────────────
/// Applies a key press to a line of text. Returns `Some(true)` on Enter, `Some(false)` when text changed.
pub fn edit_text(text: &mut String, ev: &KeyDownEvent) -> Option<bool> {
    let ks = &ev.keystroke;
    if ks.modifiers.control || ks.modifiers.platform || ks.modifiers.alt {
        return None;
    }
    match ks.key.as_str() {
        "enter" => Some(true),
        "backspace" => {
            text.pop();
            Some(false)
        }
        _ => {
            if let Some(ch) = &ks.key_char {
                if !ch.chars().any(|c| c.is_control()) {
                    text.push_str(ch);
                    return Some(false);
                }
            }
            None
        }
    }
}

pub fn caret(t: &Theme) -> Div {
    div().w(px(1.5)).h(px(15.)).ml(px(1.)).rounded(px(1.)).bg(t.accent)
}

pub fn search_field(t: &Theme, ic: &str, text: &str, placeholder: &str, focused: bool, w: f32) -> Div {
    let t = *t;
    let mut d = div()
        .flex()
        .flex_none()
        .items_center()
        .gap(px(8.))
        .h(px(36.))
        .w(px(w))
        .px(px(13.))
        .rounded(px(999.))
        .bg(t.field)
        .shadow(if focused {
            let mut s = inset_well(&t);
            s.push(BoxShadow { color: alpha(t.accent, 0.45), offset: point(px(0.), px(0.)), blur_radius: px(0.), spread_radius: px(2.), inset: false });
            s
        } else {
            inset_well(&t)
        })
        .text_size(px(BODY))
        .font_weight(FontWeight::MEDIUM)
        .overflow_hidden()
        .child(icon(ic, IV::Lin, 16., if focused { t.accent } else { t.ink4 }));
    if text.is_empty() {
        d = d.child(div().text_color(t.ink4).child(placeholder.to_string()));
        if focused {
            d = d.child(caret(&t));
        }
    } else {
        d = d.child(div().text_color(t.ink1).whitespace_nowrap().child(text.to_string()));
        if focused {
            d = d.child(caret(&t));
        }
    }
    d
}
