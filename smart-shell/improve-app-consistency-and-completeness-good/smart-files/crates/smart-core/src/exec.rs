//! Tiny execution shim: real threads on native, run-inline in the browser (no threads there).
pub const THREADED: bool = cfg!(not(target_arch = "wasm32"));

pub fn spawn<F: FnOnce() + Send + 'static>(f: F) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::thread::spawn(f);
    }
    #[cfg(target_arch = "wasm32")]
    {
        f();
    }
}

pub fn now_millis() -> i64 {
    chrono::Utc::now().timestamp_millis()
}
