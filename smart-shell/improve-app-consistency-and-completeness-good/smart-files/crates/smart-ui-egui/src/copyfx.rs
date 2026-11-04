//! Transfer dialog with a hand-drawn "files flying folder to folder" animation — a nod to the Windows XP copy
//! dialog, rebuilt with springs. Sheets leave the source folder on a lifted arc, tumble through the air and drop
//! into the destination, whose flap opens to catch them and squashes on every landing.
use crate::app::*;
use crate::chrome::*;
use crate::overlays::win_rect;
use crate::paint::*;
use crate::theme::*;
use egui::{pos2, vec2, Align2, Color32, Context, Id, Order, Painter, Pos2, Rect, Sense, Shape, Stroke, Vec2};
use smart_core::*;

fn rot(c: Pos2, v: Vec2, ang: f32) -> Pos2 {
    let (s, co) = ang.sin_cos();
    c + vec2(v.x * co - v.y * s, v.x * s + v.y * co)
}

fn smooth(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// A sheet of paper with a folded corner and ruled lines, rotated about `c`.
pub fn sheet(p: &Painter, c: Pos2, w: f32, h: f32, ang: f32, body: Color32, ink: Color32, a: f32) {
    if a <= 0.01 {
        return;
    }
    let (hw, hh, fold) = (w / 2.0, h / 2.0, w * 0.3);
    let pts = |dx: f32, dy: f32| -> Vec<Pos2> { [(-hw, -hh), (hw - fold, -hh), (hw, -hh + fold), (hw, hh), (-hw, hh)].iter().map(|(x, y)| rot(c, vec2(*x + dx, *y + dy), ang)).collect() };
    p.add(Shape::convex_polygon(pts(0.0, 2.0), alpha(Color32::BLACK, 0.20 * a), Stroke::NONE));
    p.add(Shape::convex_polygon(pts(0.0, 0.0), alpha(body, a), Stroke::new(1.0, alpha(ink, 0.35 * a))));
    let tri: Vec<Pos2> = [(hw - fold, -hh), (hw, -hh + fold), (hw - fold, -hh + fold)].iter().map(|(x, y)| rot(c, vec2(*x, *y), ang)).collect();
    p.add(Shape::convex_polygon(tri, alpha(mix(body, ink, 0.25), a), Stroke::new(1.0, alpha(ink, 0.3 * a))));
    for k in 0..3 {
        let y = -hh + h * (0.46 + 0.16 * k as f32);
        let x1 = hw - w * 0.18 - if k == 2 { w * 0.28 } else { 0.0 };
        p.line_segment([rot(c, vec2(-hw + w * 0.18, y), ang), rot(c, vec2(x1, y), ang)], Stroke::new(1.4, alpha(ink, 0.45 * a)));
    }
}

/// A folder whose flap opens (`open` 0..1) and which squashes (`squash` ±) like jelly.
fn folder(p: &Painter, th: &Theme, ctr: Pos2, w: f32, h: f32, open: f32, squash: f32, hue: Color32) {
    let sy = 1.0 - 0.07 * squash;
    let sx = 1.0 + 0.05 * squash;
    let r = Rect::from_center_size(pos2(ctr.x, ctr.y + h * (1.0 - sy) / 2.0), vec2(w * sx, h * sy));
    shadow(p, r.translate(vec2(0.0, 4.0)), 12.0, 6.0, 16.0, -4.0, alpha(th.shadow, if th.dark { 0.6 } else { 0.28 }));
    let back = Rect::from_min_max(pos2(r.left(), r.top() + h * 0.10), r.max);
    grad_fill(p, back, 12.0, mix(hue, Color32::BLACK, 0.22), mix(hue, Color32::BLACK, 0.32));
    let tab = Rect::from_min_size(pos2(r.left(), r.top()), vec2(w * 0.38 * sx, h * 0.22));
    grad_fill(p, tab, 8.0, mix(hue, Color32::BLACK, 0.12), mix(hue, Color32::BLACK, 0.22));
    // dark mouth that appears as the flap opens
    let mouth = Rect::from_min_max(pos2(r.left() + 6.0, back.top() + 5.0), pos2(r.right() - 6.0, back.top() + 5.0 + h * 0.34));
    grad_fill(p, mouth, 6.0, alpha(Color32::BLACK, 0.50 * open), alpha(Color32::BLACK, 0.0));
}

fn folder_front(p: &Painter, ctr: Pos2, w: f32, h: f32, open: f32, squash: f32, hue: Color32) {
    let sy = 1.0 - 0.07 * squash;
    let sx = 1.0 + 0.05 * squash;
    let r = Rect::from_center_size(pos2(ctr.x, ctr.y + h * (1.0 - sy) / 2.0), vec2(w * sx, h * sy));
    let top = r.top() + h * (0.34 + 0.07 * open);
    let front = Rect::from_min_max(pos2(r.left(), top), r.max);
    grad_fill(p, front, 12.0, mix(hue, Color32::WHITE, 0.30), mix(hue, Color32::WHITE, 0.04));
    rim(p, front, 12.0, 1.4, Color32::from_white_alpha(150), true, 26.0);
    rim(p, front, 12.0, 1.0, Color32::from_black_alpha(40), false, 6.0);
}

impl SmartApp {
    pub(crate) fn m_copy_progress(&mut self, ctx: &Context) {
        let Some(id) = self.copy_dialog else { return };
        let Some(job) = self.jobs.iter().find(|j| j.id == id).map(|j| j.clone_job_view()) else {
            self.copy_dialog = None;
            return;
        };
        ctx.request_repaint();
        let win = win_rect();
        let th = self.th.clone();
        let dt = ctx.input(|i| i.stable_dt).min(0.1);
        self.copy_dialog_anim += dt;
        let anim = self.copy_dialog_anim;
        let info = self.last_transfer.clone();
        let frac = (job.done as f32 / job.total.max(1) as f32).clamp(0.0, 1.0);
        let sf = spring_to(ctx, "cp-frac", frac, 90.0, 16.0).clamp(0.0, 1.0);
        let pop = spring_out(anim);
        let is_move = job.title.to_lowercase().starts_with("mov");
        egui::Area::new(Id::unique("copy-dialog")).order(Order::Foreground).fixed_pos(win.min).show(ctx, |ui| {
            let p = ui.painter().clone();
            ui.set_clip_rect(win);
            ui.set_opacity(smooth(anim * 6.0));
            let _scrim = ui.allocate_rect(win, Sense::click());
            grad_fill(&p, win, R_WIN, alpha(th.scrim, 0.55), th.scrim);
            let w = 540.0f32.min(win.width() - 32.0);
            let h = 286.0f32.min(win.height() - 32.0);
            let card = Rect::from_center_size(win.center() + vec2(0.0, (1.0 - pop) * 18.0), vec2(w, h));
            menu_surface(&p, &th, card, 22.0);
            ui.allocate_rect(card, Sense::click());
            let pad = 22.0;

            // ── header: ring orb + names (everything is measured, nothing shares a baseline) ──
            let orb = Rect::from_min_size(card.min + vec2(pad, 18.0), vec2(46.0, 46.0));
            let (a, b) = th.accent_grad();
            shadow(&p, orb, 23.0, 6.0, 16.0, -4.0, alpha(th.accent, 0.4));
            grad_fill(&p, orb, 23.0, a, b);
            rim(&p, orb, 23.0, 1.4, Color32::from_white_alpha(150), true, 30.0);
            let ctr = orb.center();
            let mut prev = pos2(ctr.x, ctr.y - 18.0);
            for i in 1..=60 {
                let ang = -std::f32::consts::FRAC_PI_2 + (i as f32 / 60.0) * std::f32::consts::TAU;
                let cur = pos2(ctr.x + ang.cos() * 18.0, ctr.y + ang.sin() * 18.0);
                p.line_segment([prev, cur], Stroke::new(2.2, if i as f32 / 60.0 <= sf { Color32::WHITE } else { Color32::from_white_alpha(70) }));
                prev = cur;
            }
            p.text(ctr, Align2::CENTER_CENTER, format!("{}", (sf * 100.0).round() as u32), f_bold(13.0), Color32::WHITE);
            let first = info.as_ref().and_then(|(s, _)| s.first()).cloned();
            let name = first.as_ref().map(|l| l.name()).unwrap_or_else(|| job.title.clone());
            let size = first.as_ref().and_then(|l| self.be.stat(l).ok()).map(|e| e.size).unwrap_or(5_000_000_000);
            let dest = info.as_ref().map(|(_, d)| d.clone()).unwrap_or_else(|| self.target_dir());
            let tx = orb.right() + 14.0;
            let paced = first.as_ref().is_some_and(|l| self.be.stat(l).map(|e| e.paced).unwrap_or(false));
            let badge_w = if paced { 84.0 } else { 0.0 };
            p.text(pos2(tx, card.top() + 32.0), Align2::LEFT_CENTER, self.fit(&p, &name, &f_bold(T_TITLE), card.right() - tx - pad - badge_w - 8.0), f_bold(T_TITLE), th.ink1);
            let verb = if is_move { "Moving" } else { "Copying" };
            p.text(pos2(tx, card.top() + 53.0), Align2::LEFT_CENTER, self.fit(&p, &format!("{verb} {} to {}", format_size(size), self.pretty_path(&dest)), &f_medium(T_META), card.right() - tx - pad), f_medium(T_META), th.ink3);
            if paced {
                let badge = Rect::from_min_size(pos2(card.right() - pad - 76.0, card.top() + 22.0), vec2(76.0, 20.0));
                grad_fill(&p, badge, 10.0, alpha(th.accent, 0.20), alpha(th.accent, 0.10));
                p.text(badge.center(), Align2::CENTER_CENTER, &format!("{} DEMO", format_size(size).to_uppercase()), f_semi(9.0), th.accent_ink);
            }

            // ── stage: folder → arcing sheets → folder ──
            let stage = Rect::from_min_max(pos2(card.left() + pad, card.top() + 76.0), pos2(card.right() - pad, card.top() + 176.0));
            let well_r = stage;
            grad_fill(&p, well_r, 16.0, alpha(th.accent, if th.dark { 0.10 } else { 0.07 }), alpha(th.accent, if th.dark { 0.03 } else { 0.02 }));
            rim(&p, well_r, 16.0, 2.0, alpha(th.shadow, if th.dark { 0.35 } else { 0.07 }), true, 12.0);
            let sp = p.with_clip_rect(stage);
            let (fw, fh) = (58.0, 44.0);
            let src_c = pos2(stage.left() + 56.0, stage.center().y + 6.0);
            let dst_c = pos2(stage.right() - 56.0, stage.center().y + 6.0);
            let hue_src = mix(th.ink3, th.accent, 0.35);
            let hue_dst = th.accent;
            const N: usize = 5;
            const T: f32 = 1.6;
            let cols = [th.accent, th.good, th.warn, hex("#d86fb0"), hex("#8f7cf5")];
            // per-sheet phase, derived landing impulses
            let mut dst_bump = 0.0f32;
            let mut src_bump = 0.0f32;
            let mut dst_open = 0.25f32;
            let mut src_open = 0.25f32;
            let mut flying: Vec<(f32, usize)> = vec![];
            let speed_k = 0.75 + 0.25 * (sf * 6.0).min(1.0);
            for k in 0..N {
                let tt = ((anim * speed_k / T) + k as f32 / N as f32).fract();
                dst_open = dst_open.max(1.0 - ((tt - 0.86).abs() / 0.18).clamp(0.0, 1.0));
                src_open = src_open.max(1.0 - ((tt - 0.07).abs() / 0.14).clamp(0.0, 1.0));
                let since_land = ((tt - 0.90).rem_euclid(1.0)) * T;
                if since_land < 0.9 {
                    dst_bump += (-since_land * 7.5).exp() * (since_land * 30.0).sin();
                }
                let since_lift = ((tt - 0.02).rem_euclid(1.0)) * T;
                if since_lift < 0.6 {
                    src_bump -= (-since_lift * 9.0).exp() * (since_lift * 26.0).sin() * 0.6;
                }
                flying.push((tt, k));
            }
            let mouth_src = pos2(src_c.x, src_c.y - fh * 0.30);
            let mouth_dst = pos2(dst_c.x, dst_c.y - fh * 0.30);
            // back halves → sheets → front halves gives the "drops into the folder" occlusion for free
            folder(&sp, &th, src_c, fw, fh, src_open, src_bump, hue_src);
            folder(&sp, &th, dst_c, fw, fh, dst_open, dst_bump, hue_dst);
            // faint guide arc
            let steps = 28;
            let arc_h = 34.0;
            for i in 0..steps {
                let (t0, t1) = (i as f32 / steps as f32, (i as f32 + 0.5) / steps as f32);
                let pt = |t: f32| pos2(mouth_src.x + (mouth_dst.x - mouth_src.x) * t, mouth_src.y - arc_h * 4.0 * t * (1.0 - t));
                if i % 2 == 0 {
                    sp.line_segment([pt(t0), pt(t1)], Stroke::new(1.0, alpha(th.accent, 0.14)));
                }
            }
            flying.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            for (tt, k) in &flying {
                let t = *tt;
                let e = smooth(t * 0.9 + 0.05);
                let x = mouth_src.x + (mouth_dst.x - mouth_src.x) * e;
                let lift = (4.0 * t * (1.0 - t)).powf(0.8);
                let y = mouth_src.y + 14.0 - (arc_h + 14.0) * lift + if t > 0.84 { (t - 0.84) / 0.16 * 26.0 * smooth((t - 0.84) / 0.16) } else { 0.0 };
                let tumble = (t - 0.5) * (1.1 + (*k as f32 * 1.7).sin() * 0.5) + (*k as f32 * 2.3).sin() * 0.25;
                let sc = 0.62 + 0.38 * (std::f32::consts::PI * t).sin().powf(0.5);
                let fade = smooth(t * 9.0) * smooth((1.0 - t) * 18.0 + 0.25);
                sheet(&sp, pos2(x, y), 20.0 * sc, 26.0 * sc, tumble, mix(Color32::WHITE, cols[*k % cols.len()], 0.14), mix(th.ink1, cols[*k % cols.len()], 0.6), fade);
                // sparkle trail
                for s in 1..=2 {
                    let tp = (t - s as f32 * 0.035).max(0.0);
                    let ep = smooth(tp * 0.9 + 0.05);
                    let px = mouth_src.x + (mouth_dst.x - mouth_src.x) * ep;
                    let py = mouth_src.y + 4.0 - arc_h * (4.0 * tp * (1.0 - tp)).powf(0.8);
                    sp.circle_filled(pos2(px, py + 6.0), 2.2 - s as f32 * 0.6, alpha(cols[*k % cols.len()], 0.35 * fade / s as f32));
                }
                // landing sparkles
                let since = ((t - 0.90).rem_euclid(1.0)) * T;
                if since < 0.45 {
                    for d in 0..5 {
                        let ang = -std::f32::consts::PI * (0.15 + 0.7 * d as f32 / 4.0);
                        let rr = since * 70.0;
                        sp.circle_filled(mouth_dst + vec2(ang.cos() * rr, ang.sin() * rr + 4.0), 1.9 * (1.0 - since / 0.45), alpha(cols[(*k + d) % cols.len()], 0.9 * (1.0 - since / 0.45)));
                    }
                }
            }
            folder_front(&sp, src_c, fw, fh, src_open, src_bump, hue_src);
            folder_front(&sp, dst_c, fw, fh, dst_open, dst_bump, hue_dst);
            let src_label = first.as_ref().map(|l| if l.fs == dest.fs { l.parent().map(|p| p.name()).unwrap_or_else(|| "Source".into()) } else { "Source".into() }).unwrap_or_else(|| "Source".into());
            p.text(pos2(src_c.x, stage.bottom() - 12.0), Align2::CENTER_CENTER, self.fit(&p, &src_label, &f_semi(10.5), 96.0), f_semi(10.5), th.ink3);
            p.text(pos2(dst_c.x, stage.bottom() - 12.0), Align2::CENTER_CENTER, self.fit(&p, &dest.name(), &f_semi(10.5), 96.0), f_semi(10.5), th.accent_ink);

            // ── progress ──
            let bar = Rect::from_min_size(pos2(card.left() + pad, card.top() + 192.0), vec2(w - 2.0 * pad, 10.0));
            well(&p, &th, bar, 5.0);
            let fill = Rect::from_min_size(bar.min, vec2((bar.width() * sf).max(10.0), 10.0));
            let (a2, b2) = th.accent_grad();
            grad_fill(&p, fill, 5.0, a2, b2);
            rim(&p, fill, 5.0, 1.0, Color32::from_white_alpha(120), true, 6.0);
            let band_x = bar.left() + ((anim * 170.0) % (bar.width() + 120.0)) - 60.0;
            let band = fill.intersect(Rect::from_min_size(pos2(band_x, bar.top()), vec2(60.0, 10.0)));
            if band.width() > 1.0 {
                grad_fill(&p, band, 4.0, Color32::from_white_alpha(80), Color32::from_white_alpha(0));
            }
            let elapsed = job.age.max(0.1);
            let speed = if frac > 0.02 { size as f32 * frac / elapsed } else { 0.0 };
            let remaining = if frac > 0.02 { elapsed * (1.0 - frac) / frac } else { 0.0 };
            let stats = format!("{} of {}  \u{b7}  {}/s  \u{b7}  {}", format_size((size as f32 * frac) as u64), format_size(size), format_size(speed.max(1.0) as u64), if remaining > 0.0 { format!("{}s left", remaining.ceil() as u32) } else { "estimating\u{2026}".into() });
            p.text(pos2(card.left() + pad, bar.bottom() + 16.0), Align2::LEFT_CENTER, stats, f_medium(T_META), th.ink3);
            p.text(pos2(card.right() - pad, bar.bottom() + 16.0), Align2::RIGHT_CENTER, self.fit(&p, &job.current.clone().unwrap_or_else(|| job.title.clone()), &f_semi(T_META), 150.0), f_semi(T_META), th.ink2);

            // ── footer ──
            let by = card.bottom() - 18.0 - 32.0;
            if self.label_button(ui, Rect::from_min_size(pos2(card.right() - pad - 140.0 - 8.0 - 82.0, by), vec2(82.0, 32.0)), None, "Cancel", "cp-cancel", Btn::Danger).clicked() {
                self.acts.push(Act::CopyDialogClose);
            }
            if self.label_button(ui, Rect::from_min_size(pos2(card.right() - pad - 140.0, by), vec2(140.0, 32.0)), None, "Run in background", "cp-bg", Btn::Accent).clicked() {
                self.copy_dialog = None;
            }
            p.text(pos2(card.left() + pad, by + 16.0), Align2::LEFT_CENTER, "Keep working \u{2014} it continues behind", f_medium(10.5), th.ink4);
        });
    }
}
