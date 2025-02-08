use palette::{IntoColor, LinSrgb};
use shark::shader::{FragOne, IntoShader, Shader, ShaderExt, primitives::checkerboard};

pub fn slide_over_time<S: Shader<FragOne>>(shader: S) -> impl Shader<FragOne> {
    (move |frag: FragOne| {
        let new_pos = frag.pos + frag.time;
        shader.shade(FragOne {
            pos: new_pos,
            ..frag
        })
    })
    .into_shader()
}

pub fn conveyor<S1: Shader<FragOne>, S2: Shader<FragOne>>(
    shader1: S1,
    shader2: S2,
    section_len: f64,
    speed: f64,
) -> impl Shader<FragOne> {
    slide_over_time(checkerboard(shader1, shader2, section_len)).scale_time(speed)
}

pub fn segments<O: IntoColor<LinSrgb<f64>> + Send + Sync>(
    segments: Vec<(Box<dyn Shader<FragOne, Output = O>>, f64)>,
) -> impl Shader<FragOne, Output = O> {
    let mut ranges = Vec::with_capacity(segments.len());
    let mut pos = 0.0;
    for i in 0..segments.len() - 1 {
        ranges.push((pos, segments[i + 1].1 + pos));
        pos += segments[i].1;
    }
    let total_len: f64 = segments.iter().map(|(_, len)| len).sum();
    (move |frag: FragOne| {
        let frag = FragOne {
            pos: frag.pos % total_len,
            ..frag
        };
        for (i, range) in ranges.iter().enumerate() {
            if range.0 <= frag.pos && frag.pos < range.1 {
                return segments[i].0.shade(frag);
            }
        }
        segments[0].0.shade(frag)
    })
    .into_shader()
}

pub fn uniform_segments<O: IntoColor<LinSrgb<f64>> + Send + Sync>(
    segments_: Vec<Box<dyn Shader<FragOne, Output = O>>>,
) -> impl Shader<FragOne> {
    let len = 1.0 / segments_.len() as f64;
    segments(
        segments_
            .into_iter()
            .map(|shader| (shader, len))
            .collect(),
    )
}
