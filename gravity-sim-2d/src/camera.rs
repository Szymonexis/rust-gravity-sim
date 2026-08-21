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
