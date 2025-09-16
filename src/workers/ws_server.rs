use crate::prelude::*;
use axum::Router;
use axum::extract::ws::{Message as WebsocketMessage, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::response::{Html, IntoResponse};
use axum::routing::{any, get};
use roboplc::controller::{Context, WResult, Worker};
use roboplc::hub::Hub;
use roboplc::{event_matches, hub};
use roboplc_derive::WorkerOpts;
use std::net::SocketAddr;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, atomic};
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::time::{Instant, sleep};
use tracing::{error, info};

#[derive(WorkerOpts)]
#[worker_opts(cpu = 2, priority = 40, scheduling = "fifo", blocking = false)]
pub struct WebSocketWorker {}

impl Worker<WorkerMessage, Variables> for WebSocketWorker {
    fn run(&mut self, context: &Context<WorkerMessage, Variables>) -> WResult {
        let runtime = Runtime::new().unwrap();

        runtime.block_on(async {
            let ngrok_domain = context.variables().ngrok_domain.clone();
            let hub = context.hub().clone();

            let server_handle = tokio::spawn(async move {
                let app_state = ServerState {
                    ws_path: format!("wss://{}/ws", ngrok_domain),
                    connection_counter: Arc::new(AtomicUsize::new(0)),
                };

                let app = Router::new()
                    .route("/", get(index_handler))
                    .route(
                        "/ws",
                        any(move |ws: WebSocketUpgrade, state: State<ServerState>| async move {
                            let connection_id = state.connection_counter.fetch_add(1, atomic::Ordering::Relaxed);
                            ws.on_upgrade(move |socket| websocket_handler(socket, hub, connection_id))
                        }),
                    )
                    .with_state(app_state);

                let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
                info!("Starting server on http://{}", addr);
                let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
                axum::serve(listener, app).await.unwrap();
            });
            let _ = tokio::try_join!(server_handle);
        });

        Ok(())
    }
}

/// Handles WebSocket connections
async fn websocket_handler(mut socket: WebSocket, hub: Hub<WorkerMessage>, connection_id: usize) {
    info!("WebSocket {} connection established", connection_id);

    // ~1 sec buffer
    let frame_buffer = 32;
    let client_options = hub::ClientOptions::new(
        &("websocket: frame sender".to_owned() + &connection_id.to_string()),
        event_matches!(WorkerMessage::Frame(_)),
    )
    .capacity(frame_buffer);
    let hc = hub.register_with_options(client_options).unwrap();

    let mut attempts = 0;
    let start_time = Instant::now();
    let mut frame_count = 0;
    let mut total_bytes = 0;

    // @TODO debug why after close stream page - WS not closed, frames stop sending from camera worker
    loop {
        match hc.try_recv() {
            Ok(worker_message) => {
                if let WorkerMessage::Frame(frame) = worker_message {
                    frame_count += 1;
                    total_bytes += frame.len();

                    match socket.send(WebsocketMessage::Binary(frame)).await {
                        Ok(_) => {
                            if frame_count % 120 == 0 {
                                let elapsed = start_time.elapsed();
                                let mb_processed = total_bytes as f64 / (1024.0 * 1024.0);
                                let average_fps = frame_count as f64 / elapsed.as_secs_f64();

                                info!("WS {}: Average FPS: {:.2}", connection_id, average_fps);
                                info!("WS {}: Elapsed: {:.2}", connection_id, elapsed.as_secs_f64());
                                info!("WS {}: MB processed: {:.2}", connection_id, mb_processed);
                                info!("WS {}: Total frames: {:.2}", connection_id, frame_count);
                            }
                        }
                        Err(e) => {
                            error!("Failed to send frame to WebSocket {} client. Error: {:?}", connection_id, e);
                            if attempts > 5 {
                                break;
                            }
                            attempts += 1;
                            sleep(Duration::from_secs(1)).await;
                        }
                    }
                }
            }
            Err(error) => {
                error!("WS {}: Error receiving message from hub: {:?}", connection_id, error);
                sleep(Duration::from_millis(1000)).await;

                continue;
            }
        }
    }

    info!("WebSocket {} connection closed", connection_id);
}

/// video stream page handler
async fn index_handler(State(state): State<ServerState>) -> impl IntoResponse {
    info!("Received request to / from {}", state.ws_path);

    let html = format!(
        r#"<!DOCTYPE html>
                <html lang="en">
                <head>
                    <meta charset="UTF-8">
                    <meta name="viewport" content="width=device-width, initial-scale=1.0">
                    <title>Video Stream</title>
                </head>
                <body>
                <h1>Video Stream</h1>
                <img id="videoStream" style="width: 640px; height: 480px;">

                <script>
                    const img = document.getElementById('videoStream');
                    const ws = new WebSocket('{}');
                    console.log('Connecting to WebSocket server...');
                    ws.onopen = function () {{
                        console.log('WebSocket connection established');
                    }};

                    ws.onmessage = function (event) {{
                        const blob = new Blob([event.data], {{type: 'image/jpeg'}});
                        const url = URL.createObjectURL(blob);
                        img.src = url;
                    }};

                    ws.onclose = function () {{
                        console.log('WebSocket connection closed');
                    }};

                    ws.onerror = function (error) {{
                        console.error('WebSocket error:', error);
                    }};
                </script>
                </body>
                </html>"#,
        state.ws_path
    );

    Html(html)
}
