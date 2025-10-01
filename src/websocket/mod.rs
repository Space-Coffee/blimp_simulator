use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use tokio;
use tokio::sync::Mutex as TMutex;

use blimp_ground_ws_interface;
use blimp_ground_ws_interface::{
    BlimpGroundWebsocketStreamPair, MessageG2V, MessageV2G, VizInterest,
};
use blimp_onboard_software::obsw_algo;
use blimp_onboard_software::obsw_algo::{MessageG2B, SensorType};

pub fn handle_ground_ws_connection() -> impl Fn(
    BlimpGroundWebsocketStreamPair<tokio::net::TcpStream>,
) -> Pin<Box<dyn Future<Output = ()> + Send>> {
    move |stream_pair: BlimpGroundWebsocketStreamPair<tokio::net::TcpStream>| {
        Box::pin(async move {
            let stream_pair = Arc::new(stream_pair);

            loop {
                if let Ok(ws_msg) = stream_pair.recv::<MessageV2G>().await {
                    // TODO
                } else {
                    eprintln!("Couldn't receive message from WS client");
                    break;
                }
            }
        })
    }
}
