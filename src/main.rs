mod app;
mod onboard_execution;
mod render;
mod simulation;
mod websocket;

use tokio;

use crate::app::get_app;
use crate::websocket::handle_ground_ws_connection;
use blimp_ground_ws_interface::BlimpGroundWebsocketServer;

fn main() {
    std::thread::spawn(|| {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async_main());
    });
    let mut app = get_app();
    app.run();
}

async fn async_main() {
    // WebSocket server for visualizations, etc.
    let mut ws_server = BlimpGroundWebsocketServer::new("127.0.0.1:8765");
    ws_server.bind().await.expect("Failed to bind WS server");
    let server_task = tokio::spawn(async move {
        ws_server
            .run(handle_ground_ws_connection())
            .await
            .expect("Error occurred while running WS server");
    });

    println!("Hello, world!");

    tokio::signal::ctrl_c().await.unwrap();
    server_task.abort();
}
