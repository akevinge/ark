use std::{
    collections::{hash_map::Iter, HashMap},
    time::Instant,
};

use pnet_datalink::MacAddr;

pub struct MacCache {
    cache: HashMap<MacAddr, Instant>,
}

impl MacCache {
    pub fn new() -> Self {
        MacCache {
            cache: HashMap::new(),
        }
    }

    pub fn add(&mut self, mac: MacAddr) {
        self.cache.insert(mac, Instant::now());
    }

    pub fn delete(&mut self, mac: &MacAddr) {
        self.cache.remove(mac);
    }

    pub fn iter(&self) -> Iter<MacAddr, Instant> {
        self.cache.iter()
    }

    pub fn size(&self) -> usize {
        self.cache.keys().len()
    }
}
