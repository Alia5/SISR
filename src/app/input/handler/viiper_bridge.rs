use std::collections::HashMap;
use std::net::SocketAddr;

use anyhow::{Result, anyhow};
use tracing::{info, warn};
use viiper_client::{DeviceStream, ViiperClient};

use crate::app::input::device::Device;

pub(super) struct ViiperBridge {
    client: Option<ViiperClient>,
    // TODO: handle device_disconnect from SERVER
    streams: HashMap<u32, DeviceStream>,
}

impl ViiperBridge {
    pub fn new(viiper_address: Option<SocketAddr>) -> Self {
        Self {
            client: match viiper_address {
                Some(addr) => Some(ViiperClient::new(addr)),
                None => {
                    warn!("No VIIPER address provided; VIIPER integration disabled");
                    None
                }
            },
            streams: HashMap::new(),
        }
    }

    pub fn create_device(&self, device: &mut Device, bus_id: &mut Option<u32>) -> Result<()> {
        let bus_id = match bus_id {
            Some(id) => *id,
            None => self.create_bus(bus_id)?,
        };

        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow!("No VIIPER client available"))?;

        let response = client
            .bus_device_add(
                bus_id,
                &viiper_client::types::DeviceCreateRequest {
                    r#type: Some(device.viiper_type.clone()),
                    id_vendor: None,
                    id_product: None,
                },
            )
            .map_err(|e| anyhow!("Failed to create VIIPER device: {}", e))?;

        info!("Created VIIPER device with {:?}", response);
        device.viiper_device = Some(response);
        Ok(())
    }

    pub fn connect_device(&mut self, device: &mut Device) -> Result<()> {
        let viiper_dev = device
            .viiper_device
            .as_ref()
            .ok_or_else(|| anyhow!("Device has no VIIPER device"))?;

        let client = self
            .client
            .as_mut()
            .ok_or_else(|| anyhow!("No VIIPER client available"))?;

        let dev_stream = client
            .connect_device(viiper_dev.bus_id, &viiper_dev.dev_id)
            .map_err(|e| anyhow!("Failed to connect VIIPER device: {}", e))?;

        self.streams.insert(device.id, dev_stream);
        info!("Connected VIIPER device {:?}", device.viiper_device);
        Ok(())
    }

    pub fn create_bus(&self, bus_id: &mut Option<u32>) -> Result<u32> {
        if bus_id.is_some() {
            warn!("VIIPER bus already created; Recreating");
        }

        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow!("No VIIPER client available"))?;

        let response = client
            .bus_create(None)
            .map_err(|e| anyhow!("Failed to create VIIPER bus: {}", e))?;

        info!("Created VIIPER bus with ID {}", response.bus_id);
        *bus_id = Some(response.bus_id);
        Ok(response.bus_id)
    }

    pub fn disconnect_device(&mut self, which: u32) {
        if self.streams.remove(&which).is_some() {
            info!("Disconnected VIIPER device with ID {}", which);
        } else {
            warn!("No VIIPER device found with ID {}", which);
        }
    }
}
