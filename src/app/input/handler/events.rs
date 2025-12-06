use crate::app::input::handler::ViiperEvent;

pub enum HandlerEvent {
    ViiperEvent(ViiperEvent),
    IgnoreDeviceEvent { device_id: u64 },
}
