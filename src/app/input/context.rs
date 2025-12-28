use std::sync::{Arc, Mutex, atomic::AtomicU64};

use dashmap::DashMap;

use crate::{app::input::device::Device, config::CONFIG};

pub struct Context {
    pub devices: DashMap<u64, Arc<Mutex<Device>>>,
    pub viiper_address: Option<std::net::SocketAddr>,
    pub viiper_available: bool,
    pub viiper_version: Option<String>,
    pub keyboard_mouse_emulation: bool,
    pub steam_overlay_open: bool,
    pub next_device_id: AtomicU64,
}
impl Context {
    pub fn new(viiper_address: Option<std::net::SocketAddr>) -> Self {
        Self {
            devices: DashMap::new(),
            viiper_address,
            viiper_available: false,
            viiper_version: None,
            keyboard_mouse_emulation: CONFIG
                .read()
                .expect("WTF: config not initialized")
                .as_ref()
                .expect("WTF: config not set")
                .kbm_emulation
                .unwrap_or(false),
            steam_overlay_open: false,
            next_device_id: AtomicU64::new(1),
        }
    }

    pub fn device_for_sdl_id(&self, sdl_id: u32) -> Option<Arc<Mutex<Device>>> {
        for device_mtx in self.devices.iter() {
            let device = device_mtx.value().lock().ok()?;
            if device.sdl_devices.iter().any(|d| d.id == sdl_id) {
                return Some(device_mtx.value().clone());
            }
        }
        None
    }

    pub fn device_for_id(&self, device_id: u64) -> Option<Arc<Mutex<Device>>> {
        self.devices.get(&device_id).map(|d| d.value().clone())
    }
}
