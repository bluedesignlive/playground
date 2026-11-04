//! Plain data types shared by every frontend. No UI, no I/O.
use chrono::{DateTime, Datelike, Local, TimeZone, Timelike};
use serde::{Deserialize, Serialize};

/// A place in the virtual namespace: which file system, and a `/`-separated absolute path in it.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Location {
    pub fs: String,
    pub path: String,
}

impl Location {
    pub fn new(fs: impl Into<String>, path: impl Into<String>) -> Self {
        let mut path: String = path.into();
        if path.is_empty() {
            path = "/".into();
        }
        while path.len() > 1 && path.ends_with('/') {
            path.pop();
        }
        Self { fs: fs.into(), path }
    }
    pub fn file(path: impl Into<String>) -> Self {
        Self::new("file", path)
    }
    pub fn mem(path: impl Into<String>) -> Self {
        Self::new("mem", path)
    }
    pub fn is_root(&self) -> bool {
        self.path == "/"
    }
    pub fn parent(&self) -> Option<Location> {
        if self.is_root() {
            return None;
        }
        let idx = self.path.rfind('/')?;
        let p = if idx == 0 { "/" } else { &self.path[..idx] };
        Some(Location::new(self.fs.clone(), p))
    }
    pub fn name(&self) -> String {
        if self.is_root() {
            return self.fs.clone();
        }
        self.path.rsplit('/').next().unwrap_or("").to_string()
    }
    pub fn join(&self, name: &str) -> Location {
        let base = if self.is_root() { String::new() } else { self.path.clone() };
        Location::new(self.fs.clone(), format!("{}/{}", base, name))
    }
    pub fn starts_with(&self, other: &Location) -> bool {
        self.fs == other.fs
            && (self.path == other.path
                || other.is_root()
                || self.path.starts_with(&format!("{}/", other.path)))
    }
    /// Path segments from the root, for breadcrumbs.
    pub fn ancestors(&self) -> Vec<Location> {
        let mut v = vec![];
        let mut cur = Some(self.clone());
        while let Some(c) = cur {
            cur = c.parent();
            v.push(c);
        }
        v.reverse();
        v
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Kind {
    Folder,
    Image,
    Audio,
    Video,
    Pdf,
    Document,
    Text,
    Code,
    Archive,
    App,
    Vector,
    Project,
    Other,
}

impl Kind {
    pub fn from_ext(ext: &str) -> Kind {
        match ext.to_ascii_lowercase().as_str() {
            "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp" | "tif" | "tiff" | "heic" | "avif" | "ico" => Kind::Image,
            "svg" => Kind::Vector,
            "wav" | "mp3" | "flac" | "ogg" | "oga" | "m4a" | "aac" | "opus" | "aif" | "aiff" => Kind::Audio,
            "mp4" | "mkv" | "mov" | "webm" | "avi" | "m4v" => Kind::Video,
            "pdf" => Kind::Pdf,
            "doc" | "docx" | "odt" | "rtf" | "pages" | "xls" | "xlsx" | "ods" | "ppt" | "pptx" | "odp" | "key" => Kind::Document,
            "txt" | "md" | "markdown" | "log" | "csv" | "json" | "toml" | "yaml" | "yml" | "ini" | "conf" | "cfg" => Kind::Text,
            "rs" | "js" | "ts" | "tsx" | "jsx" | "py" | "c" | "h" | "cpp" | "hpp" | "go" | "java" | "kt" | "swift" | "sh" | "html" | "css" | "lua" | "rb" | "php" => Kind::Code,
            "zip" | "tar" | "gz" | "xz" | "bz2" | "zst" | "7z" | "rar" | "tgz" => Kind::Archive,
            "appimage" | "exe" | "msi" | "app" | "deb" | "rpm" | "apk" | "dmg" | "flatpak" => Kind::App,
            "flp" | "als" | "logicx" | "ptx" | "psd" | "fig" | "sketch" | "blend" | "xd" | "ai" => Kind::Project,
            _ => Kind::Other,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Kind::Folder => "Folder",
            Kind::Image => "Image",
            Kind::Audio => "Audio",
            Kind::Video => "Video",
            Kind::Pdf => "PDF document",
            Kind::Document => "Document",
            Kind::Text => "Text",
            Kind::Code => "Source code",
            Kind::Archive => "Archive",
            Kind::App => "Application",
            Kind::Vector => "Vector image",
            Kind::Project => "Project",
            Kind::Other => "Document",
        }
    }
    /// Name of the icon in the frontend's Solar icon set.
    pub fn icon(self) -> &'static str {
        match self {
            Kind::Folder => "folder",
            Kind::Image => "gallery",
            Kind::Audio => "music-note",
            Kind::Video => "clapperboard",
            Kind::Pdf | Kind::Document => "document-text",
            Kind::Text => "text",
            Kind::Code => "terminal",
            Kind::Archive => "archive",
            Kind::App => "widget",
            Kind::Vector => "stars-minimalistic",
            Kind::Project | Kind::Other => "file",
        }
    }
    pub fn previewable(self) -> bool {
        matches!(self, Kind::Image | Kind::Audio | Kind::Text | Kind::Code | Kind::Video | Kind::Pdf | Kind::Vector)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub name: String,
    pub loc: Location,
    pub is_dir: bool,
    pub size: u64,
    /// Unix seconds.
    pub modified: Option<i64>,
    pub kind: Kind,
    pub ext: String,
    pub hidden: bool,
    pub symlink: bool,
    pub readonly: bool,
    /// Size is a demo promise, not the bytes on disk (the 5 GB showcase file). Transfers of it are paced.
    pub paced: bool,
}

impl Entry {
    pub fn new(loc: Location, is_dir: bool, size: u64, modified: Option<i64>) -> Entry {
        let name = loc.name();
        let ext = if is_dir { String::new() } else { ext_of(&name) };
        let kind = if is_dir { Kind::Folder } else { Kind::from_ext(&ext) };
        Entry { hidden: name.starts_with('.'), name, loc, is_dir, size, modified, kind, ext, symlink: false, readonly: false, paced: false }
    }
    pub fn kind_label(&self) -> String {
        if self.is_dir {
            return "Folder".into();
        }
        let e = self.ext.to_ascii_uppercase();
        match self.kind {
            Kind::Image if !e.is_empty() => format!("{} image", if e == "JPG" { "JPEG".into() } else { e }),
            Kind::Audio if !e.is_empty() => format!("{} audio", e),
            Kind::Video if !e.is_empty() => format!("{} video", e),
            Kind::Archive if !e.is_empty() => format!("{} archive", e),
            Kind::Text if e == "MD" => "Markdown".into(),
            Kind::Text if e == "TXT" => "Plain text".into(),
            Kind::Project if e == "FLP" => "FL Studio project".into(),
            Kind::App if e == "APPIMAGE" => "Linux app".into(),
            k if e.is_empty() => k.label().into(),
            k => format!("{} {}", e, k.label().to_lowercase()),
        }
    }
}

pub fn ext_of(name: &str) -> String {
    match name.rfind('.') {
        Some(i) if i > 0 && i + 1 < name.len() => name[i + 1..].to_string(),
        _ => String::new(),
    }
}

// ───────────────────────────── formatting ─────────────────────────────

pub fn format_size(bytes: u64) -> String {
    const U: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    if bytes < 1000 {
        return format!("{} B", bytes);
    }
    let mut v = bytes as f64;
    let mut i = 0;
    while v >= 1000.0 && i < 4 {
        v /= 1000.0;
        i += 1;
    }
    if v >= 100.0 || i == 1 {
        format!("{:.0} {}", v, U[i])
    } else {
        format!("{:.1} {}", v, U[i])
    }
}

pub fn now_secs() -> i64 {
    chrono::Utc::now().timestamp()
}

fn local(ts: i64) -> DateTime<Local> {
    Local.timestamp_opt(ts, 0).single().unwrap_or_else(|| Local.timestamp_opt(0, 0).unwrap())
}

/// "Today, 09:41" · "Yesterday, 18:02" · "Oct 28, 09:15" · "Oct 28, 2023"
pub fn format_date(ts: i64) -> String {
    let d = local(ts);
    let now = Local::now();
    let days = (now.date_naive() - d.date_naive()).num_days();
    let hm = format!("{:02}:{:02}", d.hour(), d.minute());
    let mon = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"][d.month0() as usize];
    match days {
        0 => format!("Today, {}", hm),
        1 => format!("Yesterday, {}", hm),
        _ if d.year() == now.year() => format!("{} {}, {}", mon, d.day(), hm),
        _ => format!("{} {}, {}", mon, d.day(), d.year()),
    }
}

pub fn format_full_date(ts: i64) -> String {
    local(ts).format("%A %-d %B %Y, %H:%M").to_string()
}

pub fn clock_hm() -> String {
    Local::now().format("%H:%M").to_string()
}
pub fn clock_date() -> String {
    Local::now().format("%a %-d %B").to_string()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortKey {
    Name,
    Modified,
    Size,
    Kind,
}

impl SortKey {
    pub const ALL: [SortKey; 4] = [SortKey::Name, SortKey::Modified, SortKey::Size, SortKey::Kind];
    pub fn label(self) -> &'static str {
        match self {
            SortKey::Name => "Name",
            SortKey::Modified => "Date modified",
            SortKey::Size => "Size",
            SortKey::Kind => "Kind",
        }
    }
    pub fn short(self) -> &'static str {
        match self {
            SortKey::Modified => "Date",
            o => o.label(),
        }
    }
}

/// Natural, case-insensitive comparison ("img2" < "img10").
pub fn natural_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    let (mut ai, mut bi) = (a.chars().peekable(), b.chars().peekable());
    loop {
        match (ai.peek().copied(), bi.peek().copied()) {
            (None, None) => return std::cmp::Ordering::Equal,
            (None, _) => return std::cmp::Ordering::Less,
            (_, None) => return std::cmp::Ordering::Greater,
            (Some(x), Some(y)) if x.is_ascii_digit() && y.is_ascii_digit() => {
                let mut na = String::new();
                let mut nb = String::new();
                while let Some(c) = ai.peek().copied().filter(|c| c.is_ascii_digit()) {
                    na.push(c);
                    ai.next();
                }
                while let Some(c) = bi.peek().copied().filter(|c| c.is_ascii_digit()) {
                    nb.push(c);
                    bi.next();
                }
                let (na, nb) = (na.trim_start_matches('0'), nb.trim_start_matches('0'));
                let o = na.len().cmp(&nb.len()).then_with(|| na.cmp(nb));
                if o != std::cmp::Ordering::Equal {
                    return o;
                }
            }
            (Some(x), Some(y)) => {
                let o = x.to_lowercase().cmp(y.to_lowercase());
                if o != std::cmp::Ordering::Equal {
                    return o;
                }
                ai.next();
                bi.next();
            }
        }
    }
}

pub fn sort_entries(v: &mut [Entry], key: SortKey, ascending: bool, folders_first: bool) {
    v.sort_by(|a, b| {
        if folders_first && a.is_dir != b.is_dir {
            return b.is_dir.cmp(&a.is_dir);
        }
        let o = match key {
            SortKey::Name => natural_cmp(&a.name, &b.name),
            SortKey::Modified => b.modified.cmp(&a.modified).then_with(|| natural_cmp(&a.name, &b.name)),
            SortKey::Size => b.size.cmp(&a.size).then_with(|| natural_cmp(&a.name, &b.name)),
            SortKey::Kind => a.kind_label().cmp(&b.kind_label()).then_with(|| natural_cmp(&a.name, &b.name)),
        };
        if ascending { o } else { o.reverse() }
    });
}
