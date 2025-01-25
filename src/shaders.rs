use palette::LinSrgb;
use shark::shader::{
    primitives::{checkerboard, color, mix, off}, FragOne, FragThree, IntoShader, Shader, ShaderExt
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

pub fn battery_indicator(voltage: f64) -> impl Shader<FragThree> {
    let low_voltage_color = color(LinSrgb::new(1.0, 0.03, 0.01));
    let high_voltage_color = color(LinSrgb::new(0.03, 1.0, 0.04));

    let color = mix(
        high_voltage_color,
        low_voltage_color,
        (12.0 - voltage) / 12.0,
    );
    let inverted_color = mix(mix(
        high_voltage_color,
        low_voltage_color,
        (12.0 - voltage) / 12.0,
    ), off(), 0.8);

    conveyor(
        color,
        inverted_color,
        0.1,
        0.2,
    )
    .extrude()
    .extrude()
}
