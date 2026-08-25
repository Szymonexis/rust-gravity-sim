const FACTOR: f32 = 1.1;
const LIMIT: f32 = 10.0;

pub const STEP_TICKS: i32 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Speed {
    index: i32,
}

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
    pub const NORMAL: Self = Self { index: 1 };

    pub fn faster(&mut self) {
        self.index = (self.index + 1).min(MAX_INDEX);
    }

    pub fn slower(&mut self) {
        self.index = (self.index - 1).max(-MAX_INDEX);
    }

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
