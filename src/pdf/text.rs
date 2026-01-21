use lopdf::{
    Document, Object,
    content::{Content, Operation},
};

use rustybuzz::{GlyphBuffer, UnicodeBuffer};

use crate::{
    error::{BoxedError, GenericError},
    fonts::{
        FontType, FontWeight, fit_text_font_size, get_font, get_font_pdf_name, text_width,
        wrap_text,
    },
    pdf::{TextAlignment, VerticalAlign},
};

use super::{color::Color, get_page_media_box};

// Hard wrap a string by character count, preferably breaking on whitespace.
pub fn hard_wrap_by_chars(text: &str, max_chars: usize) -> String {
    let mut out = String::with_capacity(text.len() + text.len() / max_chars.max(1));
    let mut line_char_count = 0;

    for ch in text.chars() {
        if ch == '\n' {
            line_char_count = 0;
            out.push('\n');
            continue;
        }

        // if about to exceed limit, break line even without space
        if line_char_count >= max_chars {
            out.push('\n');
            line_char_count = 0;
        }

        out.push(ch);
        line_char_count += 1;
    }

    out
}

#[derive(Debug)]
pub struct DrawTextOptions<'a> {
    pub text: &'a str,
    pub font_size: f32,
    pub font_weight: Option<FontWeight>,
    pub font_type: Option<FontType>,
    pub text_alignment: Option<TextAlignment>,
    pub align_v: Option<VerticalAlign>,
    pub color: Option<Color>,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}
pub fn draw_text<'a>(
    document: &'a mut Document,
    page: &(u32, u16),
    options: DrawTextOptions<'a>,
) -> Result<(), BoxedError> {
    let (_, _, _page_w, page_h) = get_page_media_box(document, page)?;

    let color = options.color.unwrap_or(Color::from_rgb(0, 0, 0));

    let font_weight = options.font_weight.unwrap_or(FontWeight::Regular);
    let font_type = options.font_type.unwrap_or(FontType::Serif);

    let font_face = get_font(&font_type, &font_weight, false);
    let font_name = get_font_pdf_name(&font_type, &font_weight, false);

    let upem = font_face.units_per_em() as f32;
    let scale = options.font_size / upem;

    let mut buffer = UnicodeBuffer::new();
    buffer.push_str(options.text);
    let glyph_buffer: GlyphBuffer = rustybuzz::shape(font_face, &[], buffer);

    let mut content = Content { operations: vec![] };

    content.operations.push(Operation::new(
        "rg",
        vec![color.r().into(), color.g().into(), color.b().into()],
    )); // Color
    content.operations.push(Operation::new("BT", vec![])); // Begin text
    content.operations.push(Operation::new(
        "Tf",
        vec![
            Object::Name(font_name.as_bytes().to_vec()),
            options.font_size.into(),
        ],
    )); // Font size and face

    let baseline_y = page_h - options.y - options.font_size * 0.2;

    content.operations.push(Operation::new(
        "Tm",
        vec![
            1.into(),
            0.into(),
            0.into(),
            1.into(),
            options.x.into(),
            baseline_y.into(),
        ],
    )); // Move text position

    // Render each glyph with correct advance

    if let FontType::Cursive = font_type {
        let mut chars = options.text.chars();
        for pos in glyph_buffer.glyph_positions() {
            let text = format!("{}", chars.next().unwrap_or(' ')); // placeholder for actual mapping
            content
                .operations
                .push(Operation::new("Tj", vec![Object::string_literal(text)]));
            if pos.x_advance != 0 {
                let adv = pos.x_advance as f32 * scale - options.font_size * scale * 0.75;

                content
                    .operations
                    .push(Operation::new("Td", vec![adv.into(), 0.into()]));
            }
        }
    } else {
        content.operations.push(Operation::new(
            "Tj",
            vec![Object::string_literal(options.text)],
        ));
    }

    content.operations.push(
        Operation::new("ET", vec![]), // End text
    );

    let encoded_content = content.encode()?;

    document.add_page_contents(page.to_owned(), encoded_content)?;

    Ok(())
}

pub fn draw_text_wrap<'a>(
    document: &'a mut Document,
    page: &(u32, u16),
    options: DrawTextOptions<'a>,
) -> Result<(), BoxedError> {
    let font_weight = options.font_weight.unwrap_or(FontWeight::Regular);
    let font_type = options.font_type.unwrap_or(FontType::Serif);
    let font_face = get_font(&font_type, &font_weight, false);

    let estimated_chars_per_line =
        ((options.w / options.font_size.max(1.0)) * 1.2).clamp(5.0, 120.0) as usize;
    let pre_wrapped = hard_wrap_by_chars(options.text, estimated_chars_per_line);

    let (text, line_count, _) = wrap_text(
        font_face, // Use the font_face
        &pre_wrapped,
        options.font_size,
        options.w,
    );

    let line_height = options.font_size * 1.2;

    let total_text_height = (line_count as f32) * line_height;
    let y_offset = match options.align_v.unwrap_or(VerticalAlign::Top) {
        VerticalAlign::Top => 0.0,
        VerticalAlign::Middle => (options.h - total_text_height) / 2.0,
        VerticalAlign::Bottom => options.h - total_text_height,
    };

    for (index, line) in text.split('\n').enumerate() {
        let line_y = options.y + y_offset + (index as f32) * line_height;
        // --- End Fix ---

        let width = text_width(font_face, line, options.font_size);
        let line_x = match options.text_alignment {
            Some(TextAlignment::Center) => options.x + (options.w - width) / 2.0,
            Some(TextAlignment::Right) => options.x + (options.w - width),
            _ => options.x,
        };

        draw_text(
            document,
            page,
            DrawTextOptions {
                text: line,
                font_size: options.font_size,
                font_weight: options.font_weight,
                font_type: options.font_type,
                text_alignment: None,
                align_v: None,
                color: options.color,
                x: line_x,
                y: line_y,
                w: options.w,
                h: line_height, // Use line_height, not original box height
            },
        )?;
    }
    Ok(())
}

pub fn draw_text_fit<'a>(
    document: &'a mut Document,
    page: &(u32, u16),
    options: DrawTextOptions<'a>,
) -> Result<(), BoxedError> {
    let font_weight = options.font_weight.unwrap_or(FontWeight::Regular);
    let font_type = options.font_type.unwrap_or(FontType::Serif);
    let font_face = get_font(&font_type, &font_weight, false);

    let pre_wrapped_text = hard_wrap_by_chars(options.text, 25);

    // Find the best font size that fits
    let (text, font_size) =
        fit_text_font_size(font_face, &pre_wrapped_text, options.w, options.h).ok_or(
            GenericError(format!("Unable to fit text: {}", options.text)),
        )?;

    let line_height = font_size * 1.2;
    let lines: Vec<&str> = text.split('\n').collect(); // Get lines
    let line_count = lines.len(); // Get line_count

    let total_text_height = (line_count as f32) * line_height;

    // Correction to align baseline properly with draw_text
    let correction_bottom = font_size * 1.0;
    let correction_middle = font_size * 0.6;
    let correction_top = font_size * 0.25;

    let y_offset = match options.align_v.unwrap_or(VerticalAlign::Top) {
        VerticalAlign::Top => correction_top,
        VerticalAlign::Middle => (options.h - total_text_height) / 2.0 + correction_middle,
        VerticalAlign::Bottom => options.h - total_text_height + correction_bottom,
    };

    for (index, line) in lines.into_iter().enumerate() {
        let line_y = options.y + y_offset + (index as f32) * line_height;

        let width = text_width(font_face, line, font_size);
        let aligned_x = match options.text_alignment.clone() {
            Some(TextAlignment::Center) => options.x + (options.w - width) / 2f32,
            Some(TextAlignment::Right) => options.x + (options.w - width),
            _ => options.x,
        };

        draw_text(
            document,
            page,
            DrawTextOptions {
                text: line,
                font_size,
                font_weight: options.font_weight,
                font_type: options.font_type,
                text_alignment: None,
                align_v: None,
                color: options.color,
                x: aligned_x,
                y: line_y,
                w: options.w,
                h: line_height,
            },
        )?;
    }
    Ok(())
}

pub fn draw_text_wrap_fit<'a>(
    document: &'a mut Document,
    page: &(u32, u16),
    opts: DrawTextOptions<'a>,
) -> Result<(), BoxedError> {
    let DrawTextOptions {
        text,
        mut font_size,
        font_weight,
        font_type,
        text_alignment,
        align_v,
        color,
        x,
        y,
        w,
        h,
    } = opts;

    let font_face = get_font(
        &font_type.clone().unwrap_or(FontType::Serif),
        &font_weight.clone().unwrap_or(FontWeight::Regular),
        false,
    );

    let words: Vec<&str> = text.split_whitespace().collect();
    let mut lines: Vec<String> = Vec::new();

    //dynamic shrinking
    loop {
        lines.clear();
        let mut current_line = String::new();

        for word in &words {
            let test_line = if current_line.is_empty() {
                word.to_string()
            } else {
                format!("{} {}", current_line, word)
            };

            if text_width(&font_face, &test_line, font_size) <= w {
                current_line = test_line;
            } else {
                if !current_line.is_empty() {
                    lines.push(current_line);
                }
                current_line = word.to_string();
            }
        }
        if !current_line.is_empty() {
            lines.push(current_line);
        }

        let total_height = lines.len() as f32 * font_size;

        let fits_width = lines
            .iter()
            .all(|l| text_width(&font_face, l, font_size) <= w);
        let fits_height = total_height <= h;

        if fits_width && fits_height {
            break;
        }

        font_size -= 0.5; // shrink gradually
        if font_size <= 1.0 {
            break; // minimum font size
        }
    }

    let line_height = font_size;
    let total_height = lines.len() as f32 * line_height;

    // --- Vertical alignment ---
    let start_y = match align_v.unwrap_or(VerticalAlign::Top) {
        VerticalAlign::Top => y + (line_height * 0.9), //3.0, //+offset
        VerticalAlign::Middle => y + (h - total_height) / 2.0 + (line_height * 0.75),
        VerticalAlign::Bottom => y + h - total_height + (line_height * 0.75),
    };

    let mut current_y = start_y;
    for line in &lines {
        let line_width = text_width(&font_face, line, font_size);
        let x_offset = match text_alignment.as_ref().unwrap_or(&TextAlignment::Left) {
            TextAlignment::Left => 0.0,
            TextAlignment::Center => (w - line_width) / 2.0,
            TextAlignment::Right => w - line_width,
        };

        draw_text(
            document,
            page,
            DrawTextOptions {
                text: &line,
                font_size,
                font_weight,
                font_type: font_type.clone(),
                text_alignment: None,
                align_v: None,
                color,
                x: x + x_offset,
                y: current_y,
                w,
                h: line_height,
            },
        )?;

        current_y += line_height; // move down for next line
    }

    Ok(())
}
