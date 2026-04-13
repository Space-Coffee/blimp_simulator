use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use serde_json;
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
    camera_switch_tx: watch::Sender<u8>,
) -> impl Fn(
    BlimpGroundWebsocketStreamPair<tokio::net::TcpStream>,
) -> Pin<Box<dyn Future<Output = ()> + Send>> {
    move |stream_pair: BlimpGroundWebsocketStreamPair<tokio::net::TcpStream>| {
        let messages_g2b_tx = messages_g2b_tx.clone();
        let mut messages_b2g_rx = messages_b2g_tx.subscribe();
        let camera_switch_tx = camera_switch_tx.clone();
        Box::pin(async move {
            let stream_pair = Arc::new(stream_pair);

            let (interest_tx, interest_rx) = watch::channel(VizInterest::new());

            {
                let stream_pair = stream_pair.clone();
                tokio::spawn(async move {
                    loop {
                        if let Ok(msg) = messages_b2g_rx.recv().await {
                            match msg {
                                MessageB2G::Ping(_) => {}
                                MessageB2G::Pong(_) => {}
                                MessageB2G::ForwardAction(blimp_action) => match blimp_action {
                                    obsw_algo::BlimpAction::SetServo { servo, location } => {
                                        if interest_rx.borrow().servos {
                                            stream_pair
                                                .send(MessageG2V::ServoPosition {
                                                    id: servo,
                                                    angle: location,
                                                })
                                                .await
                                                .unwrap();
                                        }
                                    }
                                    obsw_algo::BlimpAction::SetMotor { motor, speed } => {
                                        if interest_rx.borrow().motors {
                                            stream_pair
                                                .send(MessageG2V::MotorSpeed { id: motor, speed })
                                                .await
                                                .unwrap();
                                        }
                                    }
                                    obsw_algo::BlimpAction::SendMsg(_) => {}
                                    obsw_algo::BlimpAction::NavLights(_) => {}
                                },
                                MessageB2G::ForwardEvent(blimp_event) => match blimp_event {
                                    obsw_algo::BlimpEvent::Control(_) => {}
                                    obsw_algo::BlimpEvent::GetMsg(_) => {}
                                    obsw_algo::BlimpEvent::SensorDataF64(
                                        sensor_type,
                                        sensor_data,
                                    ) => {
                                        if interest_rx.borrow().sensors {
                                            stream_pair
                                                .send(MessageG2V::SensorData {
                                                    id: serde_json::to_string(&sensor_type)
                                                        .unwrap(),
                                                    data: sensor_data,
                                                })
                                                .await
                                                .unwrap();
                                        }
                                    }
                                },
                                MessageB2G::BlimpState(blimp_state) => {
                                    if interest_rx.borrow().state {
                                        stream_pair
                                            .send(MessageG2V::State(blimp_state))
                                            .await
                                            .unwrap();
                                    }
                                }
                            }
                        } else {
                            eprintln!("Couldn't receive B2G message!");
                            break;
                        }
                    }
                });
            }

            loop {
                if let Ok(ws_msg) = stream_pair.recv::<MessageV2G>().await {
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
                        MessageV2G::CycleCamera => {
                            // println!("Received cycle camera message");
                            let prev_camera = camera_switch_tx.borrow().clone();
                            // println!("Read previous camera");
                            camera_switch_tx
                                .send(match prev_camera {
                                    0 => 1,
                                    1 => 0,
                                    _ => unreachable!(),
                                })
                                .unwrap();
                            // println!("Cycled camera");
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
