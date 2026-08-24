use std::fs;
use std::path::Path;

use fontdue::{Font, FontSettings};

use crate::color::ColorRGBA;
use crate::ui::quad::Quad;

/// The overlay covers printable ASCII and nothing else. It is a debug readout,
/// not a text engine, so one contiguous range keeps lookup to an array index.
const FIRST: u32 = 0x20; // space
const LAST: u32 = 0x7e; // tilde
const COUNT: usize = (LAST - FIRST + 1) as usize;
/// Stand-in for anything outside the range above.
const MISSING: char = '?';

/// One byte per pixel, so 512 wide is already a multiple of wgpu's 256-byte
/// row alignment and the upload never needs padded rows.
const ATLAS_WIDTH: u32 = 512;
/// An empty pixel between neighbours, so linear filtering at a glyph's edge
/// can't reach into the one packed next to it.
const GUTTER: u32 = 1;

/// The font the app falls back on when the config names none, or names one
/// that won't load. Compiled into the executable rather than read from disk, so
/// it is there whatever machine the binary lands on, and whatever directory it
/// is started from.
const BUNDLED: &[u8] = include_bytes!("../../JetBrains_Mono/JetBrainsMono-VariableFont_wght.ttf");
const BUNDLED_NAME: &str = "JetBrains Mono (bundled)";

/// Where one glyph sits in the atlas, and how to place it on a line.
#[derive(Clone, Copy, Default)]
struct Glyph {
    uv: [f32; 4],
    size: [f32; 2],
    /// From the pen position *on the baseline* to the top-left of the bitmap.
    offset: [f32; 2],
    advance: f32,
}

/// A parsed font file, still at no particular size.
pub struct FontFace {
    font: Font,
    /// Where the face came from - a path, or the name of the bundled one.
    pub source: String,
}

impl FontFace {
    /// Never fails. The configured font is a preference; the one it falls back
    /// to is part of the binary, so there is always a face to draw with.
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

    /// Rasterise every covered character at `px` and shelf-pack the results
    /// into one texture, so the whole overlay draws from a single binding.
    pub fn rasterize(&self, px: f32) -> Atlas {
        let px = px.max(4.0);

        let mut glyphs = [Glyph::default(); COUNT];
        // Only glyphs that actually inked something need atlas space; a space
        // is pure advance.
        let mut inked: Vec<(usize, Vec<u8>)> = Vec::with_capacity(COUNT);

        for index in 0..COUNT {
            let character = char::from_u32(FIRST + index as u32).expect("ascii is valid utf-8");
            let (metrics, bitmap) = self.font.rasterize(character, px);

            glyphs[index] = Glyph {
                uv: [0.0; 4],
                size: [metrics.width as f32, metrics.height as f32],
                offset: [
                    metrics.xmin as f32,
                    // `ymin` measures up from the baseline to the bottom of the
                    // bitmap; the overlay places quads by their top edge.
                    -((metrics.ymin + metrics.height as i32) as f32),
                ],
                advance: metrics.advance_width,
            };

            if metrics.width > 0 && metrics.height > 0 {
                inked.push((index, bitmap));
            }
        }

        // Shelf packing: fill a row left to right, drop to a new row when the
        // next glyph won't fit. Cheap, and near-perfect for one font size where
        // every glyph is roughly the same height.
        let mut placements = Vec::with_capacity(inked.len());
        let (mut pen_x, mut pen_y, mut shelf_height) = (GUTTER, GUTTER, 0);

        for (index, _) in &inked {
            let [width, height] = glyphs[*index].size.map(|value| value as u32);

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
            let [width, glyph_height] = glyph.size.map(|value| value as u32);

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

/// Every covered glyph at one size, packed into a single coverage texture.
pub struct Atlas {
    pub width: u32,
    pub height: u32,
    /// One byte of coverage per pixel, row-major.
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

    /// Width the line will occupy, in physical pixels.
    pub fn measure(&self, text: &str) -> f32 {
        text.chars()
            .map(|character| self.glyph(character).advance)
            .sum()
    }

    /// Append one line of text. `origin` is the top-left of the line box, so
    /// callers stack lines by [`Atlas::line_height`] and never think in
    /// baselines.
    pub fn push_line(&self, quads: &mut Vec<Quad>, origin: [f32; 2], text: &str, color: ColorRGBA) {
        let baseline = origin[1] + self.ascent;
        let mut pen = origin[0];

        for character in text.chars() {
            let glyph = self.glyph(character);

            if glyph.size[0] > 0.0 && glyph.size[1] > 0.0 {
                quads.push(Quad::glyph(
                    [
                        pen + glyph.offset[0],
                        baseline + glyph.offset[1],
                        glyph.size[0],
                        glyph.size[1],
                    ],
                    glyph.uv,
                    color,
                ));
            }

            pen += glyph.advance;
        }
    }
}
