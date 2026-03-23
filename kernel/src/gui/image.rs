use alloc::vec::Vec;

use crate::gui::structure::{AABB, Color, GuiWindow};

#[derive(Debug)]
pub struct QoiImage {
    header: qoi::Header,
    pixels: Vec<u8>,
}

impl QoiImage {
    pub fn new(bytes: &[u8]) -> Result<Self, qoi::Error> {
        let (header, pixels) = qoi::decode_to_vec(bytes)?;
        Ok(Self { header, pixels })
    }

    pub fn header(&self) -> &qoi::Header {
        &self.header
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    // Draw an image to a GUI window at the specified AABB.
    // If the image is smaller than the AABB, it will only draw the part that fits.
    // If the image is larger than the AABB, it will be clipped.
    pub fn draw_to_window(&self, window: &mut GuiWindow, aabb: AABB) {
        match self.header.channels {
            qoi::Channels::Rgb => {
                for y in 0..aabb.size.1 {
                    for x in 0..aabb.size.0 {
                        let img_x = x as u32;
                        let img_y = y as u32;
                        if img_x >= self.header.width || img_y >= self.header.height {
                            continue;
                        }

                        let pixel_index = (img_y * self.header.width + img_x) as usize;
                        let byte_index = pixel_index * 3;

                        let color = Color {
                            r: self.pixels[byte_index],
                            g: self.pixels[byte_index + 1],
                            b: self.pixels[byte_index + 2],
                            a: 255,
                        };

                        let window_pos = (aabb.pos.0 + x, aabb.pos.1 + y);
                        window.set_pixel(window_pos, color);
                    }
                }
            }
            qoi::Channels::Rgba => {
                for y in 0..aabb.size.1 {
                    for x in 0..aabb.size.0 {
                        let img_x = x as u32;
                        let img_y = y as u32;
                        if img_x >= self.header.width || img_y >= self.header.height {
                            continue;
                        }

                        let pixel_index = (img_y * self.header.width + img_x) as usize;
                        let byte_index = pixel_index * 4;

                        let color = Color {
                            r: self.pixels[byte_index],
                            g: self.pixels[byte_index + 1],
                            b: self.pixels[byte_index + 2],
                            a: self.pixels[byte_index + 3],
                        };

                        let window_pos = (aabb.pos.0 + x, aabb.pos.1 + y);
                        window.set_pixel(window_pos, color);
                    }
                }
            }
        }
    }
}
