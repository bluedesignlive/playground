//! User-owned metadata: tags, recents, pinned places and preferences. Serialised as JSON.
use crate::model::{now_secs, Location, SortKey};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

pub type SharedLibrary = Arc<RwLock<Library>>;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TagDef {
    pub id: String,
    pub name: String,
    pub color: String, // #rrggbb
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Recent {
    pub loc: Location,
    pub at: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub dark: bool,
    pub grid: bool,
    pub sort: SortKey,
    pub ascending: bool,
    pub show_hidden: bool,
    pub folders_first: bool,
    pub sidebar_collapsed: bool,
    pub wallpaper: usize,
    pub search_contents: bool,
    pub thumb_scale: f32,
    pub show_extensions: bool,
    pub confirm_delete: bool,
    pub copy_animation_secs: f32,
    pub reduce_motion: bool,
    pub autoplay: bool,
    pub accent: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            dark: false,
            grid: true,
            sort: SortKey::Name,
            ascending: true,
            show_hidden: false,
            folders_first: true,
            sidebar_collapsed: false,
            wallpaper: 0,
            search_contents: false,
            thumb_scale: 1.0,
            show_extensions: true,
            confirm_delete: true,
            copy_animation_secs: 5.0,
            reduce_motion: false,
            autoplay: false,
            accent: "#2f6feb".into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Library {
    pub tags: Vec<TagDef>,
    pub assignments: BTreeMap<String, Vec<String>>,
    pub recents: Vec<Recent>,
    pub pins: Vec<Location>,
    pub settings: Settings,
}

impl Default for Library {
    fn default() -> Self {
        Self {
            tags: vec![
                TagDef { id: "design".into(), name: "Design".into(), color: "#2f6feb".into() },
                TagDef { id: "amapiano".into(), name: "Amapiano".into(), color: "#c9861a".into() },
                TagDef { id: "archive".into(), name: "Archive".into(), color: "#2b9b6d".into() },
            ],
            assignments: BTreeMap::new(),
            recents: vec![],
            pins: vec![],
            settings: Settings::default(),
        }
    }
}

fn key(l: &Location) -> String {
    format!("{}:{}", l.fs, l.path)
}

impl Library {
    pub fn shared(self) -> SharedLibrary {
        Arc::new(RwLock::new(self))
    }
    pub fn tags_of(&self, l: &Location) -> Vec<&TagDef> {
        self.assignments
            .get(&key(l))
            .map(|ids| ids.iter().filter_map(|i| self.tags.iter().find(|t| &t.id == i)).collect())
            .unwrap_or_default()
    }
    pub fn tag_ids_of(&self, l: &Location) -> Vec<String> {
        self.assignments.get(&key(l)).cloned().unwrap_or_default()
    }
    pub fn has_tag(&self, l: &Location, id: &str) -> bool {
        self.assignments.get(&key(l)).is_some_and(|v| v.iter().any(|t| t == id))
    }
    pub fn toggle_tag(&mut self, l: &Location, id: &str) {
        let e = self.assignments.entry(key(l)).or_default();
        if let Some(i) = e.iter().position(|t| t == id) {
            e.remove(i);
        } else {
            e.push(id.to_string());
        }
        if e.is_empty() {
            self.assignments.remove(&key(l));
        }
    }
    pub fn tag_count(&self, id: &str) -> usize {
        self.assignments.values().filter(|v| v.iter().any(|t| t == id)).count()
    }
    pub fn tagged(&self, id: &str) -> Vec<Location> {
        self.assignments
            .iter()
            .filter(|(_, v)| v.iter().any(|t| t == id))
            .filter_map(|(k, _)| k.split_once(':').map(|(fs, p)| Location::new(fs, p)))
            .collect()
    }
    pub fn add_tag(&mut self, name: &str, color: &str) -> String {
        let id = name.trim().to_lowercase().replace(' ', "-");
        if !self.tags.iter().any(|t| t.id == id) {
            self.tags.push(TagDef { id: id.clone(), name: name.trim().to_string(), color: color.to_string() });
        }
        id
    }
    /// Keep tags attached when files move or are renamed.
    pub fn rekey(&mut self, from: &Location, to: &Location) {
        let prefix = format!("{}/", key(from));
        let moves: Vec<(String, String)> = self
            .assignments
            .keys()
            .filter(|k| **k == key(from) || k.starts_with(&prefix))
            .map(|k| (k.clone(), format!("{}{}", key(to), &k[key(from).len()..])))
            .collect();
        for (a, b) in moves {
            if let Some(v) = self.assignments.remove(&a) {
                self.assignments.insert(b, v);
            }
        }
    }
    pub fn touch_recent(&mut self, l: &Location) {
        self.recents.retain(|r| &r.loc != l);
        self.recents.insert(0, Recent { loc: l.clone(), at: now_secs() });
        self.recents.truncate(60);
    }
    pub fn is_pinned(&self, l: &Location) -> bool {
        self.pins.contains(l)
    }
    pub fn toggle_pin(&mut self, l: &Location) {
        if let Some(i) = self.pins.iter().position(|p| p == l) {
            self.pins.remove(i);
        } else {
            self.pins.push(l.clone());
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }
    pub fn from_json(s: &str) -> Library {
        serde_json::from_str(s).unwrap_or_default()
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn path() -> Option<std::path::PathBuf> {
        dirs::config_dir().map(|d| d.join("smart-files").join("library.json"))
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load() -> Library {
        Self::path().and_then(|p| std::fs::read_to_string(p).ok()).map(|s| Self::from_json(&s)).unwrap_or_default()
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn save(&self) {
        if let Some(p) = Self::path() {
            let _ = std::fs::create_dir_all(p.parent().unwrap());
            let _ = std::fs::write(p, self.to_json());
        }
    }
}
