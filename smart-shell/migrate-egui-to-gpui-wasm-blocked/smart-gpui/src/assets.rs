//! Embedded assets: Solar icons (linear / duotone / bold), Inter + JetBrains Mono, wallpapers.
use gpui_kit::*;
use include_dir::{Dir, include_dir};
use std::borrow::Cow;

static ICONS: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets/icons");
static THUMBS: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets/thumbs");

pub struct Assets;

/// Real photography for the demo Pictures folder.
pub fn thumb_path(name: &str) -> Option<&'static str> {
    Some(match name {
        "4K Frosted Glass.jpg" => "thumbs/frosted.jpg",
        "Blossom Study.jpg" => "thumbs/blossom.jpg",
        "Cycling in Soweto.png" => "thumbs/cycling.jpg",
        "Green Fold.png" => "thumbs/greenfold.jpg",
        "Nixio Dashboard.png" => "thumbs/dash.jpg",
        "Orbit Poster.png" => "thumbs/orbit.jpg",
        "Soweto Moto Ad.png" => "thumbs/moto.jpg",
        "Window Study.png" => "thumbs/swatches.jpg",
        "Pack Cover.jpg" => "thumbs/orbit.jpg",
        "Landing Draft.png" => "thumbs/dash.jpg",
        "Package B.png" => "thumbs/swatches.jpg",
        "Reference 01.jpg" => "thumbs/blossom.jpg",
        "Layout Study.png" => "thumbs/greenfold.jpg",
        _ => return None,
    })
}

impl AssetSource for Assets {
    fn load(&self, path: &str) -> gpui_kit::Result<Option<Cow<'static, [u8]>>> {
        if let Some(name) = path.strip_prefix("icons/") {
            return Ok(ICONS.get_file(name).map(|f| Cow::Borrowed(f.contents())));
        }
        if let Some(name) = path.strip_prefix("thumbs/") {
            return Ok(THUMBS.get_file(name).map(|f| Cow::Borrowed(f.contents())));
        }
        Ok(match path {
            "wallpapers/light.jpg" => Some(Cow::Borrowed(include_bytes!("../assets/wallpapers/light.jpg").as_slice())),
            "wallpapers/dark.jpg" => Some(Cow::Borrowed(include_bytes!("../assets/wallpapers/dark.jpg").as_slice())),
            _ => None,
        })
    }
    fn list(&self, _path: &str) -> gpui_kit::Result<Vec<SharedString>> {
        Ok(vec![])
    }
}

pub fn install_fonts(cx: &mut App) {
    let fonts: Vec<Cow<'static, [u8]>> = vec![
        Cow::Borrowed(include_bytes!("../assets/fonts/Inter-Regular.ttf").as_slice()),
        Cow::Borrowed(include_bytes!("../assets/fonts/Inter-Medium.ttf").as_slice()),
        Cow::Borrowed(include_bytes!("../assets/fonts/Inter-SemiBold.ttf").as_slice()),
        Cow::Borrowed(include_bytes!("../assets/fonts/Inter-Bold.ttf").as_slice()),
        Cow::Borrowed(include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf").as_slice()),
        Cow::Borrowed(include_bytes!("../assets/fonts/JetBrainsMono-Bold.ttf").as_slice()),
        // the web platform resolves `.SystemUIFont` to IBM Plex Sans; it must exist
        Cow::Borrowed(include_bytes!("../assets/fonts/IBMPlexSans-Regular.ttf").as_slice()),
    ];
    cx.text_system().add_fonts(fonts).expect("fonts");
}

#[derive(Clone, Copy, PartialEq)]
pub enum IV {
    Lin,
    Duo,
    Bold,
}

pub fn icon(name: &str, v: IV, size: f32, color: Hsla) -> Svg {
    let suffix = match v {
        IV::Lin => "",
        IV::Duo => "-d",
        IV::Bold => "-f",
    };
    svg()
        .path(format!("icons/{name}{suffix}.svg"))
        .size(px(size))
        .flex_none()
        .text_color(color)
}
