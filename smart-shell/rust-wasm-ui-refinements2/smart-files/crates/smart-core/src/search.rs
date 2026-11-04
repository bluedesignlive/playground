//! Search: query language + cached name index + fuzzy ranking (nucleo) + optional content scan.
//!
//! * `report kind:pdf size:>1mb modified:week tag:design in:name`  — terms are fuzzy, filters are exact.
//! * First search under a root walks the tree while streaming matches, then caches the index;
//!   later searches rank the cached index in parallel (rayon) and answer in milliseconds.
use crate::exec::{now_millis, spawn};
use crate::library::SharedLibrary;
use crate::model::*;
use crate::ops::Registry;
use crate::vfs::Vfs;
use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, RwLock};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Filters {
    pub kinds: Vec<Kind>,
    pub exts: Vec<String>,
    pub min_size: Option<u64>,
    pub max_size: Option<u64>,
    pub modified_after: Option<i64>,
    pub modified_before: Option<i64>,
    pub tags: Vec<String>,
    pub dirs: Option<bool>,
    pub content: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Query {
    pub raw: String,
    pub terms: Vec<String>,
    pub filters: Filters,
}

fn parse_size(s: &str) -> Option<u64> {
    let s = s.to_lowercase();
    let idx = s.find(|c: char| !(c.is_ascii_digit() || c == '.')).unwrap_or(s.len());
    let n: f64 = s[..idx].parse().ok()?;
    let mul = match s[idx..].trim() {
        "" | "b" => 1.0,
        "k" | "kb" => 1e3,
        "m" | "mb" => 1e6,
        "g" | "gb" => 1e9,
        "t" | "tb" => 1e12,
        _ => return None,
    };
    Some((n * mul) as u64)
}

fn midnight(days_ago: i64) -> i64 {
    use chrono::{Duration, Local, TimeZone};
    let d = Local::now().date_naive() - Duration::days(days_ago);
    Local.from_local_datetime(&d.and_hms_opt(0, 0, 0).unwrap()).single().map(|t| t.timestamp()).unwrap_or(0)
}

impl Query {
    pub fn parse(raw: &str) -> Query {
        let mut q = Query { raw: raw.to_string(), ..Default::default() };
        let mut chars = raw.trim().chars().peekable();
        let mut toks: Vec<String> = vec![];
        let mut cur = String::new();
        let mut quoted = false;
        while let Some(c) = chars.next() {
            match c {
                '"' => quoted = !quoted,
                c if c.is_whitespace() && !quoted => {
                    if !cur.is_empty() {
                        toks.push(std::mem::take(&mut cur));
                    }
                }
                c => cur.push(c),
            }
        }
        if !cur.is_empty() {
            toks.push(cur);
        }
        for t in toks {
            let Some((k, v)) = t.split_once(':').filter(|(k, v)| !k.is_empty() && !v.is_empty() && k.chars().all(|c| c.is_ascii_alphabetic())) else {
                q.terms.push(t);
                continue;
            };
            let f = &mut q.filters;
            match k.to_lowercase().as_str() {
                "kind" | "type" | "is" => match v.to_lowercase().as_str() {
                    "image" | "images" | "photo" | "picture" => f.kinds.extend([Kind::Image, Kind::Vector]),
                    "audio" | "music" | "sound" => f.kinds.push(Kind::Audio),
                    "video" | "movie" | "movies" => f.kinds.push(Kind::Video),
                    "pdf" => f.kinds.push(Kind::Pdf),
                    "doc" | "docs" | "document" => f.kinds.extend([Kind::Document, Kind::Pdf, Kind::Text]),
                    "text" => f.kinds.push(Kind::Text),
                    "code" | "source" => f.kinds.push(Kind::Code),
                    "archive" | "zip" => f.kinds.push(Kind::Archive),
                    "app" => f.kinds.push(Kind::App),
                    "folder" | "dir" | "directory" => f.dirs = Some(true),
                    "file" => f.dirs = Some(false),
                    _ => q.terms.push(t.clone()),
                },
                "ext" => f.exts.push(v.trim_start_matches('.').to_lowercase()),
                "tag" => f.tags.push(v.to_lowercase()),
                "content" | "text-in" | "contains" => f.content = Some(v.to_string()),
                "size" => {
                    if let Some(r) = v.strip_prefix('>') {
                        f.min_size = parse_size(r);
                    } else if let Some(r) = v.strip_prefix('<') {
                        f.max_size = parse_size(r);
                    } else {
                        f.min_size = parse_size(v);
                    }
                }
                "modified" | "date" | "changed" => match v.to_lowercase().as_str() {
                    "today" => f.modified_after = Some(midnight(0)),
                    "yesterday" => {
                        f.modified_after = Some(midnight(1));
                        f.modified_before = Some(midnight(0));
                    }
                    "week" => f.modified_after = Some(midnight(7)),
                    "month" => f.modified_after = Some(midnight(30)),
                    "year" => f.modified_after = Some(midnight(365)),
                    _ => q.terms.push(t.clone()),
                },
                _ => q.terms.push(t.clone()),
            }
        }
        q
    }
    /// “big videos from last week” → `kind:video size:>100mb modified:week`. Everything stays on-device.
    pub fn from_natural(text: &str) -> Option<(String, String)> {
        let lower = text.to_lowercase();
        let words: Vec<&str> = lower.split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()).collect();
        let has = |ws: &[&str]| words.iter().any(|w| ws.contains(w));
        let mut parts: Vec<String> = vec![];
        let mut why: Vec<String> = vec![];
        for (ws, f, label) in [
            (&["image", "images", "photo", "photos", "picture", "pictures", "screenshots"][..], "kind:image", "images"),
            (&["music", "audio", "song", "songs", "sounds", "samples"][..], "kind:audio", "audio"),
            (&["video", "videos", "movie", "movies", "clips"][..], "kind:video", "videos"),
            (&["pdf", "pdfs"][..], "kind:pdf", "PDFs"),
            (&["document", "documents", "docs", "notes"][..], "kind:doc", "documents"),
            (&["archive", "archives", "zip", "zips"][..], "kind:archive", "archives"),
            (&["code", "source", "scripts"][..], "kind:code", "code"),
            (&["folder", "folders", "directories"][..], "kind:folder", "folders"),
        ] {
            if has(ws) {
                parts.push(f.into());
                why.push(label.into());
                break;
            }
        }
        let joined = lower.as_str();
        for (needle, f, label) in [
            ("today", "modified:today", "today"),
            ("yesterday", "modified:yesterday", "yesterday"),
            ("this week", "modified:week", "this week"),
            ("last week", "modified:week", "last 7 days"),
            ("past week", "modified:week", "last 7 days"),
            ("this month", "modified:month", "this month"),
            ("last month", "modified:month", "last 30 days"),
            ("this year", "modified:year", "this year"),
            ("recent", "modified:week", "recently"),
        ] {
            if joined.contains(needle) {
                parts.push(f.into());
                why.push(label.into());
                break;
            }
        }
        if has(&["large", "big", "huge", "heavy", "biggest", "largest"]) {
            parts.push("size:>100mb".into());
            why.push("over 100 MB".into());
        } else if has(&["small", "tiny"]) {
            parts.push("size:<100kb".into());
            why.push("under 100 KB".into());
        }
        if let Some(i) = words.iter().position(|w| *w == "tagged" || *w == "tag") {
            if let Some(t) = words.get(i + 1) {
                parts.push(format!("tag:{}", t));
                why.push(format!("tagged {}", t));
            }
        }
        for key in ["named", "called", "containing", "with"] {
            if let Some(i) = words.iter().position(|w| *w == key) {
                if let Some(t) = words.get(i + 1) {
                    if key == "containing" || key == "with" {
                        parts.push(format!("content:{}", t));
                    } else {
                        parts.push((*t).to_string());
                    }
                    why.push(format!("“{}”", t));
                }
            }
        }
        if parts.is_empty() {
            return None;
        }
        Some((parts.join(" "), why.join(" · ")))
    }
    pub fn is_empty(&self) -> bool {
        self.terms.is_empty() && self.filters == Filters::default()
    }
    /// Turn the free-text terms into a content search (used by the “Search inside files” toggle).
    pub fn into_content_search(mut self) -> Query {
        if self.filters.content.is_none() && !self.terms.is_empty() {
            self.filters.content = Some(self.terms.join(" "));
            self.terms.clear();
        }
        self
    }
}

#[derive(Clone, Debug)]
pub struct IndexEntry {
    pub name: String,
    pub path: String,
    pub dir: bool,
    pub size: u64,
    pub modified: i64,
    pub kind: Kind,
}

impl IndexEntry {
    fn from_entry(e: &Entry) -> Self {
        Self { name: e.name.clone(), path: e.loc.path.clone(), dir: e.is_dir, size: e.size, modified: e.modified.unwrap_or(0), kind: e.kind }
    }
    fn to_entry(&self, fs: &str) -> Entry {
        Entry::new(Location::new(fs, self.path.clone()), self.dir, self.size, Some(self.modified))
    }
}

pub struct IndexData {
    fs: String,
    root: String,
    entries: Vec<IndexEntry>,
    built_at: i64,
}

#[derive(Clone, Debug)]
pub struct Hit {
    pub entry: Entry,
    pub score: u32,
    /// Char indices in `entry.name` that matched (for highlighting).
    pub matched: Vec<u32>,
    pub snippet: Option<String>,
}

pub enum SearchMsg {
    Hits { id: u64, hits: Vec<Hit> },
    Done { id: u64, scanned: usize, elapsed_ms: i64, cached: bool },
}

struct Scorer {
    matcher: Matcher,
    pats: Vec<Pattern>,
    terms: Vec<String>,
    buf: Vec<char>,
}

impl Scorer {
    fn new(terms: &[String]) -> Self {
        Self {
            matcher: Matcher::new(Config::DEFAULT.match_paths()),
            pats: terms.iter().map(|t| Pattern::parse(t, CaseMatching::Smart, Normalization::Smart)).collect(),
            terms: terms.iter().map(|t| t.to_lowercase()).collect(),
            buf: Vec::new(),
        }
    }
    fn score(&mut self, e: &IndexEntry) -> Option<u32> {
        if self.pats.is_empty() {
            return Some(1);
        }
        let mut total = 0u32;
        for p in &self.pats {
            match p.score(Utf32Str::new(&e.name, &mut self.buf), &mut self.matcher) {
                Some(s) => total += s * 4,
                None => total += p.score(Utf32Str::new(&e.path, &mut self.buf), &mut self.matcher)?,
            }
        }
        // cheap tie-breakers (only on matches): prefix / exact names, short names, fresh files, folders
        let n = e.name.to_lowercase();
        for t in &self.terms {
            if n == *t || n.rsplit_once('.').is_some_and(|(s, _)| s == t) {
                total += 600;
            } else if n.starts_with(t.as_str()) {
                total += 240;
            }
        }
        total = total.saturating_sub(e.name.len() as u32);
        if e.modified > now_secs() - 7 * 86400 {
            total += 24;
        }
        if e.dir {
            total += 12;
        }
        Some(total)
    }
    fn indices(&mut self, name: &str) -> Vec<u32> {
        let mut out = vec![];
        for p in &self.pats {
            let mut idx = vec![];
            if p.indices(Utf32Str::new(name, &mut self.buf), &mut self.matcher, &mut idx).is_some() {
                out.extend(idx);
            }
        }
        out.sort_unstable();
        out.dedup();
        out
    }
}

fn passes(f: &Filters, e: &IndexEntry, fs: &str, lib: &SharedLibrary) -> bool {
    if let Some(d) = f.dirs {
        if e.dir != d {
            return false;
        }
    }
    if !f.kinds.is_empty() && !f.kinds.contains(&e.kind) {
        return false;
    }
    if !f.exts.is_empty() && !f.exts.iter().any(|x| e.name.to_lowercase().ends_with(&format!(".{}", x))) {
        return false;
    }
    if f.min_size.is_some_and(|m| e.size < m) || f.max_size.is_some_and(|m| e.size > m) {
        return false;
    }
    if f.modified_after.is_some_and(|m| e.modified < m) || f.modified_before.is_some_and(|m| e.modified >= m) {
        return false;
    }
    if !f.tags.is_empty() {
        let loc = Location::new(fs, e.path.clone());
        let l = lib.read().unwrap();
        let have = l.tag_ids_of(&loc);
        if !f.tags.iter().all(|t| have.iter().any(|h| h == t)) {
            return false;
        }
    }
    true
}

const JUNK: &[&str] = &["node_modules", "target", "__pycache__", "venv", ".git", ".cache", "site-packages", "proc", "sys", "dev", "snap", "lost+found"];

fn walk(v: &dyn Vfs, root: &str, cancel: &AtomicBool, mut visit: impl FnMut(IndexEntry) -> bool) -> usize {
    let mut stack = vec![root.to_string()];
    let mut n = 0;
    while let Some(dir) = stack.pop() {
        if cancel.load(Ordering::Relaxed) {
            break;
        }
        let Ok(list) = v.list(&dir) else { continue };
        for e in list {
            if e.hidden || (e.is_dir && JUNK.contains(&e.name.as_str())) {
                continue;
            }
            n += 1;
            if e.is_dir && !e.symlink {
                stack.push(e.loc.path.clone());
            }
            if !visit(IndexEntry::from_entry(&e)) || n > 600_000 {
                return n;
            }
        }
    }
    n
}

fn text_like(e: &IndexEntry) -> bool {
    !e.dir && e.size < 2_000_000 && matches!(e.kind, Kind::Text | Kind::Code | Kind::Other | Kind::Document)
}

fn content_hit(v: &dyn Vfs, e: &IndexEntry, needle: &str) -> Option<String> {
    let data = v.read(&e.path, 2_000_000).ok()?;
    if data.iter().take(4096).any(|b| *b == 0) {
        return None;
    }
    let lower: Vec<u8> = data.iter().map(|b| b.to_ascii_lowercase()).collect();
    let at = memchr::memmem::find(&lower, needle.to_lowercase().as_bytes())?;
    let s = data[..at].iter().rposition(|b| *b == b'\n').map(|i| i + 1).unwrap_or(0);
    let e2 = data[at..].iter().position(|b| *b == b'\n').map(|i| at + i).unwrap_or(data.len());
    let line = String::from_utf8_lossy(&data[s..e2.min(s + 400)]).trim().to_string();
    Some(line.chars().take(140).collect())
}

pub struct SearchEngine {
    reg: Arc<Registry>,
    lib: SharedLibrary,
    indexes: Arc<RwLock<Vec<Arc<IndexData>>>>,
}

impl SearchEngine {
    pub fn new(reg: Arc<Registry>, lib: SharedLibrary) -> Self {
        Self { reg, lib, indexes: Arc::new(RwLock::new(vec![])) }
    }

    /// Forget cached indexes that cover `loc` (call after files change).
    pub fn invalidate(&self, loc: &Location) {
        self.indexes.write().unwrap().retain(|i| !(i.fs == loc.fs && (loc.path.starts_with(&i.root) || i.root.starts_with(&loc.path))));
    }

    pub fn indexed_count(&self) -> usize {
        self.indexes.read().unwrap().iter().map(|i| i.entries.len()).sum()
    }

    pub fn start(&self, id: u64, query: Query, scope: Location, tx: Sender<SearchMsg>) -> Arc<AtomicBool> {
        let cancel = Arc::new(AtomicBool::new(false));
        let (reg, lib, indexes, c2) = (self.reg.clone(), self.lib.clone(), self.indexes.clone(), cancel.clone());
        spawn(move || run(id, query, scope, reg, lib, indexes, c2, tx));
        cancel
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn score_all(entries: &[IndexEntry], terms: &[String], f: &Filters, fs: &str, lib: &SharedLibrary, root: &str) -> Vec<(u32, usize)> {
    use rayon::prelude::*;
    entries
        .par_chunks(16_384)
        .enumerate()
        .flat_map_iter(|(ci, chunk)| {
            let mut s = Scorer::new(terms);
            chunk
                .iter()
                .enumerate()
                .filter_map(|(i, e)| {
                    if !under(&e.path, root) || !passes(f, e, fs, lib) {
                        return None;
                    }
                    s.score(e).map(|sc| (sc, ci * 16_384 + i))
                })
                .collect::<Vec<_>>()
        })
        .collect()
}
#[cfg(target_arch = "wasm32")]
fn score_all(entries: &[IndexEntry], terms: &[String], f: &Filters, fs: &str, lib: &SharedLibrary, root: &str) -> Vec<(u32, usize)> {
    let mut s = Scorer::new(terms);
    entries
        .iter()
        .enumerate()
        .filter_map(|(i, e)| if under(&e.path, root) && passes(f, e, fs, lib) { s.score(e).map(|sc| (sc, i)) } else { None })
        .collect()
}

fn under(path: &str, root: &str) -> bool {
    root == "/" || (path.starts_with(root) && (path.len() == root.len() || path.as_bytes()[root.len()] == b'/')) && path != root
}

fn content_filter(v: &dyn Vfs, cands: Vec<(u32, IndexEntry)>, needle: &str) -> Vec<(u32, IndexEntry, Option<String>)> {
    let mut cands: Vec<_> = cands.into_iter().filter(|(_, e)| text_like(e)).collect();
    cands.sort_by(|a, b| b.1.modified.cmp(&a.1.modified));
    cands.truncate(6000);
    #[cfg(not(target_arch = "wasm32"))]
    {
        use rayon::prelude::*;
        cands.into_par_iter().filter_map(|(s, e)| content_hit(v, &e, needle).map(|sn| (s + 50, e, Some(sn)))).collect()
    }
    #[cfg(target_arch = "wasm32")]
    {
        cands.into_iter().filter_map(|(s, e)| content_hit(v, &e, needle).map(|sn| (s + 50, e, Some(sn)))).collect()
    }
}

#[allow(clippy::too_many_arguments)]
fn run(id: u64, q: Query, scope: Location, reg: Arc<Registry>, lib: SharedLibrary, indexes: Arc<RwLock<Vec<Arc<IndexData>>>>, cancel: Arc<AtomicBool>, tx: Sender<SearchMsg>) {
    let t0 = now_millis();
    let Some(v) = reg.get(&scope.fs) else {
        let _ = tx.send(SearchMsg::Done { id, scanned: 0, elapsed_ms: 0, cached: false });
        return;
    };
    let cached = indexes.read().unwrap().iter().find(|i| i.fs == scope.fs && under_or_eq(&scope.path, &i.root) && now_secs() - i.built_at < 600).cloned();
    let mk_hits = |ranked: Vec<(u32, IndexEntry, Option<String>)>, scorer: &mut Scorer, fs: &str| -> Vec<Hit> {
        ranked.into_iter().map(|(score, e, snippet)| Hit { matched: scorer.indices(&e.name), entry: e.to_entry(fs), score, snippet }).collect()
    };

    if let Some(ix) = cached {
        let ranked = score_all(&ix.entries, &q.terms, &q.filters, &ix.fs, &lib, &scope.path);
        let mut ranked = ranked;
        ranked.sort_unstable_by(|a, b| b.0.cmp(&a.0));
        ranked.truncate(if q.filters.content.is_some() { 20_000 } else { 400 });
        let mut list: Vec<(u32, IndexEntry, Option<String>)> = ranked.into_iter().map(|(s, i)| (s, ix.entries[i].clone(), None)).collect();
        if let Some(needle) = &q.filters.content {
            let c = content_filter(v.as_ref(), list.into_iter().map(|(s, e, _)| (s, e)).collect(), needle);
            list = c;
            list.sort_by(|a, b| b.0.cmp(&a.0));
            list.truncate(400);
        }
        let mut sc = Scorer::new(&q.terms);
        let hits = mk_hits(list, &mut sc, &ix.fs);
        let _ = tx.send(SearchMsg::Hits { id, hits });
        let _ = tx.send(SearchMsg::Done { id, scanned: ix.entries.len(), elapsed_ms: now_millis() - t0, cached: true });
        return;
    }

    // Cold path: walk, match as we go, stream batches, then keep the index.
    let mut scorer = Scorer::new(&q.terms);
    let mut all: Vec<IndexEntry> = Vec::new();
    let mut batch: Vec<(u32, IndexEntry, Option<String>)> = vec![];
    let mut last_flush = now_millis();
    let mut matches = 0usize;
    let content = q.filters.content.is_some();
    let scanned = walk(v.as_ref(), &scope.path, &cancel, |e| {
        if !content && matches < 3000 && passes(&q.filters, &e, &scope.fs, &lib) {
            if let Some(s) = scorer.score(&e) {
                matches += 1;
                batch.push((s, e.clone(), None));
            }
        }
        all.push(e);
        if batch.len() >= 64 || (now_millis() - last_flush > 80 && !batch.is_empty()) {
            let b = std::mem::take(&mut batch);
            let mut sc2 = Scorer::new(&q.terms);
            let hits = mk_hits(b, &mut sc2, &scope.fs);
            last_flush = now_millis();
            return tx.send(SearchMsg::Hits { id, hits }).is_ok();
        }
        true
    });
    if cancel.load(Ordering::Relaxed) {
        return;
    }
    if content {
        let cands: Vec<(u32, IndexEntry)> = all.iter().filter(|e| passes(&q.filters, e, &scope.fs, &lib)).filter_map(|e| scorer.score(e).map(|s| (s, e.clone()))).collect();
        let mut list = content_filter(v.as_ref(), cands, q.filters.content.as_deref().unwrap_or(""));
        list.sort_by(|a, b| b.0.cmp(&a.0));
        list.truncate(400);
        let hits = mk_hits(list, &mut scorer, &scope.fs);
        let _ = tx.send(SearchMsg::Hits { id, hits });
    } else if !batch.is_empty() {
        let hits = mk_hits(batch, &mut scorer, &scope.fs);
        let _ = tx.send(SearchMsg::Hits { id, hits });
    }
    indexes.write().unwrap().retain(|i| !(i.fs == scope.fs && i.root.starts_with(&scope.path)));
    indexes.write().unwrap().push(Arc::new(IndexData { fs: scope.fs.clone(), root: scope.path.clone(), entries: all, built_at: now_secs() }));
    let _ = tx.send(SearchMsg::Done { id, scanned, elapsed_ms: now_millis() - t0, cached: false });
}

fn under_or_eq(path: &str, root: &str) -> bool {
    path == root || under(path, root)
}
