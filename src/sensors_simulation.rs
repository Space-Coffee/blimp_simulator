use tokio::sync::{mpsc, watch};

use blimp_onboard_software::obsw_algo::SensorType;

pub async fn start_sensors(
    pos_rx: watch::Receiver<(f32, f32, f32)>,
    rot_rx: watch::Receiver<nalgebra::Rotation3<f32>>,
    sensors_tx: mpsc::Sender<(SensorType, f64)>,
) {
    tokio::spawn(async move {
        loop {
            let pressure: f64 = 101325.0f64
                * (-9.81f64 * 28.9644 * (pos_rx.borrow().1 as f64 + 200.0 - 0.0)
                    / 8.31e3
                    / (273.15 + 15.0))
                    .exp();
            sensors_tx
                .send((SensorType::Barometer, pressure))
                .await
                .unwrap();

            let grav_acc: nalgebra::Vector3<f32> =
                rot_rx.borrow().inverse() * nalgebra::Vector3::<f32>::new(0.0, -9.81, 0.0);
            sensors_tx
                .send((SensorType::AccelerometerX, grav_acc.x as f64))
                .await
                .unwrap();
            sensors_tx
                .send((SensorType::AccelerometerY, grav_acc.y as f64))
                .await
                .unwrap();
            sensors_tx
                .send((SensorType::AccelerometerZ, grav_acc.z as f64))
                .await
                .unwrap();

            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    });
}
