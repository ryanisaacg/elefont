use image::{ImageBuffer, Rgba};
use font_cache::FontCache;
use rusttype::Font;

fn main() {
    let font_data = include_bytes!("../DejaVuSans.ttf");
    let font = Font::from_bytes(font_data as &[u8]).expect("Error constructing Font");

    let image = ImageBuffer::new(200, 200);

    let mut cache = FontCache::new(Box::new(font), image);
    cache.render_string("Hello, world!", 24.0).for_each(|r| {
        r.unwrap();
    }); 
    cache.render_string("こんにちは世界！", 16.0).for_each(|r| {
        r.unwrap();
    }); 
    cache.render_string("Привет, мир!", 16.0).for_each(|r| {
        r.unwrap();
    }); 
    cache.texture().save("result.png").expect("Failed to save file");
}

