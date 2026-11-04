//! Solar icons (outline / duotone / filled) rasterised at build time, tinted at paint time.
use crate::icons_gen;
use egui::{Color32, Context, Painter, Rect, TextureHandle, TextureOptions};
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Ic {
    Outline,
    Duo,
    Filled,
}

pub struct Icons {
    pub ctx: Context,
    cache: RefCell<HashMap<(&'static str, u8, u16), Option<TextureHandle>>>,
}

impl Icons {
    pub fn new(ctx: &Context) -> Self {
        Self { ctx: ctx.clone(), cache: RefCell::new(HashMap::new()) }
    }

    pub fn paint(&self, p: &Painter, name: &'static str, v: Ic, rect: Rect, tint: Color32) {
        let ppp = self.ctx.pixels_per_point();
        let px = ((rect.width() * ppp).round() as u16).clamp(8, 256);
        let code = match v {
            Ic::Outline => b'o',
            Ic::Duo => b'd',
            Ic::Filled => b'f',
        };
        let mut cache = self.cache.borrow_mut();
        let entry = cache.entry((name, code, px)).or_insert_with(|| {
            let bytes = icons_gen::bytes(name, code)?;
            let img = image::load_from_memory(bytes).ok()?.to_rgba8();
            let img = if img.width() != px as u32 {
                image::imageops::resize(&img, px as u32, px as u32, image::imageops::FilterType::Lanczos3)
            } else {
                img
            };
            let ci = egui::ColorImage::from_rgba_unmultiplied([img.width() as usize, img.height() as usize], img.as_raw());
            Some(self.ctx.load_texture(format!("ic-{name}-{code}-{px}"), ci, TextureOptions::LINEAR))
        });
        if let Some(t) = entry {
            p.image(t.id(), rect, Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), tint);
        }
    }

    pub fn paint_at(&self, p: &Painter, name: &'static str, v: Ic, center: egui::Pos2, size: f32, tint: Color32) {
        self.paint(p, name, v, Rect::from_center_size(center, egui::vec2(size, size)), tint);
    }
}
