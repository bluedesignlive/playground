use crate::vfs::Vfs;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// All mounted providers, addressable by `Location::fs`. Providers can appear at runtime (USB, phone, cloud).
#[derive(Default)]
pub struct Registry {
    map: RwLock<HashMap<String, Arc<dyn Vfs>>>,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn insert(&self, v: Arc<dyn Vfs>) {
        self.map.write().unwrap().insert(v.id().to_string(), v);
    }
    pub fn get(&self, id: &str) -> Option<Arc<dyn Vfs>> {
        self.map.read().unwrap().get(id).cloned()
    }
    pub fn remove(&self, id: &str) {
        self.map.write().unwrap().remove(id);
    }
    pub fn ids(&self) -> Vec<String> {
        self.map.read().unwrap().keys().cloned().collect()
    }
}
