use std::sync::{Arc, OnceLock};

use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum TrayEvent {
    SetWindowState(bool),
    UpdateAvailable(String),
    ForceConfigChanged(bool),
}

static EVENT_SENDER: OnceLock<Arc<mpsc::UnboundedSender<TrayEvent>>> = OnceLock::new();

pub fn init(tx: mpsc::UnboundedSender<TrayEvent>) {
    EVENT_SENDER.set(Arc::new(tx)).unwrap();
}

pub fn get_event_sender() -> Arc<mpsc::UnboundedSender<TrayEvent>> {
    EVENT_SENDER.get().cloned().expect("Not initialized")
}

pub fn try_get_event_sender() -> Option<Arc<mpsc::UnboundedSender<TrayEvent>>> {
    EVENT_SENDER.get().cloned()
}

pub fn send_and_wake(event: TrayEvent) {
    if let Some(sender) = try_get_event_sender() {
        let _ = sender.send(event);
    }
    #[cfg(windows)]
    {
        use windows_sys::Win32::UI::WindowsAndMessaging::{PostThreadMessageW, WM_NULL};
        let thread_id = super::TRAY_THREAD_ID.load(std::sync::atomic::Ordering::SeqCst);
        if thread_id != 0 {
            unsafe { PostThreadMessageW(thread_id, WM_NULL, 0, 0); }
        }
    }
}
