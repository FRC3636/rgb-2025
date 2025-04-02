use std::{
    thread::sleep,
    time::{Duration, Instant},
};

use network_tables::NtReactives;
use palette::LinSrgb;
use shaders::{ShaderExt2, box_shader, boxtube_shader, transition};
use shark::shader::{ShaderExt, primitives::color};
use shrewnit::Seconds;

mod drivers;
mod network_tables;
mod renderer;
mod shaders;
mod strips;

const DESIRED_FPS: f64 = 101.0;
const SLEEP_DURATION: Duration = Duration::from_millis((1.0 / DESIRED_FPS * 1000.0) as u64);

fn main() {
    let NtReactives {
        coral_state,

        movement_state,
        position_relative_to_align_target,

        topics_last_changed,
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
            boxtube_shader(
                *coral_state.lock().unwrap(),
                *movement_state.lock().unwrap(),
                *position_relative_to_align_target.lock().unwrap(),
            )
            .to_linsrgb(),
            0.4 * Seconds,
            *topics_last_changed.read().unwrap(),
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
