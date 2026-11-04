//! On-device “smart” helpers: duplicate finder, organise-by-kind planner, stale-file finder.
use crate::model::*;
use crate::registry::Registry;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

fn fingerprint(reg: &Registry, e: &Entry) -> Option<u64> {
    let v = reg.get(&e.loc.fs)?;
    let data = v.read(&e.loc.path, 256 * 1024).ok()?;
    let mut h = std::collections::hash_map::DefaultHasher::new();
    data.hash(&mut h);
    e.size.hash(&mut h);
    Some(h.finish())
}

/// Groups of files in `entries` with identical size and leading content.
pub fn find_duplicates(reg: &Registry, entries: &[Entry]) -> Vec<Vec<Entry>> {
    let mut by_size: HashMap<u64, Vec<&Entry>> = HashMap::new();
    for e in entries.iter().filter(|e| !e.is_dir && e.size > 0) {
        by_size.entry(e.size).or_default().push(e);
    }
    let mut out = vec![];
    for (_, group) in by_size.into_iter().filter(|(_, g)| g.len() > 1) {
        let mut by_fp: HashMap<u64, Vec<Entry>> = HashMap::new();
        for e in group {
            if let Some(f) = fingerprint(reg, e) {
                by_fp.entry(f).or_default().push(e.clone());
            }
        }
        out.extend(by_fp.into_values().filter(|g| g.len() > 1));
    }
    out.sort_by_key(|g| std::cmp::Reverse(g[0].size * g.len() as u64));
    out
}

pub fn group_name(k: Kind) -> &'static str {
    match k {
        Kind::Image | Kind::Vector => "Images",
        Kind::Audio => "Audio",
        Kind::Video => "Video",
        Kind::Pdf | Kind::Document | Kind::Text => "Documents",
        Kind::Code => "Code",
        Kind::Archive => "Archives",
        Kind::App => "Apps",
        Kind::Project => "Projects",
        _ => "Other",
    }
}

/// Plan: move every loose file into a folder named after its kind. Returns (file, target folder name).
pub fn organise_by_kind(entries: &[Entry]) -> Vec<(Location, String)> {
    entries.iter().filter(|e| !e.is_dir && !e.hidden).map(|e| (e.loc.clone(), group_name(e.kind).to_string())).collect()
}

pub fn older_than(entries: &[Entry], days: i64) -> Vec<Entry> {
    let cutoff = now_secs() - days * 86400;
    let mut v: Vec<Entry> = entries.iter().filter(|e| !e.is_dir && e.modified.is_some_and(|m| m < cutoff)).cloned().collect();
    v.sort_by_key(|e| e.modified);
    v
}
