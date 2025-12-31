use std::mem::discriminant;
use std::sync::{Arc, Mutex};

use sdl3_sys::events::SDL_Event;

use crate::app::input::context::Context;
use crate::app::input::device::ViiperDevice;
use crate::app::input::device_state::DeviceState;
use crate::app::input::event::handler::on_viiper_event::device_output::{keyboard, xbox360};
use crate::app::input::event::handler_events::HandlerEvent;
use crate::app::input::event::router::{EventHandler, ListenEvent, RoutedEvent};
use crate::app::input::sdl_loop::Subsystems;
use crate::app::input::viiper_bridge::{DeviceOutput, ViiperBridge, ViiperEvent};
use crate::app::steam_utils::binding_enforcer::binding_enforcer;
use crate::app::window;

pub struct Handler {
    ctx: Arc<Mutex<Context>>,
    viiper_bridge: Arc<Mutex<ViiperBridge>>,
}
impl Handler {
    pub fn new(ctx: Arc<Mutex<Context>>, viiper_bridge: Arc<Mutex<ViiperBridge>>) -> Self {
        Self { ctx, viiper_bridge }
    }
}

impl EventHandler for Handler {
    fn handle_event(
        &self,
        _subsystems: &Subsystems,
        event: &Option<RoutedEvent>,
        _sdl_event: &SDL_Event,
    ) {
        tracing::debug!(event = ?event);
        let event = match event {
            Some(RoutedEvent::UserEvent(event)) => event,
            _ => {
                tracing::warn!("Received non-handler event ");
                return;
            }
        };
        let viiper_event = match event {
            HandlerEvent::ViiperEvent(event) => event,
            _ => {
                tracing::warn!("Received non-ViiperEvent event ");
                return;
            }
        };

        match viiper_event {
            ViiperEvent::DeviceCreated {
                device_id,
                viiper_device,
            } => {
                let Ok(ctx) = self.ctx.lock() else {
                    tracing::error!("Failed to lock state for VIIPER device created handling");
                    return;
                };
                let Some(device_mtx) = ctx.device_for_id(*device_id) else {
                    tracing::warn!("Received created event for unknown device ID {}", device_id);
                    return;
                };
                drop(ctx);

                let Ok(mut device) = device_mtx.lock() else {
                    tracing::error!(
                        "Failed to lock device mutex for VIIPER device created handling"
                    );
                    return;
                };

                device.viiper_type = Some(viiper_device.r#type.clone());
                device.viiper_device = Some(ViiperDevice {
                    device: viiper_device.clone(),
                    state: DeviceState::Empty,
                });

                let Ok(mut viiper) = self.viiper_bridge.lock() else {
                    tracing::error!("Failed to lock ViiperBridge mutex");
                    return;
                };
                viiper.connect_device(&device);
                window::request_redraw();
            }
            ViiperEvent::DeviceConnected { device_id } => {
                let Ok(ctx) = self.ctx.lock() else {
                    tracing::error!("Failed to lock state for VIIPER device connected handling");
                    return;
                };
                let Some(device_mtx) = ctx.device_for_id(*device_id) else {
                    tracing::warn!(
                        "Received connected event for unknown device ID {}",
                        device_id
                    );
                    return;
                };
                drop(ctx);
                let Ok(device) = device_mtx.lock() else {
                    tracing::error!(
                        "Failed to lock device mutex for VIIPER device connected handling"
                    );
                    return;
                };
                if device.steam_handle != 0 {
                    let Ok(mut enforcer) = binding_enforcer().lock() else {
                        tracing::error!("Failed to lock binding enforcer mutex");
                        window::request_redraw();

                        return;
                    };
                    if !enforcer.is_active() {
                        enforcer.activate();
                    }
                }
                window::request_redraw();
            }
            //
            ViiperEvent::ServerDisconnected { device_id } => {
                let Ok(ctx) = self.ctx.lock() else {
                    tracing::error!("Failed to lock state for VIIPER server disconnected handling");
                    return;
                };
                let Some(device_mtx) = ctx.device_for_id(*device_id) else {
                    tracing::warn!(
                        "Received server disconnected event for unknown device ID {}",
                        device_id
                    );
                    return;
                };
                drop(ctx);
                let Ok(mut device) = device_mtx.lock() else {
                    tracing::error!(
                        "Failed to lock device mutex for VIIPER server disconnected handling"
                    );
                    return;
                };
                device.viiper_device = None;
                if device.steam_handle != 0 {
                    let Ok(mut enforcer) = binding_enforcer().lock() else {
                        tracing::error!("Failed to lock binding enforcer mutex");
                        window::request_redraw();

                        return;
                    };
                    if enforcer.is_active() {
                        enforcer.deactivate();
                    }
                }
                window::request_redraw();
            }
            //
            ViiperEvent::DeviceOutput(output) => match output {
                DeviceOutput::Xbox360 {
                    device_id,
                    rumble_l,
                    rumble_r,
                } => {
                    xbox360::handle_output(self.ctx.clone(), device_id, rumble_l, rumble_r);
                }
                DeviceOutput::Keyboard { device_id, leds } => {
                    keyboard::handle_output(self.ctx.clone(), device_id, leds);
                }
            },
            //
            ViiperEvent::ErrorCreateDevice { device_id } => {
                tracing::error!("Error creating VIIPER device with ID {}", device_id);
                let Ok(ctx) = self.ctx.lock() else {
                    tracing::error!("Failed to lock state for VIIPER server disconnected handling");
                    return;
                };
                let Some(device_mtx) = ctx.device_for_id(*device_id) else {
                    tracing::warn!(
                        "Received server disconnected event for unknown device ID {}",
                        device_id
                    );
                    return;
                };
                drop(ctx);
                let Ok(mut device) = device_mtx.lock() else {
                    tracing::error!(
                        "Failed to lock device mutex for VIIPER server disconnected handling"
                    );
                    return;
                };
                device.viiper_device = None;
                window::request_redraw();
            }
            ViiperEvent::ErrorConnectDevice { device_id } => {
                tracing::error!("Error connecting VIIPER device with ID {}", device_id);
                let Ok(ctx) = self.ctx.lock() else {
                    tracing::error!("Failed to lock state for VIIPER server disconnected handling");
                    return;
                };
                let Some(device_mtx) = ctx.device_for_id(*device_id) else {
                    tracing::warn!(
                        "Received server disconnected event for unknown device ID {}",
                        device_id
                    );
                    return;
                };
                drop(ctx);
                let Ok(mut device) = device_mtx.lock() else {
                    tracing::error!(
                        "Failed to lock device mutex for VIIPER server disconnected handling"
                    );
                    return;
                };
                device.viiper_device = None;
                window::request_redraw();
            }
        };
    }

    fn listen_events(&self) -> Vec<ListenEvent> {
        vec![
            // TODO: FIXME: Currently listening for all Viiper events, doesn't care which we pass for discriminant
            ListenEvent::HandlerEvent(discriminant(&HandlerEvent::ViiperEvent(
                ViiperEvent::ServerDisconnected { device_id: 0 },
            ))),
        ]
    }
}
