use palette::{rgb::channels::Argb, IntoColor, LinSrgb, Srgb};
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