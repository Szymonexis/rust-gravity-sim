/// Ratio between two neighbouring rungs of the speed ladder.
const FACTOR: f32 = 1.1;
/// Hard clamp on the magnitude, in both directions.
const LIMIT: f32 = 10.0;

/// How far a single arrow press moves the world while the simulation is paused.
pub const STEP_TICKS: i32 = 10;

/// Speeds live on a geometric ladder indexed by an integer, because a
/// multiplier can never *reach* zero by being divided - and without zero there
/// is no way across to reverse.
///
/// Index 0 is a full stop. Every step away from it multiplies the previous
/// magnitude by [`FACTOR`], so index 1 is +1x, index -1 is -1x, and the two
/// ends land on [`LIMIT`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Speed {
    index: i32,
}

/// Smallest index whose magnitude has reached the clamp. Computed rather than
/// written down so [`FACTOR`] and [`LIMIT`] stay the only knobs.
const MAX_INDEX: i32 = {
    let mut index = 1;
    let mut magnitude = 1.0;
    while magnitude < LIMIT {
        magnitude *= FACTOR;
        index += 1;
    }
    index
};

impl Default for Speed {
    fn default() -> Self {
        Self::NORMAL
    }
}

impl Speed {
    /// Real time: one simulated tick per configured tick period.
    pub const NORMAL: Self = Self { index: 1 };

    pub fn faster(&mut self) {
        self.index = (self.index + 1).min(MAX_INDEX);
    }

    pub fn slower(&mut self) {
        self.index = (self.index - 1).max(-MAX_INDEX);
    }

    /// Multiplier on the tick rate. Negative runs the world backwards.
    pub fn value(self) -> f32 {
        match self.index.signum() {
            0 => 0.0,
            sign => sign as f32 * FACTOR.powi(self.index.abs() - 1).min(LIMIT),
        }
    }
}

impl std::fmt::Display for Speed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = self.value();
        if value == 0.0 {
            write!(f, "halted")
        } else {
            write!(f, "{value:+.2}x")
        }
    }
}
