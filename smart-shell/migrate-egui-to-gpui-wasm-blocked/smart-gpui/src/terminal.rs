//! Smart Terminal — a tiny shell over the same in-memory filesystem as Files.
use crate::data::*;
use crate::sig::*;
use crate::theme::*;
use crate::ui::edit_text;
use gpui_kit::prelude::*;
use gpui_kit::*;

#[derive(Clone, Copy, PartialEq)]
enum Tone {
    Plain,
    Prompt,
    Dim,
    Good,
    Bad,
    Accent,
}

pub struct TermView {
    focus: FocusHandle,
    scroll: ScrollHandle,
    lines: Vec<(String, Tone)>,
    input: String,
    cwd: Vec<String>,
    hist: Vec<String>,
    hist_i: Option<usize>,
}

impl TermView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let mut s = Self {
            focus: cx.focus_handle(),
            scroll: ScrollHandle::new(),
            lines: vec![],
            input: String::new(),
            cwd: vec!["Home".into()],
            hist: vec![],
            hist_i: None,
        };
        s.push("Smart Terminal 0.1 \u{b7} GPUI \u{b7} wasm32", Tone::Accent);
        s.push("Type `help` for commands. Try `neofetch`, `ls`, `tree`, `theme dark`.", Tone::Dim);
        s
    }

    fn push(&mut self, s: impl Into<String>, t: Tone) {
        for l in s.into().split('\n') {
            self.lines.push((l.to_string(), t));
        }
    }

    fn prompt(&self) -> String {
        let p = if self.cwd.is_empty() { "/".to_string() } else { format!("~/{}", self.cwd[1..].join("/")).replace("~/", if self.cwd.len() == 1 { "~" } else { "~/" }) };
        format!("smart@shell {}", if self.cwd.len() == 1 { "~".to_string() } else { p })
    }

    fn resolve(&self, arg: &str) -> Vec<String> {
        let mut p = if arg.starts_with('/') { vec![] } else { self.cwd.clone() };
        for part in arg.split('/').filter(|s| !s.is_empty()) {
            match part {
                "." => {}
                ".." => {
                    p.pop();
                }
                "~" => p = vec!["Home".into()],
                x => p.push(x.to_string()),
            }
        }
        p
    }

    fn run(&mut self, line: &str, cx: &mut Context<Self>) {
        let pr = self.prompt();
        self.push(format!("{pr} \u{276f} {line}"), Tone::Prompt);
        let mut it = line.split_whitespace();
        let Some(cmd) = it.next() else { return };
        let args: Vec<&str> = it.collect();
        let fsr = cx.global::<FsG>().0.clone();
        match cmd {
            "help" => self.push("help  ls [-l]  cd  pwd  cat  tree  mkdir  touch  echo  clear  whoami  date  uname  df\nneofetch  theme <dark|light>  open <files|photos|kit>  toast <msg>", Tone::Plain),
            "clear" => self.lines.clear(),
            "pwd" => self.push(format!("/{}", self.cwd.join("/")), Tone::Plain),
            "whoami" => self.push("smart", Tone::Plain),
            "echo" => self.push(args.join(" "), Tone::Plain),
            "uname" => self.push("SmartOS 0.1 wasm32-unknown-unknown gpui", Tone::Plain),
            "date" => {
                let d = js_sys::Date::new_0();
                self.push(String::from(d.to_string()), Tone::Plain);
            }
            "df" => self.push("Filesystem   Size   Used  Avail  Use%\nsmart-ssd    512G   318G   194G   62%\nsandisk       64G    23G    41G   36%\nstudio-nas   8.0T   5.9T   2.1T   74%", Tone::Plain),
            "ls" => {
                let long = args.contains(&"-l");
                let target = args.iter().find(|a| !a.starts_with('-')).map(|a| self.resolve(a)).unwrap_or_else(|| self.cwd.clone());
                let fs = fsr.borrow();
                match fs.dir(&target) {
                    None => self.push(format!("ls: {}: No such directory", args.join(" ")), Tone::Bad),
                    Some(d) => {
                        let mut ch: Vec<&Node> = d.children.iter().collect();
                        ch.sort_by(|a, b| b.is_dir().cmp(&a.is_dir()).then(a.name.to_lowercase().cmp(&b.name.to_lowercase())));
                        let mut out = vec![];
                        for n in ch {
                            if long {
                                out.push((format!("{} {:>8}  {:<14}{}", if n.is_dir() { "d" } else { "-" }, fmt_size(n.total()), fmt_ago(n.hours), n.name), if n.is_dir() { Tone::Accent } else { Tone::Plain }));
                            } else {
                                out.push((n.name.clone() + if n.is_dir() { "/" } else { "" }, if n.is_dir() { Tone::Accent } else { Tone::Plain }));
                            }
                        }
                        drop(fs);
                        if long {
                            self.lines.extend(out);
                        } else {
                            // pack into rows
                            let mut row = String::new();
                            for (s, tone) in out {
                                if tone == Tone::Accent {
                                    self.lines.push((s, tone));
                                } else {
                                    if row.len() + s.len() > 70 {
                                        self.lines.push((std::mem::take(&mut row), Tone::Plain));
                                    }
                                    row += &format!("{s}   ");
                                }
                            }
                            if !row.is_empty() {
                                self.lines.push((row, Tone::Plain));
                            }
                        }
                    }
                }
            }
            "cd" => {
                let target = args.first().map(|a| self.resolve(a)).unwrap_or_else(|| vec!["Home".into()]);
                if fsr.borrow().dir(&target).is_some() {
                    self.cwd = target;
                } else {
                    self.push(format!("cd: {}: No such directory", args.first().unwrap_or(&"")), Tone::Bad);
                }
            }
            "cat" => {
                let Some(a) = args.first() else { return self.push("cat: missing file", Tone::Bad) };
                let mut p = self.resolve(a);
                let name = p.pop().unwrap_or_default();
                let fs = fsr.borrow();
                match fs.dir(&p).and_then(|d| d.children.iter().find(|c| c.name == name)) {
                    Some(n) if n.text.is_some() => {
                        let t = n.text.clone().unwrap();
                        drop(fs);
                        self.push(t, Tone::Plain)
                    }
                    Some(n) if n.is_dir() => self.push(format!("cat: {name}: Is a directory"), Tone::Bad),
                    Some(_) => self.push(format!("cat: {name}: binary file"), Tone::Dim),
                    None => self.push(format!("cat: {name}: No such file"), Tone::Bad),
                }
            }
            "tree" => {
                fn walk(n: &Node, prefix: &str, out: &mut Vec<(String, Tone)>, depth: usize) {
                    let len = n.children.len();
                    for (i, c) in n.children.iter().enumerate() {
                        let last = i + 1 == len;
                        out.push((format!("{prefix}{}{}{}", if last { "\u{2514}\u{2500}\u{2500} " } else { "\u{251c}\u{2500}\u{2500} " }, c.name, if c.is_dir() { "/" } else { "" }), if c.is_dir() { Tone::Accent } else { Tone::Plain }));
                        if c.is_dir() && depth < 2 {
                            walk(c, &format!("{prefix}{}", if last { "    " } else { "\u{2502}   " }), out, depth + 1);
                        }
                    }
                }
                let mut out = vec![];
                if let Some(d) = fsr.borrow().dir(&self.cwd) {
                    walk(d, "", &mut out, 0);
                }
                self.lines.extend(out);
            }
            "mkdir" | "touch" => {
                let Some(a) = args.first() else { return self.push(format!("{cmd}: missing operand"), Tone::Bad) };
                let mut fs = fsr.borrow_mut();
                let folder = cmd == "mkdir";
                if let Some(d) = fs.dir_mut(&self.cwd) {
                    d.children.push(Node { name: a.to_string(), kind: if folder { Kind::Folder } else { kind_of(a) }, size: 0, hours: 0.0, hue: 0.6, children: vec![], text: if folder { None } else { Some(String::new()) }, tag: 0 });
                }
            }
            "neofetch" => {
                self.push("        \u{2584}\u{2588}\u{2588}\u{2588}\u{2588}\u{2584}        smart@shell", Tone::Accent);
                self.push("      \u{2584}\u{2588}\u{2588}\u{2580}\u{2580}\u{2588}\u{2588}\u{2584}      \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}", Tone::Accent);
                self.push("     \u{2588}\u{2588}\u{2580}    \u{2580}\u{2588}\u{2588}     OS       SmartOS 0.1 wasm32", Tone::Accent);
                self.push("     \u{2588}\u{2588}\u{2584}    \u{2584}\u{2588}\u{2588}     UI       GPUI + gpui-kit", Tone::Accent);
                self.push("      \u{2580}\u{2588}\u{2588}\u{2584}\u{2584}\u{2588}\u{2588}\u{2580}      Renderer WebGPU / WebGL (wgpu)", Tone::Accent);
                self.push("        \u{2580}\u{2588}\u{2588}\u{2588}\u{2588}\u{2580}        Lang     100% Rust", Tone::Accent);
                self.push("                         JS/TS    0 lines", Tone::Good);
            }
            "theme" => match args.first().copied() {
                Some("dark") => sig(cx).borrow_mut().dark = Some(true),
                Some("light") => sig(cx).borrow_mut().dark = Some(false),
                _ => self.push("usage: theme <dark|light>", Tone::Dim),
            },
            "open" => match args.first().copied() {
                Some("files") => sig(cx).borrow_mut().launch.push(AppId::Files),
                Some("photos") => sig(cx).borrow_mut().launch.push(AppId::Photos),
                Some("kit") => sig(cx).borrow_mut().launch.push(AppId::Kit),
                _ => self.push("usage: open <files|photos|kit>", Tone::Dim),
            },
            "toast" => toast(cx, args.join(" ")),
            other => self.push(format!("smart: command not found: {other}"), Tone::Bad),
        }
    }

    /// Key input is forwarded here by the Shell so it works regardless of the
    /// GPUI focus path across embedded app views.
    pub fn handle_key(&mut self, ev: &KeyDownEvent, cx: &mut Context<Self>) {
        let k = ev.keystroke.key.as_str();
        match k {
            "up" => {
                if !self.hist.is_empty() {
                    let i = self.hist_i.map(|i| i.saturating_sub(1)).unwrap_or(self.hist.len() - 1);
                    self.hist_i = Some(i);
                    self.input = self.hist[i].clone();
                }
            }
            "down" => {
                if let Some(i) = self.hist_i {
                    if i + 1 < self.hist.len() {
                        self.hist_i = Some(i + 1);
                        self.input = self.hist[i + 1].clone();
                    } else {
                        self.hist_i = None;
                        self.input.clear();
                    }
                }
            }
            _ => {
                if ev.keystroke.modifiers.control && k == "l" {
                    self.lines.clear();
                } else if let Some(true) = edit_text(&mut self.input, ev) {
                    let line = std::mem::take(&mut self.input);
                    if !line.trim().is_empty() {
                        self.hist.push(line.clone());
                    }
                    self.hist_i = None;
                    self.run(&line, cx);
                }
            }
        }
        self.scroll.scroll_to_bottom();
        cx.notify();
    }
}

impl Render for TermView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = theme(cx);
        let focused = self.focus.is_focused(window);
        let col = |tone: Tone| match tone {
            Tone::Plain => if t.dark { hexc(0xd5deee) } else { hexc(0x26324a) },
            Tone::Prompt => t.ink1,
            Tone::Dim => t.ink4,
            Tone::Good => t.good,
            Tone::Bad => t.danger,
            Tone::Accent => t.accent,
        };
        let mut body = div().flex().flex_col().p(px(16.)).pt(px(8.)).gap(px(1.));
        for (s, tone) in &self.lines {
            let s = if s.is_empty() { " ".to_string() } else { s.clone() };
            body = body.child(div().whitespace_nowrap().text_color(col(*tone)).when(*tone == Tone::Prompt, |d| d.font_weight(FontWeight::BOLD)).child(s));
        }
        body = body.child(
            div()
                .flex()
                .items_center()
                .child(div().text_color(t.good).font_weight(FontWeight::BOLD).child(format!("{} \u{276f} ", self.prompt())))
                .child(div().text_color(t.ink1).whitespace_nowrap().child(self.input.clone()))
                .child(div().w(px(8.)).h(px(16.)).ml(px(1.)).bg(if focused { t.accent } else { alpha(t.accent, 0.35) })),
        );
        let bg = if t.dark { bg_grad(c(7, 11, 19, 0.82), c(5, 8, 14, 0.9)) } else { bg_grad(c(248, 250, 255, 0.78), c(238, 243, 252, 0.82)) };
        div()
            .id("term")
            .track_focus(&self.focus)
            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, window, cx| {
                window.focus(&this.focus, cx);
                cx.notify();
            }))
            .on_key_down(cx.listener(|this, ev: &KeyDownEvent, _, cx| this.handle_key(ev, cx)))
            .flex()
            .flex_col()
            .size_full()
            .bg(bg)
            .font_family("JetBrains Mono")
            .text_size(px(12.5))
            .line_height(px(19.))
            .child(div().h(px(48.)).flex_none().on_mouse_down(MouseButton::Left, |ev, _, cx| start_drag(cx, AppId::Terminal, ev.position)).flex().items_center().justify_center().text_size(px(12.)).font_family("Inter").font_weight(FontWeight::SEMIBOLD).text_color(t.ink3).child("smart@shell \u{2014} 96\u{d7}28"))
            .child(div().id("term-scroll").track_scroll(&self.scroll).flex_1().min_h(px(0.)).overflow_y_scroll().child(body))
    }
}
