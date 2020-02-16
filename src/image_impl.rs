use super::*;

use image::{ImageBuffer, Rgba};

impl Texture for ImageBuffer<Rgba<u8>, Vec<u8>> {
    fn width(&self) -> u32 {
        ImageBuffer::width(self)
    }

    fn height(&self) -> u32 {
        ImageBuffer::height(self)
    }

    fn put_rect(&mut self, pixel: PixelType, data: &[u8], gpu: &TextureGlyph) {
        use PixelType::*;

        match pixel {
            Alpha => {
                for x in 0..gpu.width {
                    for y in 0..gpu.height {
                        for i in 0..3 {
                            self.get_pixel_mut(gpu.x + x, gpu.y + y).0[i] = 255;
                        }
                        self.get_pixel_mut(gpu.x + x, gpu.y + y).0[3] =
                            data[(x + y * gpu.width) as usize];
                    }
                }
            }
            RGBA => {
                for x in gpu.x..gpu.width {
                    for y in gpu.y..gpu.height {
                        let index = ((x + y * gpu.height) * 4) as usize;
                        for i in 0..4 {
                            self.get_pixel_mut(x, y).0[i] = data[index + i];
                        }
                    }
                }
            }
        }
    }
}
