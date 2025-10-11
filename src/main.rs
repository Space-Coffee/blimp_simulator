mod app;
mod onboard_execution;
mod render;
mod simulation;
mod websocket;

use tokio;

use crate::app::get_app;
use crate::onboard_execution::start_onboard;
use crate::websocket::handle_ground_ws_connection;
use blimp_ground_ws_interface::BlimpGroundWebsocketServer;

struct AsyncSyncBridge {
    pub motors_servos_rx: tokio::sync::watch::Receiver<([f32; 4], [f32; 12])>,
}

fn main() {
    let (as_bridge_tx, as_bridge_rx) = tokio::sync::oneshot::channel::<AsyncSyncBridge>();

    std::thread::spawn(|| {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async_main(as_bridge_tx));
    });

    let mut app = get_app(as_bridge_rx);
    app.run();
}

async fn async_main(as_bridge_tx: tokio::sync::oneshot::Sender<AsyncSyncBridge>) {
    let (messages_g2b_tx, messages_b2g_tx, /*events_tx,*/ motors_servos_rx) = start_onboard().await;
    as_bridge_tx
        .send(AsyncSyncBridge { motors_servos_rx })
        .map_err(|_| "Couldn't send data thourgh async-sync bridge")
        .unwrap();

    // WebSocket server for visualizations, etc.
    let mut ws_server = BlimpGroundWebsocketServer::new("127.0.0.1:8765");
    ws_server.bind().await.expect("Failed to bind WS server");
    let server_task = tokio::spawn(async move {
        ws_server
            .run(handle_ground_ws_connection(
                messages_g2b_tx,
                messages_b2g_tx,
            ))
            .await
            .expect("Error occurred while running WS server");
    });

    println!("Hello, world!");

    tokio::signal::ctrl_c().await.unwrap();
    server_task.abort();
}
