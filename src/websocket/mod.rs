use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use tokio;
use tokio::sync::{broadcast, mpsc, watch, Mutex as TMutex};

use blimp_ground_ws_interface;
use blimp_ground_ws_interface::{
    BlimpGroundWebsocketStreamPair, MessageG2V, MessageV2G, VizInterest,
};
use blimp_onboard_software::obsw_algo;
use blimp_onboard_software::obsw_algo::{MessageB2G, MessageG2B, SensorType};

pub fn handle_ground_ws_connection(
    messages_g2b_tx: mpsc::Sender<MessageG2B>,
    messages_b2g_tx: broadcast::Sender<MessageB2G>,
) -> impl Fn(
    BlimpGroundWebsocketStreamPair<tokio::net::TcpStream>,
) -> Pin<Box<dyn Future<Output = ()> + Send>> {
    move |stream_pair: BlimpGroundWebsocketStreamPair<tokio::net::TcpStream>| {
        let messages_g2b_tx = messages_g2b_tx.clone();
        let mut messages_b2g_rx = messages_b2g_tx.subscribe();
        Box::pin(async move {
            let stream_pair = Arc::new(stream_pair);

            let (interest_tx, interest_rx) = watch::channel(VizInterest::new());

            {
                let stream_pair = stream_pair.clone();
                tokio::spawn(async move {
                    loop {
                        if let Ok(msg) = messages_b2g_rx.recv().await {
                        } else {
                            eprintln!("Couldn't receive B2G message!");
                            break;
                        }
                    }
                });
            }

            loop {
                if let Ok(ws_msg) = stream_pair.recv::<MessageV2G>().await {
                    // TODO
                    match ws_msg {
                        MessageV2G::DeclareInterest(viz_interest) => {
                            interest_tx.send(viz_interest).unwrap();
                        }
                        MessageV2G::Controls(controls) => {
                            messages_g2b_tx
                                .send(MessageG2B::Control(controls))
                                .await
                                .unwrap();
                        }
                    }
                } else {
                    eprintln!("Couldn't receive message from WS client");
                    break;
                }
            }
        })
    }
}
