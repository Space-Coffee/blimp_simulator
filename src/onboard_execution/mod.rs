use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use tokio;

use blimp_onboard_software::obsw_algo::{
    BlimpAction, BlimpMainAlgo, BlimpState, MessageB2G, MessageG2B, SensorType,
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
    tokio::sync::broadcast::Sender<OnboardSimEvent>,
) {
    let (messages_g2b_tx, messages_g2b_rx) = tokio::sync::mpsc::channel::<MessageG2B>(64);
    let (events_tx, _) = tokio::sync::broadcast::channel::<OnboardSimEvent>(64);

    let action_callback: Arc<
        dyn Fn(BlimpAction) -> Pin<Box<dyn Future<Output = ()> + Send + Sync>> + Send + Sync,
    > = Arc::new(move |action| {
        Box::pin(async move {
            match action {
                BlimpAction::SendMsg(msg) => {
                    // println!("Got message:\n{:#?}", msg);
                }
                BlimpAction::SetServo { servo, location } => {}
                BlimpAction::SetMotor { motor, speed } => {}
                BlimpAction::NavLights(_) => {}
            }
        })
    });

    tokio::spawn(async move {
        let blimp_algo = BlimpMainAlgo::new();
        blimp_algo.set_action_callback(action_callback).await;
        loop {
            println!("Stepping...");
            blimp_algo.step().await;

            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    });

    (messages_g2b_tx, events_tx)
}
