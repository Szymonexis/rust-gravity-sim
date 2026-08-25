use crate::color::ColorRGBA;
use crate::math::Vec2;
use crate::ui::font::Atlas;
use crate::ui::quad::Quad;

const MARGIN: f32 = 12.0;
const PADDING: f32 = 8.0;
const BACKGROUND: ColorRGBA = [0.02, 0.02, 0.04, 0.62];

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Anchor {
    TopLeft,
    TopRight,
}

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

    pub fn blank(&mut self) {
        self.lines.push(Line::default());
    }

    pub fn layout(
        &self,
        quads: &mut Vec<Quad>,
        atlas: &Atlas,
        anchor: Anchor,
        resolution: Vec2,
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
            Anchor::TopRight => resolution.x - margin - padding - width,
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
            let mut pen = match anchor {
                Anchor::TopLeft => left,
                Anchor::TopRight => left + width - line.width(atlas),
            };
            let line_top = top + atlas.line_height * row as f32;

            for span in &line.spans {
                atlas.push_line(quads, Vec2::new(pen, line_top), &span.text, span.color);
                pen += atlas.measure(&span.text);
            }
        }
    }
}
