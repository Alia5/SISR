use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use sdl3::event::EventSender;
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc;
use tracing::{error, info, warn};
use viiper_client::AsyncViiperClient;
use viiper_client::devices::keyboard;
use viiper_client::devices::mouse;
use viiper_client::devices::xbox360;

use crate::app::input::device::Device;

type SdlWaker = Arc<Mutex<Option<EventSender>>>;
type OutputReader<R> = Arc<tokio::sync::Mutex<R>>;

pub(super) enum StreamCommand {
    SendXbox360Input(xbox360::Xbox360Input),
    SendKeyboardInput(keyboard::KeyboardInput),
    SendMouseInput(mouse::MouseInput),
}

pub enum ViiperEvent {
    ServerDisconnected {
        device_id: u64,
    },
    DeviceCreated {
        device_id: u64,
        viiper_device: viiper_client::Device,
    },
    DeviceConnected {
        device_id: u64,
    },
    DeviceRumble {
        device_id: u64,
        l: u8,
        r: u8,
    },

    //
    ErrorCreateDevice {
        device_id: u64,
    },
    ErrorConnectDevice {
        device_id: u64,
    },
}

pub(super) struct ViiperBridge {
    client: Option<Arc<AsyncViiperClient>>,
    bus_id: Arc<tokio::sync::Mutex<Option<u32>>>,
    stream_senders: Arc<Mutex<HashMap<u64, mpsc::UnboundedSender<StreamCommand>>>>,
    sdl_waker: SdlWaker,
    async_handle: tokio::runtime::Handle,
}

impl ViiperBridge {
    pub fn new(
        viiper_address: Option<SocketAddr>,
        sdl_waker: SdlWaker,
        async_handle: tokio::runtime::Handle,
    ) -> Self {
        Self {
            client: match viiper_address {
                Some(addr) => Some(Arc::new(AsyncViiperClient::new(addr))),
                None => {
                    warn!("No VIIPER address provided; VIIPER integration disabled");
                    None
                }
            },
            stream_senders: Arc::new(Mutex::new(HashMap::new())),
            sdl_waker,
            async_handle,
            bus_id: Arc::new(tokio::sync::Mutex::new(None)),
        }
    }

    pub fn create_device(&self, device: &Device) {
        let Some(client) = self.client.clone() else {
            error!("No VIIPER client available to create device");
            return Self::push_event(
                &self.sdl_waker,
                ViiperEvent::ErrorCreateDevice {
                    device_id: device.id,
                },
            );
        };
        let sdl_waker = self.sdl_waker.clone();
        let bus_id = self.bus_id.clone();
        let device_id = device.id;
        let device_type = device.viiper_type.clone();

        self.async_handle.spawn(async move {
            let bus_id = {
                let mut bus_guard = bus_id.lock().await;
                let id = match Self::ensure_bus(&client, *bus_guard).await {
                    Ok(id) => id,
                    Err(e) => {
                        error!("Failed to ensure VIIPER bus exists: {}", e);
                        return Self::push_event(
                            &sdl_waker,
                            ViiperEvent::ErrorCreateDevice { device_id },
                        );
                    }
                };

                *bus_guard = Some(id);
                id
            };

            let response = match client
                .bus_device_add(
                    bus_id,
                    &viiper_client::types::DeviceCreateRequest {
                        r#type: Some(device_type),
                        id_vendor: None,
                        id_product: None,
                    },
                )
                .await
            {
                Ok(resp) => resp,
                Err(e) => {
                    error!("Failed to create VIIPER device: {}", e);
                    return Self::push_event(
                        &sdl_waker,
                        ViiperEvent::ErrorCreateDevice { device_id },
                    );
                }
            };
            info!("Created VIIPER device with {:?}", response);
            Self::push_event(
                &sdl_waker,
                ViiperEvent::DeviceCreated {
                    device_id,
                    viiper_device: response,
                },
            );
        });
    }

    pub fn connect_device(&mut self, device: &Device) {
        let Some(viiper_dev) = device.viiper_device.clone() else {
            error!("No VIIPER client available to create device");
            return Self::push_event(
                &self.sdl_waker,
                ViiperEvent::ErrorConnectDevice {
                    device_id: device.id,
                },
            );
        };

        let Some(client) = self.client.clone() else {
            error!("No VIIPER client available to create device");
            return Self::push_event(
                &self.sdl_waker,
                ViiperEvent::ErrorConnectDevice {
                    device_id: device.id,
                },
            );
        };
        let sdl_waker = self.sdl_waker.clone();
        let stream_senders = self.stream_senders.clone();
        let device_id = device.id;
        let device_type = device.viiper_type.clone();

        self.async_handle.spawn(async move {
            let mut dev_stream = match client
                .connect_device(viiper_dev.bus_id, &viiper_dev.dev_id)
                .await
            {
                Ok(stream) => stream,
                Err(e) => {
                    error!("Failed to connect VIIPER device: {}", e);
                    return Self::push_event(
                        &sdl_waker,
                        ViiperEvent::ErrorConnectDevice { device_id },
                    );
                }
            };
            let disco_sdl_waker = sdl_waker.clone();
            dev_stream
                .on_disconnect(move || {
                    info!("VIIPER server disconnected device {}", device_id);
                    Self::push_event(
                        &disco_sdl_waker,
                        ViiperEvent::ServerDisconnected { device_id },
                    );
                })
                .map_err(|e| {
                    error!(
                        "Failed to set disconnect callback for VIIPER device {}: {}",
                        device_id, e
                    );
                })
                .ok();

            let output_sdl_waker = sdl_waker.clone();
            let device_type_clone = device_type.clone();
            dev_stream
                .on_output(move |reader| {
                    let sdl_waker = output_sdl_waker.clone();
                    let dev_type = device_type_clone.clone();
                    async move {
                        Self::handle_device_output(reader, sdl_waker, device_id, &dev_type).await
                    }
                })
                .map_err(|e| {
                    error!(
                        "Failed to set output callback for VIIPER device {}: {}",
                        device_id, e
                    );
                })
                .ok();

            let (tx, mut rx) = mpsc::unbounded_channel::<StreamCommand>();
            if let Ok(mut senders) = stream_senders.lock() {
                senders.insert(device_id, tx);
            } else {
                error!("Failed to lock VIIPER stream senders");
            }

            info!("Connected VIIPER device {:?}", viiper_dev);
            Self::push_event(&sdl_waker, ViiperEvent::DeviceConnected { device_id });

            while let Some(cmd) = rx.recv().await {
                let e = match cmd {
                    StreamCommand::SendXbox360Input(input) => {
                        if device_type != "xbox360" {
                            warn!(
                                "Ignoring xbox360 input for non-xbox360 device {} (type={})",
                                device_id, device_type
                            );
                            continue;
                        }
                        dev_stream.send(&input).await.err()
                    }
                    StreamCommand::SendKeyboardInput(input) => {
                        if device_type != "keyboard" {
                            warn!(
                                "Ignoring keyboard input for non-keyboard device {} (type={})",
                                device_id, device_type
                            );
                            continue;
                        }
                        dev_stream.send(&input).await.err()
                    }
                    StreamCommand::SendMouseInput(input) => {
                        if device_type != "mouse" {
                            warn!(
                                "Ignoring mouse input for non-mouse device {} (type={})",
                                device_id, device_type
                            );
                            continue;
                        }
                        dev_stream.send(&input).await.err()
                    }
                };
                if let Some(err) = e {
                    error!(
                        "Failed to send input to VIIPER device {}: {}",
                        device_id, err
                    );
                }
            }
        });
    }

    async fn ensure_bus(client: &AsyncViiperClient, bus_id: Option<u32>) -> Result<u32> {
        if let Some(id) = bus_id {
            let buses = client.bus_list().await?;
            if buses.buses.contains(&id) {
                return Ok(id);
            }
            warn!("Bus {} no longer exists, recreating...", id);
        }

        let response = client.bus_create(None).await?;

        info!("Created VIIPER bus with ID {}", response.bus_id);
        Ok(response.bus_id)
    }

    pub fn remove_device(&mut self, device_id: u64) {
        if let Ok(mut senders) = self.stream_senders.lock()
            && senders.remove(&device_id).is_some()
        {
            info!("Disconnected VIIPER device with ID {}", device_id);
        }
    }

    pub fn update_device_state(&self, device_id: u64, cmd: StreamCommand) {
        let Ok(senders) = self.stream_senders.lock() else {
            error!("Failed to lock VIIPER stream senders");
            return;
        };
        if let Some(tx) = senders.get(&device_id) {
            if let Err(e) = tx.send(cmd) {
                error!("Failed to send input to VIIPER device {}: {}", device_id, e);
            }
        } else {
            // warn!("No stream sender found for VIIPER device {}", device_id);
        }
    }

    fn push_event(sdl_waker: &SdlWaker, event: ViiperEvent) {
        if let Ok(guard) = sdl_waker.lock()
            && let Some(sender) = &*guard
            && let Err(e) = sender.push_custom_event(super::HandlerEvent::ViiperEvent(event))
        {
            error!("Failed to push VIIPER event: {}", e);
        }
    }

    async fn handle_device_output<R>(
        reader: OutputReader<R>,
        sdl_waker: SdlWaker,
        device_id: u64,
        device_type: &str,
    ) -> std::io::Result<()>
    where
        R: tokio::io::AsyncRead + Unpin + Send,
    {
        match device_type {
            "xbox360" => Self::process_xbox360_rumble_output(reader, &sdl_waker, device_id).await,
            "keyboard" => Self::process_keyboard_output(reader, &sdl_waker, device_id).await,
            _ => {
                warn!("Unknown device type for output: {}", device_type);
                reader.lock().await.read_to_end(&mut vec![]).await?;
                Ok(())
            }
        }
    }

    async fn process_xbox360_rumble_output<R>(
        reader: OutputReader<R>,
        sdl_waker: &SdlWaker,
        device_id: u64,
    ) -> std::io::Result<()>
    where
        R: tokio::io::AsyncRead + Unpin + Send,
    {
        let mut buf = vec![0u8; xbox360::OUTPUT_SIZE];
        let mut guard = reader.lock().await;
        guard.read_exact(&mut buf).await?;
        drop(guard);

        if buf.len() < 2 {
            warn!(
                "VIIPER xbox360 output too short for device {} (len={})",
                device_id,
                buf.len()
            );
            return Ok(());
        }

        Self::push_event(
            sdl_waker,
            ViiperEvent::DeviceRumble {
                device_id,
                l: buf[0],
                r: buf[1],
            },
        );
        Ok(())
    }

    async fn process_keyboard_output<R>(
        reader: OutputReader<R>,
        _sdl_waker: &SdlWaker,
        _device_id: u64,
    ) -> std::io::Result<()>
    where
        R: tokio::io::AsyncRead + Unpin + Send,
    {
        // TODO:
        let mut buf = vec![0u8; keyboard::OUTPUT_SIZE];
        let mut guard = reader.lock().await;
        guard.read_exact(&mut buf).await?;
        drop(guard);
        Ok(())
    }
}
