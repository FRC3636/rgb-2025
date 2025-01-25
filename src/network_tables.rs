use std::{net::{Ipv4Addr, SocketAddrV4}, sync::{Arc, Mutex}};

use async_compat::Compat;
use futures::{select, FutureExt};
use network_tables::{v4::{Client, Config, Type}, Value};

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

pub fn start_nt_daemon_task() -> Arc<Mutex<f64>> {
    let voltage = Arc::new(Mutex::new(0.0));
    let voltage_clone = voltage.clone();
    std::thread::spawn(move || {
        smol::block_on(Compat::new(async {
            let client = setup_nt_client().await.unwrap();
            let mut voltage_sub = client.subscribe(&["/battery_voltage"]).await.unwrap();
            loop {
                select! {
                    data = voltage_sub.next().fuse() => {
                        let value = data.unwrap().data.as_f64().unwrap();
                        let mut lock = voltage_clone.lock().unwrap();
                        *lock = value;
                    }
                }
            }
        }))
    });
    voltage
}