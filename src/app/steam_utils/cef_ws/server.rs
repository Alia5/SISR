use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, error, info, warn};
use winit::event_loop::EventLoopProxy;

use crate::app::steam_utils::cef_ws::CefMessage;
use crate::app::window::RunnerEvent;

use super::handler::Handler;
use super::messages::WsResponse;

pub struct WebSocketServer {
    listener: TcpListener,
    port: u16,
}

impl WebSocketServer {
    pub async fn new() -> Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .context("Failed to bind WebSocket server to random port")?;

        let addr = listener
            .local_addr()
            .context("Failed to get local address of WebSocket server")?;
        let port = addr.port();

        info!("CEF Debug WebSocket server bound to port {}", port);

        Ok(Self { listener, port })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn run(
        self,
        handle: tokio::runtime::Handle,
        winit_waker: Arc<Mutex<Option<EventLoopProxy<RunnerEvent>>>>,
        sdl_waker: Arc<Mutex<Option<sdl3::event::EventSender>>>,
    ) {
        handle.spawn(async move {
            info!("CEF Debug WebSocket server listening on port {}", self.port);

            loop {
                match self.listener.accept().await {
                    Ok((stream, addr)) => {
                        debug!("New CEF Debug WebSocket connection from: {}", addr);
                        let winit_waker = winit_waker.clone();
                        let sdl_waker = sdl_waker.clone();

                        tokio::spawn(async move {
                            if let Err(e) =
                                Self::handle_connection(stream, addr, winit_waker, sdl_waker).await
                            {
                                error!(
                                    "Error handling CEF Debug WebSocket connection from {}: {}",
                                    addr, e
                                );
                            }
                        });
                    }
                    Err(e) => {
                        error!("Failed to accept CEF Debug WebSocket connection: {}", e);
                    }
                }
            }
        });
    }

    async fn handle_connection(
        stream: TcpStream,
        addr: SocketAddr,
        winit_waker: Arc<Mutex<Option<EventLoopProxy<RunnerEvent>>>>,
        sdl_waker: Arc<Mutex<Option<sdl3::event::EventSender>>>,
    ) -> Result<()> {
        let ws_stream = tokio_tungstenite::accept_async(stream)
            .await
            .context("Failed to accept WebSocket handshake")?;

        info!("CEF Debug WebSocket connection established with {}", addr);

        Self::process_messages(ws_stream, addr, winit_waker, sdl_waker).await
    }

    async fn process_messages(
        mut ws_stream: WebSocketStream<TcpStream>,
        addr: SocketAddr,
        winit_waker: Arc<Mutex<Option<EventLoopProxy<RunnerEvent>>>>,
        sdl_waker: Arc<Mutex<Option<sdl3::event::EventSender>>>,
    ) -> Result<()> {
        let handler = Handler::new(winit_waker, sdl_waker);

        while let Some(msg) = ws_stream.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    debug!("CEF Debug WebSocket received from {}: {}", addr, text);

                    let response = match serde_json::from_str::<CefMessage>(&text) {
                        Ok(message) => handler.handle(message),
                        Err(e) => {
                            warn!(
                                "Failed to parse CEF Debug WebSocket message from {}: {}",
                                addr, e
                            );
                            WsResponse::error(format!("Invalid message: {}", e))
                        }
                    };

                    let response_text = serde_json::to_string(&response)
                        .context("Failed to serialize WebSocket response")?;

                    ws_stream
                        .send(Message::Text(response_text.into()))
                        .await
                        .context("Failed to send WebSocket response")?;
                }
                Ok(Message::Close(_)) => {
                    info!("CEF Debug WebSocket connection closed by client: {}", addr);
                    break;
                }
                Ok(Message::Ping(data)) => {
                    ws_stream
                        .send(Message::Pong(data))
                        .await
                        .context("Failed to send pong")?;
                }
                Ok(_) => {
                    // Ignore other message types (Binary, Pong, Frame)
                }
                Err(e) => {
                    error!("CEF Debug WebSocket error from {}: {}", addr, e);
                    break;
                }
            }
        }

        info!("CEF Debug WebSocket connection closed with {}", addr);
        Ok(())
    }
}
