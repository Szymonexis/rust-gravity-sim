//! The 2D camera: which part of the world is on screen, and at what scale.
//!
//! The camera is two values — [`Camera::pan`] and [`Camera::zoom`] — that
//! define the world→screen mapping implemented in `src/gpu/shader.wgsl`:
//!
//! ```text
//! screen_px = world * zoom + pan
//! ```
//!
//! "Screen" here means physical pixels measured from the *window center*,
//! with +y up (the same orientation as clip space, so the shader needs no
//! flip). Raw window events use top-left origin, +y down; `src/input.rs`
//! converts before anything reaches this module.
//!
//! Because the mapping is defined in pixels, the window size appears nowhere
//! in it — that is the invariant that keeps the scene from scaling or moving
//! when the window is resized. Resizing only changes how much of the world is
//! visible around the center.

/// Pixels per world unit at startup. The square in the shader is 1 world unit
/// wide, so it starts out `INITIAL_ZOOM` pixels wide in any window.
pub const INITIAL_ZOOM: f32 = 400.0;

/// Zoom clamp range: 0.1 px/unit (very far out) to 10k px/unit (very close).
/// Widen these if the simulation ever needs more range.
pub const MIN_ZOOM: f32 = 0.1;
pub const MAX_ZOOM: f32 = 10_000.0;

/// Multiplier applied per scroll "line". Exponential zoom (factor per step,
/// not amount per step) feels uniform at every scale: each notch changes the
/// on-screen size by 10%.
pub const ZOOM_STEP: f32 = 1.1;

pub struct Camera {
    /// Screen position of the world origin (the scene center), in pixels from
    /// the window center, +y up. `[0, 0]` = scene centered in the window.
    pub pan: [f32; 2],
    /// Pixels per world unit.
    pub zoom: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            pan: [0.0, 0.0],
            zoom: INITIAL_ZOOM,
        }
    }
}

impl Camera {
    /// Translate the scene by a screen-space delta (pixels, +y up).
    ///
    /// Pan is applied *after* zoom in the shader, so the scene follows the
    /// cursor 1:1 regardless of zoom level.
    pub fn pan_by(&mut self, delta: [f32; 2]) {
        self.pan[0] += delta[0];
        self.pan[1] += delta[1];
    }

    /// Zoom by `scroll_lines` wheel notches, keeping the world point under
    /// `anchor` (screen px, +y up) fixed on screen — "zoom to cursor".
    ///
    /// Derivation: a world point `w` is rendered at screen position
    /// `a = w * zoom + pan`. Requiring the same `a` after changing zoom by
    /// factor `k = zoom' / zoom` gives
    ///
    /// ```text
    /// pan' = a - (a - pan) * k
    /// ```
    ///
    /// i.e. the world origin slides away from / toward the anchor by exactly
    /// the zoom factor. Same trick as in every map application.
    pub fn zoom_by(&mut self, scroll_lines: f32, anchor: [f32; 2]) {
        let new_zoom = (self.zoom * ZOOM_STEP.powf(scroll_lines)).clamp(MIN_ZOOM, MAX_ZOOM);
        let k = new_zoom / self.zoom;
        self.pan[0] = anchor[0] - (anchor[0] - self.pan[0]) * k;
        self.pan[1] = anchor[1] - (anchor[1] - self.pan[1]) * k;
        self.zoom = new_zoom;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The world→screen mapping, exactly as the shader computes it
    /// (`shader.wgsl`: `pixels = world * zoom + pan`). Tests assert against
    /// this projection so they verify what actually ends up on screen.
    fn project(cam: &Camera, world: [f32; 2]) -> [f32; 2] {
        [
            world[0] * cam.zoom + cam.pan[0],
            world[1] * cam.zoom + cam.pan[1],
        ]
    }

    #[test]
    fn starts_centered_at_initial_zoom() {
        let cam = Camera::default();
        assert_eq!(cam.pan, [0.0, 0.0]);
        assert_eq!(cam.zoom, INITIAL_ZOOM);
        // The square spans world ±0.5, so its on-screen width is exactly
        // INITIAL_ZOOM pixels, centered on the window center.
        assert_eq!(project(&cam, [0.5, 0.0])[0], INITIAL_ZOOM / 2.0);
    }

    #[test]
    fn pan_moves_scene_one_to_one() {
        let mut cam = Camera::default();
        cam.pan_by([150.0, -80.0]);
        // Every world point shifts by exactly the cursor delta, in pixels.
        assert_eq!(project(&cam, [0.0, 0.0]), [150.0, -80.0]);
        assert_eq!(project(&cam, [0.3, 0.2])[0], 0.3 * INITIAL_ZOOM + 150.0);
    }

    #[test]
    fn zoom_is_exponential_in_notches() {
        let mut cam = Camera::default();
        cam.zoom_by(5.0, [0.0, 0.0]);
        assert!((cam.zoom - INITIAL_ZOOM * ZOOM_STEP.powi(5)).abs() < 1e-3);
        cam.zoom_by(-5.0, [0.0, 0.0]);
        assert!((cam.zoom - INITIAL_ZOOM).abs() < 1e-3);
    }

    #[test]
    fn zoom_keeps_anchor_point_fixed() {
        let mut cam = Camera {
            pan: [37.0, -12.0],
            zoom: 400.0,
        };
        // The world point currently under the anchor...
        let anchor = [100.0, 50.0];
        let world_at_anchor = [
            (anchor[0] - cam.pan[0]) / cam.zoom,
            (anchor[1] - cam.pan[1]) / cam.zoom,
        ];
        // ...must still be under the anchor after zooming, in or out.
        cam.zoom_by(3.0, anchor);
        let after_in = project(&cam, world_at_anchor);
        assert!((after_in[0] - anchor[0]).abs() < 1e-3);
        assert!((after_in[1] - anchor[1]).abs() < 1e-3);

        cam.zoom_by(-7.0, anchor);
        let after_out = project(&cam, world_at_anchor);
        assert!((after_out[0] - anchor[0]).abs() < 1e-3);
        assert!((after_out[1] - anchor[1]).abs() < 1e-3);
    }

    #[test]
    fn zoom_clamps_to_range() {
        let mut cam = Camera::default();
        cam.zoom_by(1000.0, [0.0, 0.0]);
        assert_eq!(cam.zoom, MAX_ZOOM);
        cam.zoom_by(-10_000.0, [0.0, 0.0]);
        assert_eq!(cam.zoom, MIN_ZOOM);
    }
}
