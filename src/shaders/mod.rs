pub mod atoms;
pub mod field_relative;
pub mod pride;
pub mod utils;

use palette::LinSrgb;
use shark::shader::{
    FragOne, FragThree, Fragment, Shader, ShaderExt,
    primitives::{color, off, time_rainbow},
};

pub use atoms::*;
pub use utils::*;

use crate::network_tables::CoralState;

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

pub fn coral_state_indicator(state: CoralState) -> impl Shader<FragThree> {
    match state {
        CoralState::None => box_shader(Box::new(flowy_rainbow().to_linsrgb())),
        CoralState::Held => box_shader(Box::new(
            conveyor(
                color(LinSrgb::new(0.0, 1.0, 0.1)),
                color(LinSrgb::new(0.2, 0.5, 0.6)),
                0.3,
                0.5,
            )
            .to_linsrgb()
            .volume_blur(0.1, 12)
            .extrude()
            .extrude(),
        )),
        CoralState::Transit => box_shader(Box::new(
            conveyor(
                color(LinSrgb::new(0.03, 1.0, 0.32)),
                color(LinSrgb::new(1.0, 1.0, 1.0)),
                0.1,
                0.5,
            )
            .to_linsrgb()
            .extrude()
            .extrude(),
        )),
    }
}
