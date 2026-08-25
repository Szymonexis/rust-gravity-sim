use super::{ColorRGBA, WHITE, mix};

pub struct ColorRamp {
    stops: Vec<(f32, ColorRGBA)>,
}

impl ColorRamp {
    pub fn new(mut stops: Vec<(f32, ColorRGBA)>) -> Self {
        stops.sort_by(|(left, _), (right, _)| left.total_cmp(right));
        Self { stops }
    }

    pub fn sample(&self, value: f32) -> ColorRGBA {
        let Some(&(first_at, first)) = self.stops.first() else {
            return WHITE;
        };
        if value.is_nan() || value <= first_at {
            return first;
        }

        let upper = self.stops.partition_point(|(at, _)| *at <= value);
        let (from_at, from) = self.stops[upper - 1];
        let Some(&(to_at, to)) = self.stops.get(upper) else {
            return from;
        };

        let span = to_at - from_at;
        let delta = if span > 0.0 {
            (value - from_at) / span
        } else {
            0.0
        };

        mix(from, to, delta)
    }
}
