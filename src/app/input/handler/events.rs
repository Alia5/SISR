use crate::app::input::handler::ViiperEvent;

pub enum HandlerEvent {
    ViiperEvent(ViiperEvent),
    IgnoreDevice { device_id: u64 },
    ConnectViiperDevice { device_id: u64 },
    DisconnectViiperDevice { device_id: u64 },
    CefDebugReady { port: u16 },
    OverlayStateChanged { open: bool },
}
