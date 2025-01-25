use palette::LinSrgb;
use shark::shader::{primitives::color, FragThree, Shader};

pub fn battery_indicator(voltage: f64) -> impl Shader<FragThree> {
    let low_voltage_color = color(LinSrgb::new(1.0, 0.0, 0.0));
    let high_voltage_color = color(LinSrgb::new(0.0, 1.0, 0.0));

    shark::shader::primitives::mix(high_voltage_color, low_voltage_color, (12.0 - voltage) / 12.0)
}