use std::{future::Future, pin::Pin, sync::Arc};

use tokio::sync::{broadcast, mpsc, Mutex as TMutex, RwLock as TRwLock};

use blimp_onboard_software::obsw_algo::{
    BlimpAction, BlimpEvent, BlimpMainAlgo, BlimpState, MessageB2G, MessageG2B, SensorType,
};
use blimp_onboard_software::obsw_interface::BlimpAlgorithm;

struct SimBlimp {
    // This thing is not Send nor Sync. Why?!!
    // coord_mat: nalgebra::Affine3<f64>,
    main_algo: TRwLock<BlimpMainAlgo>,
}

struct Simulation {
    blimp: SimBlimp,
    _earth_radius: f64,
}

// impl Send for Simulation {}

impl Simulation {
    fn new() -> Self {
        let blimp_main_algo = BlimpMainAlgo::new();
        //blimp_main_algo.set_action_callback();
        //blimp_main_algo.set_action_callback(action_callback);
        Self {
            blimp: SimBlimp {
                // coord_mat: nalgebra::Affine3::identity(),
                main_algo: TRwLock::new(blimp_main_algo),
            },
            _earth_radius: 6371000.0,
        }
    }

    async fn step(&self) {
        self.blimp.main_algo.read().await.step().await;
    }
}

pub struct SimChannels {
    pub msg_egress_tx: tokio::sync::mpsc::Sender<MessageG2B>,
    pub motors_rx: tokio::sync::broadcast::Receiver<(u8, f32)>,
    pub servos_rx: tokio::sync::broadcast::Receiver<(u8, f32)>,
    pub sensors_rx: tokio::sync::broadcast::Receiver<(SensorType, f64)>,
    pub state_rx: broadcast::Receiver<BlimpState>,
}

impl SimChannels {
    pub fn resubscribe(&self) -> Self {
        Self {
            msg_egress_tx: self.msg_egress_tx.clone(),
            motors_rx: self.motors_rx.resubscribe(),
            servos_rx: self.servos_rx.resubscribe(),
            sensors_rx: self.sensors_rx.resubscribe(),
            state_rx: self.state_rx.resubscribe(),
        }
    }
}

pub async fn sim_start(shutdown_tx: tokio::sync::broadcast::Sender<()>) -> SimChannels {
    // When simulated blimp wants to set motors, it will be sent to this channel
    let (motors_tx, motors_rx) = tokio::sync::broadcast::channel::<(u8, f32)>(64);
    let (servos_tx, servos_rx) = tokio::sync::broadcast::channel::<(u8, f32)>(64);
    let (sensors_tx, sensors_rx) = tokio::sync::broadcast::channel::<(SensorType, f64)>(64);
    let (state_tx, state_rx) = tokio::sync::broadcast::channel::<BlimpState>(64);

    let sim: Arc<TMutex<Simulation>> = Arc::new(TMutex::new(Simulation::new()));

    let blimp_action_callback: Arc<
        dyn Fn(BlimpAction) -> Pin<Box<dyn Future<Output = ()> + Send + Sync>> + Send + Sync,
    > = {
        let sim = sim.clone();
        let motors_tx = motors_tx.clone();
        let servos_tx = servos_tx.clone();
        let sensors_tx = sensors_tx.clone();
        let state_tx = state_tx.clone();
        Arc::new(move |action| {
            let sim = sim.clone();
            let motors_tx = motors_tx.clone();
            let servos_tx = servos_tx.clone();
            let sensors_tx = sensors_tx.clone();
            let state_tx = state_tx.clone();
            Box::pin(async move {
                // println!("Action {:#?}", action);
                match action {
                    BlimpAction::SendMsg(msg) => {
                        // println!("Got message:\n{:#?}", msg);

                        match msg.as_ref() {
                            MessageB2G::Ping(ping_id) => {
                                let sim_locked = sim.lock().await;
                                // let sim_locked=sim.blocking_lock();
                                let main_algo_locked = sim_locked.blimp.main_algo.read().await;
                                // let main_algo_locked=sim_locked.blimp.main_algo.blocking_lock();
                                main_algo_locked
                                    .handle_event(BlimpEvent::GetMsg(MessageG2B::Pong(*ping_id)))
                                    .await;
                            }
                            MessageB2G::Pong(_ping_id) => {}
                            MessageB2G::ForwardAction(fwd_action) => match fwd_action {
                                BlimpAction::SetMotor { motor, speed } => {
                                    motors_tx.send((*motor, *speed)).unwrap();
                                }
                                BlimpAction::SetServo { servo, location } => {
                                    servos_tx.send((*servo, *location)).unwrap();
                                }
                                _ => {}
                            },
                            MessageB2G::ForwardEvent(fwd_event) => match fwd_event {
                                BlimpEvent::SensorDataF64(sns, data) => {
                                    sensors_tx.send((sns.clone(), *data)).unwrap();
                                }
                                _ => {}
                            },
                            MessageB2G::BlimpState(blimp_state) => {
                                state_tx.send(blimp_state.clone()).unwrap();
                            }
                        }
                    }
                    _ => {}
                }
            })
        })
    };
    sim.lock()
        .await
        .blimp
        .main_algo
        .write()
        .await
        .set_action_callback(blimp_action_callback)
        .await;

    {
        // Execute blimp's algorithm steps

        let mut shutdown_rx = shutdown_tx.subscribe();
        let sim = sim.clone();
        tokio::spawn(async move {
            loop {
                sim.lock().await.step().await;

                tokio::select! {
                    _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {},
                    _ = shutdown_rx.recv() => {
                        break;
                    },
                };
            }
        });
    }

    let blimp_send_msg_tx = {
        // Channel for sending messages to blimp

        let mut shutdown_rx = shutdown_tx.subscribe();
        let sim = sim.clone();
        let (blimp_send_msg_tx, mut blimp_send_msg_rx) =
            tokio::sync::mpsc::channel::<MessageG2B>(64);
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    msg = blimp_send_msg_rx.recv() => {
                        if let Some(msg) = msg {
                            // println!("Sending to blimp: {:?}", msg);
                            sim.lock()
                                .await
                                .blimp
                                .main_algo
                                .read()
                                .await
                                .handle_event(BlimpEvent::GetMsg(
                                    msg))
                                .await;
                        }
                        else {
                            break;
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        break;
                    }
                };
            }
        });
        blimp_send_msg_tx
    };

    {
        // Ping the blimp

        let mut shutdown_rx = shutdown_tx.subscribe();
        let msg_tx = blimp_send_msg_tx.clone();
        tokio::spawn(async move {
            let mut i: u32 = 0;
            loop {
                println!("Pinging the blimp with id {}", i);
                msg_tx.send(MessageG2B::Ping(i)).await.unwrap();
                i += 1;

                tokio::select! {
                    _ = tokio::time::sleep(tokio::time::Duration::from_millis(1000))=>{
                    }
                    _ = shutdown_rx.recv() => {
                        break;
                    }
                };
            }
        });
    }

    {
        // Sensors simulation
        let mut shutdown_rx = shutdown_tx.subscribe();
        let sim = sim.clone();
        tokio::spawn(async move {
            let mut counter: i64 = 0;
            loop {
                sim.lock()
                    .await
                    .blimp
                    .main_algo
                    .read()
                    .await
                    .handle_event(BlimpEvent::SensorDataF64(
                        SensorType::Barometer,
                        (counter as f64 * 0.1).sin() * 2000.0 + 101300.0,
                    ))
                    .await;

                tokio::select! {
                    _ = tokio::time::sleep(tokio::time::Duration::from_millis(250)) => {
                    }
                    _ = shutdown_rx.recv() => {
                        break;
                    }
                };

                counter += 1;
            }
        });
    }

    SimChannels {
        msg_egress_tx: blimp_send_msg_tx,
        motors_rx,
        servos_rx,
        sensors_rx,
        state_rx,
    }
}
