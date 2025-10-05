use tokio;

use blimp_onboard_software::obsw_algo::{BlimpState, MessageG2B, SensorType};

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

    (messages_g2b_tx, events_tx)
}
