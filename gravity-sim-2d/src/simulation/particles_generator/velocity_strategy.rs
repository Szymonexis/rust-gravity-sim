#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VelocityStrategy {
    Stationary,
    CommonVector { velocity: [f32; 2] },
    Random { max_speed: f32 },
    Orbital { angular_speed: f32 },
    Radial { speed: f32 },
}
