mod sim;
mod websocket;

use std::sync::Arc;

use fixed;
use futures_util::{SinkExt, StreamExt};
use nalgebra;
use postcard;
use simba;
use tokio;
use tokio::sync::Mutex as TMutex;
use tokio_tungstenite;
use typenum;

use blimp_ground_ws_interface;
use blimp_onboard_software;
use blimp_onboard_software::obsw_interface::BlimpAlgorithm;

#[tokio::main]
async fn main() {
    let mut ws_conns: Arc<TMutex<std::collections::BTreeMap<u32, tokio::sync::mpsc::Sender<()>>>> =
        Arc::new(TMutex::new(std::collections::BTreeMap::new()));

    let (shutdown_tx, mut shutdown_rx) = tokio::sync::broadcast::channel::<()>(1);

    let sim_channels = crate::sim::sim_start(shutdown_tx.clone()).await;

    // WebSocket server for visualizations, etc.
    crate::websocket::ws_server_start(shutdown_tx.clone(), &sim_channels).await;

    println!("Hello, world!");

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap_or(());
        shutdown_tx.send(()).unwrap();
    });

    shutdown_rx.recv().await.unwrap();
}
