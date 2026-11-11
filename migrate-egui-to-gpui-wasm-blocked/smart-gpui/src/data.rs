//! Seeded in-memory world (same content as the egui build's demo mode) + a tiny VFS.
use gpui_kit::*;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Folder,
    Image,
    Audio,
    Video,
    Doc,
    Code,
    Archive,
    Text,
    App,
    Other,
}

impl Kind {
    pub fn label(self) -> &'static str {
        match self {
            Kind::Folder => "Folder",
            Kind::Image => "Image",
            Kind::Audio => "Audio",
            Kind::Video => "Video",
            Kind::Doc => "Document",
            Kind::Code => "Source code",
            Kind::Archive => "Archive",
            Kind::Text => "Text",
            Kind::App => "Application",
            Kind::Other => "File",
        }
    }
    pub fn icon(self) -> &'static str {
        match self {
            Kind::Folder => "folder",
            Kind::Image => "gallery",
            Kind::Audio => "music-notes",
            Kind::Video => "clapperboard",
            Kind::Doc => "document-text",
            Kind::Code => "terminal",
            Kind::Archive => "archive",
            Kind::Text => "text",
            Kind::App => "widget",
            Kind::Other => "file",
        }
    }
}

pub fn kind_of(name: &str) -> Kind {
    let ext = name.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "jpg" | "jpeg" | "png" | "svg" | "gif" | "webp" => Kind::Image,
        "wav" | "mp3" | "flac" | "flp" => Kind::Audio,
        "mp4" | "mov" | "mkv" => Kind::Video,
        "pdf" | "xlsx" | "docx" => Kind::Doc,
        "rs" | "py" | "js" | "toml" => Kind::Code,
        "zip" | "sparse" | "tar" => Kind::Archive,
        "txt" | "md" => Kind::Text,
        "appimage" => Kind::App,
        _ => Kind::Other,
    }
}

#[derive(Clone)]
pub struct Node {
    pub name: String,
    pub kind: Kind,
    pub size: u64,
    pub hours: f32,
    pub hue: f32,
    pub children: Vec<Node>,
    pub text: Option<String>,
    pub tag: u8,
}

impl Node {
    pub fn is_dir(&self) -> bool {
        self.kind == Kind::Folder
    }
    pub fn total(&self) -> u64 {
        if self.is_dir() { self.children.iter().map(|c| c.total()).sum() } else { self.size }
    }
}

const KB: u64 = 1_000;
const MB: u64 = 1_000_000;
const GB: u64 = 1_000_000_000;

fn hue_of(name: &str) -> f32 {
    let mut h: u32 = 2166136261;
    for b in name.bytes() {
        h = (h ^ b as u32).wrapping_mul(16777619);
    }
    (h % 360) as f32 / 360.0
}

fn f(name: &str, size: u64, hours: f32) -> Node {
    Node { name: name.into(), kind: kind_of(name), size, hue: hue_of(name), hours, children: vec![], text: None, tag: 0 }
}
fn ft(name: &str, size: u64, hours: f32, text: &str) -> Node {
    let mut n = f(name, size, hours);
    n.text = Some(text.into());
    n
}
fn d(name: &str, hours: f32, children: Vec<Node>) -> Node {
    Node { name: name.into(), kind: Kind::Folder, size: 0, hue: 0.6, hours, children, text: None, tag: 0 }
}

pub struct Fs {
    pub root: Node,
    pub trash: Vec<(Vec<String>, Node)>,
}

impl Fs {
    pub fn dir(&self, path: &[String]) -> Option<&Node> {
        let mut n = &self.root;
        for p in path {
            n = n.children.iter().find(|c| &c.name == p && c.is_dir())?;
        }
        Some(n)
    }
    pub fn dir_mut(&mut self, path: &[String]) -> Option<&mut Node> {
        let mut n = &mut self.root;
        for p in path {
            n = n.children.iter_mut().find(|c| &c.name == p && c.is_dir())?;
        }
        Some(n)
    }
    pub fn recents(&self) -> Vec<(Vec<String>, Node)> {
        fn walk(n: &Node, path: &mut Vec<String>, out: &mut Vec<(Vec<String>, Node)>) {
            for c in &n.children {
                if c.is_dir() {
                    path.push(c.name.clone());
                    walk(c, path, out);
                    path.pop();
                } else {
                    out.push((path.clone(), c.clone()));
                }
            }
        }
        let mut out = vec![];
        walk(&self.root, &mut vec![], &mut out);
        out.sort_by(|a, b| a.1.hours.partial_cmp(&b.1.hours).unwrap());
        out.truncate(24);
        out
    }
    pub fn unique_name(&self, path: &[String], base: &str) -> String {
        let Some(dir) = self.dir(path) else { return base.into() };
        if !dir.children.iter().any(|c| c.name == base) {
            return base.into();
        }
        for i in 2.. {
            let cand = match base.rsplit_once('.') {
                Some((stem, ext)) if !stem.is_empty() => format!("{stem} {i}.{ext}"),
                _ => format!("{base} {i}"),
            };
            if !dir.children.iter().any(|c| c.name == cand) {
                return cand;
            }
        }
        base.into()
    }
}

pub struct FsG(pub Rc<RefCell<Fs>>);
impl Global for FsG {}

pub fn demo() -> Fs {
    let pics = d(
        "Pictures",
        200.0,
        vec![
            f("4K Frosted Glass.jpg", 1500 * KB, 0.1),
            f("Blossom Study.jpg", 812 * KB, 96.0),
            f("Cycling in Soweto.png", 977 * KB, 72.0),
            f("Green Fold.png", 868 * KB, 16.0),
            f("Nixio Dashboard.png", 1700 * KB, 1.0),
            f("Orbit Poster.png", 1200 * KB, 60.0),
            f("Soweto Moto Ad.png", 1100 * KB, 67.0),
            f("Window Study.png", 878 * KB, 12.0),
            d(
                "Amapiano Pack Vol. 1",
                7.0,
                vec![
                    f("01 Log Drum 126 BPM.wav", 18_400 * KB, 7.0),
                    f("02 Shaker Groove.wav", 6_200 * KB, 7.2),
                    f("Amapiano Pack Vol. 1.flp", 4_800 * KB, 14.0),
                    f("Pack Cover.jpg", 2_100 * KB, 9.0),
                ],
            ),
            d(
                "Log Drum Sessions",
                20.0,
                vec![
                    f("Log Drum 126.wav", 12_800 * KB, 20.0),
                    f("Log Drum 128.wav", 6_400 * KB, 20.2),
                    ft("Session Notes.txt", 12 * KB, 20.5, "Session notes\n=============\n\nLog drum at 126 BPM sits best with the shaker pushed 12 ms late.\nTODO: bounce stems, label takes, archive the raw takes."),
                ],
            ),
            d(
                "SMART-DAW Beta",
                30.0,
                vec![f("smart-daw-0.4.2.AppImage", 22_400 * KB, 30.0), f("Layout Study.png", 1_400 * KB, 31.0), ft("beta-notes.md", 12 * KB, 36.0, "# SMART-DAW beta\n\n- Piano roll snaps to the 8 px grid\n- Log drum preset needs more punch\n- Ship the AppImage on Friday")],
            ),
            d("Brand \u{2014} Naledzi", 96.0, vec![f("Naledzi Wordmark.svg", 12 * KB, 96.0), f("Reference 01.jpg", 1_600 * KB, 97.0), f("Type Specimen.pdf", 4_200 * KB, 98.0)]),
        ],
    );
    let docs = d(
        "Documents",
        180.0,
        vec![
            ft("Brand Brief.md", 3 * KB, 5.0, "# Naledzi brand brief\n\nCobalt is the only accent. Depth comes from tone and shadow, never from strokes.\n\n## Voice\nCalm, precise, a little warm."),
            ft("Q4 Roadmap.txt", 2 * KB, 30.0, "Q4 roadmap\n\n1. Smart Files 0.2 - cloud accounts and phone continuity\n2. Smart DAW beta - log drum instrument\n3. Design contract v0.2"),
            ft("invoice-2026-014.txt", 1 * KB, 50.0, "INVOICE 2026-014\nClient: Soweto Moto\nAmount: R 24 500\nStatus: awaiting payment"),
            f("Type Specimen.pdf", 4_200 * KB, 98.0),
            f("Budget 2026.xlsx", 212 * KB, 120.0),
            f("Studio Master Reel 2026.mov", 5 * GB, 6.0),
            f("Backup Snapshot 2026.sparse", 5 * GB, 48.0),
            ft("main.rs", 1 * KB, 70.0, "fn main() {\n    println!(\"Hello from Smart Files\");\n}"),
        ],
    );
    let downloads = d(
        "Downloads",
        60.0,
        vec![f("Brand Master.zip", 248 * MB, 22.0), f("Landing Draft.png", 1_250 * KB, 26.0), f("Package B.png", 1_400 * KB, 40.0), f("smart-daw-0.4.2.AppImage", 22_400 * KB, 41.0)],
    );
    let music = d("Music", 60.0, vec![f("Pad Warm.wav", 9 * MB, 60.0), f("Piano Lead.wav", 7 * MB, 61.0), f("Bass Deep.wav", 8 * MB, 62.0), f("Log Drum Conga.wav", 6 * MB, 63.0)]);
    let movies = d("Movies", 80.0, vec![f("Soweto Ride Teaser.mp4", 842 * MB, 80.0), f("Studio Tour.mov", 1_300 * MB, 90.0)]);
    let desktop = d("Desktop", 2.0, vec![f("Screenshot 2026-11-01.png", 2_300 * KB, 2.0), ft("Todo.txt", 1 * KB, 3.0, "- reply to Naledzi\n- export album cover\n- eject the SanDisk before leaving")]);
    let mut home = d("Home", 1.0, vec![desktop, docs, downloads, pics, music, movies]);
    let tags: &[(&str, u8)] = &[("Amapiano Pack Vol. 1", 1), ("Brand \u{2014} Naledzi", 2), ("Log Drum Sessions", 1), ("SMART-DAW Beta", 2), ("Blossom Study.jpg", 3), ("Cycling in Soweto.png", 3), ("Green Fold.png", 2), ("Orbit Poster.png", 2), ("Soweto Moto Ad.png", 1)];
    fn tag_walk(n: &mut Node, tags: &[(&str, u8)]) {
        for c in n.children.iter_mut() {
            if let Some((_, t)) = tags.iter().find(|(nm, _)| *nm == c.name) { c.tag = *t; }
            tag_walk(c, tags);
        }
    }
    tag_walk(&mut home, tags);
    let usb = d(
        "SanDisk",
        2.0,
        vec![f("Amapiano Pack Vol. 1.flp", 4_800 * KB, 2.0), f("Pack Cover.jpg", 2_100 * KB, 5.0), ft("Handover.txt", 4 * KB, 30.0, "Handover notes for the studio.\nKeep the stems in /Stems.")],
    );
    let nas = d(
        "Studio NAS",
        3.0,
        vec![f("Brand Master.zip", 248 * MB, 14.0), f("Type Specimen.pdf", 4_200 * KB, 90.0), d("Backups", 3.0, vec![f("Session Backup 01.wav", 18_400 * KB, 2.0)])],
    );
    Fs { root: d("/", 0.0, vec![home, usb, nas]), trash: vec![] }
}

pub fn fmt_size(b: u64) -> String {
    let b = b as f64;
    if b >= 1e9 {
        format!("{:.1} GB", b / 1e9)
    } else if b >= 1e6 {
        let v = b / 1e6;
        if v < 100.0 { format!("{:.1} MB", v) } else { format!("{:.0} MB", v) }
    } else if b >= 1e3 {
        format!("{:.0} KB", b / 1e3)
    } else {
        format!("{} B", b as u64)
    }
}

pub fn tag_color_id(n: &Node) -> u8 { n.tag }

pub fn fmt_when(h: f32) -> String {
    let mins = ((h * 60.0) as u32) % 1440;
    let hh = (17 + (mins / 60)) % 24;
    let mm = (mins % 60 + 37) % 60;
    if h < 24.0 {
        format!("Today, {hh:02}:{mm:02}")
    } else if h < 48.0 {
        format!("Yesterday, {hh:02}:{mm:02}")
    } else {
        let day = 10 - ((h / 24.0) as u32).min(9);
        format!("Nov {day}, {hh:02}:{mm:02}")
    }
}

pub fn fmt_ago(h: f32) -> String {
    if h < 0.02 {
        "Just now".into()
    } else if h < 1.0 {
        format!("{} min ago", (h * 60.0).round() as u32)
    } else if h < 24.0 {
        format!("{} h ago", h.round() as u32)
    } else if h < 24.0 * 14.0 {
        let dd = (h / 24.0).round() as u32;
        if dd == 1 { "Yesterday".into() } else { format!("{dd} days ago") }
    } else {
        format!("{} weeks ago", (h / 24.0 / 7.0).round() as u32)
    }
}
