use palette::LinSrgb;
use shark::shader::{primitives::color, FragThree, IntoShader, Shader};

pub fn alliance() -> impl Shader<FragThree> {
    (|frag: FragThree| {
        if frag.pos[0] > 500.0 {
            color(LinSrgb::new(0.0, 0.0, 1.0)).shade(frag)
        } else {
            color(LinSrgb::new(1.0, 0.0, 0.0)).shade(frag)
        }
    }).into_shader()
} 