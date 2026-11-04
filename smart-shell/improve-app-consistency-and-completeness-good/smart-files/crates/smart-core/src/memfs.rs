//! In-memory file system. Powers the web build (browsers have no real disk) and the test-suite.
use crate::model::{now_secs, Entry, Location};
use crate::vfs::*;
use std::collections::BTreeMap;
use std::io::{Cursor, Read, Write};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct Node {
    dir: bool,
    data: Arc<Vec<u8>>,
    /// Displayed size when it differs from the stored bytes (demo content uses small stand-ins).
    size_hint: Option<u64>,
    modified: i64,
}

pub struct MemVfs {
    id: String,
    nodes: Arc<Mutex<BTreeMap<String, Node>>>,
    capacity: Option<(u64, u64)>,
}

fn norm(p: &str) -> String {
    let mut s = p.trim_end_matches('/').to_string();
    if s.is_empty() {
        s = "/".into();
    }
    s
}
fn parent_of(p: &str) -> String {
    match p.rfind('/') {
        Some(0) | None => "/".into(),
        Some(i) => p[..i].to_string(),
    }
}

impl MemVfs {
    pub fn new(id: &str) -> Self {
        let mut m = BTreeMap::new();
        m.insert("/".to_string(), Node { dir: true, data: Arc::new(vec![]), size_hint: None, modified: now_secs() });
        Self { id: id.into(), nodes: Arc::new(Mutex::new(m)), capacity: None }
    }
    pub fn with_capacity(mut self, total: u64, free: u64) -> Self {
        self.capacity = Some((total, free));
        self
    }
    pub fn mkdir_p(&self, path: &str, modified: i64) {
        let path = norm(path);
        let mut cur = String::new();
        let mut n = self.nodes.lock().unwrap();
        for seg in path.split('/').filter(|s| !s.is_empty()) {
            cur.push('/');
            cur.push_str(seg);
            n.entry(cur.clone()).or_insert(Node { dir: true, data: Arc::new(vec![]), size_hint: None, modified });
        }
    }
    pub fn put(&self, path: &str, data: Vec<u8>, size_hint: Option<u64>, modified: i64) {
        let path = norm(path);
        self.mkdir_p(&parent_of(&path), modified);
        self.nodes.lock().unwrap().insert(path, Node { dir: false, data: Arc::new(data), size_hint, modified });
    }
    fn entry(&self, path: &str, n: &Node) -> Entry {
        let size = if n.dir { 0 } else { n.size_hint.unwrap_or(n.data.len() as u64) };
        let mut e = Entry::new(Location::new(self.id.clone(), path), n.dir, size, Some(n.modified));
        e.paced = n.size_hint.is_some_and(|h| h >= 1_000_000_000);
        e
    }
}

struct MemWriter {
    nodes: Arc<Mutex<BTreeMap<String, Node>>>,
    path: String,
    buf: Vec<u8>,
}
impl Write for MemWriter {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        self.buf.extend_from_slice(b);
        Ok(b.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.commit();
        Ok(())
    }
}
impl MemWriter {
    fn commit(&mut self) {
        let data = Arc::new(self.buf.clone());
        self.nodes.lock().unwrap().insert(self.path.clone(), Node { dir: false, data, size_hint: None, modified: now_secs() });
    }
}
impl Drop for MemWriter {
    fn drop(&mut self) {
        self.commit();
    }
}

impl Vfs for MemVfs {
    fn id(&self) -> &str {
        &self.id
    }
    fn list(&self, path: &str) -> FsResult<Vec<Entry>> {
        let path = norm(path);
        let n = self.nodes.lock().unwrap();
        match n.get(&path) {
            None => return Err(FsError::NotFound),
            Some(x) if !x.dir => return Err(FsError::Other("not a folder".into())),
            _ => {}
        }
        let prefix = if path == "/" { "/".to_string() } else { format!("{}/", path) };
        let mut out = vec![];
        for (k, v) in n.range(prefix.clone()..) {
            if !k.starts_with(&prefix) {
                break;
            }
            let rest = &k[prefix.len()..];
            if rest.is_empty() || rest.contains('/') {
                continue;
            }
            out.push(self.entry(k, v));
        }
        Ok(out)
    }
    fn stat(&self, path: &str) -> FsResult<Entry> {
        let path = norm(path);
        let n = self.nodes.lock().unwrap();
        n.get(&path).map(|v| self.entry(&path, v)).ok_or(FsError::NotFound)
    }
    fn open_read(&self, path: &str) -> FsResult<Box<dyn Read + Send>> {
        let n = self.nodes.lock().unwrap();
        match n.get(&norm(path)) {
            Some(x) if !x.dir => Ok(Box::new(Cursor::new(x.data.as_ref().clone()))),
            Some(_) => Err(FsError::Other("is a folder".into())),
            None => Err(FsError::NotFound),
        }
    }
    fn create_write(&self, path: &str) -> FsResult<Box<dyn Write + Send>> {
        let path = norm(path);
        if !self.nodes.lock().unwrap().contains_key(&parent_of(&path)) {
            return Err(FsError::NotFound);
        }
        Ok(Box::new(MemWriter { nodes: self.nodes.clone(), path, buf: vec![] }))
    }
    fn mkdir(&self, path: &str) -> FsResult<()> {
        let path = norm(path);
        let mut n = self.nodes.lock().unwrap();
        if n.contains_key(&path) {
            return Err(FsError::Exists);
        }
        if !n.contains_key(&parent_of(&path)) {
            return Err(FsError::NotFound);
        }
        n.insert(path, Node { dir: true, data: Arc::new(vec![]), size_hint: None, modified: now_secs() });
        Ok(())
    }
    fn rename(&self, from: &str, to: &str) -> FsResult<()> {
        let (from, to) = (norm(from), norm(to));
        let mut n = self.nodes.lock().unwrap();
        if !n.contains_key(&from) {
            return Err(FsError::NotFound);
        }
        if n.contains_key(&to) {
            return Err(FsError::Exists);
        }
        let prefix = format!("{}/", from);
        let moved: Vec<String> = n.keys().filter(|k| **k == from || k.starts_with(&prefix)).cloned().collect();
        for k in moved {
            let v = n.remove(&k).unwrap();
            n.insert(format!("{}{}", to, &k[from.len()..]), v);
        }
        Ok(())
    }
    fn remove(&self, path: &str, _to_trash: bool) -> FsResult<()> {
        let path = norm(path);
        let mut n = self.nodes.lock().unwrap();
        if !n.contains_key(&path) {
            return Err(FsError::NotFound);
        }
        let prefix = format!("{}/", path);
        n.retain(|k, _| *k != path && !k.starts_with(&prefix));
        Ok(())
    }
    fn capacity(&self, _path: &str) -> Option<Capacity> {
        self.capacity.map(|(total, free)| Capacity { total, free })
    }
    fn set_size_hint(&self, path: &str, hint: Option<u64>) -> FsResult<()> {
        let mut n = self.nodes.lock().unwrap();
        let node = n.get_mut(&norm(path)).ok_or(FsError::NotFound)?;
        node.size_hint = hint;
        Ok(())
    }
}
