use crate::app::input::handler::ViiperEvent;
use crate::app::input::kbm_events;

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
}
