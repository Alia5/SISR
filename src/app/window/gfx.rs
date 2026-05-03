use std::sync::Arc;

use tracing::debug;
#[cfg(windows)]
use wgpu::Dx12BackendOptions;
use wgpu::{
    Backends, CompositeAlphaMode, Device, Instance, InstanceDescriptor, PresentMode, Queue,
    Surface, SurfaceConfiguration, TextureUsages,
};
#[cfg(windows)]
use wgpu_types::Dx12SwapchainKind;
use winit::window::Window;

pub struct Gfx {
    pub surface: Surface<'static>,
    pub device: Device,
    pub queue: Queue,
    pub config: SurfaceConfiguration,
    pub max_texture_dimension: u32,
}

impl Gfx {
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);

        // On Windows, use DX12 with DirectComposition (DxgiFromVisual) for transparency.
        // AMD's Vulkan driver on Windows only supports Opaque alpha mode because it
        // implements swapchains on top of DXGI. DirectComposition is required for transparency.
        // See: https://github.com/gfx-rs/wgpu/issues/5368
        //
        // On other platforms (Linux/Mac), use the default primary backend (Vulkan/Metal).
        #[cfg(windows)]
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::DX12,
            backend_options: wgpu::BackendOptions {
                dx12: Dx12BackendOptions {
                    presentation_system: Dx12SwapchainKind::DxgiFromVisual,
                    ..Default::default()
                },
                ..Default::default()
            },
            ..InstanceDescriptor::new_without_display_handle()
        });

        #[cfg(not(windows))]
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::PRIMARY | Backends::VULKAN | Backends::GL,
            ..InstanceDescriptor::new_without_display_handle()
        });

        let surface = instance
            .create_surface(window)
            .expect("Failed to create wgpu surface");
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to find an appropriate adapter");
        // let adapter_info = adapter.get_info();
        let required_features = wgpu::Features::empty();

        let device_desc = wgpu::DeviceDescriptor {
            label: None,
            required_features,
            required_limits: wgpu::Limits::default(),
            experimental_features: Default::default(),
            memory_hints: Default::default(),
            trace: Default::default(),
        };
        let (device, queue) = adapter
            .request_device(&device_desc)
            .await
            .expect("Failed to create device");
        let surface_caps = surface.get_capabilities(&adapter);
        debug!(
            "Surface capabilities: alpha_modes={:?}, formats={:?}",
            surface_caps.alpha_modes, surface_caps.formats
        );
        // Choose best alpha mode for transparency - prefer PreMultiplied, fall back to PostMultiplied or Auto
        let alpha_mode = if surface_caps
            .alpha_modes
            .contains(&CompositeAlphaMode::PreMultiplied)
        {
            CompositeAlphaMode::PreMultiplied
        } else if surface_caps
            .alpha_modes
            .contains(&CompositeAlphaMode::PostMultiplied)
        {
            CompositeAlphaMode::PostMultiplied
        } else {
            // Fallback - Auto will try to do the right thing
            CompositeAlphaMode::Auto
        };
        debug!("Using alpha_mode={:?}", alpha_mode);
        let preferred_formats = [
            wgpu::TextureFormat::Bgra8UnormSrgb,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            wgpu::TextureFormat::Bgra8Unorm,
            wgpu::TextureFormat::Rgba8Unorm,
        ];
        let format = surface_caps
            .formats
            .iter()
            .find(|f| {
                if preferred_formats.contains(f) {
                    return true;
                }
                let features = adapter.get_texture_format_features(**f);
                features
                    .allowed_usages
                    .contains(TextureUsages::RENDER_ATTACHMENT)
            })
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        debug!("Using format={:?}", format);
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode: PresentMode::Fifo,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);
        let max_texture_dimension = device.limits().max_texture_dimension_2d;
        Self {
            surface,
            device,
            queue,
            config,
            max_texture_dimension,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width.min(self.max_texture_dimension);
            self.config.height = height.min(self.max_texture_dimension);
            self.surface.configure(&self.device, &self.config);
        }
    }
}
