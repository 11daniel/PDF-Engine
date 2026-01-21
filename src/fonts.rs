use lazy_static::lazy_static;
use rustybuzz::{Face, GlyphBuffer, UnicodeBuffer};

pub static SANS_SERIF_REGULAR_BYTES: &[u8] =
    include_bytes!("../static/fonts/sans-serif/OpenSans-Regular.ttf");

pub static SANS_SERIF_REGULAR_ITALIC_BYTES: &[u8] =
    include_bytes!("../static/fonts/sans-serif/OpenSans-Italic.ttf");

pub static SANS_SERIF_BOLD_BYTES: &[u8] =
    include_bytes!("../static/fonts/sans-serif/OpenSans-Bold.ttf");

pub static SANS_SERIF_BOLD_ITALIC_BYTES: &[u8] =
    include_bytes!("../static/fonts/sans-serif/OpenSans-BoldItalic.ttf");

pub static SANS_SERIF_LIGHT_BYTES: &[u8] =
    include_bytes!("../static/fonts/sans-serif/OpenSans-Light.ttf");

pub static SANS_SERIF_LIGHT_ITALIC_BYTES: &[u8] =
    include_bytes!("../static/fonts/sans-serif/OpenSans-LightItalic.ttf");

pub static SERIF_REGULAR_BYTES: &[u8] = include_bytes!("../static/fonts/serif/DejaVuSerif.ttf");

pub static SERIF_REGULAR_ITALIC_BYTES: &[u8] =
    include_bytes!("../static/fonts/serif/DejaVuSerif-Italic.ttf");

pub static SERIF_BOLD_BYTES: &[u8] = include_bytes!("../static/fonts/serif/DejaVuSerif-Bold.ttf");

pub static SERIF_BOLD_ITALIC_BYTES: &[u8] =
    include_bytes!("../static/fonts/serif/DejaVuSerif-BoldItalic.ttf");

pub static MONO_REGULAR_BYTES: &[u8] =
    include_bytes!("../static/fonts/mono/JetBrainsMonoNL-Regular.ttf");

pub static MONO_REGULAR_ITALIC_BYTES: &[u8] =
    include_bytes!("../static/fonts/mono/JetBrainsMonoNL-Italic.ttf");

pub static MONO_BOLD_BYTES: &[u8] = include_bytes!("../static/fonts/mono/JetBrainsMonoNL-Bold.ttf");

pub static MONO_BOLD_ITALIC_BYTES: &[u8] =
    include_bytes!("../static/fonts/mono/JetBrainsMonoNL-BoldItalic.ttf");

pub static MONO_LIGHT_BYTES: &[u8] =
    include_bytes!("../static/fonts/mono/JetBrainsMonoNL-Thin.ttf");

pub static MONO_LIGHT_ITALIC_BYTES: &[u8] =
    include_bytes!("../static/fonts/mono/JetBrainsMonoNL-ThinItalic.ttf");

pub static CURSIVE_REGULAR_BYTES: &[u8] =
    include_bytes!("../static/fonts/cursive/Italianno-Regular.ttf");

#[derive(Debug, Clone, Copy)]
pub enum FontType {
    Serif,
    SansSerif,
    Mono,
    Cursive,
}

#[derive(Debug, Clone, Copy)]
pub enum FontWeight {
    Light,
    Regular,
    Bold,
}

lazy_static! {
    // SANS SERIF
    pub static ref SANS_SERIF_REGULAR: Face<'static> =
        Face::from_slice(SANS_SERIF_REGULAR_BYTES, 0).expect("Invalid SANS_SERIF_REGULAR font");
    pub static ref SANS_SERIF_REGULAR_ITALIC: Face<'static> =
        Face::from_slice(SANS_SERIF_REGULAR_ITALIC_BYTES, 0)
            .expect("Invalid SANS_SERIF_REGULAR_ITALIC font");
    pub static ref SANS_SERIF_BOLD: Face<'static> =
        Face::from_slice(SANS_SERIF_BOLD_BYTES, 0).expect("Invalid SANS_SERIF_BOLD font");
    pub static ref SANS_SERIF_BOLD_ITALIC: Face<'static> =
        Face::from_slice(SANS_SERIF_BOLD_ITALIC_BYTES, 0)
            .expect("Invalid SANS_SERIF_BOLD_ITALIC font");
    pub static ref SANS_SERIF_LIGHT: Face<'static> =
        Face::from_slice(SANS_SERIF_LIGHT_BYTES, 0).expect("Invalid SANS_SERIF_LIGHT font");
    pub static ref SANS_SERIF_LIGHT_ITALIC: Face<'static> =
        Face::from_slice(SANS_SERIF_LIGHT_ITALIC_BYTES, 0)
            .expect("Invalid SANS_SERIF_LIGHT_ITALIC font");

    // SERIF
    pub static ref SERIF_REGULAR: Face<'static> =
        Face::from_slice(SERIF_REGULAR_BYTES, 0).expect("Invalid SERIF_REGULAR font");
    pub static ref SERIF_REGULAR_ITALIC: Face<'static> =
        Face::from_slice(SERIF_REGULAR_ITALIC_BYTES, 0).expect("Invalid SERIF_REGULAR_ITALIC font");
    pub static ref SERIF_BOLD: Face<'static> =
        Face::from_slice(SERIF_BOLD_BYTES, 0).expect("Invalid SERIF_BOLD font");
    pub static ref SERIF_BOLD_ITALIC: Face<'static> =
        Face::from_slice(SERIF_BOLD_ITALIC_BYTES, 0).expect("Invalid SERIF_BOLD_ITALIC font");

    // MONOSPACE
    pub static ref MONO_REGULAR: Face<'static> =
        Face::from_slice(MONO_REGULAR_BYTES, 0).expect("Invalid MONO_REGULAR font");
    pub static ref MONO_REGULAR_ITALIC: Face<'static> =
        Face::from_slice(MONO_REGULAR_ITALIC_BYTES, 0)
            .expect("Invalid MONO_REGULAR_ITALIC font");
    pub static ref MONO_BOLD: Face<'static> =
        Face::from_slice(MONO_BOLD_BYTES, 0).expect("Invalid MONO_BOLD font");
    pub static ref MONO_BOLD_ITALIC: Face<'static> =
        Face::from_slice(MONO_BOLD_ITALIC_BYTES, 0)
            .expect("Invalid MONO_BOLD_ITALIC font");
    pub static ref MONO_LIGHT: Face<'static> =
        Face::from_slice(MONO_LIGHT_BYTES, 0).expect("Invalid MONO_LIGHT font");
    pub static ref MONO_LIGHT_ITALIC: Face<'static> =
        Face::from_slice(MONO_LIGHT_ITALIC_BYTES, 0)
            .expect("Invalid MONO_LIGHT_ITALIC font");

    pub static ref CURSIVE_REGULAR: Face<'static> =
        Face::from_slice(CURSIVE_REGULAR_BYTES, 0).expect("Invalid CURSIVE_REGULAR font");
}

pub fn get_font(
    font_type: &FontType,
    font_weight: &FontWeight,
    italics: bool,
) -> &'static Face<'static> {
    match font_type {
        FontType::SansSerif => match font_weight {
            FontWeight::Regular => match italics {
                true => &SANS_SERIF_REGULAR_ITALIC,
                false => &SANS_SERIF_REGULAR,
            },
            FontWeight::Light => match italics {
                true => &SANS_SERIF_LIGHT_ITALIC,
                false => &SANS_SERIF_LIGHT,
            },
            FontWeight::Bold => match italics {
                true => &SANS_SERIF_BOLD_ITALIC,
                false => &SANS_SERIF_BOLD,
            },
        },
        FontType::Serif => match font_weight {
            FontWeight::Regular | FontWeight::Light => match italics {
                true => &SERIF_REGULAR_ITALIC,
                false => &SERIF_REGULAR,
            },
            FontWeight::Bold => match italics {
                true => &SERIF_BOLD_ITALIC,
                false => &SERIF_BOLD,
            },
        },
        FontType::Mono => match font_weight {
            FontWeight::Regular => match italics {
                true => &MONO_REGULAR_ITALIC,
                false => &MONO_REGULAR,
            },
            FontWeight::Light => match italics {
                true => &MONO_LIGHT_ITALIC,
                false => &MONO_LIGHT,
            },
            FontWeight::Bold => match italics {
                true => &MONO_BOLD,
                false => &MONO_BOLD_ITALIC,
            },
        },
        FontType::Cursive => &CURSIVE_REGULAR,
    }
}

pub fn get_font_pdf_name(
    font_type: &FontType,
    font_weight: &FontWeight,
    italics: bool,
) -> &'static str {
    match font_type {
        FontType::SansSerif => match font_weight {
            FontWeight::Regular => match italics {
                true => "pdf-SansSerif",
                false => "pdf-SansSerif",
            },
            FontWeight::Light => match italics {
                true => "pdf-SansSerif",
                false => "pdf-SansSerif",
            },
            FontWeight::Bold => match italics {
                true => "pdf-SansSerif",
                false => "pdf-SansSerif",
            },
        },
        FontType::Serif => match font_weight {
            FontWeight::Regular => match italics {
                true => "pdf-Serif",
                false => "pdf-Serif",
            },
            FontWeight::Light => match italics {
                true => "pdf-Serif",
                false => "pdf-Serif",
            },
            FontWeight::Bold => match italics {
                true => "pdf-Serif",
                false => "pdf-Serif",
            },
        },
        FontType::Mono => match font_weight {
            FontWeight::Regular => match italics {
                true => "pdf-Mono",
                false => "pdf-Mono",
            },
            FontWeight::Light => match italics {
                true => "pdf-Mono",
                false => "pdf-Mono",
            },
            FontWeight::Bold => match italics {
                true => "pdf-Mono",
                false => "pdf-Mono",
            },
        },
        FontType::Cursive => "pdf-Cursive",
    }
}

// Estimate text width in points using ttf-parser
pub fn text_width(face: &Face, text: &str, font_size: f32) -> f32 {
    let upem = face.units_per_em() as f32;
    let scale = font_size / upem;

    // Shape the text (includes kerning, ligatures, etc.)
    let mut buffer = UnicodeBuffer::new();
    buffer.push_str(text);
    let glyph_buffer: GlyphBuffer = rustybuzz::shape(face, &[], buffer);

    // Sum advances in font units, then scale to pixels
    glyph_buffer
        .glyph_positions()
        .iter()
        .map(|pos| pos.x_advance as f32 * scale)
        .sum()
}

// Wrap text by inserting newlines
pub fn wrap_text(face: &Face, text: &str, font_size: f32, max_width: f32) -> (String, usize, f32) {
    let mut wrapped = String::new();
    let mut current = String::new();
    let mut line_count = 0usize;
    let mut high_width = 0f32;

    for word in text.split_whitespace() {
        let text = if current.is_empty() {
            word.to_string()
        } else {
            format!("{current} {word}")
        };

        let width = text_width(face, &text, font_size);

        if width <= max_width {
            current = text;
        } else {
            line_count += 1;
            high_width = if width > high_width {
                width
            } else {
                high_width
            };

            if !wrapped.is_empty() {
                wrapped.push('\n');
            }
            wrapped += &current;
            current = word.to_string();
        }
    }
    if !current.is_empty() {
        line_count += 1;
        let width = text_width(face, text, font_size);
        high_width = if width > high_width {
            width
        } else {
            high_width
        };
        if !wrapped.is_empty() {
            wrapped.push('\n');
        }
        wrapped += &current;
    }
    (wrapped, line_count, high_width)
}

/* 
pub fn fit_text_font_size(
    face: &Face,
    text: &str,
    width: f32,
    height: f32,
) -> Option<(String, f32)> {
    let mut size = height;
    let mut wrapped_text = String::new();
    let mut line_count;
    let mut calc_width = width + 1f32;
    let mut calc_height = height + 1f32;

    while size > 1f32 && (calc_width > width || calc_height > height) {
        // wrap_text likely needs height as usize
        (wrapped_text, line_count, calc_width) = wrap_text(face, text, size, width);
        println!("LINES::{}", text.clone().split("\n").count());
        calc_height = size * line_count as f32;
        println!("{calc_width} {width} {calc_height} {height} {size}");

        if calc_width > width || calc_height > height {
            size -= 1f32;
        }
    }

    println!("CCCC::{calc_height}");

    if (calc_width > width || calc_height > height) || size < 1f32 {
        None
    } else {
        Some((wrapped_text, size))
    }
}*/

pub fn fit_text_font_size(
    face: &Face,
    text: &str,
    width: f32,  // box_width
    height: f32, // box_height
) -> Option<(String, f32)> {
    // --- 1. Define Your Search Range ---
    let mut min_font = 1.0;
    let mut max_font = height; // Max guess is the box height

    let mut best_font_size = 0.0;
    let mut best_wrapped_text = String::new();

    // This multiplier must match the one used in `draw_text_fit` / `draw_text_wrap`
    // In `text.rs` you are using 1.2.
    let line_height_multiplier = 1.2;

    // --- 2. Start the Binary Search Loop ---
    // We loop as long as the range is larger than a small precision (e.g., 0.1pt)
    while (max_font - min_font) > 0.1 {
        // Calculate the middle font size to test
        let test_font = (min_font + max_font) / 2.0;

        // --- 3. Run "Numerical Verification" (your `get_text_dimensions`) ---
        // This is our "magic function" that measures the text.
        let (wrapped_text, line_count, calc_width) =
            wrap_text(face, text, test_font, width);

        // We also need to calculate the total height
        let calc_height = (line_count as f32) * (test_font * line_height_multiplier);

        // Check if it fits
        if calc_width <= width && calc_height <= height {
            // IT FITS!
            // This is a valid size. Store it as our new "best".
            best_font_size = test_font;
            best_wrapped_text = wrapped_text;

            // Now, let's try to find an EVEN BIGGER size.
            // We throw away all sizes smaller than this one.
            min_font = test_font; // For floats, we set min to the last valid test
        } else {
            // IT'S TOO BIG!
            // We throw away this size and all sizes larger than it.
            max_font = test_font; // For floats, we set max to the last invalid test
        }
    }

    // --- 4. Return the Result ---
    if best_font_size > 0.0 {
        Some((best_wrapped_text, best_font_size))
    } else {
        // We never found a size that fits (e.g., box was too small)
        None
    }
}