use std::{thread::sleep, time::Instant};

use palette::{Clamp, IntoColor, LinSrgb, Srgb};
use shark::shader::{FragThree, Shader, primitives::color};
use smart_leds::{RGB8, SmartLedsWrite};

mod spi;
mod strips;
mod network_tables;
mod shaders;

fn main() {
    let mut strip = spi::gpio_10().unwrap();
    println!("Got Strip!!!");

    let voltage = network_tables::start_nt_daemon_task();

    
    let start_instant = Instant::now();
    loop {
        let shader = shaders::battery_indicator(*voltage.lock().unwrap());

        let dt = start_instant.elapsed().as_secs_f64();
        let colors = strips::test_strip()
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
            });
        let colors = colors.collect::<Vec<_>>();
            
        strip.write(colors).unwrap();
        sleep(std::time::Duration::from_millis(100));
    }
}
