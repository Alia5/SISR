use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, error, info, trace, warn};
use winit::event_loop::EventLoopProxy;

use serde::Serialize;

use crate::app::steam_utils::cef_ws::CefMessage;
use crate::app::window::RunnerEvent;

use super::handler::Handler;
use super::messages::WsResponse;

type BroadcastSender = mpsc::UnboundedSender<Message>;

pub struct WebSocketServer {
    port: u16,
    connections: Arc<Mutex<Vec<BroadcastSender>>>,
}

impl WebSocketServer {
    pub async fn new() -> Result<(Self, TcpListener)> {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .context("Failed to bind WebSocket server to random port")?;

        let addr = listener
            .local_addr()
            .context("Failed to get local address of WebSocket server")?;
        let port = addr.port();

        info!("CEF Debug WebSocket server bound to port {}", port);

        Ok((
            Self {
                port,
                connections: Arc::new(Mutex::new(Vec::new())),
            },
            listener,
        ))
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn broadcast<T: Serialize>(&self, message: T) {
        let json = match serde_json::to_string(&message) {
            Ok(json) => json,
            Err(e) => {
                warn!("Failed to serialize broadcast message: {}", e);
                return;
            }
        };

        if let Ok(mut connections) = self.connections.lock() {
            if connections.is_empty() {
                trace!("No active WebSocket connections to broadcast to");
                return;
            }

            debug!("Broadcasting message to {} connections", connections.len());

            connections.retain(|sender| sender.send(Message::Text(json.clone().into())).is_ok());
        } else {
            warn!("Failed to acquire lock to broadcast message");
        }
    }

    pub fn run(
        self,
        listener: TcpListener,
        handle: tokio::runtime::Handle,
        winit_waker: Arc<Mutex<Option<EventLoopProxy<RunnerEvent>>>>,
        sdl_waker: Arc<Mutex<Option<sdl3::event::EventSender>>>,
    ) {
        let connections = self.connections.clone();
        let port = self.port;

        super::broadcast::set_server(Arc::new(self));

        handle.spawn(async move {
            info!("CEF Debug WebSocket server listening on port {}", port);

            loop {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        debug!("New CEF Debug WebSocket connection from: {}", addr);
                        let winit_waker = winit_waker.clone();
                        let sdl_waker = sdl_waker.clone();
                        let connections = connections.clone();

                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_connection(
                                stream,
                                addr,
                                winit_waker,
                                sdl_waker,
                                connections,
                            )
                            .await
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
        connections: Arc<Mutex<Vec<BroadcastSender>>>,
    ) -> Result<()> {
        let ws_stream = tokio_tungstenite::accept_async(stream)
            .await
            .context("Failed to accept WebSocket handshake")?;

        info!("CEF Debug WebSocket connection established with {}", addr);

        Self::process_messages(ws_stream, addr, winit_waker, sdl_waker, connections).await
    }

    async fn process_messages(
        ws_stream: WebSocketStream<TcpStream>,
        addr: SocketAddr,
        winit_waker: Arc<Mutex<Option<EventLoopProxy<RunnerEvent>>>>,
        sdl_waker: Arc<Mutex<Option<sdl3::event::EventSender>>>,
        connections: Arc<Mutex<Vec<BroadcastSender>>>,
    ) -> Result<()> {
        let handler = Handler::new(winit_waker, sdl_waker);

        let (broadcast_tx, mut broadcast_rx) = mpsc::unbounded_channel();
        let broadcast_tx_clone = broadcast_tx.clone();

        if let Ok(mut conns) = connections.lock() {
            conns.push(broadcast_tx);
            debug!("Registered WebSocket connection. Total: {}", conns.len());
        }

        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        loop {
            tokio::select! {
                msg = ws_receiver.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
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

                            ws_sender
                                .send(Message::Text(response_text.into()))
                                .await
                                .context("Failed to send WebSocket response")?;
                        }
                        Some(Ok(Message::Close(_))) => {
                            info!("CEF Debug WebSocket connection closed by client: {}", addr);
                            break;
                        }
                        Some(Ok(Message::Ping(data))) => {
                            ws_sender
                                .send(Message::Pong(data))
                                .await
                                .context("Failed to send pong")?;
                        }
                        Some(Ok(_)) => {
                            // Ignore other message types (Binary, Pong, Frame)
                        }
                        Some(Err(e)) => {
                            error!("CEF Debug WebSocket error from {}: {}", addr, e);
                            break;
                        }
                        None => {
                            debug!("CEF Debug WebSocket stream ended for {}", addr);
                            break;
                        }
                    }
                }
                Some(broadcast_msg) = broadcast_rx.recv() => {
                    if let Err(e) = ws_sender.send(broadcast_msg).await {
                        warn!("Failed to send broadcast message to {}: {}", addr, e);
                        break;
                    }
                }
            }
        }

        if let Ok(mut conns) = connections.lock() {
            conns.retain(|sender| !sender.same_channel(&broadcast_tx_clone));
            debug!("Unregistered WebSocket connection. Total: {}", conns.len());
        }

        info!("CEF Debug WebSocket connection closed with {}", addr);
        Ok(())
    }
}
