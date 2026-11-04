//! Smart Terminal — a modern, friendly shell that runs on the same virtual file system as the file manager.
//! Command blocks with status chips, syntax-highlighted input, history ghost-text, Tab completion with candidate
//! chips, clickable paths, pipes (`|`) and redirects (`>`, `>>`). Pure Rust, drawn with the shared design system.
use crate::app::*;
use crate::appkit::{Frame, SRow, SVal};
use crate::chrome::*;
use crate::overlays::{mi, MI};
use crate::icons::Ic;
use crate::paint::*;
use crate::shell::AppId;
use crate::theme::*;
use egui::{pos2, vec2, Align2, Color32, FontId, Pos2, Rect, Sense, Ui};
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
    pub full: Option<FullApp>,
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
    pub cursor: usize,
    pub blink: bool,
    pub ghost: bool,
    pub timing: bool,
    pub cards: bool,
    pub scrollback: usize,
    pub spacing: f32,
    pub ctx_block: Option<(usize, usize)>,
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
    ("top", "Live process monitor  (q to quit)"),
    ("colors", "Colour test through the vt100 engine"),
    ("clear", "Clear the screen  (Ctrl+L)"),
    ("exit", "Close this session"),
];

impl Terminal {
    pub fn new(home: Location) -> Self {
        Self { sessions: vec![], active: 0, history: vec![], font: 13.0, home, next_id: 1, booted: false, pending_run: None, cursor: 0, blink: true, ghost: true, timing: true, cards: true, scrollback: 80, spacing: 1.55, ctx_block: None }
    }

    fn new_session_at(&mut self, cwd: Location, time: f64) {
        let id = self.next_id;
        self.next_id += 1;
        let _ = time;
        let welcome = Block {
            cmd: String::new(),
            cwd: cwd.clone(),
            lines: vec![
                line(vec![sp("Try ", St::Dim), sp("Tab", St::Accent), sp(" to complete, ", St::Dim), sp("\u{2191}", St::Accent), sp(" for history, click any path to jump there.", St::Dim)]),
                TLine::Chips(vec!["ls".into(), "tree -L 2".into(), "neofetch".into(), "help".into(), "calc 21*2".into()]),
            ],
            code: 0,
            ms: 0,
        };
        self.sessions.push(Session { id, cwd, blocks: vec![welcome], input: String::new(), cursor: 0, hist_i: None, draft: String::new(), cands: vec![], cand_i: None, cand_start: 0, to_bottom: true, full: None });
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
            "top" => Out { acts: vec![Act::App("term.full:top".into())], ..Out::ok(vec![line(vec![sp("starting top \u{2014} q to quit", St::Dim)])]) },
            "colors" => {
                let mut o = String::from("\x1b[1mStandard\x1b[0m\n");
                for i in 0..16 {
                    o.push_str(&format!("\x1b[38;5;{i}m\u{2588}\u{2588}\x1b[0m "));
                }
                o.push_str("\n\n\x1b[1m256 colours\x1b[0m\n");
                for r in 0..6 {
                    for i in 16 + r * 36..16 + r * 36 + 36 {
                        o.push_str(&format!("\x1b[38;5;{i}m\u{2588}\x1b[0m"));
                    }
                    o.push('\n');
                }
                o.push_str("\n\x1b[1mTrue colour\x1b[0m\n");
                for i in 0..72 {
                    let (r, g, b) = (255 - i * 3, i * 3, 160 + i);
                    o.push_str(&format!("\x1b[38;2;{r};{g};{b}m\u{2588}\x1b[0m"));
                }
                o.push_str("\n\n\x1b[31mred\x1b[0m \x1b[32mgreen\x1b[0m \x1b[33myellow\x1b[0m \x1b[34mblue\x1b[0m \x1b[35mmagenta\x1b[0m \x1b[36mcyan\x1b[0m   \x1b[90mrendered by the vt100 state machine\x1b[0m");
                Out::ok(ansi_lines(&o))
            }
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
                let esc = has('e');
                let args: Vec<String> = args.iter().filter(|a| a.as_str() != "-e" && a.as_str() != "-n").cloned().collect();
                let s = args.join(" ").replace("$PWD", &cwd.path).replace("$HOME", &home.path).replace("$USER", &self.user_name().to_lowercase());
                if esc {
                    let s = s.replace("\\e", "\x1b").replace("\\x1b", "\x1b").replace("\\033", "\x1b").replace("\\n", "\n");
                    Out::ok(ansi_lines(&s))
                } else {
                    Out::ok(vec![plain(s)])
                }
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
            let mut lines: Vec<TLine> = vec![];
            for l in out.lines {
                match l {
                    TLine::Spans(sp_) if sp_.iter().any(|x| x.text.contains('\u{1b}')) => lines.extend(ansi_lines(&sp_.iter().map(|x| x.text.as_str()).collect::<String>())),
                    other => lines.push(other),
                }
            }
            lines.truncate(2500);
            s.blocks.push(Block { cmd: text, cwd, lines, code: out.code, ms });
            while s.blocks.len() > self.shell.term.scrollback.max(20) {
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
            St::Plain => tc("#dbe4f5", "#1c2740"),
            St::Dim => tc("#7d89a3", "#6b768e"),
            St::Accent => tc("#8db4ff", "#2f62d0"),
            St::Good => tc("#6fd9a8", "#1f8a5e"),
            St::Warn => tc("#f2c166", "#a8700c"),
            St::Err => tc("#ff8c97", "#c0364a"),
            St::Dir => tc("#8db4ff", "#2f62d0"),
            St::Pink => tc("#f29bd0", "#b0368c"),
            St::Rgb(c) => c,
        }
    }

    pub fn draw_terminal(&mut self, ui: &mut Ui, fr: &Frame) {
        let th = self.th.clone();
        TDARK.with(|c| c.set(th.dark));
        let ctx = ui.ctx().clone();
        let active = fr.active;
        let rect = fr.stage;
        if self.shell.term.sessions.is_empty() {
            let home = self.shell.term.home.clone();
            let t = self.time;
            self.shell.term.new_session_at(home, t);
        }
        let n_sessions = self.shell.term.sessions.len();
        self.shell.term.active = self.shell.term.active.min(n_sessions - 1);
        self.term_sidebar(ui, fr);
        if self.term_toolbar(ui, fr) {
            return;
        }
        let si = self.shell.term.active.min(self.shell.term.sessions.len() - 1);
        let kb = active && self.shell.settings.is_none() && !self.palette.open && self.shell.appmenu.is_none();
        let full = self.shell.term.sessions[si].full.is_some();
        if kb {
            let (cmd, t, w, plus, minus) = ctx.input(|i| (i.modifiers.command, i.key_pressed(egui::Key::T), i.key_pressed(egui::Key::W), i.key_pressed(egui::Key::Equals) || i.key_pressed(egui::Key::Plus), i.key_pressed(egui::Key::Minus)));
            if cmd && t {
                self.term_cmd("new");
            } else if cmd && w {
                self.term_cmd("close");
            } else if cmd && plus {
                self.term_cmd("bigger");
            } else if cmd && minus {
                self.term_cmd("smaller");
            }
            if full {
                self.term_full_keys(&ctx, si);
            } else {
                self.term_keys(&ctx, si);
            }
            ctx.request_repaint_after(std::time::Duration::from_millis(500));
        }
        let si = si.min(self.shell.term.sessions.len() - 1);
        if self.shell.term.sessions[si].full.is_some() {
            self.draw_term_full(ui, fr, si);
            return;
        }
        let p = ui.painter().clone();
        let font_px = self.shell.term.font;
        let mono = FontId::monospace(font_px);
        let charw = self.tw(&p, "M", &mono);
        let lh = (font_px * self.shell.term.spacing).round();
        let (ghost_on, cards, cursor_style, blink_on) = (self.shell.term.ghost, self.shell.term.cards, self.shell.term.cursor, self.shell.term.blink);

        // the terminal renders straight on the window surface: no card, no divider
        let hint_h = 28.0;
        let surf = Rect::from_min_max(pos2(rect.left() + 6.0, rect.top() + 2.0), pos2(rect.right() - 4.0, rect.bottom() - hint_h - 2.0));

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
        let mut block_menu: Option<(Pos2, usize)> = None;
        let bg_resp = self.hit(ui, surf, "tbg", Sense::click());
        if bg_resp.secondary_clicked() {
            if let Some(pos) = ui.ctx().pointer_latest_pos() {
                let items = self.term_menu_main();
                self.open_app_menu(pos, items);
            }
        }
        let mut clicked_loc: Option<Location> = None;
        let mut copy_text: Option<String> = None;
        let mut cui = ui.new_child(egui::UiBuilder::new().max_rect(inner).layout(egui::Layout::top_down(egui::Align::Min)));
        cui.set_clip_rect(inner.intersect(ui.clip_rect()));
        let mut sa = egui::ScrollArea::vertical().id_salt(("term-scroll", self.shell.term.sessions[si].id)).auto_shrink([false, false]).scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded);
        if to_bottom {
            sa = sa.vertical_scroll_offset((total_h - inner.height()).max(0.0) + 40.0);
        }
        let ghost = if ghost_on && !input.is_empty() { self.shell.term.history.iter().rev().find(|h| h.starts_with(&input) && **h != input).map(|h| h[input.len()..].to_string()) } else { None };
        let blocks_cmds: Vec<(String, Location, i32, u32)> = self.shell.term.sessions[si].blocks.iter().map(|b| (b.cmd.clone(), b.cwd.clone(), b.code, b.ms)).collect();
        let time = self.time;
        let timing_on = self.shell.term.timing;
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
                        let cresp = self.hit(ui, card, ("tblk", si, bi), Sense::click());
                        if cresp.secondary_clicked() {
                            if let Some(pos) = ui.ctx().pointer_latest_pos() {
                                block_menu = Some((pos, bi));
                            }
                        }
                    }
                    if !cmd.is_empty() {
                        if cards {
                            grad_fill(&p, card, 12.0, alpha(ovc(), 0.05), alpha(ovc(), 0.02));
                            rim(&p, card, 12.0, 1.0, alpha(ovc(), 0.07), true, 24.0);
                        }
                        // header
                        let hy = y + 17.0;
                        let path = self.short_path(bcwd);
                        let cw = self.tw(&p, &path, &f_semi(11.0)) + 26.0;
                        let chip = Rect::from_min_size(pos2(card.left() + 10.0, hy - 11.0), vec2(cw, 22.0));
                        grad_fill(&p, chip, 11.0, alpha(th.accent, 0.20), alpha(th.accent, 0.09));
                        self.ico(&p, "folder", Ic::Duo, pos2(chip.left() + 12.0, hy), 12.0, tc("#9dbdff", "#3d72e0"));
                        p.text(pos2(chip.left() + 22.0, hy), Align2::LEFT_CENTER, &path, f_semi(11.0), tc("#b6cdff", "#2f62d0"));
                        p.text(pos2(chip.right() + 10.0, hy), Align2::LEFT_CENTER, self.fit(&p, cmd, &mono, card.right() - chip.right() - 120.0), mono.clone(), St::Plain.pipe());
                        // status
                        let ok = *code == 0;
                        let label = if *code == 130 { "interrupted".to_string() } else if ok { if timing_on { format!("{} ms", ms) } else { String::new() } } else { format!("exit {}", code) };
                        let lw = self.tw(&p, &label, &f_medium(10.5));
                        let col = if ok { tc("#6fd9a8", "#1f8a5e") } else if *code == 130 { tc("#f2c166", "#a8700c") } else { tc("#ff8c97", "#c0364a") };
                        self.ico(&p, if ok { "check-circle" } else { "close-circle" }, Ic::Filled, pos2(card.right() - 16.0, hy), 14.0, col);
                        p.text(pos2(card.right() - 28.0, hy), Align2::RIGHT_CENTER, label, f_medium(10.5), alpha(col, 0.9));
                        // copy on hover
                        let hov = ui.rect_contains_pointer(card);
                        if hov {
                            let cb = Rect::from_center_size(pos2(card.right() - 40.0 - lw - 12.0, hy), vec2(24.0, 24.0));
                            let cr_ = self.hit(ui, cb, ("tcopy", si, bi), Sense::click());
                            if cr_.hovered() {
                                p.circle_filled(cb.center(), 11.0, alpha(ovc(), 0.10));
                            }
                            self.ico(&p, "copy", Ic::Outline, cb.center(), 14.0, tc("#9aa8c4", "#6f7b93"));
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
                                p.text(pos2(t.right() + 12.0, t.center().y - 7.0), Align2::LEFT_CENTER, a, f_bold(15.0), tc("#eef3fb", "#111a2e"));
                                p.text(pos2(t.right() + 12.0, t.center().y + 10.0), Align2::LEFT_CENTER, self.fit(&p, b, &f_medium(11.0), card.width() - 90.0), f_medium(11.0), tc("#7d89a3", "#6b768e"));
                            }
                            Row::Cmds(cs) => {
                                let mut x = card.left() + 12.0;
                                for c in cs {
                                    let w = self.tw(&p, c, &mono) + 24.0;
                                    let r = Rect::from_min_size(pos2(x, ry + 3.0), vec2(w, 26.0));
                                    let resp = self.hit(ui, r, ("tcmd", si, bi, c.clone()), Sense::click());
                                    let a = self.anim(ui, ("tcmdh", c.clone()), resp.hovered());
                                    grad_fill(&p, r, 13.0, alpha(ovc(), 0.07 + 0.08 * a), alpha(ovc(), 0.03 + 0.05 * a));
                                    rim(&p, r, 13.0, 1.0, alpha(ovc(), 0.10 + 0.12 * a), true, 14.0);
                                    p.text(r.center(), Align2::CENTER_CENTER, c, mono.clone(), mix(tc("#b6cdff", "#2f62d0"), tc("#ffffff", "#0b1428"), a));
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
                                    let kc = if *is_dir { tc("#8db4ff", "#2f62d0") } else { kc };
                                    grad_fill(&p, r, 9.0, alpha(kc, 0.10 + 0.14 * a), alpha(kc, 0.04 + 0.08 * a));
                                    rim(&p, r, 9.0, 1.0, alpha(ovc(), 0.07 + 0.10 * a), true, 14.0);
                                    self.ico(&p, kind.icon(), if a > 0.5 || *is_dir { Ic::Duo } else { Ic::Outline }, pos2(r.left() + 14.0, r.center().y), 14.0, kc);
                                    p.text(pos2(r.left() + 26.0, r.center().y), Align2::LEFT_CENTER, if *is_dir { format!("{name}/") } else { name.clone() }, mono.clone(), if *is_dir { tc("#a9c4ff", "#2f62d0") } else { St::Plain.pipe() });
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
                                            color = mix(color, tc("#ffffff", "#0b1428"), 0.4);
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
            p.rect_stroke(pr.shrink(0.5), cr(14.0), egui::Stroke::new(1.0, alpha(ovc(), 0.06)), egui::StrokeKind::Inside);
            let user = format!("{}@smart", self.user_name().to_lowercase());
            let uw = self.tw(&p, &user, &f_semi(11.0)) + 20.0;
            let uc = Rect::from_min_size(pos2(pr.left() + 12.0, pr.top() + 8.0), vec2(uw, 22.0));
            grad_fill(&p, uc, 11.0, alpha(tc("#6fd9a8", "#1f8a5e"), 0.20), alpha(tc("#6fd9a8", "#1f8a5e"), 0.09));
            p.text(uc.center(), Align2::CENTER_CENTER, &user, f_semi(11.0), tc("#9ae8c3", "#1a7c55"));
            let path = self.short_path(&cwd);
            let pw = self.tw(&p, &path, &f_semi(11.0)) + 28.0;
            let pc = Rect::from_min_size(pos2(uc.right() + 6.0, uc.top()), vec2(pw, 22.0));
            grad_fill(&p, pc, 11.0, alpha(th.accent, 0.22), alpha(th.accent, 0.10));
            self.ico(&p, "folder-open", Ic::Duo, pos2(pc.left() + 13.0, pc.center().y), 12.0, tc("#9dbdff", "#3d72e0"));
            p.text(pos2(pc.left() + 24.0, pc.center().y), Align2::LEFT_CENTER, &path, f_semi(11.0), tc("#b6cdff", "#2f62d0"));
            let tm = clock_hm();
            p.text(pos2(pr.right() - 14.0, uc.center().y), Align2::RIGHT_CENTER, tm, f_medium(10.5), tc("#6c7894", "#8a95ac"));
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
                    let c = if inq.is_some() || *ch == '"' || *ch == '\'' { St::Good.pipe() } else if *ch == '|' || *ch == '>' { tc("#f29bd0", "#b0368c") } else if wi == 0 { if known { tc("#8db4ff", "#2f62d0") } else { tc("#ff9aa5", "#c0364a") } } else if tok.starts_with('-') { St::Warn.pipe() } else if tok.contains('/') || tok.starts_with('~') { tc("#c9d8f5", "#33425f") } else { St::Plain.pipe() };
                    colors.push(c);
                }
            }
            let chev = pos2(ix, iy + lh / 2.0);
            p.text(chev, Align2::LEFT_CENTER, "\u{276f}", FontId::monospace(font_px), if known || input.is_empty() { tc("#6fd9a8", "#1f8a5e") } else { tc("#ff9aa5", "#c0364a") });
            for (i, ch) in in_chars.iter().enumerate() {
                let (r_, c_) = (i / prompt_cols, i % prompt_cols);
                p.text(pos2(ix + (c_ as f32 + 2.0) * charw, iy + r_ as f32 * lh + lh / 2.0), Align2::LEFT_CENTER, ch.to_string(), mono.clone(), colors[i]);
            }
            if let Some(g) = &ghost {
                let start = in_chars.len();
                for (k, ch) in g.chars().enumerate() {
                    let i = start + k;
                    let (r_, c_) = (i / prompt_cols, i % prompt_cols);
                    p.text(pos2(ix + (c_ as f32 + 2.0) * charw, iy + r_ as f32 * lh + lh / 2.0), Align2::LEFT_CENTER, ch.to_string(), mono.clone(), alpha(tc("#8b97b3", "#7b86a0"), 0.45));
                }
                if cursor >= in_chars.len() {
                    p.text(pos2(pr.right() - 14.0, iy + lh / 2.0), Align2::RIGHT_CENTER, "\u{2192} accept", f_medium(10.0), tc("#5d6985", "#96a0b6"));
                }
            }
            if input.is_empty() {
                p.text(pos2(ix + 2.0 * charw, iy + lh / 2.0), Align2::LEFT_CENTER, "type a command\u{2026}  try  help", mono.clone(), alpha(tc("#8b97b3", "#7b86a0"), 0.38));
            }
            // cursor (spring-smoothed, blinking)
            let (cr_, cc_) = (cursor / prompt_cols, cursor % prompt_cols);
            let tx = ix + (cc_ as f32 + 2.0) * charw;
            let ty = iy + cr_ as f32 * lh;
            let sx = spring_to(ui.ctx(), ("term-cx", si), tx, 900.0, 46.0);
            let sy = spring_to(ui.ctx(), ("term-cy", si), ty, 900.0, 46.0);
            let blink = if active { if blink_on { ((time * 2.0).sin() > -0.35) as u8 as f32 } else { 1.0 } } else { 0.0 };
            let cur = match cursor_style {
                1 => Rect::from_min_size(pos2(sx, sy + 2.0), vec2(2.0, lh - 4.0)),
                2 => Rect::from_min_size(pos2(sx, sy + lh - 5.0), vec2(charw.max(7.0), 2.5)),
                _ => Rect::from_min_size(pos2(sx, sy + 2.0), vec2(charw.max(7.0), lh - 4.0)),
            };
            if active {
                p.rect_filled(cur, cr(2.0), alpha(th.accent, 0.85 * blink));
                if let (Some(ch), 0) = (in_chars.get(cursor), cursor_style) {
                    p.text(pos2(tx, ty + lh / 2.0), Align2::LEFT_CENTER, ch.to_string(), mono.clone(), alpha(tc("#08101e", "#ffffff"), blink));
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
                        p.text(pos2(x + 4.0, cy + 14.0), Align2::LEFT_CENTER, format!("+{}", cands.len() - i), f_medium(11.0), tc("#7d89a3", "#6b768e"));
                        break;
                    }
                    let r = Rect::from_min_size(pos2(x, cy + 3.0), vec2(w, 24.0));
                    let on = cur_i == Some(i);
                    grad_fill(&p, r, 12.0, alpha(th.accent, if on { 0.5 } else { 0.14 }), alpha(th.accent, if on { 0.3 } else { 0.06 }));
                    rim(&p, r, 12.0, 1.0, alpha(ovc(), if on { 0.3 } else { 0.1 }), true, 12.0);
                    p.text(r.center(), Align2::CENTER_CENTER, &label, f_medium(11.5), if on { tc("#ffffff", "#0b1428") } else { tc("#b6cdff", "#2f62d0") });
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
            if x + 150.0 > hb.right() - 140.0 {
                break;
            }
            let w = self.keycap(&p, pos2(x + self.tw(&p, k, &f_semi(10.0)).max(9.0) + 9.0, hb.center().y), k, false);
            x += w + 6.0;
            p.text(pos2(x, hb.center().y), Align2::LEFT_CENTER, v, f_medium(10.5), th.ink4);
            x += self.tw(&p, v, &f_medium(10.5)) + 16.0;
        }
        p.text(pos2(hb.right(), hb.center().y), Align2::RIGHT_CENTER, format!("{}  \u{b7}  {} commands", self.shell.term.sessions[si].cwd.fs, self.shell.term.history.len()), f_medium(10.5), th.ink4);
        if let Some((pos, bi)) = block_menu {
            self.shell.term.ctx_block = Some((si, bi));
            let items = self.term_menu_block(si, bi);
            self.open_app_menu(pos, items);
        }

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

// ───────────────────────────── real terminal emulation (vt100) ─────────────────────────────
thread_local! {
    static TDARK: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
}
fn td() -> bool {
    TDARK.with(|c| c.get())
}
/// Colour that adapts to the theme: (dark value, light value).
fn tc(dark: &str, light: &str) -> Color32 {
    hex(if td() { dark } else { light })
}
/// Overlay tint for depth on the surface: light on dark themes, ink on light themes.
fn ovc() -> Color32 {
    if td() { Color32::WHITE } else { hex("#14203a") }
}

fn idx_color(i: u8, bold: bool) -> Color32 {
    let dark = td();
    let base: [&str; 16] = if dark {
        ["#4a5468", "#ff7b86", "#6fd9a8", "#f2c166", "#7fa8ff", "#e08cff", "#6fd7e6", "#dbe4f5", "#7d89a3", "#ff9aa5", "#8cebc0", "#ffd88a", "#9dbdff", "#eaa8ff", "#8ee6f2", "#ffffff"]
    } else {
        ["#2b3550", "#c0364a", "#1f8a5e", "#a8700c", "#2f62d0", "#9a3fc8", "#13808f", "#6b768e", "#5a6680", "#d4475b", "#25a06f", "#bb8110", "#4577e6", "#ae52dc", "#1b97a8", "#111a2e"]
    };
    let i = if bold && i < 8 { i + 8 } else { i };
    match i {
        0..=15 => hex(base[i as usize]),
        16..=231 => {
            let n = i - 16;
            let l = |v: u8| if v == 0 { 0 } else { 55 + 40 * v };
            Color32::from_rgb(l(n / 36), l((n / 6) % 6), l(n % 6))
        }
        _ => {
            let g = 8 + 10 * (i - 232);
            Color32::from_gray(g)
        }
    }
}

fn ansi_rgb(c: vt100::Color, bold: bool) -> Option<Color32> {
    match c {
        vt100::Color::Default => None,
        vt100::Color::Rgb(r, g, b) => Some(Color32::from_rgb(r, g, b)),
        vt100::Color::Idx(i) => Some(idx_color(i, bold)),
    }
}

/// Run raw ANSI text through the vt100 state machine and turn the resulting screen into styled lines.
pub fn ansi_lines(text: &str) -> Vec<TLine> {
    let n = text.split('\n').count();
    let mut p = vt100::Parser::new((n as u16 + 2).clamp(2, 1500), 240, 0);
    p.process(text.replace('\n', "\r\n").as_bytes());
    let scr = p.screen();
    let (rows, cols) = scr.size();
    let mut out: Vec<TLine> = vec![];
    for r in 0..rows {
        let mut spans: Vec<Span> = vec![];
        for c in 0..cols {
            let Some(cell) = scr.cell(r, c) else { continue };
            if cell.is_wide_continuation() {
                continue;
            }
            let ch = if cell.has_contents() { cell.contents() } else { " ".to_string() };
            let st = ansi_rgb(cell.fgcolor(), cell.bold()).map(St::Rgb).unwrap_or(St::Plain);
            let bold = cell.bold();
            match spans.last_mut() {
                Some(l) if l.st == st && l.bold == bold => l.text.push_str(&ch),
                _ => spans.push(Span { text: ch, st, link: None, bold }),
            }
        }
        if let Some(l) = spans.last_mut() {
            l.text = l.text.trim_end().to_string();
        }
        spans.retain(|s| !s.text.is_empty());
        out.push(TLine::Spans(spans));
    }
    while matches!(out.last(), Some(TLine::Spans(s)) if s.is_empty()) {
        out.pop();
    }
    out
}

pub struct FullApp {
    pub kind: &'static str,
    pub parser: vt100::Parser,
    pub t0: f64,
    pub last: f64,
}

fn top_frame(t: f64, cols: usize, rows: usize, tasks: usize) -> String {
    let mut o = String::from("\x1b[?25l\x1b[2J\x1b[H");
    let bw = (cols.saturating_sub(34) / 2).clamp(8, 36);
    let bar = |label: &str, v: f32| -> String {
        let n = (v * bw as f32).round() as usize;
        let col = if v < 0.5 { "32" } else if v < 0.8 { "33" } else { "31" };
        format!("\x1b[1;36m{label:<4}\x1b[0m[\x1b[{col}m{}\x1b[90m{}\x1b[0m] \x1b[1m{:>4.1}%\x1b[0m", "|".repeat(n), "|".repeat(bw - n), v * 100.0)
    };
    o.push_str(&format!("\x1b[1;34mSmart Terminal\x1b[0m \x1b[90mtop\x1b[0m  tasks: \x1b[1m{tasks}\x1b[0m   up {:.0}s\r\n", t + 3600.0));
    for c in 0..4 {
        let v = (0.42 + 0.38 * (t * (0.6 + c as f64 * 0.23) + c as f64 * 1.7).sin() + 0.1 * (t * 3.1 + c as f64).sin()).clamp(0.03, 0.99) as f32;
        o.push_str(&bar(&format!("CPU{c}"), v));
        o.push_str("\r\n");
    }
    o.push_str(&bar("Mem", (0.58 + 0.04 * (t * 0.4).sin()) as f32));
    o.push_str("\r\n");
    o.push_str(&bar("Swp", 0.07));
    o.push_str("\r\n\r\n");
    o.push_str("\x1b[7m  PID USER       CPU%   MEM%  COMMAND");
    o.push_str(&" ".repeat(cols.saturating_sub(40)));
    o.push_str("\x1b[0m\r\n");
    let procs = [("smart-files", 14.0, 6.2), ("smart-photos", 9.0, 8.8), ("egui-render", 22.0, 3.1), ("wasm-vm", 17.0, 11.4), ("smart-terminal", 4.0, 1.9), ("vfs-indexer", 7.0, 2.4), ("thumbnailer", 11.0, 4.7), ("audio-engine", 3.0, 1.2), ("compositor", 6.0, 3.9), ("zip-worker", 2.0, 0.8), ("clipboardd", 0.4, 0.2), ("syncd", 1.1, 0.6)];
    let mut v: Vec<(usize, &str, f64, f64)> = procs.iter().enumerate().map(|(i, (n, c, m))| (i, *n, (c * (0.65 + 0.35 * (t * 0.8 + i as f64 * 1.3).sin())).max(0.1), *m)).collect();
    v.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
    for (i, n, cpu, mem) in v.into_iter().take(rows.saturating_sub(11)) {
        let col = if cpu > 15.0 { "31" } else if cpu > 8.0 { "33" } else { "32" };
        o.push_str(&format!("{:>5} naledzi \x1b[{col}m{:>6.1}\x1b[0m {:>6.1}  \x1b[1;37m{n}\x1b[0m\r\n", 1200 + i * 37, cpu, mem));
    }
    o.push_str(&format!("\x1b[{};1H\x1b[7m q \x1b[0m quit   \x1b[7m Esc \x1b[0m close   \x1b[90mvt100 screen\x1b[0m", rows.max(1)));
    o
}

fn ctrl_nav(ctx: &egui::Context, k: egui::Key) -> bool {
    ctx.input(|i| i.key_pressed(k))
}

impl SmartApp {
    fn term_full_keys(&mut self, ctx: &egui::Context, si: usize) {
        let quit = ctrl_nav(ctx, egui::Key::Escape) || ctrl_nav(ctx, egui::Key::Q) || ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::C)) || ctx.input(|i| i.events.iter().any(|e| matches!(e, egui::Event::Text(t) if t == "q")));
        if quit {
            self.shell.term.sessions[si].full = None;
            self.shell.term.sessions[si].to_bottom = true;
        }
    }

    pub fn term_cmd(&mut self, c: &str) {
        if self.shell.term.sessions.is_empty() {
            self.shell.term.command(c);
            return;
        }
        let si = self.shell.term.active.min(self.shell.term.sessions.len() - 1);
        let (name, arg) = c.split_once(':').unwrap_or((c, ""));
        let blk = self.shell.term.ctx_block.take();
        match name {
            "run" => {
                let cmd = if arg == "!!" { self.shell.term.history.last().cloned().unwrap_or_default() } else { arg.to_string() };
                self.term_submit(si, cmd);
            }
            "go" => {
                if let Some(pl) = arg.parse::<usize>().ok().and_then(|i| self.places.get(i).cloned()) {
                    self.term_submit(si, format!("cd {}", quote(&pl.loc.path)));
                }
            }
            "fill" => {
                let s = &mut self.shell.term.sessions[si];
                s.input = arg.to_string();
                s.cursor = s.input.chars().count();
                s.to_bottom = true;
            }
            "full" => {
                let mut parser = vt100::Parser::new(30, 100, 0);
                parser.process(b"");
                let t = self.time;
                self.shell.term.sessions[si].full = Some(FullApp { kind: "top", parser, t0: t, last: -10.0 });
            }
            "copy_cmd" | "copy_out" | "copy_all" => {
                if let Some((s, b)) = blk {
                    if let Some(bl) = self.shell.term.sessions.get(s).and_then(|x| x.blocks.get(b)) {
                        let out = text_of(&bl.lines).join("\n");
                        let t = match name {
                            "copy_cmd" => bl.cmd.clone(),
                            "copy_out" => out,
                            _ => format!("$ {}\n{}", bl.cmd, out),
                        };
                        self.acts.push(Act::CopyText(t));
                        self.toast("Copied", true);
                    }
                }
            }
            "copy_last" => {
                if let Some(bl) = self.shell.term.sessions[si].blocks.iter().rev().find(|b| !b.cmd.is_empty()) {
                    let t = text_of(&bl.lines).join("\n");
                    self.acts.push(Act::CopyText(t));
                    self.toast("Last output copied", true);
                }
            }
            "rerun" => {
                if let Some((s, b)) = blk {
                    if let Some(cmd) = self.shell.term.sessions.get(s).and_then(|x| x.blocks.get(b)).map(|b| b.cmd.clone()) {
                        self.term_submit(si, cmd);
                    }
                }
            }
            "rm_block" => {
                if let Some((s, b)) = blk {
                    if let Some(x) = self.shell.term.sessions.get_mut(s) {
                        if b < x.blocks.len() {
                            x.blocks.remove(b);
                        }
                    }
                }
            }
            "ghost" => self.shell.term.ghost = !self.shell.term.ghost,
            "timing" => self.shell.term.timing = !self.shell.term.timing,
            "cards" => self.shell.term.cards = !self.shell.term.cards,
            "dup" => {
                let cwd = self.shell.term.sessions[si].cwd.clone();
                let t = self.time;
                self.shell.term.new_session_at(cwd, t);
            }
            "close_idx" => {
                if let Ok(i) = arg.parse::<usize>() {
                    if self.shell.term.sessions.len() > 1 && i < self.shell.term.sessions.len() {
                        self.shell.term.sessions.remove(i);
                        self.shell.term.active = self.shell.term.active.min(self.shell.term.sessions.len() - 1);
                    }
                }
            }
            "close_others" => {
                if let Ok(i) = arg.parse::<usize>() {
                    if i < self.shell.term.sessions.len() {
                        let keep = self.shell.term.sessions.remove(i);
                        self.shell.term.sessions = vec![keep];
                        self.shell.term.active = 0;
                    }
                }
            }
            "reset_font" => self.shell.term.font = 13.0,
            "clear_history" => {
                self.shell.term.history.clear();
                self.toast("History cleared", true);
            }
            _ => self.shell.term.command(c),
        }
    }

    pub(crate) fn term_menu_main(&self) -> Vec<MI> {
        let t = &self.shell.term;
        let a = |s: &str| Act::App(format!("term.{s}"));
        vec![
            mi("add", "New session", a("new")).k("Ctrl+T").q(),
            mi("copy", "Copy last output", a("copy_last")).q(),
            mi("trash-bin-trash", "Clear screen", a("clear")).k("Ctrl+L").q(),
            mi("history", "Run again", a("run:!!")).off(t.history.is_empty()),
            mi("text", "Text size", Act::Toast(String::new())).sub(vec![mi("add", "Bigger", a("bigger")).k("Ctrl+="), mi("alt-arrow-down", "Smaller", a("smaller")).k("Ctrl+-"), mi("restart", "Actual size", a("reset_font"))]).gap(),
            mi("magic-stick-3", "Ghost suggestions", a("ghost")).on(t.ghost),
            mi("clock-circle", "Show timings", a("timing")).on(t.timing),
            mi("widget", "Command cards", a("cards")).on(t.cards),
            mi("copy", "Duplicate session", a("dup")).gap(),
            mi("close-circle", "Close session", a("close")).k("Ctrl+W").off(self.shell.term.sessions.len() < 2),
            mi("settings-minimalistic", "Terminal settings\u{2026}", Act::App("settings".into())).k("Ctrl+,").gap(),
            mi("command", "Command palette", Act::Palette).k("Ctrl+K"),
        ]
    }

    fn term_menu_block(&self, si: usize, bi: usize) -> Vec<MI> {
        let a = |s: &str| Act::App(format!("term.{s}"));
        let has_out = self.shell.term.sessions.get(si).and_then(|s| s.blocks.get(bi)).is_some_and(|b| !b.lines.is_empty());
        vec![
            mi("copy", "Copy command", a("copy_cmd")).q(),
            mi("document-text", "Copy output", a("copy_out")).off(!has_out).q(),
            mi("copy", "Copy command and output", a("copy_all")),
            mi("refresh", "Run again", a("rerun")).gap().q(),
            mi("trash-bin-minimalistic", "Remove block", a("rm_block")).danger().q(),
            mi("settings-minimalistic", "Terminal settings\u{2026}", Act::App("settings".into())).gap(),
        ]
    }

    fn term_sidebar(&mut self, ui: &mut Ui, fr: &Frame) {
        if fr.side.width() < 24.0 {
            return;
        }
        let th = self.th.clone();
        let saved = ui.clip_rect();
        ui.set_clip_rect(fr.side.intersect(saved));
        let p = ui.painter().clone();
        let left = fr.win.left();
        let gear = Rect::from_center_size(pos2(left + SIDEBAR_W - 28.0, fr.win.top() + APPBAR_H / 2.0), vec2(30.0, 30.0));
        if self.icon_button(ui, gear, "settings-minimalistic", "tm-gear", Btn::Quiet, true).on_hover_text("Terminal settings  Ctrl+,").clicked() {
            self.shell.settings = Some((AppId::Terminal, 0));
        }
        let ws = Rect::from_min_size(pos2(left + 12.0, fr.win.top() + 54.0), vec2(SIDEBAR_W - 24.0, 42.0));
        let av = Rect::from_min_size(ws.min + vec2(7.0, 7.0), vec2(28.0, 28.0));
        shadow(&p, av, 9.0, 2.0, 6.0, -1.0, c(20, 30, 60, 0.4));
        grad_fill(&p, av, 9.0, hex("#2f3d5c"), hex("#111827"));
        rim(&p, av, 9.0, 1.0, Color32::from_white_alpha(90), true, 14.0);
        p.text(av.center(), Align2::CENTER_CENTER, ">_", f_bold(11.5), hex("#7dffb2"));
        let n = self.shell.term.sessions.len();
        let user = self.user_name().to_lowercase();
        p.text(pos2(av.right() + 9.0, ws.center().y - 7.5), Align2::LEFT_CENTER, "Terminal", f_semi(13.0), th.ink1);
        p.text(pos2(av.right() + 9.0, ws.center().y + 8.0), Align2::LEFT_CENTER, format!("{user}@smart \u{b7} {n} session{}", if n == 1 { "" } else { "s" }), f_medium(11.0), th.ink3);
        let si = self.shell.term.active.min(n - 1);
        let card = Rect::from_min_size(pos2(left + 12.0, fr.side.bottom() - 14.0 - 62.0), vec2(SIDEBAR_W - 24.0, 62.0));
        elevate(&p, &th, card, 14.0, 2);
        grad_fill(&p, card, 14.0, th.card_top, th.card_bottom);
        rim(&p, card, 14.0, 1.2, th.rim, true, 30.0);
        let cwd = self.short_path(&self.shell.term.sessions[si].cwd);
        p.text(card.left_top() + vec2(14.0, 19.0), Align2::LEFT_CENTER, "Working folder", f_semi(T_META), th.ink1);
        p.text(card.left_top() + vec2(14.0, 38.0), Align2::LEFT_CENTER, self.fit(&p, &cwd, &FontId::monospace(11.5), card.width() - 28.0), FontId::monospace(11.5), th.accent_ink);
        p.text(card.left_top() + vec2(14.0, 53.0), Align2::LEFT_CENTER, format!("{} \u{b7} {} commands run", self.shell.term.sessions[si].cwd.fs, self.shell.term.history.len()), f_medium(T_LABEL), th.ink4);
        let nav = Rect::from_min_max(pos2(left, ws.bottom() + 6.0), pos2(left + SIDEBAR_W, card.top() - 8.0));
        let recent: Vec<String> = {
            let mut v: Vec<String> = vec![];
            for h in self.shell.term.history.iter().rev() {
                if !v.contains(h) {
                    v.push(h.clone());
                }
                if v.len() >= 6 {
                    break;
                }
            }
            v
        };
        let places = self.places.clone();
        let quick = [("folder-open", "ls"), ("layers-minimalistic", "tree -L 2"), ("laptop", "neofetch"), ("slider-vertical", "top"), ("palette", "colors"), ("info-circle", "help")];
        let content_h = 12.0 + 22.0 + places.len() as f32 * 33.0 + 14.0 + 23.0 + recent.len().max(1) as f32 * 33.0 + 14.0 + 23.0 + quick.len() as f32 * 33.0 + 12.0;
        let off = self.vscroll(ui, nav, "tm-side", content_h);
        ui.set_clip_rect(nav.intersect(saved));
        let pp = p.with_clip_rect(nav.intersect(saved));
        let x0 = nav.left() + 12.0;
        let w = SIDEBAR_W - 24.0;
        let mut y = nav.top() - off + 12.0;
        let label = |y: f32, s: &str| pp.text(pos2(nav.left() + 20.0, y + 7.0), Align2::LEFT_CENTER, s.to_uppercase(), f_semi(T_LABEL), th.ink4);
        let vis = |y: f32| y + 32.0 > nav.top() - 4.0 && y < nav.bottom() + 4.0;
        label(y, "Places");
        y += 22.0;
        let cur = self.shell.term.sessions[si].cwd.clone();
        for (i, pl) in places.iter().enumerate() {
            if vis(y) {
                let r = self.sb_row(ui, Rect::from_min_size(pos2(x0, y), vec2(w, 32.0)), pl.icon, &pl.title, pl.loc == cur, None, false, &format!("tm-pl-{i}"));
                if r.clicked() {
                    self.push_act(Act::App(format!("term.go:{i}")));
                }
            }
            y += 33.0;
        }
        y += 14.0;
        label(y, "Recent");
        y += 23.0;
        if recent.is_empty() && vis(y) {
            pp.text(pos2(x0 + 10.0, y + 16.0), Align2::LEFT_CENTER, "Commands you run show up here", f_medium(T_META), th.ink4);
        }
        for (i, h) in recent.iter().enumerate() {
            if vis(y) {
                let r = self.sb_row(ui, Rect::from_min_size(pos2(x0, y), vec2(w, 32.0)), "history", h, false, None, false, &format!("tm-rc-{i}"));
                if r.clicked() {
                    self.push_act(Act::App(format!("term.fill:{h}")));
                }
                if r.secondary_clicked() {
                    if let Some(pos) = ui.ctx().pointer_latest_pos() {
                        self.open_app_menu(pos, vec![mi("play", "Run", Act::App(format!("term.run:{h}"))), mi("pen", "Edit in prompt", Act::App(format!("term.fill:{h}"))), mi("copy", "Copy", Act::CopyText(h.clone()))]);
                    }
                }
            }
            y += 33.0;
        }
        if recent.is_empty() {
            y += 33.0;
        }
        y += 14.0;
        label(y, "Quick commands");
        y += 23.0;
        for (i, (icon, cmd)) in quick.iter().enumerate() {
            if vis(y) {
                let r = self.sb_row(ui, Rect::from_min_size(pos2(x0, y), vec2(w, 32.0)), icon, cmd, false, None, false, &format!("tm-q-{i}"));
                if r.clicked() {
                    self.push_act(Act::App(format!("term.run:{cmd}")));
                }
            }
            y += 33.0;
        }
        ui.set_clip_rect(saved);
    }

    /// Session tabs live in the toolbar. Returns true when the session list changed (caller redraws next frame).
    fn term_toolbar(&mut self, ui: &mut Ui, fr: &Frame) -> bool {
        let th = self.th.clone();
        let p = ui.painter().clone();
        let cy = fr.bar.center().y;
        let mut rx = fr.right_x;
        let avail = rx - fr.lead_x;
        let more = Rect::from_min_size(pos2(rx - 34.0, cy - 17.0), vec2(34.0, 34.0));
        if self.icon_button(ui, more, "menu-dots", "tm-more", Btn::Raised, true).on_hover_text("More").clicked() {
            let items = self.term_menu_main();
            self.open_app_menu(pos2(more.right() - 232.0, more.bottom() + 8.0), items);
        }
        rx = more.left() - 8.0;
        if avail > 560.0 {
            let clr = Rect::from_min_size(pos2(rx - 34.0, cy - 17.0), vec2(34.0, 34.0));
            if self.icon_button(ui, clr, "trash-bin-trash", "tm-clear", Btn::Raised, true).on_hover_text("Clear  Ctrl+L").clicked() {
                self.push_act(Act::App("term.clear".into()));
            }
            rx = clr.left() - 8.0;
        }
        if avail > 700.0 {
            let seg = Rect::from_min_size(pos2(rx - 84.0, cy - 17.0), vec2(84.0, 34.0));
            raised(&p, &th, seg, 17.0, false, false);
            let (b1, b2) = (Rect::from_min_size(seg.min, vec2(42.0, 34.0)), Rect::from_min_size(seg.min + vec2(42.0, 0.0), vec2(42.0, 34.0)));
            for (r, id, txt, cmd, tip) in [(b1, "tm-sm", "A\u{2212}", "smaller", "Smaller text  Ctrl+\u{2212}"), (b2, "tm-bg", "A+", "bigger", "Bigger text  Ctrl+=")] {
                let resp = self.hit(ui, r, id, Sense::click());
                let a = self.anim(ui, (id, "h"), resp.hovered());
                if a > 0.01 {
                    p.rect_filled(r.shrink(3.0), cr(14.0), alpha(th.row_hover, a));
                }
                p.text(r.center(), Align2::CENTER_CENTER, txt, f_semi(T_BODY), mix(th.ink2, th.ink1, a));
                resp.clone().on_hover_text(tip);
                if resp.clicked() {
                    self.push_act(Act::App(format!("term.{cmd}")));
                }
            }
            rx = seg.left() - 10.0;
        }
        // tabs
        let n = self.shell.term.sessions.len();
        let si = self.shell.term.active.min(n - 1);
        let plus_w = 34.0;
        let strip_w = (rx - fr.lead_x - plus_w - 10.0).max(80.0);
        let tw_ = (strip_w / n as f32).clamp(88.0, 176.0);
        let mut close: Option<usize> = None;
        let mut switch: Option<usize> = None;
        for i in 0..n {
            let r = Rect::from_min_size(pos2(fr.lead_x + tw_ * i as f32, cy - 16.0), vec2(tw_ - 6.0, 32.0));
            if r.right() > rx - plus_w {
                break;
            }
            let on = i == si;
            let resp = self.hit(ui, r, ("tsess", i), Sense::click());
            if on {
                raised(&p, &th, r, 16.0, false, false);
            } else if resp.hovered() {
                p.rect_filled(r, cr(16.0), th.row_hover);
            }
            self.ico(&p, "terminal", if on || resp.hovered() { Ic::Duo } else { Ic::Outline }, pos2(r.left() + 16.0, r.center().y), 15.0, if on { th.accent_ink } else { th.ink3 });
            let title = self.short_path(&self.shell.term.sessions[i].cwd);
            let showx = n > 1 && (on || resp.hovered());
            p.text(pos2(r.left() + 31.0, r.center().y), Align2::LEFT_CENTER, self.fit(&p, &title, &f_semi(T_BODY), r.width() - 40.0 - if showx { 20.0 } else { 0.0 }), f_semi(T_BODY), if on { th.ink1 } else { th.ink2 });
            if showx {
                let xr = Rect::from_center_size(pos2(r.right() - 15.0, r.center().y), vec2(20.0, 20.0));
                let xh = self.hit(ui, xr, ("tsx", i), Sense::click());
                if xh.hovered() {
                    p.circle_filled(xr.center(), 9.0, th.row_press);
                }
                self.ico(&p, "close-circle", Ic::Outline, xr.center(), 14.0, th.ink3);
                if xh.clicked() {
                    close = Some(i);
                }
            }
            if resp.clicked() && !on && close.is_none() {
                switch = Some(i);
            }
            if resp.secondary_clicked() {
                if let Some(pos) = ui.ctx().pointer_latest_pos() {
                    self.shell.term.active = i;
                    let a = |s: String| Act::App(format!("term.{s}"));
                    self.open_app_menu(pos, vec![mi("add", "New session", a("new".into())).k("Ctrl+T"), mi("copy", "Duplicate session", a("dup".into())), mi("close-circle", "Close session", a(format!("close_idx:{i}"))).off(n < 2).gap(), mi("close-circle", "Close other sessions", a(format!("close_others:{i}"))).off(n < 2)]);
                }
            }
        }
        let shown = ((rx - plus_w - fr.lead_x) / tw_).floor().max(1.0).min(n as f32);
        let plus = Rect::from_min_size(pos2(fr.lead_x + tw_ * shown, cy - 17.0), vec2(plus_w, 34.0));
        if self.icon_button(ui, plus, "add", "tsess-new", Btn::Quiet, true).on_hover_text("New session  Ctrl+T").clicked() {
            self.shell.term.command("new");
            return true;
        }
        if let Some(i) = switch {
            self.shell.term.active = i;
            return true;
        }
        if let Some(i) = close {
            self.term_cmd(&format!("close_idx:{i}"));
            return true;
        }
        false
    }

    fn draw_term_full(&mut self, ui: &mut Ui, fr: &Frame, si: usize) {
        let th = self.th.clone();
        let p = ui.painter().clone();
        let font_px = self.shell.term.font;
        let mono = FontId::monospace(font_px);
        let charw = self.tw(&p, "M", &mono);
        let lh = (font_px * 1.35).round();
        let rect = fr.stage;
        let hint_h = 28.0;
        let surf = Rect::from_min_max(pos2(rect.left() + 6.0, rect.top() + 2.0), pos2(rect.right() - 4.0, rect.bottom() - hint_h - 2.0));
        well(&p, &th, surf, 16.0);
        let inner = surf.shrink2(vec2(14.0, 10.0));
        let (cols, rows) = (((inner.width() / charw) as u16).max(20), ((inner.height() / lh) as u16).max(8));
        let t = self.time;
        let tasks = self.shell.term.sessions.len() * 3 + 42;
        if let Some(f) = self.shell.term.sessions[si].full.as_mut() {
            if t - f.last >= 0.45 {
                f.parser.set_size(rows, cols);
                let frame = top_frame(t - f.t0, cols as usize, rows as usize, tasks);
                f.parser.process(frame.as_bytes());
                f.last = t;
            }
        }
        ui.ctx().request_repaint_after(std::time::Duration::from_millis(100));
        if let Some(f) = self.shell.term.sessions[si].full.as_ref() {
            let scr = f.parser.screen();
            let pp = p.with_clip_rect(inner.expand(4.0));
            for r in 0..rows {
                let mut c = 0u16;
                while c < cols {
                    let Some(cell) = scr.cell(r, c) else { break };
                    let key = (cell.fgcolor(), cell.bgcolor(), cell.bold(), cell.inverse());
                    let mut text = String::new();
                    let mut e = c;
                    while e < cols {
                        let Some(x) = scr.cell(r, e) else { break };
                        if (x.fgcolor(), x.bgcolor(), x.bold(), x.inverse()) != key {
                            break;
                        }
                        if !x.is_wide_continuation() {
                            text.push_str(&if x.has_contents() { x.contents() } else { " ".into() });
                        }
                        e += 1;
                    }
                    let (mut fg, mut bg) = (ansi_rgb(key.0, key.2), ansi_rgb(key.1, false));
                    if key.3 {
                        let surface = if th.dark { hex("#10151f") } else { Color32::WHITE };
                        let (f2, b2) = (bg.unwrap_or(surface), fg.unwrap_or(th.ink1));
                        fg = Some(f2);
                        bg = Some(b2);
                    }
                    let x0 = inner.left() + c as f32 * charw;
                    let y0 = inner.top() + r as f32 * lh;
                    if let Some(b) = bg {
                        pp.rect_filled(Rect::from_min_size(pos2(x0, y0), vec2((e - c) as f32 * charw, lh)), cr(2.0), b);
                    }
                    if !text.trim().is_empty() {
                        pp.text(pos2(x0, y0 + lh / 2.0), Align2::LEFT_CENTER, text, mono.clone(), fg.unwrap_or(th.ink1));
                    }
                    c = e.max(c + 1);
                }
            }
        }
        let hb = Rect::from_min_max(pos2(rect.left() + 18.0, rect.bottom() - hint_h), pos2(rect.right() - 18.0, rect.bottom() - 4.0));
        let w = self.keycap(&p, pos2(hb.left() + 14.0, hb.center().y), "q", false);
        p.text(pos2(hb.left() + w + 6.0, hb.center().y), Align2::LEFT_CENTER, "quit  \u{b7}  Esc also works  \u{b7}  live full-screen app on a vt100 screen", f_medium(10.5), th.ink4);
        let r = self.hit(ui, surf, "tm-full-bg", Sense::click());
        if r.secondary_clicked() {
            if let Some(pos) = ui.ctx().pointer_latest_pos() {
                self.open_app_menu(pos, vec![mi("close-circle", "Quit top", Act::App("term.quit_full".into())).k("q"), mi("settings-minimalistic", "Terminal settings\u{2026}", Act::App("settings".into())).gap()]);
            }
        }
    }

    pub fn term_settings(&mut self, ui: &mut Ui, win: Rect) {
        let pages = [("text", "Appearance"), ("keyboard", "Behaviour"), ("history", "History")];
        let rows_for = |app: &SmartApp, page: usize| -> Vec<SRow> {
            let t = &app.shell.term;
            match page {
                0 => vec![
                    SRow::Slider { icon: "text", label: "Text size", sub: "Monospace size in the prompt and output", v: t.font, lo: 10.0, hi: 20.0, unit: " px" },
                    SRow::Slider { icon: "sort-vertical", label: "Line spacing", sub: "Space between lines of output", v: t.spacing * 10.0, lo: 12.0, hi: 20.0, unit: "" },
                    SRow::Choice { icon: "code-square", label: "Cursor shape", sub: "How the caret looks in the prompt", opts: &["Block", "Bar", "Line"], sel: t.cursor },
                    SRow::Toggle { icon: "bolt", label: "Blinking cursor", sub: "Pulse the caret while typing", on: t.blink },
                    SRow::Toggle { icon: "widget", label: "Command cards", sub: "Group each command and its output", on: t.cards },
                ],
                1 => vec![
                    SRow::Toggle { icon: "magic-stick-3", label: "Ghost suggestions", sub: "Dimmed completion from history, \u{2192} accepts", on: t.ghost },
                    SRow::Toggle { icon: "clock-circle", label: "Show timings", sub: "How long each command took", on: t.timing },
                    SRow::Slider { icon: "layers-minimalistic", label: "Scrollback", sub: "Command blocks kept per session", v: t.scrollback as f32, lo: 20.0, hi: 200.0, unit: "" },
                ],
                _ => vec![
                    SRow::Button { icon: "history", label: "Clear command history", sub: "Forget what you typed before", cta: "Clear" },
                    SRow::Button { icon: "trash-bin-trash", label: "Clear this session", sub: "Remove every block on screen", cta: "Clear" },
                    SRow::Button { icon: "restart", label: "Reset text size", sub: "Back to 13 px", cta: "Reset" },
                ],
            }
        };
        if let Some((page, row, val)) = self.draw_app_settings(ui, win, AppId::Terminal, &pages, &rows_for) {
            let t = &mut self.shell.term;
            match (page, row, val) {
                (0, 0, SVal::F(v)) => t.font = v.round(),
                (0, 1, SVal::F(v)) => t.spacing = (v.round() / 10.0).clamp(1.2, 2.0),
                (0, 2, SVal::Idx(i)) => t.cursor = i,
                (0, 3, SVal::Bool(b)) => t.blink = b,
                (0, 4, SVal::Bool(b)) => t.cards = b,
                (1, 0, SVal::Bool(b)) => t.ghost = b,
                (1, 1, SVal::Bool(b)) => t.timing = b,
                (1, 2, SVal::F(v)) => t.scrollback = v.round() as usize,
                (2, 0, _) => self.term_cmd("clear_history"),
                (2, 1, _) => self.term_cmd("clear"),
                (2, 2, _) => self.term_cmd("reset_font"),
                _ => {}
            }
        }
    }
}
