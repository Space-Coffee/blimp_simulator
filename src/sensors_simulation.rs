use std::f32::consts::PI;
use tokio::sync::{mpsc, watch};

use blimp_onboard_software::obsw_algo::SensorType;
use nalgebra::Vector3;
use crate::simulation::util::pressure_at;

pub async fn start_sensors(
    pos_rx: watch::Receiver<(f32, f32, f32)>,
    rot_rx: watch::Receiver<nalgebra::Rotation3<f32>>,
    sensors_tx: mpsc::Sender<(SensorType, f64)>,
) {
    tokio::spawn(async move {
        loop {
            let pressure = pressure_at(pos_rx.borrow().1 as f64);
            sensors_tx
                .send((SensorType::Barometer, pressure))
                .await
                .unwrap();

            let acc_transform =
                nalgebra::Matrix3::<f32>::new(1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0);
            let grav_acc: nalgebra::Vector3<f32> = acc_transform
                * rot_rx.borrow().inverse()
                * nalgebra::Vector3::<f32>::new(0.0, 9.81, 0.0);

            let forward = rot_rx.borrow().clone() * Vector3::z();
            let magnetometer_angle = PI - forward.x.atan2(forward.z);

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
            sensors_tx
                .send((SensorType::MagnetometerHeading, magnetometer_angle as f64))
                .await
                .unwrap();

            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    });
}
