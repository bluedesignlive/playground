//! Smart Shell — a clean GPUI (gpui-kit) implementation, compiled to WebAssembly.
//! No JavaScript or TypeScript is hand-written anywhere: the only script in the page is the
//! wasm-bindgen bootstrap generated at build time.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

mod assets;
mod data;
mod files;
mod kit;
mod photos;
mod shell;
mod sig;
mod terminal;
mod theme;
mod ui;

use gpui_kit::*;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

thread_local! {
    static APP: RefCell<Option<ApplicationHandle>> = const { RefCell::new(None) };
}

#[wasm_bindgen]
pub fn start() {
    console_error_panic_hook::set_once();
    gpui_kit::platform::web_init();

    // Single-threaded web platform (no SharedArrayBuffer / cross-origin isolation required).
    let platform = Rc::new(gpui_kit::web::WebPlatform::new_with_backend(false, gpui_kit::web::WebBackendPreference::Auto));
    let app = Application::with_platform(platform).with_assets(assets::Assets);

    let handle = app.run_embedded(|cx: &mut App| {
        gpui_kit::init(cx);
        assets::install_fonts(cx);
        cx.set_global(theme::ThemeG(theme::Theme::light()));
        cx.set_global(sig::SigG(Rc::new(RefCell::new(sig::Sig::default()))));
        cx.set_global(data::FsG(Rc::new(RefCell::new(data::demo()))));

        gpui_kit::open_window(WindowOptions::default(), cx, |_, cx| cx.new(shell::Shell::new)).expect("open window");
        cx.activate(true);
    });
    APP.with(|a| *a.borrow_mut() = Some(handle));
}
