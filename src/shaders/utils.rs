use std::{sync::{Arc, Mutex, RwLock}, time::Instant};

use palette::{rgb::channels::Argb, IntoColor, LinSrgb, Srgb, Mix};
use shark::shader::{FragOne, Fragment, IntoShader, Shader, primitives::color};

pub fn to_linsrgb<F: Fragment, S: Shader<F>>(shader: S) -> impl Shader<F, Output = LinSrgb<f64>> {
    (move |frag: F| shader.shade(frag).into_color()).into_shader()
}

pub struct BoxShader<F: Fragment, O: IntoColor<LinSrgb<f64>>>(Box<dyn Shader<F, Output = O>>);
impl<F: Fragment, O: IntoColor<LinSrgb<f64>> + Send + Sync> Shader<F> for BoxShader<F, O> {
    type Output = O;

    fn shade(&self, frag: F) -> Self::Output {
        self.0.shade(frag)
    }
}
pub fn box_shader<F: Fragment, O: IntoColor<LinSrgb<f64>> + Send + Sync>(
    shader: Box<dyn Shader<F, Output = O>>,
) -> BoxShader<F, O> {
    BoxShader(shader)
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

pub fn hex(hex: u32) -> Box<dyn Shader<FragOne, Output = LinSrgb<f64>>> {
    let rgb: Srgb<u8> = Srgb::from_u32::<Argb>(hex);
    Box::new(color(rgb.into_linear()))
}

pub struct TransitionShader<F: Fragment, S: Shader<F>, T: Shader<F>> {
    start: S,
    end: T,
    last_switch: Instant,
    duration: f64,

    _phantom: std::marker::PhantomData<F>,
}

impl<O: IntoColor<LinSrgb<f64>> + Send + Sync, F: Fragment, S: Shader<F, Output = O>, T: Shader<F, Output = O>> Shader<F> for TransitionShader<F, S, T> {
    type Output = LinSrgb<f64>;

    fn shade(&self, frag: F) -> Self::Output {
        let elapsed = self.last_switch.elapsed();

        let factor = (elapsed.as_secs_f64() / self.duration).min(1.0);

        let end_color = self.end.shade(frag).into_color();

        if factor == 1.0 {
            return end_color;
        }
        
        let start_color = self.start.shade(frag).into_color();

        start_color.mix(end_color, factor)
    }
}

pub fn transition<F: Fragment, S: Shader<F>, T: Shader<F>>(
    start: S,
    end: T,
    duration: f64,
    last_switch: Instant,
) -> TransitionShader<F, S, T> {
    TransitionShader {
        start,
        end,
        last_switch,
        duration,
        _phantom: std::marker::PhantomData,
    }
}