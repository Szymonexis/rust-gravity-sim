#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PositionStrategy {
    Uniform,
    Gaussian { standard_deviation: f32 },
    Ring { inner_fraction: f32 },
    Sunflower,
    Clusters { count: usize, spread: f32 },
}
