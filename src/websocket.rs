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

pub fn handle_ground_ws_connection(
    sim_channels: crate::sim::SimChannels,
) -> impl Fn(
    BlimpGroundWebsocketStreamPair<tokio::net::TcpStream>,
) -> Pin<Box<dyn Future<Output = ()> + Send>> {
    move |stream_pair: BlimpGroundWebsocketStreamPair<tokio::net::TcpStream>| {
        let sim_channels = sim_channels.resubscribe();
        Box::pin(async move {
            println!("New WebSocket connection");

            let curr_interest = Arc::new(TMutex::new(VizInterest::new()));

            let stream_pair = Arc::new(stream_pair);

            {
                let stream_pair = stream_pair.clone();
                let curr_interest = curr_interest.clone();
                let mut motors_rx = sim_channels.motors_rx;
                let mut servos_rx = sim_channels.servos_rx;
                let mut sensors_rx = sim_channels.sensors_rx;
                let mut state_rx = sim_channels.state_rx;
                tokio::spawn(async move {
                    loop {
                        tokio::select! {
                            motors_update = motors_rx.recv() => {
                                if curr_interest.lock().await.motors.clone() {
                                    let motors_update = motors_update.unwrap();
                                    stream_pair.send(MessageG2V::MotorSpeed{
                                        id: motors_update.0,
                                        speed: motors_update.1
                                    })
                                    .await.unwrap();
                                    // println!("Sent motors update");
                                }
                            }
                            servos_update = servos_rx.recv() => {
                                if curr_interest.lock().await.servos {
                                    let servos_update = servos_update.unwrap();
                                    stream_pair.send(MessageG2V::ServoPosition{
                                        id: servos_update.0,
                                        angle: servos_update.1
                                    })
                                    .await.unwrap();
                                    // println!("Sent servos update");
                                }
                            }
                            sensors_update = sensors_rx.recv() => {
                                if curr_interest.lock().await.sensors {
                                    let sensors_update = sensors_update.unwrap();
                                    stream_pair.send(MessageG2V::SensorData{
                                        id: serde_json::to_string::<SensorType>(&sensors_update.0).unwrap(),
                                        data: sensors_update.1
                                    })
                                    .await.unwrap();
                                    // println!("Sent sensors update");
                                }
                            }
                            state_update = state_rx.recv() => {
                                if curr_interest.lock().await.state {
                                    stream_pair.send(MessageG2V::State(state_update.unwrap()))
                                    .await.unwrap();
                                }
                            }
                        };
                    }
                });
            }

            loop {
                if let Ok(ws_msg) = stream_pair.recv::<MessageV2G>().await {
                    // println!("Got message: {:?}", ws_msg);

                    match ws_msg {
                        MessageV2G::DeclareInterest(interest) => {
                            *(curr_interest.lock().await) = interest;
                        }
                        MessageV2G::Controls(ctrls) => {
                            sim_channels
                                .msg_egress_tx
                                .send(MessageG2B::Control(ctrls))
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
