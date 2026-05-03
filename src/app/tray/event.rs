use std::sync::{Arc, OnceLock};

use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum TrayEvent {
    SetWindowState(bool),
}

static EVENT_SENDER: OnceLock<Arc<mpsc::UnboundedSender<TrayEvent>>> = OnceLock::new();

pub fn init(tx: mpsc::UnboundedSender<TrayEvent>) {
    EVENT_SENDER.set(Arc::new(tx)).unwrap();
}

pub fn get_event_sender() -> Arc<mpsc::UnboundedSender<TrayEvent>> {
    EVENT_SENDER.get().cloned().expect("Not initialized")
}
