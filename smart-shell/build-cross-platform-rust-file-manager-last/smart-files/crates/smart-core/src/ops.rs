//! Copy / move / delete jobs that work across any pair of providers, with progress + cancel.
use crate::model::Location;
use crate::vfs::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub use crate::registry::Registry;

#[derive(Clone, Debug)]
pub enum JobKind {
    Copy { srcs: Vec<Location>, dest: Location },
    Move { srcs: Vec<Location>, dest: Location },
    Trash { srcs: Vec<Location> },
    Delete { srcs: Vec<Location> },
    /// Duplicate in place (“name copy.ext”).
    Duplicate { srcs: Vec<Location> },
}

impl JobKind {
    pub fn srcs(&self) -> Vec<Location> {
        match self {
            JobKind::Copy { srcs, .. } | JobKind::Move { srcs, .. } | JobKind::Trash { srcs } | JobKind::Delete { srcs } | JobKind::Duplicate { srcs } => srcs.clone(),
        }
    }
    pub fn title(&self) -> String {
        let n = |s: &Vec<Location>| if s.len() == 1 { format!("“{}”", s[0].name()) } else { format!("{} items", s.len()) };
        match self {
            JobKind::Copy { srcs, dest } => format!("Copying {} to {}", n(srcs), dest.name()),
            JobKind::Move { srcs, dest } => format!("Moving {} to {}", n(srcs), dest.name()),
            JobKind::Trash { srcs } => format!("Moving {} to Trash", n(srcs)),
            JobKind::Delete { srcs } => format!("Deleting {}", n(srcs)),
            JobKind::Duplicate { srcs } => format!("Duplicating {}", n(srcs)),
        }
    }
}

#[derive(Clone, Debug)]
pub enum JobEvent {
    Started { id: u64, title: String },
    Progress { id: u64, done: u64, total: u64, current: String },
    Finished { id: u64, created: Vec<Location>, touched: Vec<Location> },
    Failed { id: u64, error: String },
}

pub struct JobCtx {
    pub id: u64,
    pub cancel: Arc<AtomicBool>,
    pub emit: Box<dyn Fn(JobEvent) + Send + Sync>,
}

fn vfs(r: &Registry, l: &Location) -> FsResult<Arc<dyn Vfs>> {
    r.get(&l.fs).ok_or_else(|| FsError::Other(format!("unknown file system {}", l.fs)))
}

/// “name.ext” → “name (1).ext” until it is free in `dir`.
pub fn unique_name(v: &dyn Vfs, dir: &Location, name: &str, suffix: Option<&str>) -> String {
    let (stem, ext) = match name.rfind('.') {
        Some(i) if i > 0 => (&name[..i], &name[i..]),
        _ => (name, ""),
    };
    if suffix.is_none() && !v.exists(&dir.join(name).path) {
        return name.to_string();
    }
    let base = match suffix {
        Some(s) => format!("{} {}", stem, s),
        None => stem.to_string(),
    };
    if suffix.is_some() {
        let c = format!("{}{}", base, ext);
        if !v.exists(&dir.join(&c).path) {
            return c;
        }
    }
    for i in 1..10_000 {
        let c = if suffix.is_some() { format!("{} {}{}", base, i + 1, ext) } else { format!("{} ({}){}", base, i, ext) };
        if !v.exists(&dir.join(&c).path) {
            return c;
        }
    }
    format!("{}-{}{}", stem, crate::model::now_secs(), ext)
}

fn measure(r: &Registry, l: &Location) -> u64 {
    let Ok(v) = vfs(r, l) else { return 0 };
    let Ok(e) = v.stat(&l.path) else { return 0 };
    if !e.is_dir {
        return e.size.max(1);
    }
    v.list(&l.path).map(|c| c.iter().map(|c| measure(r, &c.loc)).sum::<u64>()).unwrap_or(0).max(1)
}

struct Progress<'a> {
    ctx: &'a JobCtx,
    done: u64,
    total: u64,
    last: u64,
}
impl Progress<'_> {
    fn bump(&mut self, n: u64, cur: &str) {
        self.done += n;
        // coalesce events: at most ~200 per job
        if self.done - self.last >= (self.total / 200).max(1) || self.done >= self.total {
            self.last = self.done;
            (self.ctx.emit)(JobEvent::Progress { id: self.ctx.id, done: self.done, total: self.total, current: cur.to_string() });
        }
    }
}

fn copy_tree(r: &Registry, src: &Location, dst: &Location, p: &mut Progress) -> FsResult<()> {
    if p.ctx.cancel.load(Ordering::Relaxed) {
        return Err(FsError::Cancelled);
    }
    let (sv, dv) = (vfs(r, src)?, vfs(r, dst)?);
    let st = sv.stat(&src.path)?;
    if st.is_dir {
        if dst.fs == src.fs && (dst.path == src.path || dst.starts_with(src)) {
            return Err(FsError::Other("cannot copy a folder into itself".into()));
        }
        dv.mkdir(&dst.path)?;
        for c in sv.list(&src.path)? {
            copy_tree(r, &c.loc, &dst.join(&c.name), p)?;
        }
        return Ok(());
    }
    let mut rd = sv.open_read(&src.path)?;
    let mut wr = dv.create_write(&dst.path)?;
    let mut buf = vec![0u8; 1 << 18];
    loop {
        if p.ctx.cancel.load(Ordering::Relaxed) {
            drop(wr);
            let _ = dv.remove(&dst.path, false);
            return Err(FsError::Cancelled);
        }
        let n = rd.read(&mut buf)?;
        if n == 0 {
            break;
        }
        wr.write_all(&buf[..n])?;
        p.bump(n as u64, &st.name);
    }
    wr.flush()?;
    if st.paced {
        let _ = dv.set_size_hint(&dst.path, Some(st.size));
    }
    if st.size == 0 {
        p.bump(1, &st.name);
    }
    Ok(())
}

use std::io::{Read, Write};

/// Run a job to completion without emitting progress (used after a paced transfer animation).
pub fn run_sync(r: &Registry, job: &JobKind) -> FsResult<(Vec<Location>, Vec<Location>)> {
    let cancel = Arc::new(AtomicBool::new(false));
    let ctx = JobCtx { id: 0, cancel, emit: Box::new(|_| {}) };
    run_inner(r, job, &ctx)
}

pub fn run(r: &Registry, job: JobKind, ctx: &JobCtx) {
    (ctx.emit)(JobEvent::Started { id: ctx.id, title: job.title() });
    let res = run_inner(r, &job, ctx);
    match res {
        Ok((created, touched)) => (ctx.emit)(JobEvent::Finished { id: ctx.id, created, touched }),
        Err(e) => (ctx.emit)(JobEvent::Failed { id: ctx.id, error: e.to_string() }),
    }
}

fn run_inner(r: &Registry, job: &JobKind, ctx: &JobCtx) -> FsResult<(Vec<Location>, Vec<Location>)> {
    let mut created = vec![];
    let mut touched = vec![];
    match job {
        JobKind::Copy { srcs, dest } | JobKind::Move { srcs, dest } => {
            let moving = matches!(job, JobKind::Move { .. });
            let total: u64 = srcs.iter().map(|s| measure(r, s)).sum::<u64>().max(1);
            let mut p = Progress { ctx, done: 0, total, last: 0 };
            let dv = vfs(r, dest)?;
            touched.push(dest.clone());
            for s in srcs {
                let name = unique_name(dv.as_ref(), dest, &s.name(), None);
                let target = dest.join(&name);
                if moving && s.fs == dest.fs {
                    if s.parent().as_ref() == Some(dest) {
                        continue;
                    }
                    if dest.starts_with(s) {
                        return Err(FsError::Other("cannot move a folder into itself".into()));
                    }
                    match dv.rename(&s.path, &target.path) {
                        Ok(()) => {
                            p.bump(measure(r, &target), &name);
                            created.push(target);
                            touched.extend(s.parent());
                            continue;
                        }
                        Err(FsError::Cancelled) => return Err(FsError::Cancelled),
                        Err(_) => {} // cross-device: fall through to copy + delete
                    }
                }
                copy_tree(r, s, &target, &mut p)?;
                created.push(target);
                if moving {
                    vfs(r, s)?.remove(&s.path, false)?;
                    touched.extend(s.parent());
                }
            }
        }
        JobKind::Duplicate { srcs } => {
            let total: u64 = srcs.iter().map(|s| measure(r, s)).sum::<u64>().max(1);
            let mut p = Progress { ctx, done: 0, total, last: 0 };
            for s in srcs {
                let Some(dir) = s.parent() else { continue };
                let v = vfs(r, s)?;
                let name = unique_name(v.as_ref(), &dir, &s.name(), Some("copy"));
                let target = dir.join(&name);
                copy_tree(r, s, &target, &mut p)?;
                created.push(target);
                touched.push(dir);
            }
        }
        JobKind::Trash { srcs } | JobKind::Delete { srcs } => {
            let trash = matches!(job, JobKind::Trash { .. });
            for (i, s) in srcs.iter().enumerate() {
                if ctx.cancel.load(Ordering::Relaxed) {
                    return Err(FsError::Cancelled);
                }
                let v = vfs(r, s)?;
                v.remove(&s.path, trash && v.supports_trash())?;
                touched.extend(s.parent());
                (ctx.emit)(JobEvent::Progress { id: ctx.id, done: i as u64 + 1, total: srcs.len() as u64, current: s.name() });
            }
        }
    }
    touched.sort();
    touched.dedup();
    Ok((created, touched))
}
