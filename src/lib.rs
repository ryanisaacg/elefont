//! A library that handles caching rendered glyphs on the GPU
//!
//! This fits as a layer in your rendering pipeline between font rasterization and shaping and text
//! rendering. In other words, first you turn a string into a series of font glyphs. Each of those
//! glyphs is looked up against the cache, and if it hasn't been rendered, it is turned into a
//! bitmap and uploaded to the GPU. The string is laid out
//!
//! Scope of this library:
//! - DO support various font libraries / types of fonts (rusttype / fontdue for TTFs, bitmap fonts
//! of various types)
//! - DO support various types of graphics backends (GL, wgpu, vulkan, various higher-level
//! frameworks)
//! - DON'T handle complex tasks like shaping. The font stack should handle that elsewhere, and
//! provide this library the glyphs to render
//! - DON'T handle layout or rendering to the screen. This can be taken care of 

#[cfg(feature = "rusttype")]
mod rusttype_impl;

// TODO: going to want a better hashmap
use std::collections::HashMap;

pub trait FontProvider {
    fn supports_vertical(&self) -> bool;
    fn pixel_type(&self) -> PixelType;
    fn glyphs(&self, string: &str, glyphs: &mut Vec<Glyph>);
    fn line_width(&self, size: f32) -> f32;
    fn line_height(&self, size: f32) -> f32;
    fn metrics(&self, key: GlyphKey) -> Metrics;
    fn rasterize(&self, key: GlyphKey) -> Vec<u8>;
}

pub trait Texture {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn put_rect(&mut self, pixel: PixelType, data: &[u8], gpu: &GpuGlyph);
}

/// The index of the font character to render
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Glyph(pub u32);

pub struct FontCache<T: Texture> {
    glyph_buffer: Vec<Glyph>,
    cache: Cache<T>,
}

struct Cache<T: Texture> {
    font: Box<dyn FontProvider>,
    texture: T,
    map: HashMap<GlyphKey, GpuGlyph>,
    h_cursor: u32,
    v_cursor: u32,
    current_line_height: u32,
}

// TODO: probably add a better packing algorithm

impl<T: Texture> FontCache<T> {
    pub fn new(font: Box<dyn FontProvider>, texture: T) -> Self {
        FontCache {
            glyph_buffer: Vec::new(),
            cache: Cache {
                font,
                texture,
                map: HashMap::new(),
                h_cursor: 0,
                v_cursor: 0,
                current_line_height: 0,
            }
        }
    }

    pub fn clear(&mut self) {
        self.cache.clear();
    }

    pub fn render_glyph(&mut self, key: GlyphKey) -> Result<GpuGlyph, CacheError> {
        self.cache.render_glyph(key)
    }

    pub fn render_string<'a>(&'a mut self, string: &str, size: f32) -> impl 'a + Iterator<Item = Result<GpuGlyph, CacheError>> {
        #[cfg(feature = "unicode-normalization")]
        let string = {
            use unicode_normalization::UnicodeNormalization;
            &string.nfc().collect::<String>()
        };
        let size = size.to_bits();
        let glyph_buffer = &mut self.glyph_buffer;
        let cache = &mut self.cache;
        cache.font.glyphs(&string, glyph_buffer);
        glyph_buffer.drain(..).map(move |glyph| cache.render_glyph(GlyphKey {
            glyph,
            size
        }))
    }

    pub fn texture(&self) -> &T {
        &self.cache.texture
    }
}

impl<T: Texture> Cache<T> {
    pub fn clear(&mut self) {
        self.map.clear();
        self.h_cursor = 0;
        self.v_cursor = 0;
        self.current_line_height = 0;
    }

    pub fn render_glyph(&mut self, key: GlyphKey) -> Result<GpuGlyph, CacheError> {
        let metrics = self.font.metrics(key);
        if metrics.width > self.texture.width() || metrics.height > self.texture.height() {
            return Err(CacheError::TextureTooSmall);
        }
        if metrics.width + self.h_cursor > self.texture.width() {
            self.h_cursor = 0;
            self.v_cursor += self.current_line_height;
        }
        if metrics.height + self.v_cursor > self.texture.height() {
            return Err(CacheError::OutOfSpace);
        }
        let pixel_type = self.font.pixel_type();
        let data = self.font.rasterize(key);
        let gpu = GpuGlyph {
            x: self.h_cursor,
            y: self.v_cursor,
            width: metrics.width,
            height: metrics.height,
        };
        self.texture.put_rect(pixel_type, &data[..], &gpu);
        Ok(gpu)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct GlyphKey {
    pub glyph: Glyph,
    size: u32,
}

pub struct GpuGlyph {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[non_exhaustive]
pub struct Metrics {
    pub width: u32,
    pub height: u32,
    pub bearing_x: f32,
    pub advance_x: f32,
    pub bearing_y: f32,
    pub advance_y: f32,
}

pub enum CacheError {
    TextureTooSmall,
    OutOfSpace,
}

pub enum PixelType {
    Alpha, RGB, RGBA
}

