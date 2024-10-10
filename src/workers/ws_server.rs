use crate::prelude::*;
use axum::extract::ws::{Message as WebsocketMessage, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::Router;
use roboplc::controller::{Context, WResult, Worker};
use roboplc::{event_matches, hub};
use roboplc_derive::WorkerOpts;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;
use tokio::time::{sleep, Instant};
use tracing::{debug, error, info};

#[derive(WorkerOpts)]
#[worker_opts(cpu = 2, priority = 70, scheduling = "fifo", blocking = true)]
pub struct WebSocketWorker {}

impl Worker<WorkerMessage, Variables> for WebSocketWorker {
    fn run(&mut self, context: &Context<WorkerMessage, Variables>) -> WResult {
        let runtime = Runtime::new().unwrap();


        runtime.block_on(async {
            let ngrok_domain = context.variables().ngrok_domain.clone();
            let hc: Arc<Mutex<hub::Client<WorkerMessage>>> = Arc::new(Mutex::new(
                context
                    .hub()
                    .register("websocket: frame sender", event_matches!(WorkerMessage::Frame(_)))
                    .unwrap(),
            ));

            let server_handle = tokio::spawn(async move {
                let app_state = ServerState {
                    ws_path: format!("wss://{}/ws", ngrok_domain),
                };

                let app = Router::new()
                    .route("/", get(index_handler))
                    .route(
                        "/ws",
                        get(move |ws: WebSocketUpgrade| async move {
                            ws.on_upgrade(move |socket| websocket_handler(socket, hc.clone()))
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
async fn websocket_handler(mut socket: WebSocket, rx: Arc<Mutex<hub::Client<WorkerMessage>>>) {
    info!("WebSocket connection established");
    let hc = rx.lock().await;

    let mut attempts = 0;
    let start_time = Instant::now();
    let mut frame_count = 0;
    let mut total_bytes = 0;
    while let Ok(frame) = hc.recv() {
        if let WorkerMessage::Frame(frame) = frame {
            frame_count += 1;
            total_bytes += frame.len();
            match socket.send(WebsocketMessage::Binary(frame)).await {
                Ok(_) => {
                    //@todo write to logs if debug is enabled
                    if frame_count % 120 == 0 {
                        let elapsed = start_time.elapsed();
                        let mb_processed = total_bytes as f64 / (1024.0 * 1024.0);
                        let average_fps = frame_count as f64 / elapsed.as_secs_f64();
                        debug!("WS: Average FPS: {:.2}", average_fps);
                        debug!("WS: Elapsed: {:.2}", elapsed.as_secs_f64());
                        debug!("WS: MB processed: {:.2}", mb_processed);
                    }
                }
                Err(e) => {
                    error!("Failed to send frame to WebSocket client. Error: {:?}", e);
                    if attempts > 5 {
                        break;
                    }
                    attempts += 1;
                    sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }

    info!("WebSocket connection closed");
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
