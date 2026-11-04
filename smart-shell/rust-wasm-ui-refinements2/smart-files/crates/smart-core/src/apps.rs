//! “Open”, “Open With…” and default-app handling. Linux reads .desktop entries; macOS/Windows use the shell.
use std::path::Path;

#[derive(Clone, Debug, PartialEq)]
pub struct AppInfo {
    pub id: String,
    pub name: String,
    pub exec: String,
    pub icon: &'static str,
    pub is_default: bool,
}

pub trait AppProvider: Send + Sync {
    fn apps_for(&self, file_name: &str) -> Vec<AppInfo>;
    fn open_default(&self, file_name: &str, real: Option<&Path>) -> Result<String, String>;
    fn open_with(&self, app: &AppInfo, file_name: &str, real: Option<&Path>) -> Result<String, String>;
    fn set_default(&self, app: &AppInfo, file_name: &str) -> Result<(), String>;
}

pub fn mime_of(name: &str) -> String {
    mime_guess::from_path(name).first_or_octet_stream().to_string()
}

#[cfg(not(target_arch = "wasm32"))]
pub use native::SystemApps;

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use super::*;
    use std::process::{Command, Stdio};
    use std::sync::OnceLock;

    #[derive(Clone)]
    struct Desktop {
        id: String,
        name: String,
        exec: String,
        mimes: Vec<String>,
    }

    fn scan() -> Vec<Desktop> {
        let mut dirs: Vec<std::path::PathBuf> = vec!["/usr/share/applications".into(), "/usr/local/share/applications".into(), "/var/lib/flatpak/exports/share/applications".into()];
        if let Some(d) = dirs::data_dir() {
            dirs.push(d.join("applications"));
            dirs.push(d.join("flatpak/exports/share/applications"));
        }
        let mut out = vec![];
        for d in dirs {
            let Ok(rd) = std::fs::read_dir(&d) else { continue };
            for e in rd.flatten() {
                let p = e.path();
                if p.extension().and_then(|x| x.to_str()) != Some("desktop") {
                    continue;
                }
                let Ok(txt) = std::fs::read_to_string(&p) else { continue };
                let (mut name, mut exec, mut mimes, mut hidden, mut in_entry) = (String::new(), String::new(), vec![], false, false);
                for l in txt.lines() {
                    let l = l.trim();
                    if l.starts_with('[') {
                        in_entry = l == "[Desktop Entry]";
                        continue;
                    }
                    if !in_entry {
                        continue;
                    }
                    if let Some(v) = l.strip_prefix("Name=") {
                        name = v.into();
                    } else if let Some(v) = l.strip_prefix("Exec=") {
                        exec = v.into();
                    } else if let Some(v) = l.strip_prefix("MimeType=") {
                        mimes = v.split(';').filter(|s| !s.is_empty()).map(String::from).collect();
                    } else if l == "NoDisplay=true" || l == "Hidden=true" {
                        hidden = true;
                    }
                }
                if !hidden && !name.is_empty() && !exec.is_empty() {
                    out.push(Desktop { id: p.file_name().unwrap().to_string_lossy().into(), name, exec, mimes });
                }
            }
        }
        out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        out.dedup_by(|a, b| a.id == b.id);
        out
    }

    fn all() -> &'static Vec<Desktop> {
        static A: OnceLock<Vec<Desktop>> = OnceLock::new();
        A.get_or_init(scan)
    }

    fn split_exec(exec: &str, path: &str) -> Vec<String> {
        let (mut parts, mut cur, mut q) = (vec![], String::new(), false);
        for c in exec.chars() {
            match c {
                '"' => q = !q,
                c if c.is_whitespace() && !q => {
                    if !cur.is_empty() {
                        parts.push(std::mem::take(&mut cur));
                    }
                }
                c => cur.push(c),
            }
        }
        if !cur.is_empty() {
            parts.push(cur);
        }
        let mut used = false;
        let mut out: Vec<String> = parts
            .into_iter()
            .filter_map(|p| match p.as_str() {
                "%f" | "%F" | "%u" | "%U" => {
                    used = true;
                    Some(path.to_string())
                }
                "%i" | "%c" | "%k" => None,
                _ => Some(p.replace("%%", "%")),
            })
            .collect();
        if !used {
            out.push(path.to_string());
        }
        out
    }

    fn detach(mut c: Command) -> Result<(), String> {
        c.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
        c.spawn().map(|_| ()).map_err(|e| e.to_string())
    }

    pub struct SystemApps;

    impl AppProvider for SystemApps {
        fn apps_for(&self, file_name: &str) -> Vec<AppInfo> {
            if cfg!(not(target_os = "linux")) {
                return vec![];
            }
            let mime = mime_of(file_name);
            let major = format!("{}/", mime.split('/').next().unwrap_or(""));
            let default = Command::new("xdg-mime").args(["query", "default", &mime]).output().ok().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default();
            let mut v: Vec<AppInfo> = all()
                .iter()
                .filter(|d| d.mimes.iter().any(|m| *m == mime || (m.ends_with("/*") && m.starts_with(&major))))
                .map(|d| AppInfo { id: d.id.clone(), name: d.name.clone(), exec: d.exec.clone(), icon: "widget", is_default: d.id == default })
                .collect();
            v.sort_by_key(|a| !a.is_default);
            v
        }
        fn open_default(&self, file_name: &str, real: Option<&Path>) -> Result<String, String> {
            let p = real.ok_or("This item is not on a local disk")?;
            let mut c = if cfg!(target_os = "macos") {
                let mut c = Command::new("open");
                c.arg(p);
                c
            } else if cfg!(windows) {
                let mut c = Command::new("cmd");
                c.args(["/C", "start", ""]).arg(p);
                c
            } else {
                let mut c = Command::new("xdg-open");
                c.arg(p);
                c
            };
            c.stdin(Stdio::null());
            detach(c).map_err(|e| format!("Couldn’t open: {}", e))?;
            Ok(format!("Opening {}", file_name))
        }
        fn open_with(&self, app: &AppInfo, file_name: &str, real: Option<&Path>) -> Result<String, String> {
            let p = real.ok_or("This item is not on a local disk")?.to_string_lossy().to_string();
            if cfg!(target_os = "macos") {
                let mut c = Command::new("open");
                c.args(["-a", &app.name, &p]);
                detach(c)?;
            } else {
                let parts = split_exec(&app.exec, &p);
                let mut c = Command::new(&parts[0]);
                c.args(&parts[1..]);
                detach(c)?;
            }
            Ok(format!("Opening {} with {}", file_name, app.name))
        }
        fn set_default(&self, app: &AppInfo, file_name: &str) -> Result<(), String> {
            let status = Command::new("xdg-mime").args(["default", &app.id, &mime_of(file_name)]).status().map_err(|e| e.to_string())?;
            if status.success() { Ok(()) } else { Err("xdg-mime failed".into()) }
        }
    }
}

/// Browser / demo provider: pretends to launch, the frontend shows its built-in viewers.
pub struct DemoApps;
impl AppProvider for DemoApps {
    fn apps_for(&self, file_name: &str) -> Vec<AppInfo> {
        use crate::model::{ext_of, Kind};
        let k = Kind::from_ext(&ext_of(file_name));
        let mk = |id: &str, name: &str, icon, d| AppInfo { id: id.into(), name: name.into(), exec: id.into(), icon, is_default: d };
        let mut v = vec![];
        match k {
            Kind::Image | Kind::Vector => {
                v.push(mk("viewer", "Smart Image Viewer", "gallery-wide", true));
                v.push(mk("design", "Smart Design Studio", "stars-minimalistic", false));
            }
            Kind::Audio => {
                v.push(mk("player", "Smart Player", "music-note", true));
                v.push(mk("daw", "Smart DAW", "music-notes", false));
            }
            Kind::Text | Kind::Code | Kind::Document | Kind::Pdf => {
                v.push(mk("reader", "Smart Reader", "document-text", true));
                v.push(mk("editor", "Smart Text Editor", "text", false));
            }
            _ => v.push(mk("default", "Default app", "global", true)),
        }
        v
    }
    fn open_default(&self, n: &str, _: Option<&Path>) -> Result<String, String> {
        Ok(format!("Opening {} in the default app", n))
    }
    fn open_with(&self, a: &AppInfo, n: &str, _: Option<&Path>) -> Result<String, String> {
        Ok(format!("Opening {} with {}", n, a.name))
    }
    fn set_default(&self, _: &AppInfo, _: &str) -> Result<(), String> {
        Ok(())
    }
}
