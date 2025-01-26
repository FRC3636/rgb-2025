use palette::{IntoColor, LinSrgb};
use shark::shader::{
    FragOne, FragThree, Fragment, IntoShader, Shader, ShaderExt,
    primitives::{checkerboard, color, memoize, time_rainbow},
};

fn slide_over_time<S: Shader<FragOne>>(shader: S) -> impl Shader<FragOne> {
    (move |frag: FragOne| {
        let new_pos = frag.pos + frag.time;
        shader.shade(FragOne {
            pos: new_pos,
            ..frag
        })
    })
    .into_shader()
}

fn conveyor<S1: Shader<FragOne>, S2: Shader<FragOne>>(
    shader1: S1,
    shader2: S2,
    section_len: f64,
    speed: f64,
) -> impl Shader<FragOne> {
    slide_over_time(checkerboard(shader1, shader2, section_len)).scale_time(speed)
}

fn to_linsrgb<F: Fragment, S: Shader<F>>(shader: S) -> impl Shader<F, Output = LinSrgb<f64>> {
    (move |frag: F| shader.shade(frag).into_color()).into_shader()
}

pub fn battery_indicator(voltage: f64) -> impl Shader<FragThree> {
    let low_voltage_color = color(LinSrgb::new(1.0, 0.03, 0.01));

    let color = low_voltage_color.rotate_hue(voltage / 12.0 * 90.0);

    // memoize(
    to_linsrgb(conveyor(color, time_rainbow().scale_time(100.0), 0.3, 0.5))
        //     Some(0.04),
        //     true,
        // )
        .volume_blur(0.1, 12)
        .extrude()
        .extrude()
}
