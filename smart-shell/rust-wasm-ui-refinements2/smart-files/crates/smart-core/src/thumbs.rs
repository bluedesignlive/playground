//! Background thumbnail decoding (image crate). Frontends upload the RGBA to a texture.
use crate::exec::THREADED;
use crate::model::Location;
use crate::ops::Registry;
use std::collections::HashSet;
use std::sync::{Arc, Condvar, Mutex};

pub struct ThumbResult {
    pub loc: Location,
    pub size: [usize; 2],
    pub rgba: Vec<u8>,
    pub original: Option<(u32, u32)>,
    pub edge: u32,
}

#[derive(Default)]
struct Shared {
    queue: Mutex<Vec<(Location, u32)>>,
    cv: Condvar,
    out: Mutex<Vec<ThumbResult>>,
}

pub struct Thumbs {
    shared: Arc<Shared>,
    registry: Arc<Registry>,
    seen: Mutex<HashSet<(Location, u32)>>,
}

fn decode(reg: &Registry, loc: &Location, edge: u32) -> Option<ThumbResult> {
    let v = reg.get(&loc.fs)?;
    let bytes = v.read(&loc.path, 48 << 20).ok()?;
    let img = image::load_from_memory(&bytes).ok()?;
    let original = Some((img.width(), img.height()));
    let t = img.thumbnail(edge, edge).to_rgba8();
    Some(ThumbResult { loc: loc.clone(), size: [t.width() as usize, t.height() as usize], rgba: t.into_raw(), original, edge })
}

impl Thumbs {
    pub fn new(registry: Arc<Registry>) -> Self {
        let shared = Arc::new(Shared::default());
        if THREADED {
            let n = 2;
            for _ in 0..n {
                let (s, r) = (shared.clone(), registry.clone());
                crate::exec::spawn(move || loop {
                    let job = {
                        let mut q = s.queue.lock().unwrap();
                        while q.is_empty() {
                            q = s.cv.wait(q).unwrap();
                        }
                        q.pop() // newest first = what is on screen now
                    };
                    if let Some((loc, edge)) = job {
                        let res = decode(&r, &loc, edge).unwrap_or(ThumbResult { loc, size: [0, 0], rgba: vec![], original: None, edge });
                        s.out.lock().unwrap().push(res);
                    }
                });
            }
        }
        Self { shared, registry, seen: Mutex::new(HashSet::new()) }
    }

    pub fn request(&self, loc: &Location, edge: u32) {
        if !self.seen.lock().unwrap().insert((loc.clone(), edge)) {
            return;
        }
        self.shared.queue.lock().unwrap().push((loc.clone(), edge));
        self.shared.cv.notify_one();
    }

    pub fn forget(&self, loc: &Location) {
        self.seen.lock().unwrap().retain(|(l, _)| l != loc);
    }

    /// Drain finished thumbnails. In the browser this also does the decoding, a few per call.
    pub fn poll(&self) -> Vec<ThumbResult> {
        if !THREADED {
            for _ in 0..2 {
                let job = self.shared.queue.lock().unwrap().pop();
                let Some((loc, edge)) = job else { break };
                let r = decode(&self.registry, &loc, edge).unwrap_or(ThumbResult { loc, size: [0, 0], rgba: vec![], original: None, edge });
                self.shared.out.lock().unwrap().push(r);
            }
        }
        std::mem::take(&mut *self.shared.out.lock().unwrap())
    }

    pub fn pending(&self) -> bool {
        !self.shared.queue.lock().unwrap().is_empty()
    }
}
