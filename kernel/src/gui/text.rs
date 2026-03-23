use noto_sans_mono_bitmap::{FontWeight, RasterHeight, RasterizedChar, get_raster};

use noto_sans_mono_bitmap::get_raster_width;

use crate::gui::structure::{Color, GuiWindow};

/// Additional vertical space between lines
const LINE_SPACING: usize = 2;

/// Additional horizontal space between characters.
const LETTER_SPACING: usize = 0;

/// Padding from the border. Prevent that font is too close to border.
const BORDER_PADDING: usize = 1;

/// The weight of the font. The regular weight is a good choice for most use cases.
pub const FONT_WEIGHT: FontWeight = FontWeight::Regular;

/// Height of each char raster. The font size is ~0.84% of this. Thus, this is the line height that
/// enables multiple characters to be side-by-side and appear optically in one line in a natural way.
pub const CHAR_RASTER_HEIGHT: RasterHeight = RasterHeight::Size20;

/// The width of each single symbol of the mono space font.
pub const CHAR_RASTER_WIDTH: usize = get_raster_width(FONT_WEIGHT, CHAR_RASTER_HEIGHT);

/// Backup character if a desired symbol is not available by the font.
/// The '�' character requires the feature "unicode-specials".
pub const BACKUP_CHAR: char = '�';

/// Returns the raster of the given char or the raster of [`font_constants::BACKUP_CHAR`].
fn get_char_raster(c: char) -> RasterizedChar {
    fn get(c: char) -> Option<RasterizedChar> {
        get_raster(c, FONT_WEIGHT, CHAR_RASTER_HEIGHT)
    }
    get(c).unwrap_or_else(|| get(BACKUP_CHAR).expect("Should get raster of backup char."))
}

/// Draws a text to the given window at the given position. The text is drawn with the given text color and background color.
pub fn draw_to_window(
    window: &mut GuiWindow,
    text: &str,
    pos: (usize, usize),
    background_color: Color,
    text_color: Color,
) {
    let mut cur_pos = (pos.0 + BORDER_PADDING, pos.1 + BORDER_PADDING);

    for c in text.chars() {
        // Handle newline
        if c == '\n' {
            cur_pos.0 = pos.0 + BORDER_PADDING;
            cur_pos.1 += CHAR_RASTER_HEIGHT as usize + LINE_SPACING;
            continue;
        }

        let raster = get_char_raster(c).raster();

        for (raster_y, row) in raster.iter().enumerate() {
            for (raster_x, &pixel) in row.iter().enumerate() {
                let new_color = Color {
                    r: text_color.r,
                    g: text_color.g,
                    b: text_color.b,
                    a: pixel,
                };

                let new_pos = ((cur_pos.0 + raster_x) as i32, (cur_pos.1 + raster_y) as i32);

                window.overlay_opaque_pixel(new_pos, background_color);
                window.overlay_opaque_pixel(new_pos, new_color);
            }
        }

        cur_pos.0 += CHAR_RASTER_WIDTH + LETTER_SPACING;
    }
}
