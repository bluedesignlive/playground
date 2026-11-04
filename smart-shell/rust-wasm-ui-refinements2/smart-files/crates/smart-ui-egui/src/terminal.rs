//! Smart Terminal — a modern, friendly shell that runs on the same virtual file system as the file manager.
//! Command blocks with status chips, syntax-highlighted input, history ghost-text, Tab completion with candidate
//! chips, clickable paths, pipes (`|`) and redirects (`>`, `>>`). Pure Rust, drawn with the shared design system.
use crate::app::*;
use crate::chrome::*;
use crate::icons::Ic;
use crate::paint::*;
use crate::shell::AppId;
use crate::theme::*;
use egui::{pos2, vec2, Align2, Color32, FontId, Id, Rect, Sense, Ui};
use smart_core::*;

#[derive(Clone, Copy, PartialEq)]
pub enum St {
    Plain,
    Dim,
    Accent,
    Good,
    Warn,
    Err,
    Dir,
    Pink,
    Rgb(Color32),
}

#[derive(Clone)]
pub struct Span {
    pub text: String,
    pub st: St,
    pub link: Option<Location>,
    pub bold: bool,
}

#[derive(Clone)]
pub enum TLine {
    Spans(Vec<Span>),
    Gap,
    /// Entries flow like chips (friendly `ls`).
    Entries(Vec<(String, Kind, bool, Location)>),
    /// Clickable command suggestions.
    Chips(Vec<String>),
    Title(String, String),
}

fn sp(text: impl Into<String>, st: St) -> Span {
    Span { text: text.into(), st, link: None, bold: false }
}
fn plain(text: impl Into<String>) -> TLine {
    TLine::Spans(vec![sp(text, St::Plain)])
}
fn line(v: Vec<Span>) -> TLine {
    TLine::Spans(v)
}
fn err(text: impl Into<String>) -> TLine {
    TLine::Spans(vec![sp(text, St::Err)])
}

pub struct Block {
    pub cmd: String,
    pub cwd: Location,
    pub lines: Vec<TLine>,
    pub code: i32,
    pub ms: u32,
}

pub struct Session {
    pub id: u32,
    pub cwd: Location,
    pub blocks: Vec<Block>,
    pub input: String,
    pub cursor: usize,
    pub hist_i: Option<usize>,
    pub draft: String,
    pub cands: Vec<String>,
    pub cand_i: Option<usize>,
    pub cand_start: usize,
    pub to_bottom: bool,
}

pub struct Terminal {
    pub sessions: Vec<Session>,
    pub active: usize,
    pub history: Vec<String>,
    pub font: f32,
    pub home: Location,
    next_id: u32,
    pub booted: bool,
    pub pending_run: Option<String>,
}

pub const COMMANDS: &[(&str, &str)] = &[
    ("help", "Show everything this shell can do"),
    ("ls", "List a folder  (-l details, -a hidden)"),
    ("cd", "Change folder  (cd ~, cd .., cd -)"),
    ("pwd", "Where am I?"),
    ("tree", "Folder tree  (-L depth)"),
    ("cat", "Print a file"),
    ("head", "First lines of a file  (-n N)"),
    ("tail", "Last lines of a file  (-n N)"),
    ("wc", "Count lines, words and bytes"),
    ("echo", "Print text  (use > file to save)"),
    ("mkdir", "Create folders  (-p for parents)"),
    ("touch", "Create empty files"),
    ("rm", "Move to Trash  (-r for folders)"),
    ("mv", "Move or rename"),
    ("cp", "Copy files and folders"),
    ("stat", "Details about one item"),
    ("du", "Disk usage of a folder"),
    ("find", "Find by name, recursively"),
    ("grep", "Search text  (-i ignore case)"),
    ("sort", "Sort piped lines"),
    ("uniq", "Remove duplicate piped lines"),
    ("open", "Open in the right app"),
    ("files", "Open a folder in Files"),
    ("photos", "Open an image in Photos"),
    ("unzip", "Extract a ZIP  (-p password)"),
    ("zip", "Create a ZIP  (zip out.zip a b c)"),
    ("theme", "dark | light | toggle"),
    ("wallpaper", "Pick a wallpaper by name"),
    ("calc", "Calculator  (calc 2*(3+4))"),
    ("neofetch", "About this system"),
    ("date", "Date and time"),
    ("whoami", "Current user"),
    ("history", "Recent commands"),
    ("clear", "Clear the screen  (Ctrl+L)"),
    ("exit", "Close this session"),
];

impl Terminal {
    pub fn new(home: Location) -> Self {
        Self { sessions: vec![], active: 0, history: vec![], font: 13.0, home, next_id: 1, booted: false, pending_run: None }
    }

    fn new_session_at(&mut self, cwd: Location, time: f64) {
        let id = self.next_id;
        self.next_id += 1;
        let _ = time;
        let welcome = Block {
            cmd: String::new(),
            cwd: cwd.clone(),
            lines: vec![
                TLine::Title("Smart Terminal".into(), "Same file system as Files and Photos \u{b7} wasm \u{b7} pure Rust".into()),
                TLine::Gap,
                line(vec![sp("Try ", St::Dim), sp("Tab", St::Accent), sp(" to complete, ", St::Dim), sp("\u{2191}", St::Accent), sp(" for history, click any path to jump there.", St::Dim)]),
                TLine::Chips(vec!["ls".into(), "tree -L 2".into(), "neofetch".into(), "help".into(), "calc 21*2".into()]),
            ],
            code: 0,
            ms: 0,
        };
        self.sessions.push(Session { id, cwd, blocks: vec![welcome], input: String::new(), cursor: 0, hist_i: None, draft: String::new(), cands: vec![], cand_i: None, cand_start: 0, to_bottom: true });
        self.active = self.sessions.len() - 1;
    }

    pub fn cd_to(&mut self, l: Location) {
        if self.sessions.is_empty() {
            self.new_session_at(l, 0.0);
            return;
        }
        let a = self.active.min(self.sessions.len() - 1);
        self.pending_run = Some(format!("cd {}", quote(&l.path)));
        let _ = a;
        self.sessions[a].cwd = l.clone();
        self.sessions[a].blocks.push(Block { cmd: format!("cd {}", quote(&l.path)), cwd: l, lines: vec![], code: 0, ms: 0 });
        self.sessions[a].to_bottom = true;
        self.pending_run = None;
    }

    pub fn command(&mut self, c: &str) {
        match c {
            "new" => {
                let cwd = self.sessions.get(self.active).map(|s| s.cwd.clone()).unwrap_or_else(|| self.home.clone());
                self.new_session_at(cwd, 0.0);
            }
            "close" => {
                if self.sessions.len() > 1 {
                    self.sessions.remove(self.active);
                    self.active = self.active.min(self.sessions.len() - 1);
                }
            }
            "clear" => {
                if let Some(s) = self.sessions.get_mut(self.active) {
                    s.blocks.clear();
                }
            }
            "bigger" => self.font = (self.font + 1.0).min(20.0),
            "smaller" => self.font = (self.font - 1.0).max(10.0),
            _ => {}
        }
    }
}

fn quote(s: &str) -> String {
    if s.contains(' ') {
        format!("\"{}\"", s)
    } else {
        s.to_string()
    }
}

fn tokenize(s: &str) -> Vec<String> {
    let mut out = vec![];
    let mut cur = String::new();
    let mut q: Option<char> = None;
    let mut has = false;
    let mut it = s.chars().peekable();
    while let Some(ch) = it.next() {
        match (q, ch) {
            (Some(c), x) if x == c => q = None,
            (Some(_), x) => cur.push(x),
            (None, '"') | (None, '\'') => {
                q = Some(ch);
                has = true;
            }
            (None, '\\') => {
                if let Some(n) = it.next() {
                    cur.push(n);
                    has = true;
                }
            }
            (None, c) if c.is_whitespace() => {
                if has || !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                    has = false;
                }
            }
            (None, c) => cur.push(c),
        }
    }
    if has || !cur.is_empty() {
        out.push(cur);
    }
    out
}

fn text_of(lines: &[TLine]) -> Vec<String> {
    let mut v = vec![];
    for l in lines {
        match l {
            TLine::Spans(s) => v.push(s.iter().map(|x| x.text.as_str()).collect::<String>()),
            TLine::Gap => v.push(String::new()),
            TLine::Entries(e) => v.extend(e.iter().map(|x| x.0.clone())),
            TLine::Chips(c) => v.push(c.join("  ")),
            TLine::Title(a, b) => {
                v.push(a.clone());
                v.push(b.clone());
            }
        }
    }
    v
}

pub struct Out {
    pub lines: Vec<TLine>,
    pub code: i32,
    pub cd: Option<Location>,
    pub clear: bool,
    pub acts: Vec<Act>,
}

impl Out {
    fn ok(lines: Vec<TLine>) -> Self {
        Self { lines, code: 0, cd: None, clear: false, acts: vec![] }
    }
    fn fail(msg: impl Into<String>) -> Self {
        Self { lines: vec![err(msg)], code: 1, cd: None, clear: false, acts: vec![] }
    }
}

fn calc(expr: &str) -> Result<f64, String> {
    struct P<'a> {
        s: &'a [u8],
        i: usize,
    }
    impl P<'_> {
        fn ws(&mut self) {
            while self.i < self.s.len() && self.s[self.i] == b' ' {
                self.i += 1;
            }
        }
        fn expr(&mut self) -> Result<f64, String> {
            let mut v = self.term()?;
            loop {
                self.ws();
                match self.s.get(self.i) {
                    Some(b'+') => {
                        self.i += 1;
                        v += self.term()?;
                    }
                    Some(b'-') => {
                        self.i += 1;
                        v -= self.term()?;
                    }
                    _ => return Ok(v),
                }
            }
        }
        fn term(&mut self) -> Result<f64, String> {
            let mut v = self.pow()?;
            loop {
                self.ws();
                match self.s.get(self.i) {
                    Some(b'*') | Some(b'x') => {
                        self.i += 1;
                        v *= self.pow()?;
                    }
                    Some(b'/') => {
                        self.i += 1;
                        let d = self.pow()?;
                        if d == 0.0 {
                            return Err("division by zero".into());
                        }
                        v /= d;
                    }
                    Some(b'%') => {
                        self.i += 1;
                        v %= self.pow()?;
                    }
                    _ => return Ok(v),
                }
            }
        }
        fn pow(&mut self) -> Result<f64, String> {
            let b = self.atom()?;
            self.ws();
            if self.s.get(self.i) == Some(&b'^') {
                self.i += 1;
                let e = self.pow()?;
                return Ok(b.powf(e));
            }
            Ok(b)
        }
        fn atom(&mut self) -> Result<f64, String> {
            self.ws();
            match self.s.get(self.i) {
                Some(b'(') => {
                    self.i += 1;
                    let v = self.expr()?;
                    self.ws();
                    if self.s.get(self.i) != Some(&b')') {
                        return Err("missing )".into());
                    }
                    self.i += 1;
                    Ok(v)
                }
                Some(b'-') => {
                    self.i += 1;
                    Ok(-self.atom()?)
                }
                Some(c) if c.is_ascii_digit() || *c == b'.' => {
                    let st = self.i;
                    while self.i < self.s.len() && (self.s[self.i].is_ascii_digit() || self.s[self.i] == b'.') {
                        self.i += 1;
                    }
                    std::str::from_utf8(&self.s[st..self.i]).unwrap().parse::<f64>().map_err(|_| "bad number".into())
                }
                _ => Err("unexpected input".into()),
            }
        }
    }
    let mut p = P { s: expr.as_bytes(), i: 0 };
    let v = p.expr()?;
    p.ws();
    if p.i != p.s.len() {
        return Err("unexpected input".into());
    }
    Ok(v)
}

fn edit_dist(a: &str, b: &str) -> usize {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    let mut d: Vec<usize> = (0..=b.len()).collect();
    for i in 1..=a.len() {
        let mut prev = d[0];
        d[0] = i;
        for j in 1..=b.len() {
            let t = d[j];
            d[j] = (d[j] + 1).min(d[j - 1] + 1).min(prev + (a[i - 1] != b[j - 1]) as usize);
            prev = t;
        }
    }
    d[b.len()]
}

impl SmartApp {
    pub fn short_path(&self, l: &Location) -> String {
        let home = self.be.home();
        if l.fs == home.fs && l.starts_with(&home) {
            let rest = l.path.strip_prefix(&home.path).unwrap_or("");
            return format!("~{}", rest);
        }
        if l.is_root() { format!("{}:/", l.fs) } else { l.path.clone() }
    }

    fn resolve(&self, cwd: &Location, arg: &str) -> Location {
        let home = self.be.home();
        let (mut cur, rest): (Location, &str) = if arg == "~" {
            (home, "")
        } else if let Some(r) = arg.strip_prefix("~/") {
            (home, r)
        } else if let Some(r) = arg.strip_prefix('/') {
            (Location::new(cwd.fs.clone(), "/"), r)
        } else {
            (cwd.clone(), arg)
        };
        for seg in rest.split('/') {
            match seg {
                "" | "." => {}
                ".." => {
                    if let Some(p) = cur.parent() {
                        cur = p;
                    }
                }
                s => cur = cur.join(s),
            }
        }
        cur
    }

    fn list_dir(&self, l: &Location) -> Result<Vec<Entry>, String> {
        let mut v = self.be.registry.get(&l.fs).ok_or("no such file system")?.list(&l.path).map_err(|e| e.to_string())?;
        sort_entries(&mut v, SortKey::Name, true, true);
        Ok(v)
    }

    fn walk(&self, l: &Location, depth: usize, limit: &mut usize, f: &mut dyn FnMut(&Entry, usize)) {
        if let Ok(v) = self.list_dir(l) {
            for e in v {
                if *limit == 0 {
                    return;
                }
                *limit -= 1;
                f(&e, depth);
                if e.is_dir && depth < 12 {
                    self.walk(&e.loc, depth + 1, limit, f);
                }
            }
        }
    }

    fn dir_size(&self, l: &Location, budget: &mut usize) -> u64 {
        let mut total = 0;
        if let Ok(v) = self.list_dir(l) {
            for e in v {
                if *budget == 0 {
                    break;
                }
                *budget -= 1;
                total += if e.is_dir { self.dir_size(&e.loc, budget) } else { e.size };
            }
        }
        total
    }

    fn entry_span(&self, e: &Entry) -> Span {
        Span { text: if e.is_dir { format!("{}/", e.name) } else { e.name.clone() }, st: if e.is_dir { St::Dir } else { St::Plain }, link: Some(e.loc.clone()), bold: e.is_dir }
    }

    /// Run a full command line (pipes + redirect) against the shared backend.
    pub fn term_exec_line(&mut self, cwd: &Location, input: &str) -> Out {
        let (cmdline, redirect) = if let Some((a, b)) = input.split_once(">>") {
            (a.trim().to_string(), Some((b.trim().to_string(), true)))
        } else if let Some((a, b)) = input.split_once('>') {
            (a.trim().to_string(), Some((b.trim().to_string(), false)))
        } else {
            (input.trim().to_string(), None)
        };
        let stages: Vec<String> = cmdline.split(" | ").map(|s| s.trim().to_string()).collect();
        let mut stdin: Option<Vec<String>> = None;
        let mut last = Out::ok(vec![]);
        let mut cd_acc: Option<Location> = None;
        let mut acts = vec![];
        let mut cur = cwd.clone();
        for (i, st) in stages.iter().enumerate() {
            let out = self.term_exec_one(&cur, st, stdin.take());
            if let Some(c) = &out.cd {
                cur = c.clone();
                cd_acc = Some(c.clone());
            }
            acts.extend(out.acts.clone());
            if out.code != 0 && i + 1 < stages.len() {
                return Out { acts, ..out };
            }
            stdin = Some(text_of(&out.lines));
            last = out;
        }
        last.acts = acts;
        last.cd = cd_acc;
        if let Some((target, append)) = redirect {
            if target.is_empty() {
                return Out::fail("redirect: missing file name");
            }
            let dest = self.resolve(&cur, &target);
            let mut text = text_of(&last.lines).join("\n");
            text.push('\n');
            if append {
                if let Ok(old) = self.be.read_text(&dest, 4_000_000) {
                    text = format!("{old}{text}");
                }
            }
            if !self.be.stat(&dest).is_ok() {
                if let Some(parent) = dest.parent() {
                    let _ = self.be.create_file(&parent, &dest.name());
                }
            }
            return match self.be.write_text(&dest, &text) {
                Ok(()) => Out { lines: vec![line(vec![sp(if append { "appended to " } else { "wrote " }, St::Dim), Span { text: self.short_path(&dest), st: St::Accent, link: Some(dest.clone()), bold: false }])], code: 0, cd: last.cd, clear: false, acts: last.acts },
                Err(e) => Out::fail(format!("cannot write {}: {e}", target)),
            };
        }
        last
    }

    fn term_exec_one(&mut self, cwd: &Location, cmdline: &str, stdin: Option<Vec<String>>) -> Out {
        let toks = tokenize(cmdline);
        let Some(cmd) = toks.first().cloned() else { return Out::ok(vec![]) };
        let args: Vec<String> = toks[1..].to_vec();
        let flags: Vec<&String> = args.iter().filter(|a| a.starts_with('-') && a.len() > 1 && !a[1..].chars().all(|c| c.is_ascii_digit())).collect();
        let has = |f: char| flags.iter().any(|a| !a.starts_with("--") && a.contains(f));
        let pos: Vec<String> = args.iter().filter(|a| !(a.starts_with('-') && a.len() > 1)).cloned().collect();
        let opt_n = |name: &str, default: usize| -> usize { args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).and_then(|v| v.parse().ok()).unwrap_or(default) };
        match cmd.as_str() {
            "help" | "?" => {
                let mut v = vec![TLine::Title("Commands".into(), "Pipes and redirects work: ls | grep png > pics.txt".into()), TLine::Gap];
                for (c, d) in COMMANDS {
                    v.push(line(vec![sp(format!("{:<10}", c), St::Accent), sp(*d, St::Dim)]));
                }
                v.push(TLine::Gap);
                v.push(line(vec![sp("Shortcuts  ", St::Dim), sp("Tab", St::Accent), sp(" complete   ", St::Dim), sp("\u{2191}\u{2193}", St::Accent), sp(" history   ", St::Dim), sp("Ctrl+L", St::Accent), sp(" clear   ", St::Dim), sp("Ctrl+C", St::Accent), sp(" cancel line", St::Dim)]));
                Out::ok(v)
            }
            "clear" | "cls" => Out { clear: true, ..Out::ok(vec![]) },
            "exit" | "logout" => Out { acts: vec![Act::App("term.close".into())], ..Out::ok(vec![line(vec![sp("closing session\u{2026}", St::Dim)])]) },
            "pwd" => Out::ok(vec![line(vec![Span { text: cwd.path.clone(), st: St::Accent, link: Some(cwd.clone()), bold: false }, sp(format!("   {}", cwd.fs), St::Dim)])]),
            "whoami" => Out::ok(vec![plain(self.user_name().to_lowercase())]),
            "date" => Out::ok(vec![plain(format!("{}  {}", clock_date(), clock_hm()))]),
            "history" => {
                let h = self.shell.term.history.clone();
                Out::ok(h.iter().enumerate().rev().take(40).rev().map(|(i, c)| line(vec![sp(format!("{:>4}  ", i + 1), St::Dim), sp(c.clone(), St::Plain)])).collect())
            }
            "echo" => {
                let home = self.be.home();
                let s = args.join(" ").replace("$PWD", &cwd.path).replace("$HOME", &home.path).replace("$USER", &self.user_name().to_lowercase());
                Out::ok(vec![plain(s)])
            }
            "calc" | "=" => match calc(&args.join(" ")) {
                Ok(v) => {
                    let s = if v.fract() == 0.0 && v.abs() < 1e15 { format!("{}", v as i64) } else { format!("{:.6}", v).trim_end_matches('0').trim_end_matches('.').to_string() };
                    Out::ok(vec![line(vec![sp(args.join(" "), St::Dim), sp("  =  ", St::Dim), Span { text: s, st: St::Good, link: None, bold: true }])])
                }
                Err(e) => Out::fail(format!("calc: {e}")),
            },
            "theme" => {
                let want = pos.first().map(|s| s.as_str()).unwrap_or("toggle");
                let dark = self.st().dark;
                let act = match want {
                    "dark" if !dark => vec![Act::ToggleTheme],
                    "light" if dark => vec![Act::ToggleTheme],
                    "toggle" => vec![Act::ToggleTheme],
                    _ => vec![],
                };
                Out { acts: act, ..Out::ok(vec![line(vec![sp("theme \u{2192} ", St::Dim), sp(if want == "toggle" { if dark { "light" } else { "dark" } } else { want }, St::Accent)])]) }
            }
            "wallpaper" => {
                let names: Vec<&str> = WALLPAPERS.iter().map(|w| w.0).collect();
                match pos.first().and_then(|n| names.iter().position(|x| x.to_lowercase().starts_with(&n.to_lowercase()))) {
                    Some(i) => Out { acts: vec![Act::SetWallpaper(i)], ..Out::ok(vec![line(vec![sp("wallpaper \u{2192} ", St::Dim), sp(names[i], St::Accent)])]) },
                    None => Out::ok(vec![line(vec![sp("available: ", St::Dim), sp(names.join(", "), St::Accent)])]),
                }
            }
            "neofetch" => {
                let th = self.th.clone();
                let sc = self.st();
                let blocks: Vec<Span> = [th.accent, th.good, th.warn, th.danger, hex("#d86fb0"), hex("#8f7cf5"), th.ink3, th.ink1].iter().map(|c| Span { text: "\u{2588}\u{2588}\u{2588}".into(), st: St::Rgb(*c), link: None, bold: false }).collect();
                let kv = |k: &str, v: String| line(vec![sp(format!("{:<11}", k), St::Accent), sp(v, St::Plain)]);
                let up = self.time as u64;
                Out::ok(vec![
                    TLine::Title(format!("{}@smart", self.user_name().to_lowercase()), "\u{2500}".repeat(22)),
                    kv("OS", "Smart Shell 0.1 (wasm32)".into()),
                    kv("Shell", "smart-term".into()),
                    kv("Files", format!("{} places \u{b7} {} tabs", self.places.len(), self.tabs.len())),
                    kv("Theme", format!("{} \u{b7} accent {}", if sc.dark { "Dark" } else { "Light" }, "#2f6feb")),
                    kv("Icons", "Solar \u{b7} linear / duotone".into()),
                    kv("Font", "Inter + system mono".into()),
                    kv("Engine", "egui + wgpu-less glow \u{b7} Rust".into()),
                    kv("Uptime", format!("{}m {}s", up / 60, up % 60)),
                    TLine::Gap,
                    line(blocks),
                ])
            }
            "cd" => {
                let target = match pos.first().map(|s| s.as_str()) {
                    None => self.be.home(),
                    Some("-") => self.shell.term.sessions.get(self.shell.term.active).and_then(|s| s.blocks.iter().rev().find(|b| b.cmd.starts_with("cd ") && b.code == 0).map(|b| b.cwd.clone())).unwrap_or_else(|| cwd.clone()),
                    Some(p) => self.resolve(cwd, p),
                };
                match self.be.stat(&target) {
                    Ok(e) if e.is_dir => Out { cd: Some(target), ..Out::ok(vec![]) },
                    Ok(_) => Out::fail(format!("cd: not a folder: {}", pos[0])),
                    Err(_) if target.is_root() => Out { cd: Some(target), ..Out::ok(vec![]) },
                    Err(_) => Out::fail(format!("cd: no such folder: {}", pos.first().cloned().unwrap_or_default())),
                }
            }
            "ls" | "dir" | "ll" => {
                let target = pos.first().map(|p| self.resolve(cwd, p)).unwrap_or_else(|| cwd.clone());
                let long = has('l') || cmd == "ll";
                match self.list_dir(&target) {
                    Err(e) => Out::fail(format!("ls: {e}")),
                    Ok(v) => {
                        let v: Vec<Entry> = v.into_iter().filter(|e| has('a') || !e.hidden).collect();
                        if v.is_empty() {
                            return Out::ok(vec![line(vec![sp("(empty)", St::Dim)])]);
                        }
                        if long {
                            let mut rows = vec![line(vec![sp(format!("{:>9}  {:<18}  name", "size", "modified"), St::Dim)])];
                            for e in &v {
                                rows.push(line(vec![sp(format!("{:>9}  {:<18}  ", if e.is_dir { "\u{2014}".to_string() } else { format_size(e.size) }, e.modified.map(format_date).unwrap_or_default()), St::Dim), self.entry_span(e)]));
                            }
                            rows.push(line(vec![sp(format!("{} items", v.len()), St::Dim)]));
                            Out::ok(rows)
                        } else {
                            Out::ok(vec![TLine::Entries(v.iter().map(|e| (e.name.clone(), e.kind, e.is_dir, e.loc.clone())).collect())])
                        }
                    }
                }
            }
            "tree" => {
                let depth = if let Some(i) = args.iter().position(|a| a == "-L") { args.get(i + 1).and_then(|v| v.parse().ok()).unwrap_or(2) } else { 2usize };
                let target = pos.iter().find(|p| p.parse::<usize>().is_err()).map(|p| self.resolve(cwd, p)).unwrap_or_else(|| cwd.clone());
                let mut rows = vec![line(vec![Span { text: if target == *cwd { ".".into() } else { target.name() }, st: St::Dir, link: Some(target.clone()), bold: true }])];
                fn rec(app: &SmartApp, l: &Location, prefix: &str, depth: usize, max: usize, rows: &mut Vec<TLine>, n: &mut usize) {
                    let Ok(v) = app.list_dir(l) else { return };
                    let v: Vec<Entry> = v.into_iter().filter(|e| !e.hidden).collect();
                    for (i, e) in v.iter().enumerate() {
                        if *n > 300 {
                            return;
                        }
                        *n += 1;
                        let last = i + 1 == v.len();
                        rows.push(line(vec![sp(format!("{}{}", prefix, if last { "\u{2514}\u{2500}\u{2500} " } else { "\u{251c}\u{2500}\u{2500} " }), St::Dim), app.entry_span(e)]));
                        if e.is_dir && depth + 1 < max {
                            rec(app, &e.loc, &format!("{}{}", prefix, if last { "    " } else { "\u{2502}   " }), depth + 1, max, rows, n);
                        }
                    }
                }
                let mut n = 0;
                rec(self, &target, "", 0, depth, &mut rows, &mut n);
                if n > 300 {
                    rows.push(line(vec![sp("\u{2026} truncated", St::Dim)]));
                }
                Out::ok(rows)
            }
            "cat" | "head" | "tail" | "wc" => {
                let text: Result<Vec<String>, String> = if let Some(p) = pos.iter().find(|p| p.parse::<usize>().is_err()) {
                    let l = self.resolve(cwd, p);
                    match self.be.read_text(&l, 512 * 1024) {
                        Ok(t) => Ok(t.lines().map(String::from).collect()),
                        Err(e) => Err(format!("{cmd}: {p}: {e}")),
                    }
                } else if let Some(s) = stdin {
                    Ok(s)
                } else {
                    Err(format!("{cmd}: missing file"))
                };
                match text {
                    Err(e) => Out::fail(e),
                    Ok(lines) => match cmd.as_str() {
                        "head" => Out::ok(lines.iter().take(opt_n("-n", 10)).map(|l| plain(l.clone())).collect()),
                        "tail" => {
                            let n = opt_n("-n", 10);
                            Out::ok(lines.iter().skip(lines.len().saturating_sub(n)).map(|l| plain(l.clone())).collect())
                        }
                        "wc" => {
                            let words: usize = lines.iter().map(|l| l.split_whitespace().count()).sum();
                            let bytes: usize = lines.iter().map(|l| l.len() + 1).sum();
                            Out::ok(vec![line(vec![sp(format!("{} lines  {} words  {} bytes", lines.len(), words, bytes), St::Plain)])])
                        }
                        _ => Out::ok(lines.into_iter().take(2000).map(plain).collect()),
                    },
                }
            }
            "sort" | "uniq" => match stdin {
                None => Out::fail(format!("{cmd}: pipe something in, e.g. ls -a | {cmd}")),
                Some(mut l) => {
                    if cmd == "sort" {
                        l.sort();
                        if has('r') {
                            l.reverse();
                        }
                    } else {
                        l.dedup();
                    }
                    Out::ok(l.into_iter().map(plain).collect())
                }
            },
            "mkdir" => {
                if pos.is_empty() {
                    return Out::fail("mkdir: missing name");
                }
                let mut v = vec![];
                for p in &pos {
                    let l = self.resolve(cwd, p);
                    if has('p') {
                        for a in l.ancestors().into_iter().skip(1) {
                            if self.be.stat(&a).is_err() {
                                if let Some(par) = a.parent() {
                                    let _ = self.be.create_folder(&par, &a.name());
                                }
                            }
                        }
                        v.push(line(vec![sp("created ", St::Dim), Span { text: self.short_path(&l), st: St::Dir, link: Some(l.clone()), bold: false }]));
                    } else if let Some(par) = l.parent() {
                        match self.be.create_folder(&par, &l.name()) {
                            Ok(n) => v.push(line(vec![sp("created ", St::Dim), Span { text: self.short_path(&n), st: St::Dir, link: Some(n), bold: false }])),
                            Err(e) => return Out::fail(format!("mkdir: {p}: {e}")),
                        }
                    }
                }
                Out { acts: vec![Act::Refresh], ..Out::ok(v) }
            }
            "touch" => {
                if pos.is_empty() {
                    return Out::fail("touch: missing name");
                }
                for p in &pos {
                    let l = self.resolve(cwd, p);
                    if self.be.stat(&l).is_err() {
                        if let Some(par) = l.parent() {
                            if let Err(e) = self.be.create_file(&par, &l.name()) {
                                return Out::fail(format!("touch: {p}: {e}"));
                            }
                        }
                    }
                }
                Out { acts: vec![Act::Refresh], ..Out::ok(vec![line(vec![sp(format!("touched {}", pos.join(", ")), St::Dim)])]) }
            }
            "rm" => {
                if pos.is_empty() {
                    return Out::fail("rm: missing path");
                }
                let mut srcs = vec![];
                for p in &pos {
                    let l = self.resolve(cwd, p);
                    match self.be.stat(&l) {
                        Ok(e) if e.is_dir && !(has('r') || has('f')) => return Out::fail(format!("rm: {p} is a folder (use -r)")),
                        Ok(_) => srcs.push(l),
                        Err(_) => return Out::fail(format!("rm: {p}: no such file or folder")),
                    }
                }
                let n = srcs.len();
                self.submit_job(JobKind::Trash { srcs });
                Out::ok(vec![line(vec![sp(format!("moved {n} item{} to Trash", if n == 1 { "" } else { "s" }), St::Warn)])])
            }
            "mv" | "cp" => {
                if pos.len() < 2 {
                    return Out::fail(format!("{cmd}: usage: {cmd} <source...> <destination>"));
                }
                let dest = self.resolve(cwd, pos.last().unwrap());
                let srcs: Vec<Location> = pos[..pos.len() - 1].iter().map(|p| self.resolve(cwd, p)).collect();
                for s in &srcs {
                    if self.be.stat(s).is_err() {
                        return Out::fail(format!("{cmd}: {}: no such file or folder", s.name()));
                    }
                }
                let dest_is_dir = self.be.stat(&dest).map(|e| e.is_dir).unwrap_or(false);
                if dest_is_dir {
                    let n = srcs.len();
                    self.submit_job(if cmd == "mv" { JobKind::Move { srcs, dest: dest.clone() } } else { JobKind::Copy { srcs, dest: dest.clone() } });
                    return Out::ok(vec![line(vec![sp(format!("{} {n} item{} \u{2192} ", if cmd == "mv" { "moving" } else { "copying" }, if n == 1 { "" } else { "s" }), St::Dim), Span { text: self.short_path(&dest), st: St::Dir, link: Some(dest), bold: false }])]);
                }
                if srcs.len() != 1 {
                    return Out::fail(format!("{cmd}: destination must be a folder for several sources"));
                }
                let s = &srcs[0];
                if cmd == "mv" {
                    if s.parent() == dest.parent() {
                        return match self.be.rename(s, &dest.name()) {
                            Ok(_) => Out { acts: vec![Act::Refresh], ..Out::ok(vec![line(vec![sp(format!("renamed to {}", dest.name()), St::Dim)])]) },
                            Err(e) => Out::fail(format!("mv: {e}")),
                        };
                    }
                    return Out::fail("mv: destination must be an existing folder or a new name in the same folder");
                }
                match self.be.read_bytes(s, 64 * 1024 * 1024).and_then(|b| {
                    if self.be.stat(&dest).is_err() {
                        if let Some(par) = dest.parent() {
                            self.be.create_file(&par, &dest.name())?;
                        }
                    }
                    self.be.registry.get(&dest.fs).ok_or("no fs".to_string())?.write(&dest.path, &b).map_err(|e| e.to_string())
                }) {
                    Ok(()) => Out { acts: vec![Act::Refresh], ..Out::ok(vec![line(vec![sp(format!("copied to {}", dest.name()), St::Dim)])]) },
                    Err(e) => Out::fail(format!("cp: {e}")),
                }
            }
            "stat" => {
                let Some(p) = pos.first() else { return Out::fail("stat: missing path") };
                let l = self.resolve(cwd, p);
                match self.be.stat(&l) {
                    Err(e) => Out::fail(format!("stat: {e}")),
                    Ok(e) => {
                        let kv = |k: &str, v: String| line(vec![sp(format!("{:<9}", k), St::Dim), sp(v, St::Plain)]);
                        Out::ok(vec![kv("name", e.name.clone()), kv("kind", e.kind_label()), kv("size", format!("{} ({} bytes)", format_size(e.size), e.size)), kv("modified", e.modified.map(format_full_date).unwrap_or_default()), kv("where", e.loc.parent().map(|p| self.short_path(&p)).unwrap_or_default()), kv("access", if e.readonly { "read only".into() } else { "read & write".into() })])
                    }
                }
            }
            "du" => {
                let target = pos.first().map(|p| self.resolve(cwd, p)).unwrap_or_else(|| cwd.clone());
                let Ok(v) = self.list_dir(&target) else { return Out::fail("du: cannot read folder") };
                let mut rows = vec![];
                let mut total = 0;
                let mut budget = 20_000;
                let mut sizes: Vec<(Entry, u64)> = v.into_iter().filter(|e| !e.hidden).map(|e| { let s = if e.is_dir { self.dir_size(&e.loc, &mut budget) } else { e.size }; (e, s) }).collect();
                sizes.sort_by(|a, b| b.1.cmp(&a.1));
                let max = sizes.first().map(|s| s.1).unwrap_or(1).max(1);
                for (e, s) in &sizes {
                    total += s;
                    let bar = "\u{2588}".repeat(((*s as f64 / max as f64) * 18.0).round() as usize);
                    rows.push(line(vec![sp(format!("{:>9}  ", format_size(*s)), St::Dim), sp(format!("{:<19}", bar), St::Accent), self.entry_span(e)]));
                }
                rows.push(line(vec![sp(format!("{:>9}  total", format_size(total)), St::Good)]));
                Out::ok(rows)
            }
            "find" => {
                let Some(name) = pos.first().cloned() else { return Out::fail("find: usage: find <name> [folder]") };
                let root = pos.get(1).map(|p| self.resolve(cwd, p)).unwrap_or_else(|| cwd.clone());
                let q = name.to_lowercase();
                let mut hits: Vec<TLine> = vec![];
                let mut limit = 5000;
                self.walk(&root, 0, &mut limit, &mut |e, _| {
                    if e.name.to_lowercase().contains(&q) && hits.len() < 100 {
                        hits.push(line(vec![Span { text: e.loc.path.clone(), st: if e.is_dir { St::Dir } else { St::Plain }, link: Some(e.loc.clone()), bold: e.is_dir }]));
                    }
                });
                if hits.is_empty() {
                    return Out { code: 1, ..Out::ok(vec![line(vec![sp(format!("nothing called \u{201c}{name}\u{201d}"), St::Dim)])]) };
                }
                let n = hits.len();
                hits.push(line(vec![sp(format!("{n} match{}", if n == 1 { "" } else { "es" }), St::Dim)]));
                Out::ok(hits)
            }
            "grep" => {
                let Some(pat) = pos.first().cloned() else { return Out::fail("grep: usage: grep <text> [file|folder]") };
                let ci = has('i');
                let q = if ci { pat.to_lowercase() } else { pat.clone() };
                let mk = |prefix: Vec<Span>, text: &str| -> TLine {
                    let hay = if ci { text.to_lowercase() } else { text.to_string() };
                    let mut spans = prefix;
                    let mut at = 0;
                    while let Some(i) = hay[at..].find(&q) {
                        let s = at + i;
                        spans.push(sp(text[at..s].to_string(), St::Plain));
                        spans.push(Span { text: text[s..s + q.len()].to_string(), st: St::Warn, link: None, bold: true });
                        at = s + q.len();
                    }
                    spans.push(sp(text[at..].to_string(), St::Plain));
                    line(spans)
                };
                let mut rows: Vec<TLine> = vec![];
                if let Some(path) = pos.get(1) {
                    let root = self.resolve(cwd, path);
                    let mut files: Vec<Location> = vec![];
                    match self.be.stat(&root) {
                        Ok(e) if !e.is_dir => files.push(root.clone()),
                        Ok(_) => {
                            let mut limit = 3000;
                            self.walk(&root, 0, &mut limit, &mut |e, _| {
                                if !e.is_dir && e.size < 2_000_000 && matches!(e.kind, Kind::Text | Kind::Code | Kind::Document) && files.len() < 200 {
                                    files.push(e.loc.clone());
                                }
                            });
                        }
                        Err(e) => return Out::fail(format!("grep: {e}")),
                    }
                    for f in files {
                        if let Ok(t) = self.be.read_text(&f, 512 * 1024) {
                            for (n, l) in t.lines().enumerate() {
                                let hay = if ci { l.to_lowercase() } else { l.to_string() };
                                if hay.contains(&q) && rows.len() < 200 {
                                    rows.push(mk(vec![Span { text: format!("{}:{}  ", f.name(), n + 1), st: St::Accent, link: Some(f.clone()), bold: false }], l));
                                }
                            }
                        }
                    }
                } else if let Some(inp) = stdin {
                    for l in inp {
                        let hay = if ci { l.to_lowercase() } else { l.clone() };
                        if hay.contains(&q) {
                            rows.push(mk(vec![], &l));
                        }
                    }
                } else {
                    return Out::fail("grep: give it a file, a folder or pipe something in");
                }
                if rows.is_empty() {
                    return Out { code: 1, ..Out::ok(vec![line(vec![sp("no matches", St::Dim)])]) };
                }
                Out::ok(rows)
            }
            "open" | "files" | "photos" => {
                let l = pos.first().map(|p| self.resolve(cwd, p)).unwrap_or_else(|| cwd.clone());
                match self.be.stat(&l) {
                    Err(_) if !l.is_root() => Out::fail(format!("{cmd}: no such file or folder: {}", pos.first().cloned().unwrap_or_default())),
                    r => {
                        let is_dir = r.as_ref().map(|e| e.is_dir).unwrap_or(true);
                        let is_img = r.as_ref().map(|e| matches!(e.kind, Kind::Image)).unwrap_or(false);
                        let act = if cmd == "photos" || (cmd == "open" && is_img) { Act::OpenInPhotos(l.clone()) } else if is_dir { Act::Seq(vec![Act::OpenApp(AppId::Files), Act::Nav(Route::Loc(l.clone()))]) } else { Act::Seq(vec![Act::OpenApp(AppId::Files), Act::Open(l.clone())]) };
                        Out { acts: vec![act], ..Out::ok(vec![line(vec![sp("opening ", St::Dim), Span { text: self.short_path(&l), st: St::Accent, link: None, bold: false }])]) }
                    }
                }
            }
            "unzip" => {
                let Some(p) = pos.first() else { return Out::fail("unzip: usage: unzip <file.zip> [folder] [-p password]") };
                let src = self.resolve(cwd, p);
                let dest = pos.get(1).map(|d| self.resolve(cwd, d)).unwrap_or_else(|| cwd.clone());
                let pw = args.iter().position(|a| a == "-p").and_then(|i| args.get(i + 1)).cloned();
                let dest = if pos.get(1).is_none() { src.parent().unwrap_or(dest) } else { dest };
                match self.be.archive_entries(&src, pw.as_deref()) {
                    Err(e) => Out::fail(format!("unzip: {e}")),
                    Ok(entries) => {
                        self.be.archive_extract(&src, &dest, pw);
                        Out { acts: vec![Act::Refresh], ..Out::ok(vec![line(vec![sp(format!("extracting {} entries \u{2192} ", entries.len()), St::Dim), Span { text: self.short_path(&dest), st: St::Dir, link: Some(dest), bold: false }])]) }
                    }
                }
            }
            "zip" => {
                if pos.len() < 2 {
                    return Out::fail("zip: usage: zip <out.zip> <files...> [-p password]");
                }
                let out = self.resolve(cwd, &pos[0]);
                let srcs: Vec<Location> = pos[1..].iter().map(|p| self.resolve(cwd, p)).collect();
                let pw = args.iter().position(|a| a == "-p").and_then(|i| args.get(i + 1)).cloned();
                self.be.archive_compress(srcs.clone(), out.clone(), pw, 6);
                Out { acts: vec![Act::Refresh], ..Out::ok(vec![line(vec![sp(format!("compressing {} item{} \u{2192} ", srcs.len(), if srcs.len() == 1 { "" } else { "s" }), St::Dim), Span { text: out.name(), st: St::Accent, link: None, bold: false }])]) }
            }
            other => {
                let best = COMMANDS.iter().map(|(c, _)| (edit_dist(other, c), *c)).min();
                let mut v = vec![line(vec![sp(format!("command not found: {other}"), St::Err)])];
                if let Some((d, c)) = best {
                    if d <= 2 {
                        v.push(line(vec![sp("did you mean ", St::Dim), sp(c, St::Accent), sp("?", St::Dim)]));
                    }
                }
                Out { lines: v, code: 127, cd: None, clear: false, acts: vec![] }
            }
        }
    }

    pub fn term_submit(&mut self, si: usize, text: String) {
        let text = text.trim().to_string();
        if text.is_empty() {
            return;
        }
        let cwd = self.shell.term.sessions[si].cwd.clone();
        let t0 = self.time;
        if self.shell.term.history.last() != Some(&text) {
            self.shell.term.history.push(text.clone());
        }
        let out = self.term_exec_line(&cwd, &text);
        let ms = ((self.time - t0) * 1000.0) as u32 + 1;
        let acts = out.acts.clone();
        let s = &mut self.shell.term.sessions[si];
        if out.clear {
            s.blocks.clear();
        } else {
            let mut lines = out.lines;
            lines.truncate(2500);
            s.blocks.push(Block { cmd: text, cwd, lines, code: out.code, ms });
            if s.blocks.len() > 80 {
                s.blocks.remove(0);
            }
        }
        if let Some(c) = out.cd {
            s.cwd = c;
        }
        s.input.clear();
        s.cursor = 0;
        s.hist_i = None;
        s.cands.clear();
        s.cand_i = None;
        s.to_bottom = true;
        for a in acts {
            self.acts.push(a);
        }
    }

    fn term_complete(&mut self, si: usize) {
        let (input, cursor, cwd) = {
            let s = &self.shell.term.sessions[si];
            (s.input.clone(), s.cursor, s.cwd.clone())
        };
        let chars: Vec<char> = input.chars().collect();
        let before: String = chars[..cursor.min(chars.len())].iter().collect();
        // cycling through existing candidates
        {
            let s = &mut self.shell.term.sessions[si];
            if let (Some(i), true) = (s.cand_i, s.cands.len() > 1) {
                let ni = (i + 1) % s.cands.len();
                let start = s.cand_start;
                let cand = s.cands[ni].clone();
                let head: String = chars[..start].iter().collect();
                let tail: String = chars[cursor.min(chars.len())..].iter().collect();
                let _ = tail;
                s.input = format!("{head}{cand}");
                s.cursor = s.input.chars().count();
                s.cand_i = Some(ni);
                return;
            }
        }
        let word_start = before.rfind(|c: char| c.is_whitespace()).map(|i| before[..=i].chars().count()).unwrap_or(0);
        let word: String = before.chars().skip(word_start).collect();
        let first_word = before[..].split_whitespace().count() <= 1 && !before.ends_with(' ') || word_start == 0;
        let mut cands: Vec<String> = vec![];
        if first_word && !word.contains('/') {
            cands = COMMANDS.iter().map(|c| c.0.to_string()).filter(|c| c.starts_with(&word.to_lowercase())).collect();
        } else {
            let (dir_part, prefix) = match word.rfind('/') {
                Some(i) => (&word[..=i], &word[i + 1..]),
                None => ("", &word[..]),
            };
            let dir = if dir_part.is_empty() { cwd.clone() } else { self.resolve(&cwd, dir_part) };
            if let Ok(v) = self.list_dir(&dir) {
                for e in v {
                    if e.name.to_lowercase().starts_with(&prefix.to_lowercase()) && (prefix.starts_with('.') || !e.hidden) {
                        let n = if e.name.contains(' ') { format!("{}{}", dir_part, e.name.replace(' ', "\\ ")) } else { format!("{}{}", dir_part, e.name) };
                        cands.push(if e.is_dir { format!("{n}/") } else { n });
                    }
                }
            }
        }
        let s = &mut self.shell.term.sessions[si];
        if cands.is_empty() {
            return;
        }
        let head: String = chars[..word_start.min(chars.len())].iter().collect();
        if cands.len() == 1 {
            let c = cands.remove(0);
            let add = if c.ends_with('/') { "" } else { " " };
            s.input = format!("{head}{c}{add}");
            s.cursor = s.input.chars().count();
            s.cands.clear();
            s.cand_i = None;
        } else {
            let mut common = cands[0].clone();
            for c in &cands {
                while !c.to_lowercase().starts_with(&common.to_lowercase()) {
                    common.pop();
                }
            }
            if common.chars().count() > word.chars().count() {
                s.input = format!("{head}{common}");
                s.cursor = s.input.chars().count();
                s.cand_i = None;
            } else {
                s.cand_i = Some(0);
                s.input = format!("{head}{}", cands[0]);
                s.cursor = s.input.chars().count();
            }
            s.cand_start = word_start;
            s.cands = cands;
        }
    }

    fn term_keys(&mut self, ctx: &egui::Context, si: usize) {
        use egui::{Event, Key};
        ctx.input_mut(|i| {
            i.consume_key(egui::Modifiers::NONE, Key::Tab);
        });
        let events = ctx.input(|i| i.events.clone());
        for e in events {
            let mut edited = true;
            match e {
                Event::Text(t) => {
                    if t.chars().all(|c| !c.is_control()) {
                        let s = &mut self.shell.term.sessions[si];
                        let idx = s.input.char_indices().nth(s.cursor).map(|x| x.0).unwrap_or(s.input.len());
                        s.input.insert_str(idx, &t);
                        s.cursor += t.chars().count();
                    }
                }
                Event::Paste(t) => {
                    let t = t.replace(['\n', '\r'], " ");
                    let s = &mut self.shell.term.sessions[si];
                    let idx = s.input.char_indices().nth(s.cursor).map(|x| x.0).unwrap_or(s.input.len());
                    s.input.insert_str(idx, &t);
                    s.cursor += t.chars().count();
                }
                Event::Copy => {
                    let s = &mut self.shell.term.sessions[si];
                    if !s.input.is_empty() {
                        let cwd = s.cwd.clone();
                        let inp = std::mem::take(&mut s.input);
                        s.blocks.push(Block { cmd: format!("{inp}^C"), cwd, lines: vec![], code: 130, ms: 0 });
                        s.cursor = 0;
                        s.to_bottom = true;
                    }
                }
                Event::Key { key, pressed: true, modifiers, .. } => {
                    let ctrl = modifiers.ctrl || modifiers.command;
                    let s = &mut self.shell.term.sessions[si];
                    let n = s.input.chars().count();
                    match key {
                        Key::Enter => {
                            let t = s.input.clone();
                            if t.trim().is_empty() {
                                let cwd = s.cwd.clone();
                                s.blocks.push(Block { cmd: String::new(), cwd, lines: vec![], code: 0, ms: 0 });
                                s.to_bottom = true;
                            } else {
                                self.term_submit(si, t);
                            }
                        }
                        Key::Backspace => {
                            if ctrl {
                                let chars: Vec<char> = s.input.chars().collect();
                                let mut i = s.cursor;
                                while i > 0 && chars[i - 1] == ' ' {
                                    i -= 1;
                                }
                                while i > 0 && chars[i - 1] != ' ' {
                                    i -= 1;
                                }
                                s.input = chars[..i].iter().chain(chars[s.cursor..].iter()).collect();
                                s.cursor = i;
                            } else if s.cursor > 0 {
                                let idx = s.input.char_indices().nth(s.cursor - 1).map(|x| x.0).unwrap_or(0);
                                s.input.remove(idx);
                                s.cursor -= 1;
                            }
                        }
                        Key::Delete => {
                            if s.cursor < n {
                                let idx = s.input.char_indices().nth(s.cursor).map(|x| x.0).unwrap_or(0);
                                s.input.remove(idx);
                            }
                        }
                        Key::ArrowLeft => {
                            if ctrl {
                                let chars: Vec<char> = s.input.chars().collect();
                                let mut i = s.cursor;
                                while i > 0 && chars[i - 1] == ' ' {
                                    i -= 1;
                                }
                                while i > 0 && chars[i - 1] != ' ' {
                                    i -= 1;
                                }
                                s.cursor = i;
                            } else {
                                s.cursor = s.cursor.saturating_sub(1)
                            }
                            edited = false;
                        }
                        Key::ArrowRight => {
                            if s.cursor >= n {
                                // accept the ghost suggestion
                                let hist = &self.shell.term.history;
                                let sg = hist.iter().rev().find(|h| h.starts_with(&s.input) && **h != s.input).cloned();
                                if let Some(sg) = sg {
                                    s.input = sg;
                                    s.cursor = s.input.chars().count();
                                }
                            } else if ctrl {
                                let chars: Vec<char> = s.input.chars().collect();
                                let mut i = s.cursor;
                                while i < n && chars[i] != ' ' {
                                    i += 1;
                                }
                                while i < n && chars[i] == ' ' {
                                    i += 1;
                                }
                                s.cursor = i;
                            } else {
                                s.cursor += 1;
                            }
                            edited = false;
                        }
                        Key::Home => s.cursor = 0,
                        Key::End => {
                            let hist = &self.shell.term.history;
                            if s.cursor >= n {
                                if let Some(sg) = hist.iter().rev().find(|h| h.starts_with(&s.input) && **h != s.input).cloned() {
                                    s.input = sg;
                                }
                            }
                            s.cursor = s.input.chars().count();
                            edited = false;
                        }
                        Key::ArrowUp | Key::ArrowDown => {
                            let h = &self.shell.term.history;
                            if !h.is_empty() {
                                let up = key == Key::ArrowUp;
                                let cur = s.hist_i;
                                let next = match (cur, up) {
                                    (None, true) => {
                                        s.draft = s.input.clone();
                                        Some(h.len() - 1)
                                    }
                                    (Some(i), true) => Some(i.saturating_sub(1)),
                                    (Some(i), false) if i + 1 < h.len() => Some(i + 1),
                                    (Some(_), false) => None,
                                    (None, false) => None,
                                };
                                s.hist_i = next;
                                s.input = next.map(|i| h[i].clone()).unwrap_or_else(|| s.draft.clone());
                                s.cursor = s.input.chars().count();
                            }
                            edited = false;
                        }
                        Key::Tab => {
                            self.term_complete(si);
                            edited = false;
                        }
                        Key::Escape => {
                            s.cands.clear();
                            s.cand_i = None;
                            edited = false;
                        }
                        Key::L if ctrl => s.blocks.clear(),
                        Key::U if ctrl => {
                            s.input = s.input.chars().skip(s.cursor).collect();
                            s.cursor = 0;
                        }
                        Key::K if ctrl => s.input = s.input.chars().take(s.cursor).collect(),
                        Key::A if ctrl => {
                            s.cursor = 0;
                            edited = false;
                        }
                        Key::E if ctrl => {
                            s.cursor = n;
                            edited = false;
                        }
                        Key::Plus | Key::Equals if ctrl => {
                            self.shell.term.font = (self.shell.term.font + 1.0).min(20.0);
                            edited = false;
                        }
                        Key::Minus if ctrl => {
                            self.shell.term.font = (self.shell.term.font - 1.0).max(10.0);
                            edited = false;
                        }
                        _ => edited = false,
                    }
                    if edited && key != Key::Enter && si < self.shell.term.sessions.len() {
                        let s = &mut self.shell.term.sessions[si];
                        s.cands.clear();
                        s.cand_i = None;
                    }
                    continue;
                }
                _ => {
                    edited = false;
                }
            }
            if edited && si < self.shell.term.sessions.len() {
                let s = &mut self.shell.term.sessions[si];
                s.cands.clear();
                s.cand_i = None;
                s.hist_i = None;
                s.to_bottom = true;
            }
        }
    }

    // ───────────── drawing ─────────────
    fn st_color(st: St) -> Color32 {
        match st {
            St::Plain => hex("#dbe4f5"),
            St::Dim => hex("#7d89a3"),
            St::Accent => hex("#8db4ff"),
            St::Good => hex("#6fd9a8"),
            St::Warn => hex("#f2c166"),
            St::Err => hex("#ff8c97"),
            St::Dir => hex("#8db4ff"),
            St::Pink => hex("#f29bd0"),
            St::Rgb(c) => c,
        }
    }

    pub fn draw_terminal(&mut self, ui: &mut Ui, rect: Rect, active: bool) {
        let th = self.th.clone();
        let ctx = ui.ctx().clone();
        if self.shell.term.sessions.is_empty() {
            let home = self.shell.term.home.clone();
            let t = self.time;
            self.shell.term.new_session_at(home, t);
        }
        let n_sessions = self.shell.term.sessions.len();
        self.shell.term.active = self.shell.term.active.min(n_sessions - 1);
        let si = self.shell.term.active;
        if active {
            self.term_keys(&ctx, si);
            ctx.request_repaint_after(std::time::Duration::from_millis(500));
        }
        if self.shell.term.sessions.is_empty() {
            return;
        }
        let si = si.min(self.shell.term.sessions.len() - 1);
        let p = ui.painter().clone();
        let font_px = self.shell.term.font;
        let mono = FontId::monospace(font_px);
        let charw = self.tw(&p, "M", &mono);
        let lh = (font_px * 1.55).round();

        // session tabs
        let strip = Rect::from_min_size(rect.min + vec2(12.0, 6.0), vec2(rect.width() - 24.0, 32.0));
        let tw_ = ((strip.width() - 40.0) / self.shell.term.sessions.len() as f32).clamp(110.0, 190.0);
        let mut close: Option<usize> = None;
        let mut switch: Option<usize> = None;
        for i in 0..self.shell.term.sessions.len() {
            let r = Rect::from_min_size(strip.min + vec2(tw_ * i as f32, 0.0), vec2(tw_ - 6.0, strip.height()));
            let on = i == si;
            let resp = self.hit(ui, r, ("tsess", i), Sense::click());
            if on {
                raised(&p, &th, r, 12.0, false, false);
            } else if resp.hovered() {
                p.rect_filled(r, cr(12.0), th.row_hover);
            }
            self.ico(&p, "terminal", if on || resp.hovered() { Ic::Duo } else { Ic::Outline }, pos2(r.left() + 15.0, r.center().y), 15.0, if on { th.accent_ink } else { th.ink3 });
            let title = self.short_path(&self.shell.term.sessions[i].cwd);
            let showx = self.shell.term.sessions.len() > 1 && (on || resp.hovered());
            p.text(pos2(r.left() + 30.0, r.center().y), Align2::LEFT_CENTER, self.fit(&p, &title, &f_semi(T_BODY), r.width() - 36.0 - if showx { 20.0 } else { 0.0 }), f_semi(T_BODY), if on { th.ink1 } else { th.ink2 });
            if showx {
                let xr = Rect::from_center_size(pos2(r.right() - 14.0, r.center().y), vec2(20.0, 20.0));
                let xh = self.hit(ui, xr, ("tsx", i), Sense::click());
                if xh.hovered() {
                    p.circle_filled(xr.center(), 9.0, th.row_press);
                }
                self.ico(&p, "close-circle", Ic::Outline, xr.center(), 14.0, th.ink3);
                if xh.clicked() {
                    close = Some(i);
                }
            }
            if resp.clicked() && !on {
                switch = Some(i);
            }
        }
        let plus = Rect::from_center_size(pos2(strip.left() + tw_ * self.shell.term.sessions.len() as f32 + 14.0, strip.center().y), vec2(30.0, 30.0));
        if self.icon_button(ui, plus, "add", "tsess-new", Btn::Quiet, true).on_hover_text("New session").clicked() {
            self.shell.term.command("new");
        }
        if let Some(i) = switch {
            self.shell.term.active = i;
            return;
        }
        if let Some(i) = close {
            if self.shell.term.sessions.len() > 1 {
                self.shell.term.sessions.remove(i);
                self.shell.term.active = self.shell.term.active.min(self.shell.term.sessions.len() - 1);
                return;
            }
        }

        // terminal surface
        let hint_h = 30.0;
        let surf = Rect::from_min_max(pos2(rect.left() + 10.0, strip.bottom() + 6.0), pos2(rect.right() - 10.0, rect.bottom() - hint_h - 6.0));
        let (s0, s1) = (hex("#0b1120"), hex("#0e1526"));
        shadow(&p, surf, 16.0, 2.0, 8.0, 0.0, alpha(Color32::BLACK, 0.25));
        grad_fill(&p, surf, 16.0, alpha(s0, 0.97), alpha(s1, 0.97));
        rim(&p, surf, 16.0, 3.0, alpha(Color32::BLACK, 0.5), true, 14.0);
        rim(&p, surf, 16.0, 1.0, alpha(Color32::WHITE, 0.10), false, 5.0);
        // subtle accent bloom bottom-left so it never reads as flat black
        grad_fill(&p, Rect::from_min_max(pos2(surf.left(), surf.bottom() - 90.0), surf.max), 16.0, alpha(th.accent, 0.0), alpha(th.accent, 0.07));

        // layout every block for this width
        let inner = Rect::from_min_max(surf.min + vec2(14.0, 10.0), surf.max - vec2(8.0, 10.0));
        let text_w = inner.width() - 26.0;
        let cols = ((text_w / charw).floor() as usize).max(20);

        #[derive(Clone)]
        enum Row {
            Spans(Vec<Span>),
            Gap,
            Chips(Vec<(String, Kind, bool, Location)>, f32),
            Cmds(Vec<String>),
            Title(String, String),
        }
        struct BL {
            rows: Vec<(Row, f32)>,
            h: f32,
        }
        let mut layouts: Vec<BL> = vec![];
        {
            let s = &self.shell.term.sessions[si];
            for b in &s.blocks {
                let mut rows: Vec<(Row, f32)> = vec![];
                for l in &b.lines {
                    match l {
                        TLine::Gap => rows.push((Row::Gap, 8.0)),
                        TLine::Title(a, c) => rows.push((Row::Title(a.clone(), c.clone()), lh * 2.0 + 4.0)),
                        TLine::Chips(c) => rows.push((Row::Cmds(c.clone()), 34.0)),
                        TLine::Entries(es) => {
                            let mut cur: Vec<(String, Kind, bool, Location)> = vec![];
                            let mut w = 0.0;
                            for e in es {
                                let ew = (e.0.chars().count() as f32 + 1.0) * charw + 34.0;
                                if w + ew > text_w && !cur.is_empty() {
                                    rows.push((Row::Chips(std::mem::take(&mut cur), 0.0), 30.0));
                                    w = 0.0;
                                }
                                w += ew + 6.0;
                                cur.push(e.clone());
                            }
                            if !cur.is_empty() {
                                rows.push((Row::Chips(cur, 0.0), 30.0));
                            }
                        }
                        TLine::Spans(spans) => {
                            // wrap by columns, splitting spans
                            let mut cur: Vec<Span> = vec![];
                            let mut col = 0;
                            for sp_ in spans {
                                let mut rest: Vec<char> = sp_.text.chars().collect();
                                loop {
                                    let room = cols.saturating_sub(col);
                                    if rest.len() <= room {
                                        col += rest.len();
                                        cur.push(Span { text: rest.iter().collect(), ..sp_.clone() });
                                        break;
                                    }
                                    let take = room.max(1).min(rest.len());
                                    let head: String = rest[..take].iter().collect();
                                    cur.push(Span { text: head, ..sp_.clone() });
                                    rest = rest[take..].to_vec();
                                    rows.push((Row::Spans(std::mem::take(&mut cur)), lh));
                                    col = 0;
                                }
                            }
                            rows.push((Row::Spans(cur), lh));
                        }
                    }
                }
                let head = if b.cmd.is_empty() { 0.0 } else { 34.0 };
                let h = head + rows.iter().map(|r| r.1).sum::<f32>() + if rows.is_empty() { 0.0 } else { 8.0 } + 8.0;
                layouts.push(BL { rows, h });
            }
        }
        // prompt metrics
        let (input, cursor, cands, cwd) = {
            let s = &self.shell.term.sessions[si];
            (s.input.clone(), s.cursor, s.cands.clone(), s.cwd.clone())
        };
        let prompt_cols = cols.saturating_sub(2).max(10);
        let in_chars: Vec<char> = input.chars().collect();
        let in_rows = ((in_chars.len() + 1) as f32 / prompt_cols as f32).ceil().max(1.0) as usize;
        let prompt_h = 34.0 + in_rows as f32 * lh + 12.0 + if cands.is_empty() { 0.0 } else { 40.0 };
        let total_h: f32 = layouts.iter().map(|l| l.h + 6.0).sum::<f32>() + prompt_h + 8.0;
        let to_bottom = std::mem::take(&mut self.shell.term.sessions[si].to_bottom);

        let mut clicked_cmd: Option<String> = None;
        let mut clicked_loc: Option<Location> = None;
        let mut copy_text: Option<String> = None;
        let mut cui = ui.new_child(egui::UiBuilder::new().max_rect(inner).layout(egui::Layout::top_down(egui::Align::Min)));
        cui.set_clip_rect(inner.intersect(ui.clip_rect()));
        let mut sa = egui::ScrollArea::vertical().id_salt(("term-scroll", self.shell.term.sessions[si].id)).auto_shrink([false, false]).scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded);
        if to_bottom {
            sa = sa.vertical_scroll_offset((total_h - inner.height()).max(0.0) + 40.0);
        }
        let ghost = if !input.is_empty() { self.shell.term.history.iter().rev().find(|h| h.starts_with(&input) && **h != input).map(|h| h[input.len()..].to_string()) } else { None };
        let blocks_cmds: Vec<(String, Location, i32, u32)> = self.shell.term.sessions[si].blocks.iter().map(|b| (b.cmd.clone(), b.cwd.clone(), b.code, b.ms)).collect();
        let time = self.time;
        sa.show_viewport(&mut cui, |ui, vp| {
            let top = ui.max_rect().top();
            ui.allocate_space(vec2(inner.width() - 12.0, total_h));
            let p = ui.painter().clone();
            let mut y = top;
            for (bi, bl) in layouts.iter().enumerate() {
                let (cmd, bcwd, code, ms) = &blocks_cmds[bi];
                let visible = y + bl.h > top + vp.top() - 40.0 && y < top + vp.bottom() + 40.0;
                if visible {
                    let card = Rect::from_min_size(pos2(inner.left(), y), vec2(inner.width() - 14.0, bl.h));
                    if !cmd.is_empty() {
                        grad_fill(&p, card, 12.0, alpha(Color32::WHITE, 0.045), alpha(Color32::WHITE, 0.018));
                        rim(&p, card, 12.0, 1.0, alpha(Color32::WHITE, 0.07), true, 24.0);
                        // header
                        let hy = y + 17.0;
                        let path = self.short_path(bcwd);
                        let cw = self.tw(&p, &path, &f_semi(11.0)) + 26.0;
                        let chip = Rect::from_min_size(pos2(card.left() + 10.0, hy - 11.0), vec2(cw, 22.0));
                        grad_fill(&p, chip, 11.0, alpha(th.accent, 0.20), alpha(th.accent, 0.09));
                        self.ico(&p, "folder", Ic::Duo, pos2(chip.left() + 12.0, hy), 12.0, hex("#9dbdff"));
                        p.text(pos2(chip.left() + 22.0, hy), Align2::LEFT_CENTER, &path, f_semi(11.0), hex("#b6cdff"));
                        p.text(pos2(chip.right() + 10.0, hy), Align2::LEFT_CENTER, self.fit(&p, cmd, &mono, card.right() - chip.right() - 120.0), mono.clone(), St::Plain.pipe());
                        // status
                        let ok = *code == 0;
                        let label = if *code == 130 { "interrupted".to_string() } else if ok { format!("{} ms", ms) } else { format!("exit {}", code) };
                        let lw = self.tw(&p, &label, &f_medium(10.5));
                        let col = if ok { hex("#6fd9a8") } else if *code == 130 { hex("#f2c166") } else { hex("#ff8c97") };
                        self.ico(&p, if ok { "check-circle" } else { "close-circle" }, Ic::Filled, pos2(card.right() - 16.0, hy), 14.0, col);
                        p.text(pos2(card.right() - 28.0, hy), Align2::RIGHT_CENTER, label, f_medium(10.5), alpha(col, 0.9));
                        // copy on hover
                        let hov = ui.rect_contains_pointer(card);
                        if hov {
                            let cb = Rect::from_center_size(pos2(card.right() - 40.0 - lw - 12.0, hy), vec2(24.0, 24.0));
                            let cr_ = self.hit(ui, cb, ("tcopy", si, bi), Sense::click());
                            if cr_.hovered() {
                                p.circle_filled(cb.center(), 11.0, alpha(Color32::WHITE, 0.10));
                            }
                            self.ico(&p, "copy", Ic::Outline, cb.center(), 14.0, hex("#9aa8c4"));
                            if cr_.clicked() {
                                copy_text = Some(format!("$ {}\n{}", cmd, text_of(&self.shell.term.sessions[si].blocks[bi].lines).join("\n")));
                            }
                        }
                    }
                    let mut ry = y + if cmd.is_empty() { 4.0 } else { 34.0 };
                    for (ri, (row, h)) in bl.rows.iter().enumerate() {
                        match row {
                            Row::Gap => {}
                            Row::Title(a, b) => {
                                let t = Rect::from_min_size(pos2(card.left() + 12.0, ry + 4.0), vec2(30.0, 30.0));
                                let (c0, c1) = th.accent_grad();
                                grad_fill(&p, t, 10.0, c0, c1);
                                rim(&p, t, 10.0, 1.0, Color32::from_white_alpha(130), true, 14.0);
                                p.text(t.center(), Align2::CENTER_CENTER, ">_", f_bold(12.0), Color32::WHITE);
                                p.text(pos2(t.right() + 12.0, t.center().y - 7.0), Align2::LEFT_CENTER, a, f_bold(15.0), hex("#eef3fb"));
                                p.text(pos2(t.right() + 12.0, t.center().y + 10.0), Align2::LEFT_CENTER, self.fit(&p, b, &f_medium(11.0), card.width() - 90.0), f_medium(11.0), hex("#7d89a3"));
                            }
                            Row::Cmds(cs) => {
                                let mut x = card.left() + 12.0;
                                for c in cs {
                                    let w = self.tw(&p, c, &mono) + 24.0;
                                    let r = Rect::from_min_size(pos2(x, ry + 3.0), vec2(w, 26.0));
                                    let resp = self.hit(ui, r, ("tcmd", si, bi, c.clone()), Sense::click());
                                    let a = self.anim(ui, ("tcmdh", c.clone()), resp.hovered());
                                    grad_fill(&p, r, 13.0, alpha(Color32::WHITE, 0.07 + 0.08 * a), alpha(Color32::WHITE, 0.03 + 0.05 * a));
                                    rim(&p, r, 13.0, 1.0, alpha(Color32::WHITE, 0.10 + 0.12 * a), true, 14.0);
                                    p.text(r.center(), Align2::CENTER_CENTER, c, mono.clone(), mix(hex("#b6cdff"), Color32::WHITE, a));
                                    if resp.clicked() {
                                        clicked_cmd = Some(c.clone());
                                    }
                                    x += w + 8.0;
                                }
                            }
                            Row::Chips(es, _) => {
                                let mut x = card.left() + 12.0;
                                for (name, kind, is_dir, loc) in es {
                                    let w = (name.chars().count() as f32 + (if *is_dir { 1.0 } else { 0.0 })) * charw + 36.0;
                                    let r = Rect::from_min_size(pos2(x, ry + 2.0), vec2(w, 26.0));
                                    let resp = self.hit(ui, r, ("tent", si, bi, ri, name.clone()), Sense::click());
                                    let a = self.anim(ui, ("tenth", name.clone(), bi, ri), resp.hovered());
                                    let kc = crate::content::kind_color(*kind, &th);
                                    let kc = if *is_dir { hex("#8db4ff") } else { kc };
                                    grad_fill(&p, r, 9.0, alpha(kc, 0.10 + 0.14 * a), alpha(kc, 0.04 + 0.08 * a));
                                    rim(&p, r, 9.0, 1.0, alpha(Color32::WHITE, 0.07 + 0.10 * a), true, 14.0);
                                    self.ico(&p, kind.icon(), if a > 0.5 || *is_dir { Ic::Duo } else { Ic::Outline }, pos2(r.left() + 14.0, r.center().y), 14.0, kc);
                                    p.text(pos2(r.left() + 26.0, r.center().y), Align2::LEFT_CENTER, if *is_dir { format!("{name}/") } else { name.clone() }, mono.clone(), if *is_dir { hex("#a9c4ff") } else { St::Plain.pipe() });
                                    if resp.clicked() {
                                        clicked_loc = Some(loc.clone());
                                    }
                                    x += w + 6.0;
                                }
                            }
                            Row::Spans(spans) => {
                                let mut col = 0usize;
                                for (k, spn) in spans.iter().enumerate() {
                                    let n = spn.text.chars().count();
                                    let x = card.left() + 14.0 + col as f32 * charw;
                                    let mut color = Self::st_color(spn.st);
                                    let r = Rect::from_min_size(pos2(x, ry), vec2(n as f32 * charw, lh));
                                    if let Some(l) = &spn.link {
                                        let resp = self.hit(ui, r, ("tlink", si, bi, ri, k), Sense::click());
                                        if resp.hovered() {
                                            color = mix(color, Color32::WHITE, 0.4);
                                            p.line_segment([pos2(r.left(), r.bottom() - 3.0), pos2(r.right(), r.bottom() - 3.0)], egui::Stroke::new(1.0, alpha(color, 0.8)));
                                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                        }
                                        if resp.clicked() {
                                            clicked_loc = Some(l.clone());
                                        }
                                    }
                                    let f = if spn.bold { FontId::new(font_px, egui::FontFamily::Proportional) } else { mono.clone() };
                                    let f = if spn.bold { FontId::monospace(font_px) } else { f };
                                    p.text(pos2(x, ry + lh / 2.0), Align2::LEFT_CENTER, &spn.text, f, color);
                                    col += n;
                                }
                            }
                        }
                        ry += *h;
                    }
                }
                y += bl.h + 6.0;
            }
            // ── live prompt ──
            let pr = Rect::from_min_size(pos2(inner.left(), y + 2.0), vec2(inner.width() - 14.0, prompt_h));
            grad_fill(&p, pr, 14.0, alpha(th.accent, if active { 0.13 } else { 0.06 }), alpha(th.accent, if active { 0.04 } else { 0.02 }));
            rim(&p, pr, 14.0, 1.2, alpha(th.accent, if active { 0.55 } else { 0.18 }), true, pr.height());
            p.rect_stroke(pr.shrink(0.5), cr(14.0), egui::Stroke::new(1.0, alpha(Color32::WHITE, 0.06)), egui::StrokeKind::Inside);
            let user = format!("{}@smart", self.user_name().to_lowercase());
            let uw = self.tw(&p, &user, &f_semi(11.0)) + 20.0;
            let uc = Rect::from_min_size(pos2(pr.left() + 12.0, pr.top() + 8.0), vec2(uw, 22.0));
            grad_fill(&p, uc, 11.0, alpha(hex("#6fd9a8"), 0.20), alpha(hex("#6fd9a8"), 0.09));
            p.text(uc.center(), Align2::CENTER_CENTER, &user, f_semi(11.0), hex("#9ae8c3"));
            let path = self.short_path(&cwd);
            let pw = self.tw(&p, &path, &f_semi(11.0)) + 28.0;
            let pc = Rect::from_min_size(pos2(uc.right() + 6.0, uc.top()), vec2(pw, 22.0));
            grad_fill(&p, pc, 11.0, alpha(th.accent, 0.22), alpha(th.accent, 0.10));
            self.ico(&p, "folder-open", Ic::Duo, pos2(pc.left() + 13.0, pc.center().y), 12.0, hex("#9dbdff"));
            p.text(pos2(pc.left() + 24.0, pc.center().y), Align2::LEFT_CENTER, &path, f_semi(11.0), hex("#b6cdff"));
            let tm = clock_hm();
            p.text(pos2(pr.right() - 14.0, uc.center().y), Align2::RIGHT_CENTER, tm, f_medium(10.5), hex("#6c7894"));
            // input rows
            let ix = pr.left() + 14.0;
            let iy = pr.top() + 38.0;
            let words: Vec<&str> = input.split(' ').collect();
            let known = COMMANDS.iter().any(|c| c.0 == words[0]);
            // colour per char
            let mut colors: Vec<Color32> = Vec::with_capacity(in_chars.len());
            {
                let mut wi = 0;
                let mut inq: Option<char> = None;
                let mut tok_start = 0;
                for (i, ch) in in_chars.iter().enumerate() {
                    if inq.is_none() && *ch == ' ' {
                        wi += 1;
                        tok_start = i + 1;
                        colors.push(St::Plain.pipe());
                        continue;
                    }
                    if inq.is_none() && (*ch == '"' || *ch == '\'') {
                        inq = Some(*ch);
                    } else if inq == Some(*ch) {
                        inq = None;
                    }
                    let tok: String = in_chars[tok_start..].iter().take_while(|c| **c != ' ').collect();
                    let c = if inq.is_some() || *ch == '"' || *ch == '\'' { St::Good.pipe() } else if *ch == '|' || *ch == '>' { hex("#f29bd0") } else if wi == 0 { if known { hex("#8db4ff") } else { hex("#ff9aa5") } } else if tok.starts_with('-') { St::Warn.pipe() } else if tok.contains('/') || tok.starts_with('~') { hex("#c9d8f5") } else { St::Plain.pipe() };
                    colors.push(c);
                }
            }
            let chev = pos2(ix, iy + lh / 2.0);
            p.text(chev, Align2::LEFT_CENTER, "\u{276f}", FontId::monospace(font_px), if known || input.is_empty() { hex("#6fd9a8") } else { hex("#ff9aa5") });
            for (i, ch) in in_chars.iter().enumerate() {
                let (r_, c_) = (i / prompt_cols, i % prompt_cols);
                p.text(pos2(ix + (c_ as f32 + 2.0) * charw, iy + r_ as f32 * lh + lh / 2.0), Align2::LEFT_CENTER, ch.to_string(), mono.clone(), colors[i]);
            }
            if let Some(g) = &ghost {
                let start = in_chars.len();
                for (k, ch) in g.chars().enumerate() {
                    let i = start + k;
                    let (r_, c_) = (i / prompt_cols, i % prompt_cols);
                    p.text(pos2(ix + (c_ as f32 + 2.0) * charw, iy + r_ as f32 * lh + lh / 2.0), Align2::LEFT_CENTER, ch.to_string(), mono.clone(), alpha(hex("#8b97b3"), 0.45));
                }
                if cursor >= in_chars.len() {
                    p.text(pos2(pr.right() - 14.0, iy + lh / 2.0), Align2::RIGHT_CENTER, "\u{2192} accept", f_medium(10.0), hex("#5d6985"));
                }
            }
            if input.is_empty() {
                p.text(pos2(ix + 2.0 * charw, iy + lh / 2.0), Align2::LEFT_CENTER, "type a command\u{2026}  try  help", mono.clone(), alpha(hex("#8b97b3"), 0.38));
            }
            // cursor (spring-smoothed, blinking)
            let (cr_, cc_) = (cursor / prompt_cols, cursor % prompt_cols);
            let tx = ix + (cc_ as f32 + 2.0) * charw;
            let ty = iy + cr_ as f32 * lh;
            let sx = spring_to(ui.ctx(), ("term-cx", si), tx, 900.0, 46.0);
            let sy = spring_to(ui.ctx(), ("term-cy", si), ty, 900.0, 46.0);
            let blink = if active { ((time * 2.0).sin() > -0.35) as u8 as f32 } else { 0.0 };
            let cur = Rect::from_min_size(pos2(sx, sy + 2.0), vec2(charw.max(7.0), lh - 4.0));
            if active {
                p.rect_filled(cur, cr(2.0), alpha(th.accent, 0.85 * blink));
                if let Some(ch) = in_chars.get(cursor) {
                    p.text(pos2(tx, ty + lh / 2.0), Align2::LEFT_CENTER, ch.to_string(), mono.clone(), alpha(hex("#08101e"), blink));
                }
            } else {
                p.rect_stroke(cur, cr(2.0), egui::Stroke::new(1.0, alpha(th.accent, 0.5)), egui::StrokeKind::Inside);
            }
            // completion chips
            if !cands.is_empty() {
                let cy = pr.bottom() - 36.0;
                let mut x = pr.left() + 12.0;
                let cur_i = self.shell.term.sessions[si].cand_i;
                for (i, c) in cands.iter().enumerate().take(10) {
                    let label = c.trim_end_matches('/').rsplit('/').next().unwrap_or(c).to_string() + if c.ends_with('/') { "/" } else { "" };
                    let w = self.tw(&p, &label, &f_medium(11.5)) + 22.0;
                    if x + w > pr.right() - 40.0 {
                        p.text(pos2(x + 4.0, cy + 14.0), Align2::LEFT_CENTER, format!("+{}", cands.len() - i), f_medium(11.0), hex("#7d89a3"));
                        break;
                    }
                    let r = Rect::from_min_size(pos2(x, cy + 3.0), vec2(w, 24.0));
                    let on = cur_i == Some(i);
                    grad_fill(&p, r, 12.0, alpha(th.accent, if on { 0.5 } else { 0.14 }), alpha(th.accent, if on { 0.3 } else { 0.06 }));
                    rim(&p, r, 12.0, 1.0, alpha(Color32::WHITE, if on { 0.3 } else { 0.1 }), true, 12.0);
                    p.text(r.center(), Align2::CENTER_CENTER, &label, f_medium(11.5), if on { Color32::WHITE } else { hex("#b6cdff") });
                    x += w + 6.0;
                }
            }
            // keep the prompt in view after submit
            if to_bottom {
                ui.scroll_to_rect(pr.expand2(vec2(0.0, 30.0)), Some(egui::Align::BOTTOM));
            }
        });

        // hint bar
        let hb = Rect::from_min_max(pos2(rect.left() + 18.0, rect.bottom() - hint_h), pos2(rect.right() - 18.0, rect.bottom() - 4.0));
        let mut x = hb.left();
        for (k, v) in [("Tab", "complete"), ("\u{2191}\u{2193}", "history"), ("\u{2192}", "accept suggestion"), ("Ctrl L", "clear")] {
            let w = self.keycap(&p, pos2(x + self.tw(&p, k, &f_semi(10.0)).max(9.0) + 9.0, hb.center().y), k, false);
            x += w + 6.0;
            p.text(pos2(x, hb.center().y), Align2::LEFT_CENTER, v, f_medium(10.5), th.ink4);
            x += self.tw(&p, v, &f_medium(10.5)) + 16.0;
        }
        p.text(pos2(hb.right(), hb.center().y), Align2::RIGHT_CENTER, format!("{}  \u{b7}  {} commands", self.shell.term.sessions[si].cwd.fs, self.shell.term.history.len()), f_medium(10.5), th.ink4);

        if let Some(c) = clicked_cmd {
            let s = &mut self.shell.term.sessions[si];
            s.input = c.clone();
            s.cursor = c.chars().count();
            self.term_submit(si, c);
        }
        if let Some(l) = clicked_loc {
            let is_dir = self.be.stat(&l).map(|e| e.is_dir).unwrap_or(true);
            let cmd = if is_dir { format!("cd {}", quote(&l.path)) } else { format!("open {}", quote(&l.path)) };
            // paths are absolute in the VFS, so cd/open resolve them directly
            let cmd = if is_dir { cmd } else { cmd };
            self.term_submit(si, cmd);
        }
        if let Some(t) = copy_text {
            self.acts.push(Act::CopyText(t));
        }
    }
}

trait Pipe {
    fn pipe(self) -> Color32;
}
impl Pipe for St {
    fn pipe(self) -> Color32 {
        SmartApp::st_color_pub(self)
    }
}
impl SmartApp {
    pub fn st_color_pub(st: St) -> Color32 {
        Self::st_color(st)
    }
}
