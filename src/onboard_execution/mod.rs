use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use tokio;

use blimp_onboard_software::obsw_algo::{
    BlimpAction, BlimpEvent, BlimpMainAlgo, BlimpState, MessageB2G, MessageG2B, SensorType,
};
use blimp_onboard_software::obsw_interface::BlimpAlgorithm;

#[derive(Clone)]
pub enum OnboardSimEvent {
    MotorUpdate(u8, f32),
    ServoUpdate(u8, f32),
    SensorUpdate(SensorType, f64),
    StateUpdate(BlimpState),
}

pub async fn start_onboard() -> (
    tokio::sync::mpsc::Sender<MessageG2B>,
    tokio::sync::broadcast::Sender<MessageB2G>,
    // tokio::sync::broadcast::Sender<OnboardSimEvent>,
    tokio::sync::watch::Receiver<([f32; 4], [f32; 12])>,
    tokio::sync::mpsc::Sender<(SensorType, f64)>,
) {
    let (messages_g2b_tx, mut messages_g2b_rx) = tokio::sync::mpsc::channel::<MessageG2B>(64);
    let (messages_b2g_tx, _) = tokio::sync::broadcast::channel::<MessageB2G>(64);
    // let (events_tx, _) = tokio::sync::broadcast::channel::<OnboardSimEvent>(64);
    let (motors_servos_tx, motors_servos_rx) =
        tokio::sync::watch::channel::<([f32; 4], [f32; 12])>(([0.0; 4], [0.0; 12]));
    let (sensors_tx, mut sensors_rx) = tokio::sync::mpsc::channel::<(SensorType, f64)>(64);

    let action_callback: Arc<
        dyn Fn(BlimpAction) -> Pin<Box<dyn Future<Output = ()> + Send + Sync>> + Send + Sync,
    > = {
        let motors_servos_tx = motors_servos_tx.clone();
        let messages_b2g_tx = messages_b2g_tx.clone();
        Arc::new(move |action| {
            let motors_servos_tx = motors_servos_tx.clone();
            let messages_b2g_tx = messages_b2g_tx.clone();
            Box::pin(async move {
                match action {
                    BlimpAction::SendMsg(msg) => {
                        // println!("Got message:\n{:#?}", msg);
                        match msg.as_ref() {
                            MessageB2G::Ping(_) => {}
                            MessageB2G::Pong(_) => {}
                            MessageB2G::ForwardAction(blimp_action) => {}
                            MessageB2G::ForwardEvent(blimp_event) => {}
                            MessageB2G::BlimpState(blimp_state) => {}
                        }
                        _ = messages_b2g_tx.send(msg.as_ref().clone());
                    }
                    BlimpAction::SetServo { servo, location } => {
                        motors_servos_tx.send_modify(move |prev| {
                            prev.1[servo as usize] = location;
                        });
                    }
                    BlimpAction::SetMotor { motor, speed } => {
                        motors_servos_tx.send_modify(move |prev| {
                            prev.0[motor as usize] = speed;
                        });
                    }
                    BlimpAction::NavLights(_) => {}
                }
            })
        })
    };

    // Execute blimp's main algorithm
    let blimp_algo = Arc::new(BlimpMainAlgo::new());
    {
        let blimp_algo = blimp_algo.clone();
        tokio::spawn(async move {
            blimp_algo.set_action_callback(action_callback).await;
            loop {
                // println!("Stepping...");
                blimp_algo.step().await;

                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        });
    }

    // Pass G2B messages to the algo
    {
        let blimp_algo = blimp_algo.clone();
        tokio::spawn(async move {
            loop {
                let msg = messages_g2b_rx.recv().await;
                if let Some(msg) = msg {
                    blimp_algo
                        .clone()
                        .handle_event(BlimpEvent::GetMsg(msg))
                        .await;
                }
            }
        });
    }

    // Pass sensors daat to the algo
    tokio::spawn(async move {
        loop {
            let data = sensors_rx.recv().await;
            if let Some(data) = data {
                blimp_algo
                    .clone()
                    .handle_event(BlimpEvent::SensorDataF64(data.0, data.1))
                    .await;
            }
        }
    });

    (
        messages_g2b_tx,
        messages_b2g_tx,
        // events_tx,
        motors_servos_rx,
        sensors_tx,
    )
}
