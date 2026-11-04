//! # smart-core
//! The Smart Files engine. Everything a file manager *does* lives here; nothing it *looks like* does.
//!
//! * [`vfs`]      – storage abstraction (`Vfs`), with local disk, memory, ADB phone providers
//! * [`search`]   – query language, cached index, fuzzy ranking, content scan
//! * [`ops`]      – copy/move/trash/delete jobs across providers with progress + cancel
//! * [`devices`]  – real mount / unmount / eject, phones, network
//! * [`apps`]     – default app + “Open With…”
//! * [`backend`]  – the facade frontends use: call methods, `poll()` events
pub mod archive;
pub mod apps;
pub mod audio;
pub mod backend;
pub mod devices;
pub mod exec;
pub mod library;
pub mod memfs;
pub mod model;
pub mod ops;
pub mod registry;
pub mod search;
pub mod thumbs;
pub mod tools;
pub mod vfs;

#[cfg(not(target_arch = "wasm32"))]
pub mod local;

#[cfg(feature = "demo")]
pub mod demo;

pub use backend::{Backend, Event, Place};
pub use devices::{Device, DeviceKind};
pub use library::{Library, Settings, SharedLibrary, TagDef};
pub use model::*;
pub use ops::{JobEvent, JobKind};
pub use search::{Hit, Query};
