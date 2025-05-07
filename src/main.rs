mod sim;
mod websocket;

use tokio;

use crate::sim::sim_start;
use crate::websocket::handle_ground_ws_connection;
use blimp_ground_ws_interface::BlimpGroundWebsocketServer;

#[tokio::main]
async fn main() {
    // let ws_conns: Arc<TMutex<std::collections::BTreeMap<u32, tokio::sync::mpsc::Sender<()>>>> =
    //     Arc::new(TMutex::new(std::collections::BTreeMap::new()));

    let (shutdown_tx, mut shutdown_rx) = tokio::sync::broadcast::channel::<()>(1);

    let sim_channels = sim_start(shutdown_tx.clone()).await;

    // WebSocket server for visualizations, etc.
    let mut ws_server = BlimpGroundWebsocketServer::new("127.0.0.1:8765");
    ws_server.bind().await.expect("Failed to bind WS server");
    tokio::spawn(async move {
        ws_server
            .run(handle_ground_ws_connection(sim_channels))
            .await
    });

    println!("Hello, world!");

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap_or(());
        shutdown_tx.send(()).unwrap();
    });

    shutdown_rx.recv().await.unwrap();
}
