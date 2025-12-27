use crate::app::input_old::handler::ViiperEvent;
use crate::app::input_old::kbm_events;

#[derive(Debug)]
pub enum HandlerEvent {
    ViiperEvent(ViiperEvent),
    IgnoreDevice { device_id: u64 },
    ConnectViiperDevice { device_id: u64 },
    DisconnectViiperDevice { device_id: u64 },
    CefDebugReady { port: u16 },
    OverlayStateChanged { open: bool },
    SetKbmEmulationEnabled { enabled: bool },
    KbmKeyEvent(kbm_events::KbmKeyEvent),
    KbmPointerEvent(kbm_events::KbmPointerEvent),
    KbmReleaseAll(),
    ViiperReady { version: String },
}
