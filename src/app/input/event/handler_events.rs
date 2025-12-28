use crate::app::input::kbm_events;
use crate::app::input::viiper_bridge::ViiperEvent;

#[derive(Debug)]
pub enum HandlerEvent {
    ViiperEvent(ViiperEvent),
    IgnoreDevice { device_id: u64 },
    ConnectViiperDevice { device_id: u64 },
    DisconnectViiperDevice { device_id: u64 },
    CefDebugReady { port: u16 },
    OverlayStateChanged { open: bool },
    SetKbmEmulation { enabled: bool, initialize: bool },
    KbmKeyEvent(kbm_events::KbmKeyEvent),
    KbmPointerEvent(kbm_events::KbmPointerEvent),
    KbmReleaseAll(),
    ViiperReady { version: String },
}
