use palette::{IntoColor, LinSrgb};
use shark::shader::{
    primitives::{checkerboard, color, off, time_rainbow}, FragOne, FragThree, Fragment, IntoShader, Shader, ShaderExt
};

pub fn to_linsrgb<F: Fragment, S: Shader<F>>(shader: S) -> impl Shader<F, Output = LinSrgb<f64>> {
    (move |frag: F| shader.shade(frag).into_color()).into_shader()
}

#[derive(Debug)]
pub struct ArcShader<F: Fragment, S: Shader<F>>(std::sync::Arc<S>, std::marker::PhantomData<F>);
impl<F: Fragment, S: Shader<F>> Clone for ArcShader<F, S> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), std::marker::PhantomData)
    }
}
impl<F: Fragment, S: Shader<F>> Shader<F> for ArcShader<F, S> {
    type Output = S::Output;

    fn shade(&self, frag: F) -> Self::Output {
        self.0.shade(frag)
    }
}

pub fn arc_shader<F: Fragment, S: Shader<F>>(shader: S) -> ArcShader<F, S> {
    ArcShader(std::sync::Arc::new(shader), std::marker::PhantomData)
}

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

pub fn flowy_rainbow() -> impl Shader<FragThree> {
    let rainbow = || time_rainbow().scale_time(40.0);
    to_linsrgb(conveyor(rainbow(), rainbow().mix(off(), 0.7), 0.3, 0.4))
        .volume_blur(0.1, 10)
        .extrude()
        .extrude()
}
