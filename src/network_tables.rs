use std::{
    net::SocketAddrV4,
    sync::{Arc, Mutex},
};

use async_compat::Compat;
use futures::{FutureExt, select};
use network_tables::v4::{Client, Config};
use smol::Timer;

const CORAL_STATE_TOPIC: &str = "/RGB/Coral State";

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoralState {
    None = 0,
    Held = 1,
    Transit = 2,
}

pub async fn setup_nt_client() -> Client {
    loop {
        let Ok(client) = Client::try_new_w_config(
            "10.36.36.2:5810".parse::<SocketAddrV4>().unwrap(),
            Config {
                ..Default::default()
            },
        )
        .await else {
            println!("Failed to connect to network tables, retrying...");
            Timer::after(std::time::Duration::from_secs(1)).await;
            continue;
        };

        return client;
    }
}

pub struct NtReactives {
    pub voltage: Arc<Mutex<f64>>,
    pub robot_pos: Arc<Mutex<[f64; 3]>>,
    pub coral_state: Arc<Mutex<CoralState>>,
}

pub fn start_nt_daemon_task() -> NtReactives {
    let voltage = Arc::new(Mutex::new(0.0));
    let robot_pos = Arc::new(Mutex::new([0.0; 3]));
    let coral_state = Arc::new(Mutex::new(CoralState::None));

    let voltage_clone = voltage.clone();
    let robot_pos_clone = robot_pos.clone();
    let coral_state_clone = coral_state.clone();
    std::thread::spawn(move || {
        smol::block_on(Compat::new(async {
            let client = setup_nt_client().await;
            let mut voltage_sub = client.subscribe(&["/battery_voltage"]).await.unwrap();
            let mut pos_sub = client.subscribe(&["/robot_pos"]).await.unwrap();
            let mut coral_state_sub = client.subscribe(&[CORAL_STATE_TOPIC]).await.unwrap();
            loop {
                select! {
                    data = voltage_sub.next().fuse() => {
                        let value = data.unwrap().data.as_f64().unwrap();
                        let mut lock = voltage_clone.lock().unwrap();
                        *lock = value;
                    }
                    data = pos_sub.next().fuse() => {
                        let value = data.unwrap().data.as_array().unwrap().iter().map(|v| v.as_f64().unwrap()).collect::<Vec<_>>();
                        let mut lock = robot_pos_clone.lock().unwrap();
                        *lock = [value[0], value[1], value[2]];
                    }
                    data = coral_state_sub.next().fuse() => {
                        let value = data.unwrap().data.as_i64().unwrap();
                        let mut lock = coral_state_clone.lock().unwrap();
                        *lock = match value {
                            0 => CoralState::None,
                            1 => CoralState::Held,
                            2 => CoralState::Transit,
                            _ => panic!("Invalid coral state"),
                        };
                    }
                }
            }
        }))
    });

    NtReactives { voltage, robot_pos, coral_state }
}
