use std::{
    thread::sleep,
    time::{Duration, Instant},
};

use clap::{Parser, Subcommand};
use lagan::{Instance, client::Client, prelude::ValueFlags, server::Server};
use simplelog::Config;

#[derive(Parser, Debug)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand, Clone, Debug)]
enum Command {
    BatteryVoltage { voltage: f64 },
    Serve,
}

fn main() {
    let args = Args::parse();
    simplelog::TermLogger::init(
        log::LevelFilter::Info,
        Config::default(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Always,
    )
    .unwrap();

    match args.command {
        Command::BatteryVoltage { voltage } => {
            let client = Client::builder()
                .address("127.0.0.1:5810".parse().unwrap())
                .build();
            let pat_entry = client.entry("/battery_voltage");
            sleep(Duration::from_millis(10));
            for _ in 0..10 {
                pat_entry.set_value_f64(voltage).unwrap();
                sleep(Duration::from_millis(20));
            }
        }
        Command::Serve => {
            let server = Server::builder()
                .persist_filename("nt_persist.json")
                .build();
            log::info!("Server started! {server:?}");
            let voltage_entry = server.entry("/battery_voltage");
            voltage_entry.set_value_f64(0.0).unwrap();
            voltage_entry.set_flags(ValueFlags::PERSISTENT).unwrap();
            loop {
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
    }
}
