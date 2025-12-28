use std::any::Any;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use dashmap::DashMap;
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc;
use tracing::{error, info, warn};
use viiper_client::devices::keyboard;
use viiper_client::devices::mouse;
use viiper_client::devices::xbox360;
use viiper_client::{AsyncViiperClient, DeviceInput};

use crate::app::core::get_tokio_handle;
use crate::app::input::device::Device;
use crate::app::input::event::handler_events::HandlerEvent;
use crate::app::input::sdl_loop;

type OutputReader<R> = Arc<tokio::sync::Mutex<R>>;

type StreamSender = mpsc::UnboundedSender<Box<dyn Any + Send>>;

#[derive(Debug)]
pub enum DeviceOutput {
    Xbox360 {
        device_id: u64,
        rumble_l: u8,
        rumble_r: u8,
    },
    Keyboard {
        device_id: u64,
        leds: u8,
    },
}

#[derive(Debug)]
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
    //
    DeviceOutput(DeviceOutput),
    //
    ErrorCreateDevice {
        device_id: u64,
    },
    ErrorConnectDevice {
        device_id: u64,
    },
}

pub struct ViiperBridge {
    client: Option<Arc<AsyncViiperClient>>,
    bus_id: Arc<tokio::sync::Mutex<Option<u32>>>,
    stream_senders: Arc<Mutex<HashMap<u64, StreamSender>>>,
    viiper_ready: bool,
    viiper_version: Option<String>,
    create_schedule: DashMap<u64, String>,
}

impl ViiperBridge {
    pub fn new(viiper_address: Option<SocketAddr>) -> Self {
        Self {
            client: match viiper_address {
                Some(addr) => Some(Arc::new(AsyncViiperClient::new(addr))),
                None => {
                    warn!("No VIIPER address provided; VIIPER integration disabled");
                    None
                }
            },
            stream_senders: Arc::new(Mutex::new(HashMap::new())),
            bus_id: Arc::new(tokio::sync::Mutex::new(None)),
            viiper_ready: false,
            viiper_version: None,
            create_schedule: DashMap::new(),
        }
    }

    pub fn set_ready(&mut self, version: &str) {
        self.viiper_ready = true;
        self.viiper_version = Some(version.to_string());

        let scheduled: Vec<(u64, String)> = self
            .create_schedule
            .iter()
            .map(|e| (*e.key(), e.value().clone()))
            .collect();

        for (id, viiper_type) in scheduled {
            info!(
                "Creating scheduled VIIPER device {} of type {}",
                id, viiper_type
            );
            self.create_device(id, &viiper_type);
        }
        self.create_schedule.clear();
    }

    pub fn is_ready(&self) -> bool {
        self.viiper_ready
    }

    pub fn get_version(&self) -> Option<String> {
        self.viiper_version.clone()
    }

    pub fn create_device(&self, id: u64, viiper_type: &str) {
        let Some(client) = self.client.clone() else {
            error!("No VIIPER client available to create device");
            return Self::push_event(ViiperEvent::ErrorCreateDevice { device_id: id });
        };

        if !self.viiper_ready {
            self.create_schedule.insert(id, viiper_type.to_string());
            info!(
                "VIIPER not ready; scheduling creation of device {} of type {}",
                id, viiper_type
            );
            return;
        }

        let bus_id = self.bus_id.clone();
        let device_id = id;
        let device_type = viiper_type.to_string();

        get_tokio_handle().spawn(async move {
            let bus_id = {
                let mut bus_guard = bus_id.lock().await;
                let id = match Self::ensure_bus(&client, *bus_guard).await {
                    Ok(id) => id,
                    Err(e) => {
                        error!("Failed to ensure VIIPER bus exists: {}", e);
                        return Self::push_event(ViiperEvent::ErrorCreateDevice { device_id });
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
                    return Self::push_event(ViiperEvent::ErrorCreateDevice { device_id });
                }
            };
            info!("Created VIIPER device {:?}", response);
            Self::push_event(ViiperEvent::DeviceCreated {
                device_id,
                viiper_device: response,
            });
        });
    }

    pub fn connect_device(&mut self, device: &Device) {
        let Some(viiper_dev) = device.viiper_device.as_ref() else {
            tracing::error!("No VIIPER client available to create device");
            return Self::push_event(ViiperEvent::ErrorConnectDevice {
                device_id: device.id,
            });
        };
        let viiper_dev = viiper_dev.device.clone();

        let Some(client) = self.client.clone() else {
            tracing::error!("No VIIPER client available to create device");
            return Self::push_event(ViiperEvent::ErrorConnectDevice {
                device_id: device.id,
            });
        };
        let stream_senders = self.stream_senders.clone();
        let device_id = device.id;
        let Some(device_type) = device.viiper_type.clone() else {
            tracing::error!("Cannot connect created viiper device without type!");
            Self::push_event(ViiperEvent::ErrorConnectDevice { device_id });
            return;
        };

        get_tokio_handle().spawn(async move {
            let mut dev_stream = match client
                .connect_device(viiper_dev.bus_id, &viiper_dev.dev_id)
                .await
            {
                Ok(stream) => stream,
                Err(e) => {
                    error!("Failed to connect VIIPER device: {}", e);
                    return Self::push_event(ViiperEvent::ErrorConnectDevice { device_id });
                }
            };
            dev_stream
                .on_disconnect(move || {
                    info!("VIIPER server disconnected device {}", device_id);
                    Self::push_event(ViiperEvent::ServerDisconnected { device_id });
                })
                .map_err(|e| {
                    error!(
                        "Failed to set disconnect callback for VIIPER device {}: {}",
                        device_id, e
                    );
                })
                .ok();

            let device_type_clone = device_type.clone();
            dev_stream
                .on_output(move |reader| {
                    let dev_type = device_type_clone.clone();
                    async move { Self::handle_device_output(reader, device_id, &dev_type).await }
                })
                .map_err(|e| {
                    error!(
                        "Failed to set output callback for VIIPER device {}: {}",
                        device_id, e
                    );
                })
                .ok();

            let (tx, mut rx) = mpsc::unbounded_channel::<Box<dyn Any + Send>>();
            if let Ok(mut senders) = stream_senders.lock() {
                senders.insert(device_id, tx);
            } else {
                error!("Failed to lock VIIPER stream senders");
            }

            info!("Connected VIIPER device {:?}", viiper_dev);
            Self::push_event(ViiperEvent::DeviceConnected { device_id });

            macro_rules! forward_loop {
                ($type_name:literal, $input_ty:ty) => {{
                    while let Some(input) = rx.recv().await {
                        let Ok(input) = input.downcast::<$input_ty>() else {
                            warn!(
                                "Dropping invalid input for VIIPER {} device {}",
                                $type_name, device_id
                            );
                            continue;
                        };
                        if let Err(e) = dev_stream.send(&*input).await {
                            error!(
                                "Failed to send input to VIIPER {} device {}: {}",
                                $type_name, device_id, e
                            );
                        }
                    }
                }};
            }

            match device_type.as_str() {
                "xbox360" => forward_loop!("xbox360", xbox360::Xbox360Input),
                "keyboard" => forward_loop!("keyboard", keyboard::KeyboardInput),
                "mouse" => forward_loop!("mouse", mouse::MouseInput),
                _ => {
                    warn!("Unhandled VIIPER device type: {}", device_type);
                    while rx.recv().await.is_some() {}
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

    pub fn update_device_state<T>(&self, device_id: u64, input: T)
    where
        T: DeviceInput + Any + Send + 'static,
    {
        let Ok(senders) = self.stream_senders.lock() else {
            error!("Failed to lock VIIPER stream senders");
            return;
        };
        if let Some(tx) = senders.get(&device_id) {
            if let Err(e) = tx.send(Box::new(input)) {
                error!("Failed to send input to VIIPER device {}: {}", device_id, e);
            }
        } else {
            // warn!("No stream sender found for VIIPER device {}", device_id);
        }
    }

    pub fn update_device_state_boxed(&self, device_id: u64, input: Box<dyn Any + Send>) {
        let Ok(senders) = self.stream_senders.lock() else {
            error!("Failed to lock VIIPER stream senders");
            return;
        };
        if let Some(tx) = senders.get(&device_id) {
            if let Err(e) = tx.send(input) {
                error!("Failed to send input to VIIPER device {}: {}", device_id, e);
            }
        } else {
            // warn!("No stream sender found for VIIPER device {}", device_id);
        }
    }

    fn push_event(event: ViiperEvent) {
        if let Err(e) =
            sdl_loop::get_event_sender().push_custom_event(HandlerEvent::ViiperEvent(event))
        {
            error!("Failed to push VIIPER event: {}", e);
        }
    }

    async fn handle_device_output<R>(
        reader: OutputReader<R>,
        device_id: u64,
        device_type: &str,
    ) -> std::io::Result<()>
    where
        R: tokio::io::AsyncRead + Unpin + Send,
    {
        match device_type {
            "xbox360" => Self::process_xbox360_rumble_output(reader, device_id).await,
            "keyboard" => Self::process_keyboard_output(reader, device_id).await,
            _ => {
                warn!("Unknown device type for output: {}", device_type);
                reader.lock().await.read_to_end(&mut vec![]).await?;
                Ok(())
            }
        }
    }

    async fn process_xbox360_rumble_output<R>(
        reader: OutputReader<R>,
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

        Self::push_event(ViiperEvent::DeviceOutput(DeviceOutput::Xbox360 {
            device_id,
            rumble_l: buf[0],
            rumble_r: buf[1],
        }));
        Ok(())
    }

    async fn process_keyboard_output<R>(
        reader: OutputReader<R>,
        _device_id: u64,
    ) -> std::io::Result<()>
    where
        R: tokio::io::AsyncRead + Unpin + Send,
    {
        // TODO: FIXME:
        let mut buf = vec![0u8; keyboard::OUTPUT_SIZE];
        let mut guard = reader.lock().await;
        guard.read_exact(&mut buf).await?;
        drop(guard);
        Ok(())
    }
}
