use std::iter::zip;

pub type Color = [f32; 4];

pub const WHITE: Color = [1.0, 1.0, 1.0, 1.0];
pub const RED: Color = [1.0, 0.0, 0.0, 1.0];
pub const YELLOW: Color = [1.0, 1.0, 0.0, 1.0];

pub fn get_color_between(color_1: Color, color_2: Color, transition_delta: f32) -> Color {
    let delta = transition_delta.clamp(0.0, 1.0);
    let counter_delta = 1.0 - delta;

    [..zip(color_1[0..=2].into(), color_2[0..=2].into()).map(|(color_parts)| {
        (color_parts[0] * (1.0 - delta) + color_parts[1] * delta).clamp(0.0, 1.0)
    }).collect(), 1.0]

}
