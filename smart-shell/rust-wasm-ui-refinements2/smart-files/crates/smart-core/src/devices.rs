//! Devices & locations: internal disks, USB sticks, phones, network and cloud. Mount/unmount are real.
use crate::model::Location;
use crate::vfs::Vfs;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeviceKind {
    Internal,
    Removable,
    Optical,
    Network,
    Phone,
    Cloud,
}

impl DeviceKind {
    pub fn icon(self) -> &'static str {
        match self {
            DeviceKind::Internal | DeviceKind::Removable => "ssd-square",
            DeviceKind::Optical => "sd-card",
            DeviceKind::Network => "global",
            DeviceKind::Phone => "smartphone",
            DeviceKind::Cloud => "cloud",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            DeviceKind::Internal => "Built-in",
            DeviceKind::Removable => "Removable",
            DeviceKind::Optical => "Optical",
            DeviceKind::Network => "Network",
            DeviceKind::Phone => "Continuity",
            DeviceKind::Cloud => "Cloud",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub kind: DeviceKind,
    pub detail: String,
    pub mounted: bool,
    pub loc: Option<Location>,
    pub total: u64,
    pub free: u64,
    pub fs_type: String,
    pub can_unmount: bool,
    pub can_eject: bool,
    pub battery: Option<u8>,
}

impl Device {
    pub fn used_fraction(&self) -> f32 {
        if self.total == 0 { 0.0 } else { 1.0 - self.free as f32 / self.total as f32 }
    }
    pub fn capacity_text(&self) -> String {
        if let (Some(b), true) = (self.battery, self.total == 0) {
            return format!("{}% battery", b);
        }
        if self.total == 0 {
            return self.detail.clone();
        }
        format!("{} free of {}", crate::model::format_size(self.free), crate::model::format_size(self.total))
    }
}

pub struct Mounted {
    pub loc: Location,
    /// Provider to add to the registry (phones, cloud); local mounts reuse the `file` provider.
    pub vfs: Option<Arc<dyn Vfs>>,
}

pub trait DeviceProvider: Send + Sync {
    fn list(&self) -> Vec<Device>;
    fn mount(&self, id: &str) -> Result<Mounted, String>;
    fn unmount(&self, id: &str) -> Result<(), String>;
    fn eject(&self, id: &str) -> Result<(), String> {
        self.unmount(id)
    }
    fn owns(&self, id: &str) -> bool;
}

#[cfg(not(target_arch = "wasm32"))]
pub mod native {
    use super::*;
    use crate::local::path_to_loc;
    use std::process::Command;

    fn run(cmd: &str, args: &[&str]) -> Result<String, String> {
        let o = Command::new(cmd).args(args).output().map_err(|e| format!("{} is not available ({})", cmd, e))?;
        if o.status.success() {
            Ok(String::from_utf8_lossy(&o.stdout).to_string())
        } else {
            let e = String::from_utf8_lossy(&o.stderr).trim().to_string();
            Err(if e.is_empty() { format!("{} failed", cmd) } else { e })
        }
    }

    const REAL_FS: &[&str] = &["ext4", "ext3", "ext2", "btrfs", "xfs", "f2fs", "vfat", "exfat", "ntfs", "ntfs3", "fuseblk", "apfs", "hfs", "zfs", "iso9660", "udf", "nfs", "nfs4", "cifs", "smb3", "fuse.sshfs", "NTFS", "FAT32", "exFAT", "NTFS3", "APFS", "ReFS"];

    pub struct SystemDevices;

    impl SystemDevices {
        fn lsblk() -> Vec<serde_json::Value> {
            let Ok(out) = run("lsblk", &["-J", "-b", "-o", "NAME,PATH,LABEL,SIZE,FSTYPE,MOUNTPOINT,RM,TRAN,TYPE,HOTPLUG,MODEL"]) else { return vec![] };
            let v: serde_json::Value = serde_json::from_str(&out).unwrap_or_default();
            let mut flat = vec![];
            fn rec(n: &serde_json::Value, out: &mut Vec<serde_json::Value>) {
                out.push(n.clone());
                if let Some(c) = n.get("children").and_then(|c| c.as_array()) {
                    for x in c {
                        rec(x, out);
                    }
                }
            }
            for n in v.get("blockdevices").and_then(|b| b.as_array()).cloned().unwrap_or_default() {
                rec(&n, &mut flat);
            }
            flat
        }
    }

    impl DeviceProvider for SystemDevices {
        fn list(&self) -> Vec<Device> {
            let mut out: Vec<Device> = vec![];
            let disks: Vec<sysinfo::Disk> = sysinfo::Disks::new_with_refreshed_list().map(Vec::from).unwrap_or_default();
            for d in disks.iter() {
                let fs = d.file_system().to_string_lossy().to_string();
                let mp = d.mount_point().to_string_lossy().to_string();
                let is_root = mp == "/" || (cfg!(windows) && mp.len() <= 3);
                if !REAL_FS.contains(&fs.as_str()) && !is_root {
                    continue;
                }
                if ["/boot", "/snap", "/var/lib", "/etc", "/usr", "/nix", "/System/Volumes/VM", "/System/Volumes/Preboot", "/System/Volumes/Update"].iter().any(|p| mp.starts_with(p)) {
                    continue;
                }
                if out.iter().any(|o| o.loc.as_ref().is_some_and(|l| l.path == mp)) {
                    continue;
                }
                let name_raw = d.name().to_string_lossy().to_string();
                let kind = if is_root {
                    DeviceKind::Internal
                } else if matches!(fs.as_str(), "nfs" | "nfs4" | "cifs" | "smb3" | "fuse.sshfs") {
                    DeviceKind::Network
                } else if fs == "iso9660" || fs == "udf" {
                    DeviceKind::Optical
                } else if d.is_removable() || mp.starts_with("/media") || mp.starts_with("/run/media") || mp.starts_with("/Volumes") {
                    DeviceKind::Removable
                } else {
                    DeviceKind::Internal
                };
                let label = mp.rsplit('/').next().filter(|s| !s.is_empty()).map(|s| s.to_string());
                let name = if is_root { if cfg!(target_os = "macos") { "Macintosh HD".into() } else { "Computer".into() } } else { label.unwrap_or(name_raw.clone()) };
                out.push(Device {
                    id: if name_raw.starts_with("/dev/") { name_raw } else { mp.clone() },
                    name,
                    kind,
                    detail: fs.clone(),
                    mounted: true,
                    loc: Some(path_to_loc(d.mount_point())),
                    total: d.total_space(),
                    free: d.available_space(),
                    fs_type: fs,
                    can_unmount: !is_root,
                    can_eject: kind == DeviceKind::Removable || kind == DeviceKind::Optical,
                    battery: None,
                });
            }
            // block devices that are present but not mounted (USB sticks, SD cards…)
            for n in Self::lsblk() {
                let s = |k: &str| n.get(k).and_then(|x| x.as_str()).unwrap_or("").to_string();
                let mounted = !s("mountpoint").is_empty();
                let removable = n.get("rm").map(|x| x.as_bool().unwrap_or(false) || x.as_str() == Some("1")).unwrap_or(false)
                    || n.get("hotplug").map(|x| x.as_bool().unwrap_or(false) || x.as_str() == Some("1")).unwrap_or(false);
                let fs = s("fstype");
                if mounted || !removable || fs.is_empty() || s("type") == "disk" && !s("fstype").is_empty() && false {
                    continue;
                }
                if matches!(fs.as_str(), "swap" | "crypto_LUKS" | "LVM2_member") {
                    continue;
                }
                let total = n.get("size").and_then(|x| x.as_u64().or_else(|| x.as_str().and_then(|s| s.parse().ok()))).unwrap_or(0);
                let label = s("label");
                let model = s("model");
                out.push(Device {
                    id: s("path"),
                    name: if !label.is_empty() { label } else if !model.is_empty() { model.trim().to_string() } else { s("name") },
                    kind: DeviceKind::Removable,
                    detail: format!("{} · {}", s("tran").to_uppercase(), fs),
                    mounted: false,
                    loc: None,
                    total,
                    free: 0,
                    fs_type: fs,
                    can_unmount: false,
                    can_eject: true,
                    battery: None,
                });
            }
            out
        }

        fn mount(&self, id: &str) -> Result<Mounted, String> {
            if cfg!(target_os = "macos") {
                run("diskutil", &["mount", id])?;
                return Err("Mounted — refreshing".into());
            }
            let out = run("udisksctl", &["mount", "-b", id, "--no-user-interaction"]).or_else(|e| run("gio", &["mount", "-d", id]).map_err(|_| e))?;
            let mp = out.rsplit(" at ").next().unwrap_or("").trim().trim_end_matches('.').to_string();
            if mp.starts_with('/') {
                Ok(Mounted { loc: Location::file(mp), vfs: None })
            } else {
                Err("Mounted — refreshing".into())
            }
        }

        fn unmount(&self, id: &str) -> Result<(), String> {
            if cfg!(target_os = "macos") {
                return run("diskutil", &["unmount", id]).map(|_| ());
            }
            if cfg!(windows) {
                return Err("Use “Safely remove hardware” on Windows".into());
            }
            if id.starts_with("/dev/") {
                run("udisksctl", &["unmount", "-b", id, "--no-user-interaction"]).map(|_| ())
            } else {
                run("umount", &[id]).or_else(|_| run("gio", &["mount", "-u", id])).map(|_| ())
            }
        }

        fn eject(&self, id: &str) -> Result<(), String> {
            if cfg!(target_os = "macos") {
                return run("diskutil", &["eject", id]).map(|_| ());
            }
            let _ = self.unmount(id);
            // power the drive off (parent disk) so it can be unplugged
            let parent = id.trim_end_matches(|c: char| c.is_ascii_digit()).trim_end_matches('p');
            run("udisksctl", &["power-off", "-b", parent, "--no-user-interaction"]).map(|_| ())
        }

        fn owns(&self, id: &str) -> bool {
            id.starts_with("/dev/") || id.starts_with('/') || id.len() <= 3
        }
    }

    /// Android phones over ADB (USB debugging or `adb connect`). Browse, pull and push for real.
    pub mod adb {
        use super::*;
        use crate::model::Entry;
        use crate::vfs::*;
        use std::io::{Read, Write};
        use std::process::Stdio;

        pub fn adb_available() -> bool {
            Command::new("adb").arg("version").output().map(|o| o.status.success()).unwrap_or(false)
        }

        pub struct AdbVfs {
            pub serial: String,
            pub id: String,
        }

        fn q(p: &str) -> String {
            format!("'{}'", p.replace('\'', "'\\''"))
        }

        impl AdbVfs {
            pub fn new(serial: &str) -> Self {
                Self { serial: serial.into(), id: format!("adb:{}", serial) }
            }
            fn sh(&self, cmd: &str) -> FsResult<String> {
                let o = Command::new("adb").args(["-s", &self.serial, "shell", cmd]).output().map_err(|e| FsError::Other(e.to_string()))?;
                let s = String::from_utf8_lossy(&o.stdout).to_string();
                if !o.status.success() || s.contains("No such file") {
                    return Err(if s.contains("No such file") { FsError::NotFound } else { FsError::Other(String::from_utf8_lossy(&o.stderr).to_string()) });
                }
                Ok(s)
            }
            fn parse(&self, dir: &str, line: &str) -> Option<Entry> {
                // drwxrws--- 2 u0_a98 media_rw 4096 2024-05-01 10:22 DCIM
                let mut it = line.split_whitespace();
                let perms = it.next()?;
                let _links = it.next()?;
                let _o = it.next()?;
                let _g = it.next()?;
                let size: u64 = it.next()?.parse().ok()?;
                let date = it.next()?;
                let time = it.next()?;
                let name_start = line.find(time)? + time.len();
                let mut name = line[name_start..].trim().to_string();
                if let Some((n, _)) = name.split_once(" -> ") {
                    name = n.to_string();
                }
                let ts = chrono::NaiveDateTime::parse_from_str(&format!("{} {}", date, time), "%Y-%m-%d %H:%M").ok().map(|d| d.and_utc().timestamp());
                let is_dir = perms.starts_with('d') || perms.starts_with('l');
                let path = if dir == "/" { format!("/{}", name) } else { format!("{}/{}", dir, name) };
                Some(Entry::new(Location::new(self.id.clone(), path), is_dir, size, ts))
            }
        }

        impl Vfs for AdbVfs {
            fn id(&self) -> &str {
                &self.id
            }
            fn list(&self, path: &str) -> FsResult<Vec<Entry>> {
                let out = self.sh(&format!("ls -lAL {}", q(path)))?;
                Ok(out.lines().filter_map(|l| self.parse(path, l)).collect())
            }
            fn stat(&self, path: &str) -> FsResult<Entry> {
                let out = self.sh(&format!("ls -ldL {}", q(path)))?;
                let parent = path.rsplit_once('/').map(|(a, _)| if a.is_empty() { "/" } else { a }).unwrap_or("/");
                let line = out.lines().next().ok_or(FsError::NotFound)?;
                let mut e = self.parse(parent, line).ok_or(FsError::NotFound)?;
                e.loc = Location::new(self.id.clone(), path);
                e.name = e.loc.name();
                Ok(e)
            }
            fn open_read(&self, path: &str) -> FsResult<Box<dyn Read + Send>> {
                let mut c = Command::new("adb").args(["-s", &self.serial, "exec-out", &format!("cat {}", q(path))]).stdout(Stdio::piped()).spawn().map_err(|e| FsError::Other(e.to_string()))?;
                Ok(Box::new(c.stdout.take().ok_or(FsError::NotFound)?))
            }
            fn create_write(&self, path: &str) -> FsResult<Box<dyn Write + Send>> {
                struct W(std::process::Child);
                impl Write for W {
                    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
                        self.0.stdin.as_mut().unwrap().write(b)
                    }
                    fn flush(&mut self) -> std::io::Result<()> {
                        Ok(())
                    }
                }
                impl Drop for W {
                    fn drop(&mut self) {
                        drop(self.0.stdin.take());
                        let _ = self.0.wait();
                    }
                }
                let c = Command::new("adb").args(["-s", &self.serial, "shell", &format!("cat > {}", q(path))]).stdin(Stdio::piped()).spawn().map_err(|e| FsError::Other(e.to_string()))?;
                Ok(Box::new(W(c)))
            }
            fn mkdir(&self, path: &str) -> FsResult<()> {
                self.sh(&format!("mkdir {}", q(path))).map(|_| ())
            }
            fn rename(&self, from: &str, to: &str) -> FsResult<()> {
                self.sh(&format!("mv {} {}", q(from), q(to))).map(|_| ())
            }
            fn remove(&self, path: &str, _t: bool) -> FsResult<()> {
                self.sh(&format!("rm -rf {}", q(path))).map(|_| ())
            }
        }

        pub struct AdbDevices;
        impl AdbDevices {
            fn query(&self) -> Vec<(String, String)> {
                let Ok(out) = run("adb", &["devices", "-l"]) else { return vec![] };
                out.lines()
                    .skip(1)
                    .filter(|l| l.contains(" device "))
                    .filter_map(|l| {
                        let serial = l.split_whitespace().next()?.to_string();
                        let model = l.split_whitespace().find_map(|p| p.strip_prefix("model:")).unwrap_or("Android").replace('_', " ");
                        Some((serial, model))
                    })
                    .collect()
            }
        }
        impl DeviceProvider for AdbDevices {
            fn list(&self) -> Vec<Device> {
                self.query()
                    .into_iter()
                    .map(|(serial, model)| {
                        let battery = run("adb", &["-s", &serial, "shell", "dumpsys battery | grep level"]).ok().and_then(|s| s.split(':').nth(1).and_then(|n| n.trim().parse().ok()));
                        Device { id: format!("adb:{}", serial), name: model, kind: DeviceKind::Phone, detail: "Android · ADB".into(), mounted: true, loc: Some(Location::new(format!("adb:{}", serial), "/sdcard")), total: 0, free: 0, fs_type: "adb".into(), can_unmount: false, can_eject: false, battery }
                    })
                    .collect()
            }
            fn mount(&self, id: &str) -> Result<Mounted, String> {
                let serial = id.trim_start_matches("adb:");
                Ok(Mounted { loc: Location::new(id, "/sdcard"), vfs: Some(Arc::new(AdbVfs::new(serial))) })
            }
            fn unmount(&self, _id: &str) -> Result<(), String> {
                Err("Disconnect the phone to unmount".into())
            }
            fn owns(&self, id: &str) -> bool {
                id.starts_with("adb:")
            }
        }
    }
}
