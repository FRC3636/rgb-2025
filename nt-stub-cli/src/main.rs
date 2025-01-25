use clap::{Parser, Subcommand, ValueEnum};
use lagan::{Instance, client::Client, prelude::ValueFlags, server::Server};

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

    match args.command {
        Command::BatteryVoltage { voltage } => {
            let client = Client::builder()
                .address("127.0.0.1:5810".parse().unwrap())
                .build();
            let pat_entry = client.entry("/battery_voltage");
            pat_entry.set_value_f64(voltage).unwrap();
        }
        Command::Serve => {
            let _server = Server::builder()
                .persist_filename("nt_persist.json")
                .build();
            loop {
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
    }
}
