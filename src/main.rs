use std::{
    sync::{Arc, Barrier, Mutex},
    thread::{sleep, spawn},
    time::{Duration, Instant},
};

use palette::{Clamp, IntoColor, LinSrgb};
use shark::shader::{FragThree, Shader};
use smart_leds::{RGB8, SmartLedsWrite};

mod network_tables;
mod shaders;
mod spi;
mod strips;

const DESIRED_FPS: f64 = 320.0;
const SLEEP_DURATION: Duration = Duration::from_millis((1.0 / DESIRED_FPS * 1000.0) as u64);

fn main() {
    let voltage = network_tables::start_nt_daemon_task();

    let start_instant = Instant::now();

    let points = strips::test_strip().collect::<Vec<_>>();
    let colors = Arc::new(Mutex::new(vec![RGB8::default(); points.len()]));
    let barrier = Arc::new(Barrier::new(2));

    // We are bottlenecked by SPI syscalls, so spawn a thread to write while we run shaders
    spawn({
        let barrier = barrier.clone();
        let colors = colors.clone();
        move || {
            let mut strip = spi::gpio_10().unwrap();

            loop {
                barrier.wait();
                strip.write(colors.lock().unwrap().iter().cloned()).unwrap();
            }
        }
    });

    loop {
        let shader = shaders::battery_indicator(*voltage.lock().unwrap());
        let loop_start = Instant::now();

        let dt = start_instant.elapsed().as_secs_f64();

        let new_colors = points
            .iter()
            .map(|point| {
                shader.shade(FragThree {
                    pos: [point.x, point.y, point.z],
                    time: dt,
                })
            })
            .map(|c| {
                let c: LinSrgb<f64> = c.into_color();
                c.clamp()
            })
            .map(|c| {
                RGB8::new(
                    (c.red * 256.0) as u8,
                    (c.green * 256.0) as u8,
                    (c.blue * 256.0) as u8,
                )
            })
            .collect::<Vec<_>>();
        let shade_dur = loop_start.elapsed();

        colors.lock().unwrap().copy_from_slice(&new_colors);

        barrier.wait();

        let sleep_dur = SLEEP_DURATION.saturating_sub(loop_start.elapsed());
        print!(
            "\rShade Time: {}us Loop Time: {}ms Sleeping for {}ms",
            shade_dur.as_micros(),
            loop_start.elapsed().as_millis(),
            sleep_dur.as_millis()
        );
        sleep(sleep_dur);
    }
}
