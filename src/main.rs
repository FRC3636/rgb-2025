use std::{thread::sleep, time::Instant};

use palette::{Clamp, IntoColor, LinSrgb, Srgb};
use shark::shader::{FragThree, Shader, primitives::color};
use smart_leds::{RGB8, SmartLedsWrite};

mod spi;
mod strips;

fn main() {
    let mut strip = spi::gpio_10().unwrap();
    println!("Got Strip!!!");
    let shader = color::<FragThree>(Srgb::new(0.5, 0.0, 0.5));

    let start_instant = Instant::now();
    loop {
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
                println!("Color: {:?}", c);
                RGB8::new(
                    (c.red * 256.0) as u8,
                    (c.green * 256.0) as u8,
                    (c.blue * 256.0) as u8,
                )
            });

        strip.write(colors).unwrap();
        sleep(std::time::Duration::from_millis(100));
    }
}
