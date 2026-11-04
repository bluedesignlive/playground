//! The storage abstraction every provider implements (local disk, memory, phone, cloud…).
use crate::model::{Entry, Location};
use std::io::{Read, Write};
use std::path::PathBuf;

#[derive(Debug, Clone, thiserror::Error, PartialEq)]
pub enum FsError {
    #[error("not found")]
    NotFound,
    #[error("permission denied")]
    PermissionDenied,
    #[error("already exists")]
    Exists,
    #[error("not supported here")]
    Unsupported,
    #[error("cancelled")]
    Cancelled,
    #[error("{0}")]
    Other(String),
}

pub type FsResult<T> = Result<T, FsError>;

impl From<std::io::Error> for FsError {
    fn from(e: std::io::Error) -> Self {
        use std::io::ErrorKind::*;
        match e.kind() {
            NotFound => FsError::NotFound,
            PermissionDenied => FsError::PermissionDenied,
            AlreadyExists => FsError::Exists,
            _ => FsError::Other(e.to_string()),
        }
    }
}

pub struct Capacity {
    pub total: u64,
    pub free: u64,
}

/// Synchronous, thread-safe file-system provider. Paths are absolute, `/`-separated, per-provider.
pub trait Vfs: Send + Sync {
    fn id(&self) -> &str;
    fn list(&self, path: &str) -> FsResult<Vec<Entry>>;
    fn stat(&self, path: &str) -> FsResult<Entry>;
    fn open_read(&self, path: &str) -> FsResult<Box<dyn Read + Send>>;
    fn create_write(&self, path: &str) -> FsResult<Box<dyn Write + Send>>;
    fn mkdir(&self, path: &str) -> FsResult<()>;
    fn rename(&self, from: &str, to: &str) -> FsResult<()>;
    /// `to_trash` moves to the platform trash where one exists, otherwise deletes.
    fn remove(&self, path: &str, to_trash: bool) -> FsResult<()>;

    /// Real on-disk path when the provider is backed by one (enables “Open with…”).
    fn real_path(&self, _path: &str) -> Option<PathBuf> {
        None
    }
    fn capacity(&self, _path: &str) -> Option<Capacity> {
        None
    }
    fn supports_trash(&self) -> bool {
        false
    }
    fn is_writable(&self) -> bool {
        true
    }

    /// Providers that support “virtual” sizes (the web demo) can preserve them when copying.
    fn set_size_hint(&self, _path: &str, _hint: Option<u64>) -> FsResult<()> {
        Err(FsError::Unsupported)
    }

    fn read(&self, path: &str, limit: usize) -> FsResult<Vec<u8>> {
        let mut r = self.open_read(path)?;
        let mut buf = Vec::new();
        r.by_ref().take(limit as u64).read_to_end(&mut buf).map_err(FsError::from)?;
        Ok(buf)
    }
    fn write(&self, path: &str, data: &[u8]) -> FsResult<()> {
        let mut w = self.create_write(path)?;
        w.write_all(data)?;
        w.flush()?;
        Ok(())
    }
    fn exists(&self, path: &str) -> bool {
        self.stat(path).is_ok()
    }
    fn loc(&self, path: &str) -> Location {
        Location::new(self.id(), path)
    }
}
