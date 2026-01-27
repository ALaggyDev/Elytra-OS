use alloc::vec::Vec;
use bootloader_api::info::{FrameBuffer, FrameBufferInfo, PixelFormat};

#[derive(Debug)]
pub struct FrameBufferWriter {
    buffer: &'static mut [u8],
    info: FrameBufferInfo,
}

impl FrameBufferWriter {
    pub fn new(framebuffer: FrameBuffer) -> Self {
        let info = framebuffer.info();
        Self {
            buffer: framebuffer.into_buffer(),
            info,
        }
    }

    pub fn create_new_window(&self) -> GuiWindow {
        GuiWindow::new((self.info.width as i32, self.info.height as i32))
    }

    pub fn write_to_framebuffer(&mut self, window: &GuiWindow) {
        assert!(self.info.width == window.size.0 as usize);
        assert!(self.info.height == window.size.1 as usize);

        for y in 0..self.info.height {
            for x in 0..self.info.width {
                let pixel = window.get_pixel((x as i32, y as i32)).unwrap();

                let color = match self.info.pixel_format {
                    PixelFormat::Rgb => [pixel.r, pixel.g, pixel.b, 0],
                    PixelFormat::Bgr => [pixel.b, pixel.g, pixel.r, 0],
                    other => {
                        panic!("pixel format {:?} not supported.", other)
                    }
                };

                let fb_pixel_index = y * self.info.stride + x;
                let fb_byte_index = fb_pixel_index * self.info.bytes_per_pixel;

                self.buffer[fb_byte_index..fb_byte_index + self.info.bytes_per_pixel]
                    .copy_from_slice(&color[..self.info.bytes_per_pixel]);
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const BLACK: Self = Self { r: 0, g: 0, b: 0 };
    pub const WHITE: Self = Self {
        r: 255,
        g: 255,
        b: 255,
    };
    pub const RED: Self = Self { r: 255, g: 0, b: 0 };
    pub const GREEN: Self = Self { r: 0, g: 255, b: 0 };
    pub const BLUE: Self = Self { r: 0, g: 0, b: 255 };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AABB {
    pub pos: (i32, i32),
    pub size: (i32, i32),
}

#[derive(Debug, Clone)]
pub struct GuiWindow {
    pub buffer: Vec<Color>,

    pub size: (i32, i32),
}

impl GuiWindow {
    pub fn new(size: (i32, i32)) -> Self {
        Self {
            buffer: vec![Color::BLACK; (size.0 * size.1) as usize],
            size,
        }
    }

    pub fn aabb(&self) -> AABB {
        AABB {
            pos: (0, 0),
            size: self.size,
        }
    }

    pub fn pixel_pos(&self, pos: (i32, i32)) -> Option<usize> {
        if pos.0 < 0 || pos.1 < 0 || pos.0 >= self.size.0 || pos.1 >= self.size.1 {
            return None;
        }
        Some(pos.1 as usize * self.size.0 as usize + pos.0 as usize)
    }

    pub fn get_pixel(&self, pos: (i32, i32)) -> Option<Color> {
        if let Some(index) = self.pixel_pos(pos) {
            Some(self.buffer[index])
        } else {
            None
        }
    }

    pub fn set_pixel(&mut self, pos: (i32, i32), color: Color) {
        if let Some(index) = self.pixel_pos(pos) {
            self.buffer[index] = color;
        }
    }

    pub fn set_pixels(&mut self, aabb: AABB, color: Color) {
        for y in 0..aabb.size.1 {
            for x in 0..aabb.size.0 {
                let pos = (aabb.pos.0 + x, aabb.pos.1 + y);
                self.set_pixel(pos, color);
            }
        }
    }

    pub fn fill_all(&mut self, color: Color) {
        for pixel in self.buffer.iter_mut() {
            *pixel = color;
        }
    }
}

#[macro_export]
macro_rules! color {
    (#$s:tt) => {
        const {
            let strings = stringify!($s);
            assert!(strings.len() == 6, "Color string must be 6 hex digits.");
            let Ok(num) = u32::from_str_radix(strings, 16) else {
                panic!("Invalid hex string");
            };
            Color {
                r: (num >> 16) as u8,
                g: (num >> 8) as u8,
                b: num as u8,
            }
        }
    };
}
