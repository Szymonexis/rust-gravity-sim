use crate::color::ColorRGBA;
use crate::ui::font::Atlas;
use crate::ui::quad::Quad;

/// Gap between the panel's backing plate and the window edge, in logical
/// pixels - everything here is scaled by the display's scale factor on the way
/// to the GPU.
const MARGIN: f32 = 12.0;
/// Gap between the backing plate and the text inside it.
const PADDING: f32 = 8.0;
const BACKGROUND: ColorRGBA = [0.02, 0.02, 0.04, 0.62];

/// Which corner the panel grows out of. Panels only stack downwards, so the
/// corner fixes both the origin and the alignment of the text.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Anchor {
    TopLeft,
    TopRight,
}

/// A run of characters sharing one colour. A line is a list of them, which is
/// what lets a dim label and a bright value sit on the same row.
struct Span {
    text: String,
    color: ColorRGBA,
}

#[derive(Default)]
pub struct Line {
    spans: Vec<Span>,
}

impl Line {
    pub fn span(mut self, text: impl Into<String>, color: ColorRGBA) -> Self {
        self.spans.push(Span {
            text: text.into(),
            color,
        });
        self
    }

    fn width(&self, atlas: &Atlas) -> f32 {
        self.spans
            .iter()
            .map(|span| atlas.measure(&span.text))
            .sum()
    }
}

#[derive(Default)]
pub struct Panel {
    lines: Vec<Line>,
}

impl Panel {
    pub fn push(&mut self, line: Line) {
        self.lines.push(line);
    }

    /// A vertical gap. Occupies a row, draws nothing.
    pub fn blank(&mut self) {
        self.lines.push(Line::default());
    }

    /// Turn the panel into quads: one backing plate, then a quad per inked
    /// glyph. The plate goes first, so the painter ordering within the single
    /// draw call puts the text on top of it.
    pub fn layout(
        &self,
        quads: &mut Vec<Quad>,
        atlas: &Atlas,
        anchor: Anchor,
        resolution: [f32; 2],
        scale: f32,
    ) {
        if self.lines.is_empty() {
            return;
        }

        let margin = MARGIN * scale;
        let padding = PADDING * scale;

        let width = self
            .lines
            .iter()
            .map(|line| line.width(atlas))
            .fold(0.0, f32::max);
        let height = atlas.line_height * self.lines.len() as f32;

        let left = match anchor {
            Anchor::TopLeft => margin + padding,
            Anchor::TopRight => resolution[0] - margin - padding - width,
        };
        let top = margin + padding;

        quads.push(Quad::rect(
            [
                left - padding,
                top - padding,
                width + padding * 2.0,
                height + padding * 2.0,
            ],
            BACKGROUND,
        ));

        for (row, line) in self.lines.iter().enumerate() {
            // Right-aligned panels keep their column pinned to the corner even
            // as the numbers inside change width.
            let mut pen = match anchor {
                Anchor::TopLeft => left,
                Anchor::TopRight => left + width - line.width(atlas),
            };
            // Top of the line box, not the baseline - `push_line` finds the
            // baseline from here by stepping down the font's ascent.
            let line_top = top + atlas.line_height * row as f32;

            for span in &line.spans {
                atlas.push_line(quads, [pen, line_top], &span.text, span.color);
                pen += atlas.measure(&span.text);
            }
        }
    }
}
