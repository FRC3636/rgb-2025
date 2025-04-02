pub mod atoms;
pub mod field_relative;
pub mod pride;
pub mod utils;

use palette::{LinSrgb, Mix};
use shark::shader::{
    FragOne, FragThree, Fragment, IntoShader, Shader, ShaderExt,
    primitives::{color, off, time_rainbow},
};

pub use atoms::*;
use shrewnit::{Dimension, Length, Meters};
pub use utils::*;

use crate::network_tables::{CoralState, MovementState};

pub trait ShaderExt2<F: Fragment>: Shader<F> + Sized {
    fn to_linsrgb(self) -> impl Shader<F, Output = LinSrgb<f64>> {
        to_linsrgb(self)
    }
    fn arc(self) -> ArcShader<F, Self> {
        arc_shader(self)
    }
    fn boxed(this: Box<Self>) -> BoxShader<F, Self::Output>
    where
        Self: 'static,
    {
        box_shader(this)
    }
}
impl<F: Fragment, S: Shader<F>> ShaderExt2<F> for S {}

pub fn battery_indicator(voltage: f64) -> impl Shader<FragThree> {
    let low_voltage_color = color(LinSrgb::new(1.0, 0.03, 0.01));

    let color = low_voltage_color.rotate_hue(voltage / 12.0 * 90.0);

    conveyor(color, time_rainbow().scale_time(100.0), 0.3, 0.5)
        .to_linsrgb()
        .volume_blur(0.1, 12)
        .extrude()
        .extrude()
}

pub fn flowy_rainbow() -> impl Shader<FragThree> {
    let rainbow = || time_rainbow().scale_time(40.0);
    conveyor(rainbow(), rainbow().mix(off(), 0.7), 0.3, 0.4)
        .to_linsrgb()
        .volume_blur(0.1, 10)
        .extrude()
        .extrude()
}

pub fn random_pride_flag() -> impl Shader<FragOne> {
    let num_flags = pride::FLAGS.len();
    let index: usize = rand::random_range(0..num_flags);
    let flag = pride::FLAGS[index];
    box_shader(flag())
}

fn coral_state_indicator(coral_state: CoralState) -> impl Shader<FragThree> {
    (move |frag: FragThree| match coral_state {
        // CoralState::None => flowy_rainbow().to_linsrgb().shade(frag),
        CoralState::None => conveyor(
            color(LinSrgb::new(0.0, 0.4, 0.8)),
            color(LinSrgb::new(0.0, 0.4, 0.8)).mix(off(), 0.4),
            0.2,
            0.5,
        )
        .to_linsrgb()
        .volume_blur(0.03, 8)
        .extrude()
        .extrude()
        .shade(frag),

        CoralState::Transit => conveyor(
            color(LinSrgb::new(0.03, 1.0, 0.32)),
            color(LinSrgb::new(1.0, 1.0, 1.0)),
            0.1,
            0.5,
        )
        .to_linsrgb()
        .extrude()
        .extrude()
        .shade(frag),

        CoralState::Held => conveyor(
            color(LinSrgb::new(0.2, 0.6, 0.8)),
            color(LinSrgb::new(1.0, 0.2, 1.0)),
            0.3,
            0.5,
        )
        .to_linsrgb()
        .volume_blur(0.1, 12)
        .extrude()
        .extrude()
        .shade(frag),
    })
    .into_shader()
}

fn auto_align_indicator(
    movement_state: MovementState,
    relative_pos: [Length; 2],
) -> impl Shader<FragThree> {
    (move |frag: FragThree| {
        let relative_frag = FragThree {
            pos: [
                frag.pos[0] + relative_pos[0].to::<Meters>(),
                frag.pos[1] + relative_pos[1].to::<Meters>(),
                frag.pos[2],
            ],
            ..frag
        };
        match movement_state {
            MovementState::AutoAlignPath => color(LinSrgb::new(1.0, 1.0, 1.0))
                .subtract(distance_shader([0.0, 0.0], 1.0))
                .extrude()
                .multiply(color(LinSrgb::new(0.4, 0.8, 0.2)))
                .shade(relative_frag),
            MovementState::AutoAlignPid => color(LinSrgb::new(1.0, 1.0, 1.0))
                .subtract(distance_shader([0.0, 0.0], 1.0))
                .extrude()
                .multiply(color(LinSrgb::new(0.3, 1.0, 0.8)))
                .shade(relative_frag),
            MovementState::SuccessfullyAligned => conveyor(
                color(LinSrgb::new(0.05, 1.0, 0.0)),
                color(LinSrgb::new(0.05, 1.0, 0.1)),
                0.2,
                0.5,
            )
            .to_linsrgb()
            .volume_blur(0.1, 7)
            .extrude()
            .extrude()
            .shade(frag),

            MovementState::Driver => unreachable!(),
        }
    })
    .into_shader()
}

pub fn boxtube_shader(
    coral_state: CoralState,
    movement_state: MovementState,
    relative_pos: [Length; 2],
) -> impl Shader<FragThree> {
    match movement_state {
        MovementState::Driver => {
            box_shader(Box::new(coral_state_indicator(coral_state).to_linsrgb()))
        }
        state => box_shader(Box::new(
            auto_align_indicator(state, relative_pos).to_linsrgb(),
        )),
    }
}
