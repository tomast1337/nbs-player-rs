//! Glyph rasterization (fontdue) into a shelf-packed RGBA atlas.

use fontdue::Font;
use std::collections::HashMap;

pub const ATLAS_SIZE: u32 = 1024;

#[derive(Clone, Copy)]
pub struct Glyph {
    /// Atlas rectangle in texels.
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    /// Offset of the bitmap's left edge from the pen and of its bottom edge from the
    /// baseline (up is positive), in raster pixels.
    pub xmin: i32,
    pub ymin: i32,
    pub advance: f32,
}

/// A bitmap that still has to be uploaded to the GPU atlas.
pub struct Upload {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    /// RGBA8, white with coverage in alpha.
    pub rgba: Vec<u8>,
}

pub struct GlyphAtlas {
    font: Font,
    glyphs: HashMap<(char, u16), Glyph>,
    cursor_x: u32,
    cursor_y: u32,
    row_h: u32,
}

impl GlyphAtlas {
    pub fn new(ttf: &[u8]) -> Result<Self, String> {
        let font = Font::from_bytes(ttf, fontdue::FontSettings::default())
            .map_err(|e| format!("font: {e}"))?;
        Ok(Self {
            font,
            glyphs: HashMap::new(),
            cursor_x: 1,
            cursor_y: 1,
            row_h: 0,
        })
    }

    /// Distance from the top of the line to the baseline at `px`.
    pub fn ascent(&self, px: u16) -> f32 {
        self.font
            .horizontal_line_metrics(px as f32)
            .map(|m| m.ascent)
            .unwrap_or(px as f32)
    }

    pub fn advance(&self, c: char, px: u16) -> f32 {
        self.font.metrics(c, px as f32).advance_width
    }

    /// Look up (rasterizing and packing on first use) a glyph. `uploads` collects bitmaps
    /// that must be written to the atlas texture before drawing.
    pub fn glyph(&mut self, c: char, px: u16, uploads: &mut Vec<Upload>) -> Option<Glyph> {
        if let Some(g) = self.glyphs.get(&(c, px)) {
            return Some(*g);
        }
        let (m, coverage) = self.font.rasterize(c, px as f32);
        let (w, h) = (m.width as u32, m.height as u32);

        let (x, y) = if w == 0 || h == 0 {
            (0, 0)
        } else {
            if self.cursor_x + w + 1 > ATLAS_SIZE {
                self.cursor_x = 1;
                self.cursor_y += self.row_h + 1;
                self.row_h = 0;
            }
            if self.cursor_y + h + 1 > ATLAS_SIZE {
                log::warn!("glyph atlas full, dropping {c:?}");
                return None;
            }
            let pos = (self.cursor_x, self.cursor_y);
            self.cursor_x += w + 1;
            self.row_h = self.row_h.max(h);

            let mut rgba = Vec::with_capacity(coverage.len() * 4);
            for a in coverage {
                rgba.extend_from_slice(&[255, 255, 255, a]);
            }
            uploads.push(Upload {
                x: pos.0,
                y: pos.1,
                w,
                h,
                rgba,
            });
            pos
        };

        let glyph = Glyph {
            x,
            y,
            w,
            h,
            xmin: m.xmin,
            ymin: m.ymin,
            advance: m.advance_width,
        };
        self.glyphs.insert((c, px), glyph);
        Some(glyph)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nbs_player_core::config::FontID;

    #[test]
    fn glyphs_are_cached_and_do_not_overlap() {
        let mut atlas = GlyphAtlas::new(FontID::PixelPlay.ttf_bytes()).unwrap();
        let mut uploads = Vec::new();
        let a = atlas.glyph('A', 16, &mut uploads).unwrap();
        let b = atlas.glyph('B', 16, &mut uploads).unwrap();
        assert_eq!(uploads.len(), 2);

        // second lookup is a cache hit: no new upload, same slot
        let again = atlas.glyph('A', 16, &mut uploads).unwrap();
        assert_eq!(uploads.len(), 2);
        assert_eq!((again.x, again.y), (a.x, a.y));

        let overlap = a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h;
        assert!(!overlap);
        assert!(a.w > 0 && a.h > 0 && a.advance > 0.0);
    }

    #[test]
    fn whitespace_has_advance_but_no_bitmap() {
        let mut atlas = GlyphAtlas::new(FontID::PixelPlay.ttf_bytes()).unwrap();
        let mut uploads = Vec::new();
        let space = atlas.glyph(' ', 16, &mut uploads).unwrap();
        assert!(uploads.is_empty());
        assert_eq!((space.w, space.h), (0, 0));
        assert!(space.advance > 0.0);
    }
}
