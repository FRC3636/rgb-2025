use std::{
    io::{Read, Write},
    net::TcpStream,
    thread::sleep,
    time::Duration,
};

use clap::{Parser, Subcommand};
use lagan::{Instance, client::Client, prelude::ValueFlags, server::Server};
use simplelog::Config;

mod position_sim;

#[derive(Parser, Debug)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand, Clone, Debug)]
enum Command {
    BatteryVoltage { voltage: f64 },
    Serve,
    ImuListener,
    MovementSim,
}

fn get_accel(stream: &mut TcpStream) -> (f32, f32, f32) {
    stream.write_all(&[100]).unwrap();

    let mut buf = [0u8; 12 + 2];
    stream.read_exact(&mut buf).unwrap();

    if buf.len() < size_of::<f32>() * 3 {
        log::warn!("Invalid IMU data: {buf:?} {}", buf.len());
        return (0.0, 0.0, 0.0);
    }

    (
        f32::from_le_bytes(buf[1..5].try_into().unwrap()),
        f32::from_le_bytes(buf[5..9].try_into().unwrap()),
        f32::from_le_bytes(buf[9..13].try_into().unwrap()) + 9.81,
    )
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
        Command::ImuListener => {
            let mut log_file = std::fs::File::create("imu_log.csv").unwrap();
            log_file
                .write_all(b"time,x,y,z,vel_x,vel_y,vel_z\n")
                .unwrap();

            let client = Client::builder()
                .address("127.0.0.1:5810".parse().unwrap())
                .build();
            let imu_entry = client.entry("/robot_pos");
            imu_entry.set_value_f64_array(vec![0.0, 0.0, 0.0]).unwrap();
            let mut stream = loop {
                // let Ok(stream) = TcpStream::connect("127.0.0.1:3680") else {
                let Ok(stream) = TcpStream::connect("192.168.0.17:3680") else {
                    continue;
                };
                break stream;
            };
            let mut velocity = (0.0, 0.0, 0.0);
            let mut position = (0.0, 0.0, 0.0);

            let mut calibration_samples = Vec::new();
            const NUM_SAMPLES: usize = 100;
            for _ in 0..NUM_SAMPLES {
                calibration_samples.push(get_accel(&mut stream));
            }
            let calibration: (f32, f32, f32) =
                calibration_samples.iter().fold((0.0, 0.0, 0.0), |acc, &x| {
                    (acc.0 + x.0, acc.1 + x.1, acc.2 + x.2)
                });
            let calibration = (
                calibration.0 / NUM_SAMPLES as f32,
                calibration.1 / NUM_SAMPLES as f32,
                calibration.2 / NUM_SAMPLES as f32,
            );

            let start_time = std::time::Instant::now();
            loop {
                let accel = get_accel(&mut stream);
                let accel = (
                    accel.0 - calibration.0,
                    accel.1 - calibration.1,
                    accel.2 - calibration.2,
                );
                log::info!("IMU accel: {:?}", accel);
                // Integrate
                // if accel.0.abs() > 0.2 {
                    velocity.0 += accel.0;
                // }
                // if accel.1.abs() > 0.2 {
                    velocity.1 += accel.1;
                // }
                // if accel.2.abs() > 0.2 {
                    velocity.2 += accel.2;
                // }
                log::info!("IMU vel: {:?}", velocity);
                // second integrate
                position.0 += velocity.0;
                position.1 += velocity.1;
                position.2 += velocity.2;
                // log::info!("IMU Pos: {:?}", pos);

                imu_entry
                    .set_value_f64_array(vec![
                        position.0 as f64,
                        position.1 as f64,
                        position.2 as f64,
                    ])
                    .unwrap();
                log_file
                    .write_all(
                        format!(
                            "{},{},{},{},{},{},{}\n",
                            start_time.elapsed().as_secs_f32(),
                            position.0,
                            position.1,
                            position.2,
                            velocity.0,
                            velocity.1,
                            velocity.2
                        )
                        .as_bytes(),
                    )
                    .unwrap();
            }
        },
        Command::MovementSim => {
            position_sim::run();
        }
    }
}
