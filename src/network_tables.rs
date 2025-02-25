use std::{
    net::SocketAddrV4,
    sync::{Arc, Mutex},
};

use async_compat::Compat;
use futures::{FutureExt, select};
use network_tables::v4::{Client, Config};

pub async fn setup_nt_client() -> Result<Client, network_tables::Error> {
    let client = Client::try_new_w_config(
        "192.168.0.48:5810".parse::<SocketAddrV4>().unwrap(),
        Config {
            ..Default::default()
        },
    )
    .await?;

    Ok(client)
}

pub struct NtReactives {
    pub voltage: Arc<Mutex<f64>>,
    pub robot_pos: Arc<Mutex<[f64; 3]>>,
}

pub fn start_nt_daemon_task() -> NtReactives {
    let voltage = Arc::new(Mutex::new(0.0));
    let robot_pos = Arc::new(Mutex::new([0.0; 3]));

    let voltage_clone = voltage.clone();
    let robot_pos_clone = robot_pos.clone();
    std::thread::spawn(move || {
        smol::block_on(Compat::new(async {
            let client = setup_nt_client().await.unwrap();
            let mut voltage_sub = client.subscribe(&["/battery_voltage"]).await.unwrap();
            let mut pos_sub = client.subscribe(&["/robot_pos"]).await.unwrap();
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
                }
            }
        }))
    });

    NtReactives { voltage, robot_pos }
}
