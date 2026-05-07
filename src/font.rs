use rusttype::{Font, Scale, GlyphId, point};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct GlyphBitmap {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
    pub x_offset: i32,
    pub y_offset: i32,
    pub advance_width: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct FontMetrics {
    pub ascent: f32,
    pub descent: f32,
    pub line_gap: f32,
    pub line_height: f32,
}

pub struct FontCache<'a> {
    pub font: Font<'a>,
    glyphs: HashMap<(char, u32), Arc<GlyphBitmap>>,
}

impl<'a> FontCache<'a> {
    // font file from memory
    pub fn new(font_data: &'a [u8]) -> Option<Self> {
        let font = Font::try_from_bytes(font_data)?;
        Some(FontCache {
            font,
            glyphs: HashMap::new(),
        })
    }

    // Parses font metrics
    pub fn metrics(&self, size: f32) -> FontMetrics {
        let scale = Scale::uniform(size);
        let v_metrics = self.font.v_metrics(scale);
        FontMetrics {
            ascent: v_metrics.ascent,
            descent: v_metrics.descent,
            line_gap: v_metrics.line_gap,
            line_height: v_metrics.ascent - v_metrics.descent + v_metrics.line_gap,
        }
    }

    // Maps a character to its glyph ID using the font's cmap table
    pub fn glyph_id_for_char(&self, c: char) -> Option<u16> {
        let id = self.font.glyph(c).id().0;
        if id == 0 { None } else { Some(id) }
    }

    // Applies kerning between two characters solving the glyphID's
    pub fn kerning(&self, c1: char, c2: char, size: f32) -> f32 {
        self.font.pair_kerning(Scale::uniform(size), c1, c2)
    }

    // Lays out a string of characters: maps each char to a glyph ID,
    // applies kerning between consecutive pairs, and returns a list of
    // (glyph_id, x_position, y_position) along with the total advance width.
    pub fn layout_text(&self, text: &str, size: f32) -> (Vec<(GlyphId, f32, f32)>, f32) {
        let scale = Scale::uniform(size);
        let v_metrics = self.font.v_metrics(scale);
        let mut positions = Vec::new();
        let mut x = 0.0f32;
        let y = v_metrics.ascent;

        let glyph_ids: Vec<GlyphId> = text
            .chars()
            .map(|c| self.font.glyph(c).id())
            .filter(|gid| gid.0 != 0)
            .collect();

        for (i, &gid) in glyph_ids.iter().enumerate() {
            if i > 0 {
                let prev = glyph_ids[i - 1];
                x += self.font.pair_kerning(scale, prev, gid);
            }
            positions.push((gid, x, y));

            let glyph = self.font.glyph(gid).scaled(scale);
            x += glyph.h_metrics().advance_width;
        }

        (positions, x)
    }

    // Retrieves cached glyph bitmaps
    pub fn get_glyph(&mut self, c: char, size: f32) -> Arc<GlyphBitmap> {
        let size_key = size.round() as u32;
        if let Some(bitmap) = self.glyphs.get(&(c, size_key)) {
            return bitmap.clone();
        }

        let scale = Scale::uniform(size);
        let glyph = self.font.glyph(c).scaled(scale).positioned(point(0.0, 0.0));
        let h_metrics = glyph.unpositioned().h_metrics();

        let mut pixels = Vec::new();
        let mut w = 0;
        let mut h = 0;
        let mut x_off = 0;
        let mut y_off = 0;

        if let Some(bb) = glyph.pixel_bounding_box() {
            w = bb.width() as u32;
            h = bb.height() as u32;
            x_off = bb.min.x;
            y_off = bb.min.y;
            pixels.resize((w * h) as usize, 0);

            glyph.draw(|x, y, v| {
                let v = (v * 255.0).clamp(0.0, 255.0) as u8;
                pixels[(y * w + x) as usize] = v;
            });
        }

        let bitmap = Arc::new(GlyphBitmap {
            width: w,
            height: h,
            pixels,
            x_offset: x_off,
            y_offset: y_off,
            advance_width: h_metrics.advance_width,
        });

        self.glyphs.insert((c, size_key), bitmap.clone());
        bitmap
    }
}
