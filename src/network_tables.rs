use std::{
    net::SocketAddrV4,
    sync::{Arc, Mutex, RwLock},
};

use async_compat::Compat;
use futures::{FutureExt, select};
use network_tables::v4::{Client, Config};
use smol::Timer;

const CORAL_STATE_TOPIC: &str = "/RGB/Coral State";
const MOVEMENT_STATE_TOPIC: &str = "/RGB/Movement State";

const POSITION_RELATIVE_TO_ALIGN_TARGET_TOPIC: &str =
    "/RGB/Auto Align/Position Relative to Align Target";

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoralState {
    None = 0,
    Held = 1,
    Transit = 2,
}
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovementState {
    Driver = 0,
    AutoAlignPath = 1,
    AutoAlignPid = 2,
    SuccessfullyAligned = 3,
}

pub async fn setup_nt_client() -> Client {
    loop {
        let Ok(client) = Client::try_new_w_config(
            "10.36.36.2:5810".parse::<SocketAddrV4>().unwrap(),
            Config {
                ..Default::default()
            },
        )
        .await
        else {
            println!("Failed to connect to network tables, retrying...");
            Timer::after(std::time::Duration::from_secs(1)).await;
            continue;
        };

        return client;
    }
}

pub struct NtReactives {
    pub coral_state: Arc<Mutex<CoralState>>,

    pub movement_state: Arc<Mutex<MovementState>>,
    pub position_relative_to_align_target: Arc<Mutex<[f64; 2]>>,

    pub topics_last_changed: Arc<RwLock<std::time::Instant>>,
}

pub fn start_nt_daemon_task() -> NtReactives {
    // Reactive values
    let coral_state = Arc::new(Mutex::new(CoralState::None));

    let movement_state = Arc::new(Mutex::new(MovementState::Driver));
    let position_relative_to_align_target = Arc::new(Mutex::new([0.0, 0.0]));

    let topics_last_changed = Arc::new(RwLock::new(std::time::Instant::now()));

    // Clone the reactive values for the async task
    let coral_state_clone = coral_state.clone();
    let topics_last_changed_clone = topics_last_changed.clone();

    let movement_state_clone = movement_state.clone();
    let position_relative_to_align_target_clone = position_relative_to_align_target.clone();
    std::thread::spawn(move || {
        smol::block_on(Compat::new(async {
            let client = setup_nt_client().await;

            let mut coral_state_sub = client.subscribe(&[CORAL_STATE_TOPIC]).await.unwrap();
            let mut movement_state_sub = client.subscribe(&[MOVEMENT_STATE_TOPIC]).await.unwrap();
            let mut position_relative_to_align_target_sub = client
                .subscribe(&[POSITION_RELATIVE_TO_ALIGN_TARGET_TOPIC])
                .await
                .unwrap();

            loop {
                select! {
                    data = coral_state_sub.next().fuse() => {
                        let value = data.unwrap().data.as_i64().unwrap();
                        let mut lock = coral_state_clone.lock().unwrap();
                        *lock = match value {
                            0 => CoralState::None,
                            1 => CoralState::Held,
                            2 => CoralState::Transit,
                            _ => panic!("Invalid coral state"),
                        };
                    },
                    data = movement_state_sub.next().fuse() => {
                        let value = data.unwrap().data.as_i64().unwrap();
                        let mut lock = movement_state_clone.lock().unwrap();
                        *lock = match value {
                            0 => MovementState::Driver,
                            1 => MovementState::AutoAlignPath,
                            2 => MovementState::AutoAlignPid,
                            3 => MovementState::SuccessfullyAligned,
                            _ => panic!("Invalid movement state"),
                        };
                    },
                    data = position_relative_to_align_target_sub.next().fuse() => {
                        let value = data.unwrap().data.as_array().unwrap().iter().map(|value| value.as_f64().unwrap()).collect::<Vec<_>>();
                        let mut lock = position_relative_to_align_target_clone.lock().unwrap();
                        *lock = [value[0], value[1]];
                    },
                }

                let mut lock = topics_last_changed_clone.write().unwrap();
                *lock = std::time::Instant::now();
            }
        }))
    });

    NtReactives {
        coral_state,

        movement_state,
        position_relative_to_align_target,

        topics_last_changed,
    }
}
