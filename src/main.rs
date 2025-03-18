use std::{
    thread::sleep,
    time::{Duration, Instant},
};

use network_tables::NtReactives;
use palette::LinSrgb;
use shaders::{ShaderExt2, box_shader, coral_state_indicator, transition};
use shark::shader::{ShaderExt, primitives::color};

mod drivers;
mod network_tables;
mod renderer;
mod shaders;
mod strips;

const DESIRED_FPS: f64 = 101.0;
const SLEEP_DURATION: Duration = Duration::from_millis((1.0 / DESIRED_FPS * 1000.0) as u64);

fn main() {
    let NtReactives {
        voltage: _voltage,
        robot_pos: _robot_pos,
        coral_state,
        coral_state_last_change,
    } = network_tables::start_nt_daemon_task();

    let start_instant = Instant::now();

    let mut underglow_shader = box_shader(Box::new(
        color(LinSrgb::new(0.0, 0.0, 0.0)).extrude().extrude(),
    ))
    .arc();

    let boxtube_points = strips::box_tube_to_intake().collect::<Vec<_>>();

    let pin_10_renderer = renderer::Renderer::new(2, drivers::spi::gpio_10().unwrap());

    loop {
        let loop_start = Instant::now();

        underglow_shader = box_shader(Box::new(transition(
            underglow_shader,
            coral_state_indicator(*coral_state.lock().unwrap()).to_linsrgb(),
            0.8,
            *coral_state_last_change.read().unwrap(),
        )))
        .arc();

        let time = start_instant.elapsed().as_secs_f64();

        pin_10_renderer.render(underglow_shader.clone(), boxtube_points.clone(), time);

        let sleep_dur = SLEEP_DURATION.saturating_sub(loop_start.elapsed());
        print!(
            "\rLoop Time: {}us Sleeping for {}ms",
            loop_start.elapsed().as_micros(),
            sleep_dur.as_millis()
        );
        sleep(sleep_dur);
    }
}
