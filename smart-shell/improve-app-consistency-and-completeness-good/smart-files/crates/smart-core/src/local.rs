//! The real disk. Native only.
use crate::model::{Entry, Location};
use crate::vfs::*;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub struct LocalVfs;

fn unix(t: std::io::Result<std::time::SystemTime>) -> Option<i64> {
    t.ok()?.duration_since(std::time::UNIX_EPOCH).ok().map(|d| d.as_secs() as i64)
}

fn to_pathbuf(p: &str) -> PathBuf {
    #[cfg(windows)]
    {
        let t = p.trim_start_matches('/');
        return PathBuf::from(if t.len() == 2 { format!("{}/", t) } else { t.to_string() });
    }
    #[cfg(not(windows))]
    PathBuf::from(p)
}

pub fn path_to_loc(p: &Path) -> Location {
    let s = p.to_string_lossy().replace('\\', "/");
    #[cfg(windows)]
    let s = format!("/{}", s);
    Location::file(s)
}

fn entry_from(path: &Path, md: &std::fs::Metadata, symlink: bool) -> Entry {
    let mut e = Entry::new(path_to_loc(path), md.is_dir(), if md.is_dir() { 0 } else { md.len() }, unix(md.modified()));
    e.symlink = symlink;
    e.readonly = md.permissions().readonly();
    e
}

impl Vfs for LocalVfs {
    fn id(&self) -> &str {
        "file"
    }
    fn list(&self, path: &str) -> FsResult<Vec<Entry>> {
        let rd = std::fs::read_dir(to_pathbuf(path))?;
        let mut out = Vec::new();
        for ent in rd.flatten() {
            let p = ent.path();
            let sym = ent.file_type().map(|t| t.is_symlink()).unwrap_or(false);
            // follow symlinks for metadata; broken links fall back to the link itself
            let md = std::fs::metadata(&p).or_else(|_| ent.metadata());
            if let Ok(md) = md {
                out.push(entry_from(&p, &md, sym));
            }
        }
        Ok(out)
    }
    fn stat(&self, path: &str) -> FsResult<Entry> {
        let p = to_pathbuf(path);
        let sym = std::fs::symlink_metadata(&p).map(|m| m.file_type().is_symlink()).unwrap_or(false);
        Ok(entry_from(&p, &std::fs::metadata(&p)?, sym))
    }
    fn open_read(&self, path: &str) -> FsResult<Box<dyn Read + Send>> {
        Ok(Box::new(std::fs::File::open(to_pathbuf(path))?))
    }
    fn create_write(&self, path: &str) -> FsResult<Box<dyn Write + Send>> {
        Ok(Box::new(std::io::BufWriter::with_capacity(1 << 18, std::fs::File::create(to_pathbuf(path))?)))
    }
    fn mkdir(&self, path: &str) -> FsResult<()> {
        Ok(std::fs::create_dir(to_pathbuf(path))?)
    }
    fn rename(&self, from: &str, to: &str) -> FsResult<()> {
        if to_pathbuf(to).exists() {
            return Err(FsError::Exists);
        }
        Ok(std::fs::rename(to_pathbuf(from), to_pathbuf(to))?)
    }
    fn remove(&self, path: &str, to_trash: bool) -> FsResult<()> {
        let p = to_pathbuf(path);
        if to_trash {
            return trash::delete(&p).map_err(|e| FsError::Other(e.to_string()));
        }
        let md = std::fs::symlink_metadata(&p)?;
        if md.is_dir() { Ok(std::fs::remove_dir_all(&p)?) } else { Ok(std::fs::remove_file(&p)?) }
    }
    fn real_path(&self, path: &str) -> Option<PathBuf> {
        Some(to_pathbuf(path))
    }
    fn supports_trash(&self) -> bool {
        true
    }
    fn capacity(&self, path: &str) -> Option<Capacity> {
        let disks: Vec<sysinfo::Disk> = sysinfo::Disks::new_with_refreshed_list().map(Vec::from).unwrap_or_default();
        let p = to_pathbuf(path);
        disks
            .iter()
            .filter(|d| p.starts_with(d.mount_point()))
            .max_by_key(|d| d.mount_point().as_os_str().len())
            .map(|d| Capacity { total: d.total_space(), free: d.available_space() })
    }
}
