use std::collections::HashMap;
use std::ffi::CString;
use std::sync::{Mutex, Once, OnceLock};

use sdl3_sys::hidapi;

static HID_INIT: Once = Once::new();

#[derive(Copy, Clone)]
struct HidDev(*mut hidapi::SDL_hid_device);

unsafe impl Send for HidDev {}
unsafe impl Sync for HidDev {}

static OPEN_DEVS: OnceLock<Mutex<HashMap<String, HidDev>>> = OnceLock::new();

fn ensure_hid_init() {
    HID_INIT.call_once(|| unsafe {
        let r = hidapi::SDL_hid_init();
        if r != 0 {
            tracing::warn!("steamdeck_hid: SDL_hid_init failed (rc={})", r);
        }
    });
}

fn devs() -> &'static Mutex<HashMap<String, HidDev>> {
    OPEN_DEVS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn get_or_open(path: &str) -> Option<*mut hidapi::SDL_hid_device> {
    ensure_hid_init();

    let Ok(mut map) = devs().lock() else {
        tracing::warn!("steamdeck_hid: failed to lock device cache");
        return None;
    };

    if let Some(&dev) = map.get(path) {
        if !dev.0.is_null() {
            return Some(dev.0);
        }
        map.remove(path);
    }

    let c_path = match CString::new(path) {
        Ok(s) => s,
        Err(_) => {
            tracing::warn!("steamdeck_hid: path contains NUL bytes");
            return None;
        }
    };

    let dev = unsafe { hidapi::SDL_hid_open_path(c_path.as_ptr()) };
    if dev.is_null() {
        tracing::warn!("steamdeck_hid: SDL_hid_open_path failed for {path}");
        return None;
    }

    map.insert(path.to_string(), HidDev(dev));
    Some(dev)
}

pub fn close_path(path: &str) {
    let Ok(mut map) = devs().lock() else {
        return;
    };

    let Some(dev) = map.remove(path) else {
        return;
    };

    if !dev.0.is_null() {
        unsafe { hidapi::SDL_hid_close(dev.0) };
    }
}

pub fn send_whatever(path: &str, send_whatever: &Vec<u8> /*64 bytes, always!*/) {
    let Some(dev) = get_or_open(path) else {
        tracing::error!(
            "steamdeck_hid: failed to get or open device for path {}",
            path
        );
        return;
    };

    if send_whatever.len() != 64 {
        tracing::warn!(
            "steamdeck_hid: send_whatever called with invalid length {} (expected 64)",
            send_whatever.len()
        );
        return;
    }

    tracing::trace!(
        "steamdeck_hid: sending {} bytes to path {}; first byte: {}",
        send_whatever.len(),
        path,
        send_whatever[0]
    );

    let mut buf = [0u8; 65];
    buf[1..].copy_from_slice(send_whatever);

    let res = unsafe { hidapi::SDL_hid_send_feature_report(dev, buf.as_ptr(), buf.len()) };
    if res < 0 {
        tracing::warn!(
            "steamdeck_hid: SDL_hid_send_feature_report failed for path {} (rc={})",
            path,
            res
        );
    }
}

pub fn cleanup_all() {
    tracing::debug!("steamdeck_hid: cleaning up all open devices");

    let Ok(mut map) = devs().lock() else {
        tracing::warn!("steamdeck_hid: failed to lock device cache during cleanup");
        return;
    };

    // Close all open devices
    for (path, dev) in map.drain() {
        if !dev.0.is_null() {
            unsafe { hidapi::SDL_hid_close(dev.0) };
            tracing::debug!("steamdeck_hid: closed device at path {}", path);
        }
    }

    // Clean up HID subsystem
    unsafe { hidapi::SDL_hid_exit() };
    tracing::debug!("steamdeck_hid: SDL_hid_exit() called");
}
