//! Small shared signal bus between the shell and the app views.
use gpui_kit::*;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AppId {
    Files,
    Photos,
    Terminal,
    Kit,
}

impl AppId {
    pub fn name(self) -> &'static str {
        match self {
            AppId::Files => "Files",
            AppId::Photos => "Photos",
            AppId::Terminal => "Terminal",
            AppId::Kit => "Smart Kit",
        }
    }
    pub fn icon(self) -> &'static str {
        match self {
            AppId::Files => "folder-with-files",
            AppId::Photos => "gallery-wide",
            AppId::Terminal => "terminal",
            AppId::Kit => "widget",
        }
    }
}

#[derive(Default)]
pub struct Sig {
    pub drag: Option<(AppId, Point<Pixels>)>,
    pub toasts: Vec<String>,
    pub launch: Vec<AppId>,
    pub dark: Option<bool>,
}

pub struct SigG(pub Rc<RefCell<Sig>>);
impl Global for SigG {}

pub fn sig(cx: &App) -> Rc<RefCell<Sig>> {
    cx.global::<SigG>().0.clone()
}

pub fn toast(cx: &App, msg: impl Into<String>) {
    sig(cx).borrow_mut().toasts.push(msg.into());
}

pub fn start_drag(cx: &App, app: AppId, pos: Point<Pixels>) {
    sig(cx).borrow_mut().drag = Some((app, pos));
}

pub fn now_ms() -> f64 {
    js_sys::Date::now()
}
