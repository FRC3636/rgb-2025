use std::{
    thread::sleep,
    time::{Duration, Instant},
};

use network_tables::NtReactives;
use palette::LinSrgb;
use shaders::ShaderExt2;
use shark::shader::{primitives::color, ShaderExt};

mod network_tables;
mod renderer;
mod shaders;
mod strips;
mod drivers;

const DESIRED_FPS: f64 = 320.0;
const SLEEP_DURATION: Duration = Duration::from_millis((1.0 / DESIRED_FPS * 1000.0) as u64);

fn main() {
    unsafe { drivers::dma_pwm::timer_read_test() };

    // let NtReactives { voltage: _voltage, robot_pos } = network_tables::start_nt_daemon_task();
    // // let mut last_voltage = *voltage.lock().unwrap();

    // let start_instant = Instant::now();

    // // let shader =
    // //     shaders::slide_over_time(shaders::transgender().scale_position(1.0 / 0.5))
    // //         .scale_time(0.3)
    // //         .extrude()
    // //         .extrude()
    // //         .arc();
    // let shader = color(LinSrgb::new(0.0, 1.0, 0.0)).arc();

    // let points = strips::test_strip().collect::<Vec<_>>();

    // let pin_10_renderer = renderer::Renderer::new(1, drivers::spi::gpio_10().unwrap());

    // loop {
    //     let loop_start = Instant::now();

    //     // let voltage = voltage.lock().unwrap();
    //     // if *voltage != last_voltage {
    //     //     last_voltage = *voltage;
    //     //     shader = shaders::arc_shader(shaders::battery_indicator(last_voltage));
    //     // }
    //     // drop(voltage);

    //     let time = start_instant.elapsed().as_secs_f64();

    //     let offset_points = points
    //         .iter()
    //         .copied()
    //         .map(|mut point| {
    //             point.x += robot_pos.lock().unwrap()[0];
    //             point.y += robot_pos.lock().unwrap()[1];
    //             point.z += robot_pos.lock().unwrap()[2];
    //             point
    //         })
    //         .collect::<Vec<_>>();

    //     pin_10_renderer.render(shader.clone(), offset_points, time);

    //     let sleep_dur = SLEEP_DURATION.saturating_sub(loop_start.elapsed());
    //     print!(
    //         "\rLoop Time: {}us Sleeping for {}ms",
    //         loop_start.elapsed().as_micros(),
    //         sleep_dur.as_millis()
    //     );
    //     sleep(sleep_dur);
    // }
}
