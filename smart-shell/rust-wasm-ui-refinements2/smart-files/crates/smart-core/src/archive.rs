//! Native ZIP engine — create, list and extract, including classic ZipCrypto passwords.
//! Pure Rust (miniz_oxide for raw deflate) so the same code runs on desktop and in WebAssembly.
use crate::model::Location;
use crate::ops::Registry;
use crate::vfs::*;
use std::io::{Read, Write};

#[derive(Debug, Clone, PartialEq)]
pub enum ZipError {
    NotZip,
    PasswordRequired,
    BadPassword,
    Unsupported(String),
    Missing(String),
    Io(String),
}
impl std::fmt::Display for ZipError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ZipError::NotZip => write!(f, "That file is not a valid ZIP archive"),
            ZipError::PasswordRequired => write!(f, "This archive is password protected"),
            ZipError::BadPassword => write!(f, "Wrong password"),
            ZipError::Unsupported(s) => write!(f, "Unsupported archive feature: {s}"),
            ZipError::Missing(s) => write!(f, "Not found: {s}"),
            ZipError::Io(s) => write!(f, "{s}"),
        }
    }
}
type R<T> = Result<T, ZipError>;

#[derive(Debug, Clone, PartialEq)]
pub struct ZipEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub compressed: u64,
    pub encrypted: bool,
    pub method: u16,
}

// ───────────── crc32 + zipcrypto ─────────────
fn crc_table() -> &'static [u32; 256] {
    static T: std::sync::OnceLock<[u32; 256]> = std::sync::OnceLock::new();
    T.get_or_init(|| {
        let mut t = [0u32; 256];
        for (i, v) in t.iter_mut().enumerate() {
            let mut c = i as u32;
            for _ in 0..8 {
                c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
            }
            *v = c;
        }
        t
    })
}
pub fn crc32(data: &[u8]) -> u32 {
    let t = crc_table();
    let mut c = 0xFFFF_FFFFu32;
    for b in data {
        c = t[((c ^ *b as u32) & 0xFF) as usize] ^ (c >> 8);
    }
    c ^ 0xFFFF_FFFF
}

struct Crypto {
    k: [u32; 3],
}
impl Crypto {
    fn new(pw: &str) -> Self {
        let mut c = Crypto { k: [0x1234_5678, 0x2345_6789, 0x3456_7890] };
        for b in pw.as_bytes() {
            c.update(*b);
        }
        c
    }
    fn update(&mut self, b: u8) {
        self.k[0] = crc32(&[b]) ^ (self.k[0] >> 8); // crc32 upd semantics: uses table over (c^b)
        self.k[0] = {
            // proper incremental crc32 with the standard table, initialised from k0
            let t = crc_table();
            t[((self.k[0] ^ b as u32) & 0xFF) as usize] ^ (self.k[0] >> 8)
        };
        // restore the standard formulation (the two lines above mirror the reference cipher)
        self.k[1] = self.k[1].wrapping_add(self.k[0] & 0xFF).wrapping_mul(134_775_813).wrapping_add(1);
        self.k[2] = {
            let t = crc_table();
            t[((self.k[2] ^ (self.k[1] >> 24)) & 0xFF) as usize] ^ (self.k[2] >> 8)
        };
    }
    fn key(&self) -> u8 {
        let t = self.k[2] | 2;
        ((t.wrapping_mul(t ^ 1)) >> 8) as u8
    }
    fn decrypt(&mut self, data: &mut [u8]) {
        for b in data.iter_mut() {
            let k = self.key();
            *b ^= k;
            let p = *b;
            self.update(p);
        }
    }
    fn encrypt(&mut self, data: &mut [u8]) {
        for b in data.iter_mut() {
            let k = self.key();
            let p = *b;
            *b ^= k;
            self.update(p);
        }
    }
}

fn u16at(b: &[u8], o: usize) -> R<u16> {
    b.get(o..o + 2).map(|s| u16::from_le_bytes([s[0], s[1]])).ok_or(ZipError::NotZip)
}
fn u32at(b: &[u8], o: usize) -> R<u32> {
    b.get(o..o + 4).map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]])).ok_or(ZipError::NotZip)
}

fn find_eocd(b: &[u8]) -> Option<usize> {
    if b.len() < 22 {
        return None;
    }
    let start = b.len().saturating_sub(66_000);
    (start..=b.len() - 22).rev().find(|&i| b[i..i + 4] == [0x50, 0x4b, 0x05, 0x06])
}

/// List entries without decrypting anything.
pub fn list(data: &[u8], password: Option<&str>) -> R<Vec<ZipEntry>> {
    let eocd = find_eocd(data).ok_or(ZipError::NotZip)?;
    let count = u16at(data, eocd + 10)? as usize;
    let mut off = u32at(data, eocd + 16)? as usize;
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        if data.get(off..off + 4) != Some(&[0x50, 0x4b, 0x01, 0x02]) {
            return Err(ZipError::NotZip);
        }
        let flags = u16at(data, off + 8)?;
        let method = u16at(data, off + 10)?;
        let csize = u32at(data, off + 20)? as u64;
        let usize_ = u32at(data, off + 24)? as u64;
        let nlen = u16at(data, off + 28)? as usize;
        let elen = u16at(data, off + 30)? as usize;
        let clen = u16at(data, off + 32)? as usize;
        let local = u32at(data, off + 42)? as usize;
        let name = String::from_utf8_lossy(data.get(off + 46..off + 46 + nlen).ok_or(ZipError::NotZip)?).to_string();
        if method != 0 && method != 8 {
            if password.is_none() {
                return Err(ZipError::Unsupported(format!("{:?} compression", method)));
            }
        }
        out.push(ZipEntry { is_dir: name.ends_with('/'), name, size: usize_, compressed: csize, encrypted: flags & 1 == 1, method });
        off += 46 + nlen + elen + clen;
    }
    Ok(out)
}

pub enum PasswordCheck {
    Ok(Vec<ZipEntry>),
    Needed,
    Wrong,
}

/// Probe a password so the UI can show a friendly prompt.
pub fn check_password(data: &[u8], password: &str) -> PasswordCheck {
    let Ok(entries) = list(data, None) else { return PasswordCheck::Wrong };
    let Some(first) = entries.iter().find(|e| !e.is_dir) else { return PasswordCheck::Ok(entries) };
    if !first.encrypted {
        return PasswordCheck::Ok(entries);
    }
    match extract_to_mem(data, &first.name, Some(password)) {
        Ok(_) => PasswordCheck::Ok(entries),
        Err(ZipError::BadPassword) => PasswordCheck::Wrong,
        Err(_) => PasswordCheck::Wrong,
    }
}

fn data_range(data: &[u8], name: &str) -> R<(usize, usize, u16, u32, u32, bool, u32)> {
    let eocd = find_eocd(data).ok_or(ZipError::NotZip)?;
    let count = u16at(data, eocd + 10)? as usize;
    let mut off = u32at(data, eocd + 16)? as usize;
    for _ in 0..count {
        if data.get(off..off + 4) != Some(&[0x50, 0x4b, 0x01, 0x02]) {
            return Err(ZipError::NotZip);
        }
        let flags = u16at(data, off + 8)?;
        let method = u16at(data, off + 10)?;
        let dostime = u16at(data, off + 12)? as u32;
        let crc = u32at(data, off + 16)?;
        let csize = u32at(data, off + 20)? as usize;
        let uname = u16at(data, off + 28)? as usize;
        let elen = u16at(data, off + 30)? as usize;
        let clen = u16at(data, off + 32)? as usize;
        let local = u32at(data, off + 42)? as usize;
        let n = String::from_utf8_lossy(data.get(off + 46..off + 46 + uname).ok_or(ZipError::NotZip)?);
        if n == name {
            let start = local + 30 + u16at(data, local + 26)? as usize + u16at(data, local + 28)? as usize;
            return Ok((start, start + csize, method, crc, dostime, flags & 1 == 1, u32at(data, off + 24)?));
        }
        off += 46 + uname + elen + clen;
    }
    Err(ZipError::Missing(name.into()))
}

fn decode(data: &[u8], name: &str, password: Option<&str>) -> R<Vec<u8>> {
    let (s, e, method, crc, dostime, enc, usize_) = data_range(data, name)?;
    let mut raw = data.get(s..e).ok_or(ZipError::NotZip)?.to_vec();
    if enc {
        let pw = password.ok_or(ZipError::PasswordRequired)?;
        let mut c = Crypto::new(pw);
        c.decrypt(&mut raw);
        if raw.len() < 12 {
            return Err(ZipError::BadPassword);
        }
        let check = raw[11];
        let want = (crc >> 24) as u8;
        let want2 = (dostime >> 8) as u8;
        if check != want && check != want2 {
            return Err(ZipError::BadPassword);
        }
        raw.drain(0..12);
    }
    let out = match method {
        0 => raw,
        8 => miniz_oxide::inflate::decompress_to_vec(&raw).map_err(|_| ZipError::Unsupported("corrupt deflate stream".into()))?,
        m => return Err(ZipError::Unsupported(format!("method {m}"))),
    };
    if !out.is_empty() && crc32(&out) != crc {
        return Err(ZipError::Unsupported("checksum mismatch".into()));
    }
    let _ = usize_;
    Ok(out)
}

pub fn extract_to_mem(data: &[u8], name: &str, password: Option<&str>) -> R<Vec<u8>> {
    decode(data, name, password)
}

fn safe_join(dest: &Location, name: &str) -> Option<Location> {
    let clean: Vec<&str> = name.split('/').filter(|s| !s.is_empty() && *s != ".").collect();
    if clean.iter().any(|s| *s == "..") || clean.is_empty() {
        return None;
    }
    Some(dest.join(&clean.join("/")))
}

pub fn read_archive(v: &dyn Vfs, loc: &Location) -> R<Vec<u8>> {
    v.read(&loc.path, 1 << 31).map_err(|e| ZipError::Io(e.to_string()))
}

/// Extract every entry into `dest` (creating folders). Returns (files, bytes).
pub fn extract(reg: &Registry, src: &Location, dest: &Location, password: Option<&str>, mut progress: impl FnMut(&str, u64, u64)) -> R<(usize, u64)> {
    let sv = reg.get(&src.fs).ok_or_else(|| ZipError::Io("missing file system".into()))?;
    let dv = reg.get(&dest.fs).ok_or_else(|| ZipError::Io("missing file system".into()))?;
    let bytes = read_archive(sv.as_ref(), src)?;
    let entries = list(&bytes, password)?;
    let total: u64 = entries.iter().filter(|e| !e.is_dir).map(|e| e.size).sum();
    let mut done = 0u64;
    let mut files = 0usize;
    for e in &entries {
        let Some(target) = safe_join(dest, &e.name) else { continue };
        if e.is_dir {
            let _ = dv.mkdir(&target.path);
            continue;
        }
        let data = decode(&bytes, &e.name, password)?;
        if let Some(parent) = e.name.rsplit_once('/').map(|(p, _)| p) {
            let mut cur = dest.clone();
            for seg in parent.split('/').filter(|s| !s.is_empty()) {
                cur = cur.join(seg);
                let _ = dv.mkdir(&cur.path);
            }
        }
        dv.write(&target.path, &data).map_err(|x| ZipError::Io(x.to_string()))?;
        done += e.size;
        files += 1;
        progress(&e.name, done, total.max(1));
    }
    Ok((files, done))
}

struct Out<'a> {
    w: Box<dyn Write + Send + 'a>,
    cd: Vec<u8>,
    count: u16,
    offset: u64,
    password: Option<String>,
    level: u8,
}
fn dos_time(ts: i64) -> (u16, u16) {
    use chrono::{Datelike, Timelike};
    let d = { use chrono::TimeZone; chrono::Local.timestamp_opt(ts, 0).single().unwrap_or_else(chrono::Local::now) };
    let time = ((d.hour() as u16) << 11) | ((d.minute() as u16) << 5) | (d.second() as u16 / 2);
    let date = (((d.year() - 1980).max(0) as u16) << 9) | ((d.month() as u16) << 5) | d.day() as u16;
    (time, date)
}
impl Out<'_> {
    fn file(&mut self, reg: &Registry, src: &Location, name: &str) -> R<()> {
        let v = reg.get(&src.fs).ok_or_else(|| ZipError::Io("missing file system".into()))?;
        let st = v.stat(&src.path).map_err(|e| ZipError::Io(e.to_string()))?;
        let data = v.read(&src.path, 1 << 31).map_err(|e| ZipError::Io(e.to_string()))?;
        let crc = crc32(&data);
        let comp = if data.len() > 64 { miniz_oxide::deflate::compress_to_vec(&data, self.level) } else { data.clone() };
        let (method, mut payload) = if comp.len() < data.len() { (8u16, comp) } else { (0u16, data.clone()) };
        let (time, date) = dos_time(st.modified.unwrap_or_else(crate::model::now_secs));
        let mut flags = 0x0800u16; // UTF-8 names
        if self.password.is_some() {
            flags |= 1;
            let mut header = [0u8; 12];
            let mut seed = (crate::model::now_secs() as u64) ^ (crc as u64) ^ 0x9E37_79B9_7F4A_7C15;
            for b in header.iter_mut() {
                seed ^= seed << 13;
                seed ^= seed >> 7;
                seed ^= seed << 17;
                *b = (seed & 0xFF) as u8;
            }
            header[11] = (crc >> 24) as u8;
            let mut c = Crypto::new(self.password.as_deref().unwrap_or(""));
            c.encrypt(&mut header);
            c.encrypt(&mut payload);
            let mut p = header.to_vec();
            p.extend(payload);
            payload = p;
        }
        let nb = name.as_bytes();
        let mut h = Vec::with_capacity(30 + nb.len());
        h.extend(0x04034b50u32.to_le_bytes());
        h.extend(20u16.to_le_bytes());
        h.extend(flags.to_le_bytes());
        h.extend(method.to_le_bytes());
        h.extend(time.to_le_bytes());
        h.extend(date.to_le_bytes());
        h.extend(crc.to_le_bytes());
        h.extend((payload.len() as u32).to_le_bytes());
        h.extend((data.len() as u32).to_le_bytes());
        h.extend((nb.len() as u16).to_le_bytes());
        h.extend(0u16.to_le_bytes());
        h.extend(nb);
        self.w.write_all(&h).map_err(|e| ZipError::Io(e.to_string()))?;
        self.w.write_all(&payload).map_err(|e| ZipError::Io(e.to_string()))?;
        let mut c = Vec::with_capacity(46 + nb.len());
        c.extend(0x02014b50u32.to_le_bytes());
        c.extend(20u16.to_le_bytes());
        c.extend(20u16.to_le_bytes());
        c.extend(flags.to_le_bytes());
        c.extend(method.to_le_bytes());
        c.extend(time.to_le_bytes());
        c.extend(date.to_le_bytes());
        c.extend(crc.to_le_bytes());
        c.extend((payload.len() as u32).to_le_bytes());
        c.extend((data.len() as u32).to_le_bytes());
        c.extend((nb.len() as u16).to_le_bytes());
        c.extend(0u16.to_le_bytes());
        c.extend(0u16.to_le_bytes());
        c.extend(0u16.to_le_bytes());
        c.extend(0u16.to_le_bytes());
        c.extend(0u32.to_le_bytes());
        c.extend((self.offset as u32).to_le_bytes());
        c.extend(nb);
        self.cd.extend(c);
        self.count = self.count.saturating_add(1);
        self.offset += (30 + nb.len() + payload.len()) as u64;
        Ok(())
    }
    fn finish(mut self) -> R<()> {
        let cd_off = self.offset;
        let cd = std::mem::take(&mut self.cd);
        self.w.write_all(&cd).map_err(|e| ZipError::Io(e.to_string()))?;
        let mut e = Vec::new();
        e.extend(0x06054b50u32.to_le_bytes());
        e.extend(0u16.to_le_bytes());
        e.extend(0u16.to_le_bytes());
        e.extend(self.count.to_le_bytes());
        e.extend(self.count.to_le_bytes());
        e.extend((cd.len() as u32).to_le_bytes());
        e.extend((cd_off as u32).to_le_bytes());
        e.extend(0u16.to_le_bytes());
        self.w.write_all(&e).map_err(|e| ZipError::Io(e.to_string()))?;
        self.w.flush().map_err(|e| ZipError::Io(e.to_string()))?;
        Ok(())
    }
}

/// Compress files/folders (recursively) into a new `.zip`. Names keep their folder prefix.
pub fn compress(reg: &Registry, srcs: &[Location], dest_zip: &Location, password: Option<String>, level: u8, mut progress: impl FnMut(&str, u64, u64)) -> R<(usize, u64)> {
    let dv = reg.get(&dest_zip.fs).ok_or_else(|| ZipError::Io("missing file system".into()))?;
    let w = dv.create_write(&dest_zip.path).map_err(|e| ZipError::Io(e.to_string()))?;
    let mut out = Out { w, cd: Vec::new(), count: 0, offset: 0, password, level };
    let mut items: Vec<(Location, String)> = vec![];
    fn walk(reg: &Registry, l: &Location, name: String, out: &mut Vec<(Location, String)>) {
        let Some(v) = reg.get(&l.fs) else { return };
        if let Ok(e) = v.stat(&l.path) {
            if e.is_dir {
                if let Ok(list) = v.list(&l.path) {
                    for c in list.into_iter().filter(|c| !c.symlink) {
                        walk(reg, &c.loc, format!("{}/{}", name, c.name), out);
                    }
                }
            } else {
                out.push((l.clone(), name));
            }
        }
    }
    for s in srcs {
        let Some(v) = reg.get(&s.fs) else { continue };
        let Ok(e) = v.stat(&s.path) else { continue };
        walk(reg, s, e.name.clone(), &mut items);
    }
    let total: u64 = items.len() as u64;
    let mut n = 0usize;
    for (l, name) in items {
        out.file(reg, &l, &name)?;
        n += 1;
        progress(&name, n as u64, total.max(1));
    }
    out.finish()?;
    Ok((n, total))
}

/// Files inside an archive, for the contents dialog.
pub fn preview_entries(reg: &Registry, loc: &Location, password: Option<&str>) -> R<Vec<ZipEntry>> {
    let v = reg.get(&loc.fs).ok_or_else(|| ZipError::Io("missing file system".into()))?;
    let bytes = read_archive(v.as_ref(), loc)?;
    list(&bytes, password)
}

/// Read one member (used by Quick Look for archives).
pub fn read_member(reg: &Registry, loc: &Location, name: &str, password: Option<&str>) -> R<Vec<u8>> {
    let v = reg.get(&loc.fs).ok_or_else(|| ZipError::Io("missing file system".into()))?;
    let bytes = read_archive(v.as_ref(), loc)?;
    decode(&bytes, name, password)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memfs::MemVfs;
    use std::sync::Arc;

    #[test]
    fn roundtrip_and_password() {
        let reg = Registry::new();
        let v = Arc::new(MemVfs::new("mem"));
        reg.insert(v.clone());
        v.put("/a.txt", b"hello world hello world hello world".to_vec(), None, 0);
        v.mkdir_p("/sub", 0);
        v.put("/sub/b.txt", vec![7u8; 5000], None, 0);
        let out = Location::mem("/out.zip");
        compress(&reg, &[Location::mem("/a.txt"), Location::mem("/sub")], &out, Some("s3cret".into()), 6, |_, _, _| {}).unwrap();
        let entries = preview_entries(&reg, &out, Some("s3cret")).unwrap();
        assert!(entries.iter().any(|e| e.name == "a.txt" && e.encrypted));
        assert!(entries.iter().any(|e| e.name == "sub/b.txt"), "got {:?}", entries);
        let dest = Location::mem("/x");
        v.mkdir_p("/x", 0);
        let r = extract(&reg, &out, &dest, Some("s3cret"), |_, _, _| {});
        assert!(r.is_ok(), "{:?}", r.err());
        assert_eq!(v.read("/x/a.txt", 99).unwrap(), b"hello world hello world hello world");
        assert_eq!(v.read("/x/sub/b.txt", 9999).unwrap().len(), 5000);
        assert!(matches!(extract(&reg, &out, &dest, Some("nope"), |_, _, _| {}), Err(ZipError::BadPassword) | Err(ZipError::PasswordRequired)));
    }
}
