use std::{env, sync::Arc};

use winit::window::Window;
use wry::WebViewBuilder;
#[cfg(target_os = "linux")]
use wry::Rect;

use crate::app::api::get_api_port;

pub struct WebView {
    webview: wry::WebView,
    visible: bool,
}

impl WebView {
    pub fn new(window: Arc<Window>) -> Self {
        let webview;

        #[cfg(target_os = "linux")]
        let _ = gtk::init();

        let webview_url = if env::var("DEV") == Ok("1".to_string()) {
            "http://localhost:5173/".to_string()
        } else {
            format!("http://localhost:{}/", get_api_port().unwrap_or(5173)) 
        };

        #[cfg(target_os = "linux")]
        let size = window.inner_size();
        #[cfg(target_os = "linux")]
        let bounds = Rect {
            position: wry::dpi::LogicalPosition::new(0.0, 0.0).into(),
            size: wry::dpi::LogicalSize::new(size.width as f64, size.height as f64).into(),
        };

        #[cfg(target_os = "linux")]
        {
            webview = if std::env::var_os("WAYLAND_DISPLAY").is_some() {
                WebViewBuilder::new()
                    .with_url(&webview_url)
                    .with_transparent(true)
                    .build(window.as_ref())
                    .expect("Failed to build webview")
            } else {
                WebViewBuilder::new()
                    .with_url(&webview_url)
                    .with_transparent(true)
                    .with_bounds(bounds)
                    .build_as_child(window.as_ref())
                    .unwrap_or_else(|_| {
                        WebViewBuilder::new()
                            .with_url(&webview_url)
                            .with_transparent(true)
                            .build(window.as_ref())
                            .expect("Failed to build webview")
                    })
            };
        }

        #[cfg(not(target_os = "linux"))]
        {
            use std::env;

            let mut builder = WebViewBuilder::new()
                .with_url(&webview_url)
                .with_transparent(true);

            if env::var("DEV") == Ok("1".to_string()) {
                use wry::WebViewBuilderExtWindows;
                tracing::info!("Enabling remote debugging for webview");
                builder = builder.with_additional_browser_args("--remote-debugging-port=9223");
            }

            webview = builder
                .build(window.as_ref())
                .expect("Failed to build webview");
        }

        Self {
            webview,
            visible: true,
        }
    }

    pub fn reload(&mut self) {
        if let Err(e) = self.webview.reload() {
            tracing::warn!("Failed to reload webview: {e}");
        }
    }

    pub fn invalidate_svelte_state(&mut self) {
        if let Err(e) = self.webview.evaluate_script("window.invalidateAll();") {
            tracing::warn!("Failed to invalidate Svelte state: {e}");
        }
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn show(&mut self) {
        self.visible = true;
        if let Err(e) = self.webview.set_visible(self.visible) {
            tracing::warn!("Failed to show webview: {e}");
        }
    }
    pub fn hide(&mut self) {
        self.visible = false;
        if let Err(e) = self.webview.set_visible(self.visible) {
            tracing::warn!("Failed to hide webview by default: {e}");
        }
        if let Err(e) = self.webview.focus_parent() {
            tracing::warn!("Failed to focus parent window after hiding webview: {e}");
        }
    }
}
