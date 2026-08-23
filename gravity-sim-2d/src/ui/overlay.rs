use crate::color::ColorRGBA;
use crate::simulation::{STEP_TICKS, Speed};
use crate::ui::panel::{Line, Panel};

const HEADING: ColorRGBA = [0.46, 0.58, 0.78, 1.0];
const LABEL: ColorRGBA = [0.52, 0.57, 0.65, 1.0];
const VALUE: ColorRGBA = [0.90, 0.93, 0.97, 1.0];
const RUNNING: ColorRGBA = [0.42, 0.85, 0.52, 1.0];
const PAUSED: ColorRGBA = [1.00, 0.74, 0.25, 1.0];

/// Width of the label column, in characters. Lines up exactly only under a
/// monospaced font, which the bundled fallback is.
const LABEL_WIDTH: usize = 12;

/// A flat snapshot of everything the overlay reads. Copying it once per frame
/// keeps the ui from reaching into the camera or the simulation handle, so
/// neither has to grow accessors it doesn't otherwise need.
pub struct Status {
    pub particles: usize,
    pub tick: i64,
    pub fps: f32,
    pub tps: f32,
    pub zoom: f32,
    pub pan: [f32; 2],
    pub paused: bool,
    pub speed: Speed,
}

/// Left corner: what the world is doing.
pub fn stats(status: &Status) -> Panel {
    let mut panel = Panel::default();

    panel.push(heading("SIMULATION"));
    panel.push(entry("particles", status.particles.to_string()));
    panel.push(entry("tick", status.tick.to_string()));
    panel.push(entry("fps", format!("{:.0}", status.fps)));
    panel.push(entry("tps", format!("{:.0}", status.tps)));
    panel.push(entry("zoom", format!("{:.2}x", status.zoom)));
    panel.push(entry(
        "pan",
        format!("{:.0}, {:.0}", status.pan[0], status.pan[1]),
    ));

    panel
}

/// Right corner: what you are doing to it, and how to do more.
pub fn playback(status: &Status, show_manual: bool, config_file: Option<&str>) -> Panel {
    let mut panel = Panel::default();

    panel.push(if status.paused {
        Line::default().span("PAUSED", PAUSED)
    } else {
        Line::default().span("RUNNING", RUNNING)
    });
    panel.push(entry("speed", status.speed.to_string()));

    if show_manual {
        panel.blank();
        panel.push(heading("CONTROLS"));
        panel.push(entry("drag", "pan"));
        panel.push(entry("wheel", "zoom"));
        panel.push(entry(
            "space",
            if status.paused { "resume" } else { "pause" },
        ));
        panel.push(entry(
            "left/right",
            if status.paused {
                format!("step {STEP_TICKS} ticks")
            } else {
                "slower / faster".to_owned()
            },
        ));
        panel.push(entry("esc", "quit"));

        // Everything the sim starts with - particle count, window, colours,
        // this font - comes out of that file. Nothing in here edits it, so the
        // overlay says where it is and leaves the rest to the user's editor.
        if let Some(path) = config_file {
            panel.blank();
            panel.push(heading("SETTINGS"));
            panel.push(Line::default().span(path, VALUE));
            panel.push(Line::default().span("edit it, then restart", LABEL));
        }
    }

    panel
}

fn heading(text: &str) -> Line {
    Line::default().span(text, HEADING)
}

/// One `label   value` row, dim label against a bright value.
fn entry(label: &str, value: impl AsRef<str>) -> Line {
    Line::default()
        .span(format!("{label:<LABEL_WIDTH$}"), LABEL)
        .span(value.as_ref(), VALUE)
}
