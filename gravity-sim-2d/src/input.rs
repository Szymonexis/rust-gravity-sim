//! Mouse input: raw state tracking, and translating winit events into
//! [`Camera`] operations.
//!
//! Controls implemented here:
//!
//! | input                | action                        |
//! |----------------------|-------------------------------|
//! | scroll wheel         | zoom (anchored at the cursor) |
//! | left button + move   | pan                           |
//!
//! This module owns the coordinate convention change: winit reports cursor
//! positions in physical pixels with the origin at the window's *top-left*
//! corner and +y pointing *down*
//! (<https://docs.rs/winit/0.30/winit/event/enum.WindowEvent.html#variant.CursorMoved>),
//! while the camera and shader work in pixels from the window *center* with
//! +y pointing *up* (matching clip space). [`centered`] converts between the
//! two; nothing past this module ever sees top-left coordinates.

use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, MouseButton, MouseScrollDelta},
};

use crate::camera::Camera;

/// Scroll amount that counts as one wheel "notch" when the platform reports
/// pixel deltas (touchpads, mice with free-spinning wheels) instead of line
/// deltas. 60 px/notch is a conventional approximation.
const SCROLL_PIXELS_PER_LINE: f32 = 60.0;

/// Tracks which buttons are held and where the cursor is, and applies the
/// resulting camera motions. Owned by `App`, fed from its event handler.
#[derive(Default)]
pub struct CameraController {
    /// Last known cursor position in centered coordinates (+y up), or `None`
    /// until the cursor first enters the window.
    cursor: Option<[f32; 2]>,
    left_down: bool,
}

impl CameraController {
    /// Record button presses/releases. Called for every
    /// [`winit::event::WindowEvent::MouseInput`].
    ///
    /// Note: winit delivers a release even if the cursor has left the window
    /// mid-drag on most platforms, so no extra "cancel drag" handling is
    /// needed for the common cases.
    pub fn on_mouse_button(&mut self, button: MouseButton, state: ElementState) {
        if button == MouseButton::Left {
            self.left_down = state == ElementState::Pressed;
        }
    }

    /// Handle cursor movement: pan while the left button is held.
    ///
    /// `resolution` is the current window client size in physical pixels,
    /// needed to convert to centered coordinates.
    pub fn on_cursor_moved(
        &mut self,
        position: PhysicalPosition<f64>,
        resolution: [f32; 2],
        camera: &mut Camera,
    ) {
        let cursor = centered(position, resolution);
        // Motions need a previous position; the first event only records one.
        if let Some(prev) = self.cursor {
            if self.left_down {
                camera.pan_by([cursor[0] - prev[0], cursor[1] - prev[1]]);
            }
        }
        self.cursor = Some(cursor);
    }

    /// Handle the scroll wheel: zoom, anchored at the cursor so the world
    /// point under it stays put. Falls back to the window center when the
    /// cursor position is unknown (wheel event before any cursor event).
    pub fn on_scroll(&mut self, delta: MouseScrollDelta, camera: &mut Camera) {
        // Mice report LineDelta (whole notches); touchpads report PixelDelta.
        // https://docs.rs/winit/0.30/winit/event/enum.MouseScrollDelta.html
        let lines = match delta {
            MouseScrollDelta::LineDelta(_, y) => y,
            MouseScrollDelta::PixelDelta(p) => p.y as f32 / SCROLL_PIXELS_PER_LINE,
        };
        let anchor = self.cursor.unwrap_or([0.0, 0.0]);
        camera.zoom_by(lines, anchor);
    }
}

/// Convert a winit cursor position (top-left origin, +y down) into centered
/// screen coordinates (window-center origin, +y up).
fn centered(position: PhysicalPosition<f64>, resolution: [f32; 2]) -> [f32; 2] {
    [
        position.x as f32 - resolution[0] / 2.0,
        resolution[1] / 2.0 - position.y as f32,
    ]
}
