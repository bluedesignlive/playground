//! Audio playback. Native: rodio (feature `audio`). Web: HTMLAudioElement. Otherwise a silent stub.
#![allow(unused_variables)]

pub struct Audio {
    inner: imp::Inner,
}

impl Audio {
    pub fn new() -> Self {
        Self { inner: imp::Inner::new() }
    }
    pub fn play(&mut self, bytes: Vec<u8>, file_name: &str) -> Result<(), String> {
        self.inner.play(bytes, file_name)
    }
    pub fn pause(&mut self) {
        self.inner.pause()
    }
    pub fn resume(&mut self) {
        self.inner.resume()
    }
    pub fn stop(&mut self) {
        self.inner.stop()
    }
    pub fn is_playing(&self) -> bool {
        self.inner.is_playing()
    }
    pub fn is_loaded(&self) -> bool {
        self.inner.is_loaded()
    }
    pub fn position(&self) -> f32 {
        self.inner.position()
    }
    pub fn duration(&self) -> Option<f32> {
        self.inner.duration()
    }
    pub fn seek(&mut self, secs: f32) {
        self.inner.seek(secs)
    }
    pub fn set_volume(&mut self, v: f32) {
        self.inner.set_volume(v)
    }
}
impl Default for Audio {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(all(feature = "audio", not(target_arch = "wasm32")))]
mod imp {
    use rodio::{Decoder, Source};
    use std::io::Cursor;
    use std::time::Duration;

    pub struct Inner {
        stream: Option<rodio::MixerDeviceSink>,
        sink: Option<rodio::Player>,
        dur: Option<f32>,
        vol: f32,
    }
    impl Inner {
        pub fn new() -> Self {
            Self { stream: None, sink: None, dur: None, vol: 0.9 }
        }
        pub fn play(&mut self, bytes: Vec<u8>, _name: &str) -> Result<(), String> {
            if self.stream.is_none() {
                let mut s = rodio::DeviceSinkBuilder::open_default_sink().map_err(|e| format!("No audio output: {e}"))?;
                s.log_on_drop(false);
                self.stream = Some(s);
            }
            let stream = self.stream.as_ref().unwrap();
            let src = Decoder::try_from(Cursor::new(bytes)).map_err(|e| format!("Can’t decode audio: {e}"))?;
            self.dur = src.total_duration().map(|d| d.as_secs_f32());
            let sink = rodio::Player::connect_new(stream.mixer());
            sink.set_volume(self.vol);
            sink.append(src);
            self.sink = Some(sink);
            Ok(())
        }
        pub fn pause(&mut self) {
            if let Some(s) = &self.sink { s.pause() }
        }
        pub fn resume(&mut self) {
            if let Some(s) = &self.sink { s.play() }
        }
        pub fn stop(&mut self) {
            if let Some(s) = self.sink.take() { s.stop() }
        }
        pub fn is_playing(&self) -> bool {
            self.sink.as_ref().is_some_and(|s| !s.is_paused() && !s.empty())
        }
        pub fn is_loaded(&self) -> bool {
            self.sink.as_ref().is_some_and(|s| !s.empty())
        }
        pub fn position(&self) -> f32 {
            self.sink.as_ref().map(|s| s.get_pos().as_secs_f32()).unwrap_or(0.0)
        }
        pub fn duration(&self) -> Option<f32> {
            self.dur
        }
        pub fn seek(&mut self, secs: f32) {
            if let Some(s) = &self.sink { let _ = s.try_seek(Duration::from_secs_f32(secs.max(0.0))); }
        }
        pub fn set_volume(&mut self, v: f32) {
            self.vol = v;
            if let Some(s) = &self.sink { s.set_volume(v) }
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod imp {
    use wasm_bindgen::JsValue;
    use web_sys::{Blob, BlobPropertyBag, HtmlAudioElement, Url};

    pub struct Inner {
        el: Option<HtmlAudioElement>,
        url: Option<String>,
        vol: f64,
    }
    impl Inner {
        pub fn new() -> Self {
            Self { el: None, url: None, vol: 0.9 }
        }
        pub fn play(&mut self, bytes: Vec<u8>, name: &str) -> Result<(), String> {
            self.stop();
            let mime = crate::apps::mime_of(name);
            let arr = js_sys::Uint8Array::from(&bytes[..]);
            let parts = js_sys::Array::new();
            parts.push(&arr);
            let opts = BlobPropertyBag::new();
            opts.set_type(&mime);
            let blob = Blob::new_with_u8_array_sequence_and_options(&parts, &opts).map_err(|_| "blob failed".to_string())?;
            let url = Url::create_object_url_with_blob(&blob).map_err(|_| "url failed".to_string())?;
            let el = HtmlAudioElement::new_with_src(&url).map_err(|_| "audio failed".to_string())?;
            el.set_volume(self.vol);
            let _ = el.play().map_err(|e: JsValue| format!("{:?}", e))?;
            self.el = Some(el);
            self.url = Some(url);
            Ok(())
        }
        pub fn pause(&mut self) {
            if let Some(e) = &self.el { let _ = e.pause(); }
        }
        pub fn resume(&mut self) {
            if let Some(e) = &self.el { let _ = e.play(); }
        }
        pub fn stop(&mut self) {
            if let Some(e) = self.el.take() { let _ = e.pause(); }
            if let Some(u) = self.url.take() { let _ = Url::revoke_object_url(&u); }
        }
        pub fn is_playing(&self) -> bool {
            self.el.as_ref().is_some_and(|e| !e.paused() && !e.ended())
        }
        pub fn is_loaded(&self) -> bool {
            self.el.as_ref().is_some_and(|e| !e.ended())
        }
        pub fn position(&self) -> f32 {
            self.el.as_ref().map(|e| e.current_time() as f32).unwrap_or(0.0)
        }
        pub fn duration(&self) -> Option<f32> {
            self.el.as_ref().map(|e| e.duration() as f32).filter(|d| d.is_finite())
        }
        pub fn seek(&mut self, secs: f32) {
            if let Some(e) = &self.el { e.set_current_time(secs as f64); }
        }
        pub fn set_volume(&mut self, v: f32) {
            self.vol = v as f64;
            if let Some(e) = &self.el { e.set_volume(v as f64); }
        }
    }
}

#[cfg(not(any(all(feature = "audio", not(target_arch = "wasm32")), target_arch = "wasm32")))]
mod imp {
    pub struct Inner;
    impl Inner {
        pub fn new() -> Self { Inner }
        pub fn play(&mut self, _b: Vec<u8>, _n: &str) -> Result<(), String> { Err("Audio output is disabled in this build".into()) }
        pub fn pause(&mut self) {}
        pub fn resume(&mut self) {}
        pub fn stop(&mut self) {}
        pub fn is_playing(&self) -> bool { false }
        pub fn is_loaded(&self) -> bool { false }
        pub fn position(&self) -> f32 { 0.0 }
        pub fn duration(&self) -> Option<f32> { None }
        pub fn seek(&mut self, _s: f32) {}
        pub fn set_volume(&mut self, _v: f32) {}
    }
}
