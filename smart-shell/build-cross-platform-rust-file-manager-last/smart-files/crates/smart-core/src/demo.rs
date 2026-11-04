//! Seeded in-memory world used by the web build (and `--demo` on desktop).
use crate::devices::*;
use crate::memfs::MemVfs;
use crate::model::{now_secs, Location};
use crate::registry::Registry;
use std::sync::{Arc, Mutex};

macro_rules! img {
    ($n:literal) => {
        include_bytes!(concat!("../../../assets/thumbs/", $n, ".jpg")).to_vec()
    };
}
macro_rules! wav {
    ($n:literal) => {
        include_bytes!(concat!("../../../assets/audio/", $n, ".wav")).to_vec()
    };
}

const KB: u64 = 1_000;
const MB: u64 = 1_000_000;
const GB: u64 = 1_000_000_000;
pub const HOME: &str = "/Home";

pub fn home() -> Location {
    Location::mem(HOME)
}

fn seed_home(fs: &MemVfs) {
    let now = now_secs();
    let ago = |h: f64| now - (h * 3600.0) as i64;
    let p = |s: &str| format!("{}/{}", HOME, s);
    for d in ["Desktop", "Documents", "Downloads", "Pictures", "Music", "Movies"] {
        fs.mkdir_p(&p(d), ago(200.0));
    }
    // Pictures — straight from the prototype
    let pics: [(&str, Vec<u8>, u64, f64); 8] = [
        ("4K Frosted Glass.jpg", img!("frosted-glass"), 1500 * KB, 0.1),
        ("Blossom Study.jpg", img!("texture-01"), 812 * KB, 24.0 * 4.0),
        ("Cycling in Soweto.png", img!("talk-notes"), 977 * KB, 24.0 * 3.0),
        ("Green Fold.png", img!("design-scan"), 868 * KB, 16.0),
        ("Nixio Dashboard.png", img!("brand-board"), 1700 * KB, 1.0),
        ("Orbit Poster.png", img!("poster-01"), 1200 * KB, 24.0 * 2.5),
        ("Soweto Moto Ad.png", img!("package-a"), 1100 * KB, 24.0 * 2.8),
        ("Window Study.png", img!("ui-exploration"), 878 * KB, 12.0),
    ];
    for (n, d, s, h) in pics {
        fs.put(&p(&format!("Pictures/{}", n)), d, Some(s), ago(h));
    }
    let pk = "Pictures/Amapiano Pack Vol. 1";
    fs.put(&p(&format!("{pk}/01 Log Drum 126 BPM.wav")), wav!("kick_logdrum"), Some(18_400 * KB), ago(7.0));
    fs.put(&p(&format!("{pk}/02 Shaker Groove.wav")), wav!("shaker"), Some(6_200 * KB), ago(7.2));
    fs.put(&p(&format!("{pk}/Amapiano Pack Vol. 1.flp")), vec![0u8; 64], Some(4_800 * KB), ago(14.0));
    fs.put(&p(&format!("{pk}/Pack Cover.jpg")), img!("poster-01"), Some(2_100 * KB), ago(9.0));
    let ld = "Pictures/Log Drum Sessions";
    fs.put(&p(&format!("{ld}/Log Drum 126.wav")), wav!("logdrum_conga"), Some(12_800 * KB), ago(20.0));
    fs.put(&p(&format!("{ld}/Log Drum 128.wav")), wav!("kick_logdrum"), Some(6_400 * KB), ago(20.2));
    fs.put(&p(&format!("{ld}/Session Notes.txt")), b"Session notes\n=============\n\nLog drum at 126 BPM sits best with the shaker pushed 12 ms late.\nTODO: bounce stems, label takes, archive the raw takes.\n".to_vec(), Some(12 * KB), ago(20.5));
    let dw = "Pictures/SMART-DAW Beta";
    fs.put(&p(&format!("{dw}/smart-daw-0.4.2.AppImage")), vec![0u8; 64], Some(22_400 * KB), ago(30.0));
    fs.put(&p(&format!("{dw}/Layout Study.png")), img!("layout-study"), Some(1_400 * KB), ago(31.0));
    fs.put(&p(&format!("{dw}/beta-notes.md")), b"# SMART-DAW beta\n\n- Piano roll snaps to the 8 px grid\n- Log drum preset needs more punch\n- **Ship** the AppImage on Friday\n".to_vec(), Some(12 * KB), ago(36.0));
    let br = "Pictures/Brand \u{2014} Naledzi";
    fs.put(&p(&format!("{br}/Naledzi Wordmark.svg")), br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64"><rect width="64" height="64" rx="16" fill="#2F6FEB"/><path d="M42 22c-2.4-4-6.4-6-10.6-6-8.8 0-14.4 7.2-14.4 16.4 0 9 5.8 15.6 14 15.6 4.4 0 8.2-2 10.6-5.8l-5.2-3.4c-1.2 2-3 3.2-5.4 3.2-4.2 0-6.8-3.6-6.8-9.6 0-6.2 2.8-10 7-10 2.2 0 4 1 5.2 3l5.6-3.4Z" fill="white"/></svg>"##.to_vec(), Some(12 * KB), ago(96.0));
    fs.put(&p(&format!("{br}/Reference 01.jpg")), img!("reference-01"), Some(1_600 * KB), ago(97.0));
    fs.put(&p(&format!("{br}/Type Specimen.pdf")), b"%PDF-1.4\n% Type specimen (demo)\n".to_vec(), Some(4_200 * KB), ago(98.0));

    // Documents
    fs.put(&p("Documents/Brand Brief.md"), b"# Naledzi brand brief\n\nCobalt is the only accent. Depth comes from tone and shadow, never from strokes.\n\n## Voice\nCalm, precise, a little warm.\n\n## Invoice reminders\nSend invoice 2026-014 to the Soweto Moto team by Friday.\n".to_vec(), None, ago(5.0));
    fs.put(&p("Documents/Q4 Roadmap.txt"), b"Q4 roadmap\n\n1. Smart Files 0.2 \xe2\x80\x94 cloud accounts and phone continuity\n2. Smart DAW beta \xe2\x80\x94 log drum instrument\n3. Design contract v0.2\n".to_vec(), None, ago(30.0));
    fs.put(&p("Documents/invoice-2026-014.txt"), b"INVOICE 2026-014\nClient: Soweto Moto\nAmount: R 24 500\nStatus: awaiting payment\n".to_vec(), None, ago(50.0));
    fs.put(&p("Documents/Type Specimen.pdf"), b"%PDF-1.4\n".to_vec(), Some(4_200 * KB), ago(98.0));
    fs.put(&p("Documents/Budget 2026.xlsx"), vec![0u8; 32], Some(212 * KB), ago(120.0));
    fs.put(&p("Documents/Studio Master Reel 2026.mov"), b"SMART-DEMO-MEDIA\n".to_vec(), Some(5 * GB), ago(6.0));
    fs.put(&p("Documents/Backup Snapshot 2026.sparse"), b"SMART-DEMO-BACKUP\n".to_vec(), Some(5 * GB), ago(48.0));
    fs.put(&p("Documents/main.rs"), b"fn main() {\n    println!(\"Hello from Smart Files\");\n}\n".to_vec(), None, ago(70.0));
    // Downloads
    fs.put(&p("Downloads/Brand Master.zip"), vec![0u8; 32], Some(248 * MB), ago(22.0));
    fs.put(&p("Downloads/Landing Draft.png"), img!("landing-draft"), Some(1_250 * KB), ago(26.0));
    fs.put(&p("Downloads/Package B.png"), img!("package-b"), Some(1_400 * KB), ago(40.0));
    fs.put(&p("Downloads/smart-daw-0.4.2.AppImage"), vec![0u8; 32], Some(22_400 * KB), ago(41.0));
    // Music
    fs.put(&p("Music/Pad Warm.wav"), wav!("pad_warm"), None, ago(60.0));
    fs.put(&p("Music/Piano Lead.wav"), wav!("piano_lead"), None, ago(61.0));
    fs.put(&p("Music/Bass Deep.wav"), wav!("bass_deep"), None, ago(62.0));
    fs.put(&p("Music/Log Drum Conga.wav"), wav!("logdrum_conga"), None, ago(63.0));
    // Movies
    fs.put(&p("Movies/Soweto Ride Teaser.mp4"), vec![0u8; 32], Some(842 * MB), ago(80.0));
    fs.put(&p("Movies/Studio Tour.mov"), vec![0u8; 32], Some(1_300 * MB), ago(90.0));
    // Desktop
    fs.put(&p("Desktop/Screenshot 2026-11-01.png"), img!("design-scan"), Some(2_300 * KB), ago(2.0));
    fs.put(&p("Desktop/Todo.txt"), b"- reply to Naledzi\n- export album cover\n- eject the SanDisk before leaving\n".to_vec(), Some(1 * KB), ago(3.0));
}

pub fn build() -> (Arc<Registry>, Arc<DemoDevices>) {
    let reg = Arc::new(Registry::new());
    let drive = MemVfs::new("mem").with_capacity(512 * GB, 194 * GB);
    seed_home(&drive);
    reg.insert(Arc::new(drive));

    let now = now_secs();
    let usb = MemVfs::new("usb").with_capacity(64 * GB, 41 * GB);
    usb.put("/Amapiano Pack Vol. 1.flp", vec![0u8; 32], Some(4_800 * KB), now - 3600 * 2);
    usb.put("/Pack Cover.jpg", img!("poster-01"), Some(2_100 * KB), now - 3600 * 5);
    usb.put("/Handover.txt", b"Handover notes for the studio.\nKeep the stems in /Stems.\n".to_vec(), Some(4 * KB), now - 3600 * 30);
    reg.insert(Arc::new(usb));

    let nas = MemVfs::new("nas").with_capacity(8_000 * GB, 2_100 * GB);
    nas.put("/Brand Master.zip", vec![0u8; 32], Some(248 * MB), now - 3600 * 14);
    nas.put("/Type Specimen.pdf", b"%PDF-1.4\n".to_vec(), Some(4_200 * KB), now - 3600 * 90);
    nas.mkdir_p("/Backups", now - 3600 * 3);
    nas.put("/Backups/Session Backup 01.wav", wav!("pad_warm"), Some(18_400 * KB), now - 3600 * 2);
    reg.insert(Arc::new(nas));

    let phone = MemVfs::new("phone");
    phone.mkdir_p("/DCIM/Camera", now - 3600);
    phone.put("/DCIM/Camera/IMG_2041.jpg", img!("texture-01"), Some(3_200 * KB), now - 3600 * 5);
    phone.put("/DCIM/Camera/IMG_2042.jpg", img!("frosted-glass"), Some(2_900 * KB), now - 3600 * 5);
    phone.put("/DCIM/Camera/IMG_2043.jpg", img!("package-b"), Some(3_100 * KB), now - 3600 * 4);
    phone.mkdir_p("/Download", now - 3600 * 20);
    phone.put("/Download/boarding-pass.pdf", b"%PDF-1.4\n".to_vec(), Some(180 * KB), now - 3600 * 20);
    phone.mkdir_p("/Recordings", now - 3600 * 30);
    phone.put("/Recordings/Voice 014.wav", wav!("shaker"), None, now - 3600 * 30);
    reg.insert(Arc::new(phone));

    let cloud = MemVfs::new("cloud").with_capacity(2_000 * GB, 1_240 * GB);
    cloud.put("/Pitch Deck.pdf", b"%PDF-1.4\n".to_vec(), Some(12 * MB), now - 3600 * 48);
    cloud.put("/Moodboard.jpg", img!("reference-01"), Some(1_900 * KB), now - 3600 * 60);
    cloud.mkdir_p("/Shared with me", now - 3600 * 72);
    reg.insert(Arc::new(cloud));

    let dev = |id: &str, name: &str, kind: DeviceKind, detail: &str, mounted: bool, fs: &str, total: u64, free: u64, battery: Option<u8>| Device {
        id: id.into(),
        name: name.into(),
        kind,
        detail: detail.into(),
        mounted,
        loc: if mounted { Some(Location::new(fs, if fs == "mem" { "/" } else { "/" })) } else { None },
        total,
        free,
        fs_type: fs.into(),
        can_unmount: kind != DeviceKind::Internal,
        can_eject: kind == DeviceKind::Removable,
        battery,
    };
    let devices = vec![
        dev("mem", "Smart Drive", DeviceKind::Internal, "NVMe · APFS", true, "mem", 512 * GB, 194 * GB, None),
        dev("usb", "SanDisk Ultra", DeviceKind::Removable, "USB 3.2 · exFAT", false, "usb", 64 * GB, 41 * GB, None),
        dev("nas", "Studio NAS", DeviceKind::Network, "smb://studio-nas.local", true, "nas", 8_000 * GB, 2_100 * GB, None),
        dev("phone", "Naledzi\u{2019}s Pixel", DeviceKind::Phone, "Paired over Wi-Fi", true, "phone", 0, 0, Some(42)),
        dev("cloud", "Smart Cloud", DeviceKind::Cloud, "Synced just now", true, "cloud", 2_000 * GB, 1_240 * GB, None),
    ];
    (reg, Arc::new(DemoDevices { devices: Mutex::new(devices) }))
}

pub struct DemoDevices {
    devices: Mutex<Vec<Device>>,
}

impl DeviceProvider for DemoDevices {
    fn list(&self) -> Vec<Device> {
        self.devices.lock().unwrap().clone()
    }
    fn mount(&self, id: &str) -> Result<Mounted, String> {
        let mut d = self.devices.lock().unwrap();
        let dev = d.iter_mut().find(|x| x.id == id).ok_or("unknown device")?;
        dev.mounted = true;
        let loc = Location::new(dev.fs_type.clone(), "/");
        dev.loc = Some(loc.clone());
        Ok(Mounted { loc, vfs: None })
    }
    fn unmount(&self, id: &str) -> Result<(), String> {
        let mut d = self.devices.lock().unwrap();
        let dev = d.iter_mut().find(|x| x.id == id).ok_or("unknown device")?;
        if dev.kind == DeviceKind::Internal {
            return Err("The startup disk can\u{2019}t be unmounted".into());
        }
        dev.mounted = false;
        dev.loc = None;
        Ok(())
    }
    fn owns(&self, id: &str) -> bool {
        self.devices.lock().unwrap().iter().any(|d| d.id == id)
    }
}
