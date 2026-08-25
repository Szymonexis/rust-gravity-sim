use std::fs;
use std::path::Path;

use fontdue::{Font, FontSettings};

use crate::color::ColorRGBA;
use crate::math::Vec2;
use crate::ui::quad::Quad;

const FIRST: u32 = 0x20;
const LAST: u32 = 0x7e;
const COUNT: usize = (LAST - FIRST + 1) as usize;
const MISSING: char = '?';

const ATLAS_WIDTH: u32 = 512;
const GUTTER: u32 = 1;

const BUNDLED: &[u8] = include_bytes!("../../JetBrains_Mono/JetBrainsMono-VariableFont_wght.ttf");
const BUNDLED_NAME: &str = "JetBrains Mono (bundled)";

#[derive(Clone, Copy, Default)]
struct Glyph {
    uv: [f32; 4],
    size: Vec2,
    offset: Vec2,
    advance: f32,
}

pub struct FontFace {
    font: Font,
    pub source: String,
}

impl FontFace {
    pub fn load(configured: Option<&str>) -> Self {
        if let Some(path) = configured {
            match Self::read(Path::new(path)) {
                Ok(face) => return face,
                Err(err) => eprintln!(
                    "ui: couldn't load the configured font `{path}` ({err}); \
                     falling back to the bundled one"
                ),
            }
        }

        Self::bundled()
    }

    fn bundled() -> Self {
        Self {
            font: Self::parse(BUNDLED).expect("the bundled font is compiled in, and parses"),
            source: BUNDLED_NAME.to_owned(),
        }
    }

    fn read(path: &Path) -> Result<Self, String> {
        let bytes = fs::read(path).map_err(|err| err.to_string())?;

        Ok(Self {
            font: Self::parse(&bytes)?,
            source: path.display().to_string(),
        })
    }

    fn parse(bytes: &[u8]) -> Result<Font, String> {
        Font::from_bytes(bytes, FontSettings::default()).map_err(str::to_owned)
    }

    pub fn rasterize(&self, px: f32) -> Atlas {
        let px = px.max(4.0);

        let mut glyphs = [Glyph::default(); COUNT];
        let mut inked: Vec<(usize, Vec<u8>)> = Vec::with_capacity(COUNT);

        for index in 0..COUNT {
            let character = char::from_u32(FIRST + index as u32).expect("ascii is valid utf-8");
            let (metrics, bitmap) = self.font.rasterize(character, px);

            glyphs[index] = Glyph {
                uv: [0.0; 4],
                size: Vec2::new(metrics.width as f32, metrics.height as f32),
                offset: Vec2::new(
                    metrics.xmin as f32,
                    -((metrics.ymin + metrics.height as i32) as f32),
                ),
                advance: metrics.advance_width,
            };

            if metrics.width > 0 && metrics.height > 0 {
                inked.push((index, bitmap));
            }
        }

        let mut placements = Vec::with_capacity(inked.len());
        let (mut pen_x, mut pen_y, mut shelf_height) = (GUTTER, GUTTER, 0);

        for (index, _) in &inked {
            let width = glyphs[*index].size.x as u32;
            let height = glyphs[*index].size.y as u32;

            if pen_x + width + GUTTER > ATLAS_WIDTH {
                pen_x = GUTTER;
                pen_y += shelf_height + GUTTER;
                shelf_height = 0;
            }

            placements.push([pen_x, pen_y]);
            pen_x += width + GUTTER;
            shelf_height = shelf_height.max(height);
        }

        let height = pen_y + shelf_height + GUTTER;
        let mut pixels = vec![0u8; (ATLAS_WIDTH * height) as usize];

        for ((index, bitmap), [x, y]) in inked.iter().zip(&placements) {
            let glyph = &mut glyphs[*index];
            let width = glyph.size.x as u32;
            let glyph_height = glyph.size.y as u32;

            for row in 0..glyph_height {
                let source = (row * width) as usize;
                let target = ((y + row) * ATLAS_WIDTH + x) as usize;
                pixels[target..target + width as usize]
                    .copy_from_slice(&bitmap[source..source + width as usize]);
            }

            glyph.uv = [
                *x as f32 / ATLAS_WIDTH as f32,
                *y as f32 / height as f32,
                (x + width) as f32 / ATLAS_WIDTH as f32,
                (y + glyph_height) as f32 / height as f32,
            ];
        }

        let (ascent, line_height) = match self.font.horizontal_line_metrics(px) {
            Some(metrics) => (metrics.ascent, metrics.new_line_size),
            None => (px * 0.8, px * 1.2),
        };

        Atlas {
            width: ATLAS_WIDTH,
            height,
            pixels,
            ascent,
            line_height,
            glyphs,
        }
    }
}

pub struct Atlas {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
    pub ascent: f32,
    pub line_height: f32,
    glyphs: [Glyph; COUNT],
}

impl Atlas {
    fn glyph(&self, character: char) -> Glyph {
        let code = character as u32;
        let index = if (FIRST..=LAST).contains(&code) {
            code - FIRST
        } else {
            MISSING as u32 - FIRST
        };

        self.glyphs[index as usize]
    }

    pub fn measure(&self, text: &str) -> f32 {
        text.chars()
            .map(|character| self.glyph(character).advance)
            .sum()
    }

    pub fn push_line(&self, quads: &mut Vec<Quad>, origin: Vec2, text: &str, color: ColorRGBA) {
        let baseline = origin.y + self.ascent;
        let mut pen = origin.x;

        for character in text.chars() {
            let glyph = self.glyph(character);

            if glyph.size.x > 0.0 && glyph.size.y > 0.0 {
                quads.push(Quad::glyph(
                    [
                        pen + glyph.offset.x,
                        baseline + glyph.offset.y,
                        glyph.size.x,
                        glyph.size.y,
                    ],
                    glyph.uv,
                    color,
                ));
            }

            pen += glyph.advance;
        }
    }
}
