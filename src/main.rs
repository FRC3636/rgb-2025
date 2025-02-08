use std::{
    thread::sleep,
    time::{Duration, Instant},
};

use shaders::ShaderExt2;
use shark::shader::ShaderExt;

mod network_tables;
mod renderer;
mod shaders;
mod spi;
mod strips;

const DESIRED_FPS: f64 = 320.0;
const SLEEP_DURATION: Duration = Duration::from_millis((1.0 / DESIRED_FPS * 1000.0) as u64);

fn main() {
    // let voltage = network_tables::start_nt_daemon_task();
    // let mut last_voltage = *voltage.lock().unwrap();

    let start_instant = Instant::now();

    let mut shader =
        shaders::slide_over_time(shaders::random_pride_flag().scale_position(1.0 / 0.5))
            .scale_time(0.3)
            .extrude()
            .extrude()
            .arc();

    let points = strips::test_strip().collect::<Vec<_>>();

    let pin_10_renderer = renderer::Renderer::new(8, spi::gpio_10().unwrap());

    loop {
        let loop_start = Instant::now();

        // let voltage = voltage.lock().unwrap();
        // if *voltage != last_voltage {
        //     last_voltage = *voltage;
        //     shader = shaders::arc_shader(shaders::battery_indicator(last_voltage));
        // }
        // drop(voltage);

        let time = start_instant.elapsed().as_secs_f64();

        pin_10_renderer.render(shader.clone(), points.clone(), time);

        let sleep_dur = SLEEP_DURATION.saturating_sub(loop_start.elapsed());
        print!(
            "\rLoop Time: {}us Sleeping for {}ms",
            loop_start.elapsed().as_micros(),
            sleep_dur.as_millis()
        );
        sleep(sleep_dur);
    }
}
