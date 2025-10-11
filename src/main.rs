mod app;
mod onboard_execution;
mod render;
mod sensors_simulation;
mod simulation;
mod websocket;

use tokio;
use tokio::sync::{oneshot, watch};

use crate::app::get_app;
use crate::onboard_execution::start_onboard;
use crate::sensors_simulation::start_sensors;
use crate::websocket::handle_ground_ws_connection;
use blimp_ground_ws_interface::BlimpGroundWebsocketServer;

struct AsyncSyncBridge {
    pub motors_servos_rx: watch::Receiver<([f32; 4], [f32; 12])>,
}

struct SyncAsyncBridge {
    pub pos_rx: watch::Receiver<(f32, f32, f32)>,
    pub rot_rx: watch::Receiver<(f32, f32, f32)>,
}

fn main() {
    let (as_bridge_tx, as_bridge_rx) = oneshot::channel::<AsyncSyncBridge>();
    let (sa_bridge_tx, sa_bridge_rx) = oneshot::channel::<SyncAsyncBridge>();

    std::thread::spawn(|| {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async_main(as_bridge_tx, sa_bridge_rx));
    });

    let mut app = get_app(as_bridge_rx, sa_bridge_tx);
    app.run();
}

async fn async_main(
    as_bridge_tx: oneshot::Sender<AsyncSyncBridge>,
    sa_bridge_rx: oneshot::Receiver<SyncAsyncBridge>,
) {
    let (messages_g2b_tx, messages_b2g_tx, /*events_tx,*/ motors_servos_rx, sensors_tx) =
        start_onboard().await;

    as_bridge_tx
        .send(AsyncSyncBridge { motors_servos_rx })
        .map_err(|_| "Couldn't send data thourgh async-sync bridge")
        .unwrap();
    let sa_bridge = sa_bridge_rx.await.unwrap();

    start_sensors(sa_bridge.pos_rx, sa_bridge.rot_rx, sensors_tx).await;

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
