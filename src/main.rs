use std::{
    thread::sleep,
    time::{Duration, Instant},
};

use network_tables::NtReactives;
use palette::LinSrgb;
use shaders::ShaderExt2;
use shark::shader::{ShaderExt, primitives::color};

mod drivers;
mod network_tables;
mod renderer;
mod shaders;
mod strips;

const DESIRED_FPS: f64 = 101.0;
const SLEEP_DURATION: Duration = Duration::from_millis((1.0 / DESIRED_FPS * 1000.0) as u64);

fn main() {
    // let NtReactives {
    //     voltage: _voltage,
    //     robot_pos,
    //     coral_state,
    // } = network_tables::start_nt_daemon_task();
    // let mut last_coral_state = *coral_state.lock().unwrap();

    let start_instant = Instant::now();

    let mut underglow_shader = shaders::coral_state_indicator(network_tables::CoralState::Held).arc();

    // let shader = color(LinSrgb::new(0.0, 1.0, 0.0)).arc();

    let boxtube_points = strips::box_tube_to_intake().collect::<Vec<_>>();
    let underglow_points = strips::underglow().collect::<Vec<_>>();

    let pin_10_renderer = renderer::Renderer::new(1, drivers::spi::gpio_10().unwrap());
    let pin_18_renderer = renderer::Renderer::new(1, drivers::spi::gpio_18().unwrap());

    loop {
        let loop_start = Instant::now();

        // let coral_state = coral_state.lock().unwrap();
        // if *coral_state != last_coral_state {
        //     last_coral_state = *coral_state;
        //     underglow_shader = shaders::coral_state_indicator(*coral_state).arc();
        // }
        // drop(coral_state);

        let time = start_instant.elapsed().as_secs_f64();

        // let offset_points = underglow_points
        //     .iter()
        //     .copied()
        //     .map(|mut point| {
        //         point.x += robot_pos.lock().unwrap()[0];
        //         point.y += robot_pos.lock().unwrap()[1];
        //         point.z += robot_pos.lock().unwrap()[2];
        //         point
        //     })
        //     .collect::<Vec<_>>();

        // pin_10_renderer.render(color(LinSrgb::new(1.0, 1.0, 1.0)), offset_points, time);
        pin_10_renderer.render(
            color(LinSrgb::new(1.0, 1.0, 1.0)),
            boxtube_points.clone(),
            time,
        );

        let sleep_dur = SLEEP_DURATION.saturating_sub(loop_start.elapsed());
        print!(
            "\rLoop Time: {}us Sleeping for {}ms",
            loop_start.elapsed().as_micros(),
            sleep_dur.as_millis()
        );
        sleep(sleep_dur);
    }
}
