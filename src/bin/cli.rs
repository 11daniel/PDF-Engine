use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

use crc32fast::hash;
use dialoguer::{Confirm, Input, Select};
use lopdf::Document;

use pdfsnap_server::{
    error::GenericError,
    fonts::{CURSIVE_REGULAR_BYTES, FontType, FontWeight},
    pdf::{
        ImageVariable, PdfVariable, TextAlignment, VerticalAlign, TextVariable, VariableOptions,
        acroform::remove_acroforms,
        color::Color,
        font::{embed_ttf_font, get_most_used_font_size, reference_base_fonts},
        get_page_media_box,
        image::{DrawImageOptions, draw_image},
        link::{DrawLinkOptions, add_link},
        text::{DrawTextOptions, draw_text, draw_text_fit, draw_text_wrap, draw_text_wrap_fit},
    },
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== PDFSnap CLI - Interactive PDF Generator ===\n");

    // Get template path/URL
    let template_input: String = Input::new()
        .with_prompt("Enter PDF template path or URL")
        .interact_text()?;

    // Load template
    let template_bytes = if template_input.starts_with("http://") || template_input.starts_with("https://") {
        println!("Downloading template from URL...");
        reqwest::get(&template_input).await?.bytes().await?.to_vec()
    } else {
        println!("Loading template from file...");
        fs::read(&template_input)?
    };

    // Load PDF document
    let mut document = Document::load_mem(&template_bytes)?;
    remove_acroforms(&mut document);
    let most_used_font_size = get_most_used_font_size(&document);
    let page_refs = document.get_pages();
    reference_base_fonts(&mut document)?;

    println!("\nTemplate loaded successfully!");
    println!("Most used font size: {:.2}", most_used_font_size);
    println!("Number of pages: {}", page_refs.len());
    
    // Show available page numbers for debugging
    // Note: lopdf's get_pages() returns keys that are 1-indexed (1, 2, 3...)
    let mut page_numbers: Vec<u32> = page_refs.keys().cloned().collect();
    page_numbers.sort();
    println!("Available page numbers in PDF: {:?}", page_numbers);
    println!("(Enter page numbers as 1, 2, 3, etc. when prompted)\n");

    // Get number of variables
    let num_variables: usize = Input::new()
        .with_prompt("How many variables do you want to add?")
        .default(1)
        .interact_text()?;

    let mut variables = Vec::new();
    let mut has_signature_embedded = false;

    // Collect variables
    for i in 0..num_variables {
        println!("\n--- Variable {} ---", i + 1);
        
        let var_types = vec!["text", "signature", "image"];
        let var_type_idx = Select::new()
            .with_prompt("Variable type")
            .items(&var_types)
            .default(0)
            .interact()?;

        let var_type = var_types[var_type_idx];

        // Get common properties
        let max_page = page_refs.len();
        let page: usize = loop {
            let input: usize = Input::new()
                .with_prompt(format!("Page number (1-indexed, 1-{})", max_page))
                .default(1)
                .validate_with(|input: &usize| {
                    if *input < 1 || *input > max_page {
                        Err(format!("Page number must be between 1 and {}", max_page))
                    } else {
                        Ok(())
                    }
                })
                .interact_text()?;
            break input;
        };

        let x: f32 = Input::new()
            .with_prompt("X position")
            .default(100.0)
            .interact_text()?;

        let y: f32 = Input::new()
            .with_prompt("Y position")
            .default(100.0)
            .interact_text()?;

        let w: f32 = Input::new()
            .with_prompt("Width")
            .default(200.0)
            .interact_text()?;

        let h: f32 = Input::new()
            .with_prompt("Height")
            .default(50.0)
            .interact_text()?;

        let field: String = Input::new()
            .with_prompt("Field name (identifier)")
            .default(format!("field_{}", i + 1))
            .interact_text()?;

        match var_type {
            "text" => {
                let mut value: String = Input::new()
                    .with_prompt("Text value")
                    .interact_text()?;

                let font_size: Option<f32> = Input::new()
                    .with_prompt("Font size (press Enter for default)")
                    .allow_empty(true)
                    .interact_text()
                    .ok()
                    .and_then(|s: String| if s.is_empty() { None } else { s.parse().ok() });

                let alignment_options = vec!["left", "center", "right"];
                let alignment_idx = Select::new()
                    .with_prompt("Text alignment")
                    .items(&alignment_options)
                    .default(0)
                    .interact()?;

                let text_alignment = match alignment_options[alignment_idx] {
                    "left" => Some(TextAlignment::Left),
                    "center" => Some(TextAlignment::Center),
                    "right" => Some(TextAlignment::Right),
                    _ => None,
                };


                let v_alignment_options = vec!["top", "middle", "bottom"];
                let v_alignment_idx = Select::new()
                    .with_prompt("Vertical alignment")
                    .items(&v_alignment_options)
                    .default(0) // Default to "top"
                    .interact()?;

                let v_alignment = match v_alignment_options[v_alignment_idx] {
                    "top" => Some(VerticalAlign::Top),
                    "middle" => Some(VerticalAlign::Middle),
                    "bottom" => Some(VerticalAlign::Bottom),
                    _ => None,
                };

                let color_input: String = Input::new()
                    .with_prompt("Color (hex format, e.g., #000000 for black, or press Enter for default)")
                    .allow_empty(true)
                    .default("#000000".to_string())
                    .interact_text()?;

                let color = if color_input.is_empty() {
                    None
                } else {
                    Some(parse_color(&color_input)?)
                };

                let wrap_text: bool = Confirm::new()
                    .with_prompt("Wrap long text onto new lines (instead of shrinking to fit)?")
                    .default(true)
                    .interact()?;

                if wrap_text {
                    let char_limit_input: String = Input::new()
                        .with_prompt("Maximum characters per line (press Enter to skip)")
                        .allow_empty(true)
                        .default("".to_string())
                        .interact_text()?;

                    if let Ok(limit) = char_limit_input.trim().parse::<usize>() {
                        if limit > 0 {
                            value = wrap_text_at_char_limit(&value, limit);
                        }
                    } else if !char_limit_input.trim().is_empty() {
                        println!("⚠️ Could not parse character limit. Ignoring and using automatic wrapping.");
                    }
                }

                variables.push(PdfVariable::Text(TextVariable {
                    variable: VariableOptions {
                        x,
                        y,
                        w,
                        h,
                        page: page - 1, // Convert to 0-indexed
                        field,
                        value,
                    },
                    font_size,
                    align_h: text_alignment,
                    align_v: v_alignment,
                    color,
                    wrap: Some(wrap_text),
                }));
            }
            "signature" => {
                let value: String = Input::new()
                    .with_prompt("Signature text")
                    .interact_text()?;

                let font_size: Option<f32> = Input::new()
                    .with_prompt("Font size (press Enter for default)")
                    .allow_empty(true)
                    .interact_text()
                    .ok()
                    .and_then(|s: String| if s.is_empty() { None } else { s.parse().ok() });

                let v_alignment_options = vec!["top", "middle", "bottom"];
                let v_alignment_idx = Select::new()
                    .with_prompt("Vertical alignment")
                    .items(&v_alignment_options)
                    .default(0) // Default to "top"
                    .interact()?;

                let v_alignment = match v_alignment_options[v_alignment_idx] {
                    "top" => Some(VerticalAlign::Top),
                    "middle" => Some(VerticalAlign::Middle),
                    "bottom" => Some(VerticalAlign::Bottom),
                    _ => None,
                };

                variables.push(PdfVariable::Signature(TextVariable {
                    variable: VariableOptions {
                        x,
                        y,
                        w,
                        h,
                        page: page - 1,
                        field,
                        value,
                    },
                    font_size,
                    align_h: None,
                    align_v: v_alignment,
                    color: None,
                    wrap: None,
                }));
            }
            "image" => {
                let image_url: String = Input::new()
                    .with_prompt("Image URL")
                    .interact_text()?;

                variables.push(PdfVariable::Image(ImageVariable {
                    variable: VariableOptions {
                        x,
                        y,
                        w,
                        h,
                        page: page - 1,
                        field,
                        value: image_url,
                    },
                }));
            }
            _ => {}
        }
    }

    // Helper function to get page reference by 1-indexed page number
    // Note: lopdf's get_pages() returns a map with 1-indexed keys (1, 2, 3...)
    let get_page_ref = |page_num_1_indexed: usize| -> Result<&(u32, u16), GenericError> {
        let page_key = page_num_1_indexed as u32;
        page_refs.get(&page_key).ok_or_else(|| {
            let available: Vec<String> = page_refs.keys().map(|k| k.to_string()).collect();
            GenericError(format!(
                "Page {} (1-indexed) not found. Available pages: {:?}",
                page_num_1_indexed, available
            ))
        })
    };

    // Process variables
    println!("\nProcessing variables...");
    for variable in &variables {
        match variable {
            PdfVariable::Text(variable) => {
                // variable.variable.page is 0-indexed, convert to 1-indexed for lookup
                let page_ref = get_page_ref(variable.variable.page + 1)?;
                
                let opts = DrawTextOptions {
                    text: &variable.variable.value,
                    font_weight: None,
                    font_type: Some(FontType::SansSerif),
                    font_size: variable.font_size.unwrap_or(most_used_font_size),
                    color: variable.color,
                    x: variable.variable.x,
                    y: variable.variable.y,
                    w: variable.variable.w,
                    h: variable.variable.h,
                    text_alignment: variable.align_h.clone(),
                    align_v: variable.align_v.clone()
                };

                if variable.wrap.unwrap_or(true) {
                    draw_text_wrap_fit(&mut document, page_ref, opts)?;
                } else {
                    draw_text_fit(&mut document, page_ref, opts)?;
                }
            }
            PdfVariable::Signature(variable) => {
                if !has_signature_embedded {
                    embed_ttf_font(
                        &mut document,
                        "ABCDEE+ItalianoRegular",
                        "pdf-Cursive",
                        CURSIVE_REGULAR_BYTES,
                    )?;
                    has_signature_embedded = true;
                }

                // variable.variable.page is 0-indexed, convert to 1-indexed for lookup
                let page_ref = get_page_ref(variable.variable.page + 1)?;

                let opts = DrawTextOptions {
                    text: &variable.variable.value,
                    font_weight: Some(FontWeight::Regular),
                    font_type: Some(FontType::Cursive),
                    font_size: variable.font_size.unwrap_or(most_used_font_size),
                    color: variable.color,
                    x: variable.variable.x,
                    y: variable.variable.y,
                    w: variable.variable.w,
                    h: variable.variable.h,
                    text_alignment: None,
                    align_v: variable.align_v.clone(),
                };
                draw_text_fit(&mut document, page_ref, opts)?;
            }
            PdfVariable::Image(variable) => {
                println!("Downloading image from: {}", variable.variable.value);
                let image_req = reqwest::get(&variable.variable.value).await?;

                let image_mime = image_req
                    .headers()
                    .get("content-type")
                    .ok_or(GenericError("Missing Content-Type header".into()))?;
                let image_type = image::ImageFormat::from_mime_type(image_mime.to_str()?)
                    .ok_or(GenericError("Invalid MIME Type".into()))?;

                let image_contents = image_req.bytes().await?.to_vec();

                // variable.variable.page is 0-indexed, convert to 1-indexed for lookup
                let page_ref = get_page_ref(variable.variable.page + 1)?;

                draw_image(
                    &mut document,
                    page_ref,
                    DrawImageOptions {
                        image_data: &image_contents,
                        image_type,
                        x: variable.variable.x,
                        y: variable.variable.y,
                        w: variable.variable.w,
                        h: variable.variable.h,
                    },
                )?;
            }
        }
    }

    // Ask about hash header
    let include_hash = Select::new()
        .with_prompt("Include verification hash in header?")
        .items(&["No", "Yes"])
        .default(0)
        .interact()? == 1;

    if include_hash {
        let template_hash = hash(&template_bytes);
        let form_schema_hash = hash(serde_json::to_string(&variables)?.as_bytes());
        let generated_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        for page in document.get_pages() {
            let (_, _, page_w, page_h) = get_page_media_box(&document, &page.1)?;
            let verification_code =
                format!("{generated_timestamp}-{template_hash:x}-{form_schema_hash:x}");
            add_link(
                &mut document,
                &page.1,
                DrawLinkOptions {
                    link: &format!(
                        "https://docs.betterinternship.com?verification-code={}",
                        verification_code
                    ),
                    x: page_w - 232f32,
                    y: page_h - 20f32,
                    w: 220f32,
                    h: 9f32,
                },
            )?;
            draw_text(
                &mut document,
                &page.1,
                DrawTextOptions {
                    text: &format!(
                        "BetterInternship E-Sign Verification Code: {verification_code}"
                    ),
                    font_size: 9.0f32,
                    font_weight: Some(FontWeight::Regular),
                    font_type: Some(FontType::SansSerif),
                    color: Some(Color::from_rgb(0x80, 0x80, 0x80)),
                    text_alignment: None,
                    align_v: None,
                    x: page_w - 232f32,
                    y: page_h - 20f32,
                    w: 220f32,
                    h: 12f32,
                },
            )?;
        }
    }

    // Compress and save
    document.compress();

    let output_path: String = Input::new()
        .with_prompt("Output file path")
        .default("output.pdf".to_string())
        .interact_text()?;

    let mut output_file = fs::File::create(&output_path)?;
    document.save_to(&mut output_file)?;

    println!("\n✅ PDF generated successfully!");
    println!("📄 Saved to: {}", output_path);

    Ok(())
}

fn parse_color(hex: &str) -> Result<Color, Box<dyn std::error::Error>> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return Err("Hex color must be 6 digits".into());
    }

    let r = u8::from_str_radix(&hex[0..2], 16)?;
    let g = u8::from_str_radix(&hex[2..4], 16)?;
    let b = u8::from_str_radix(&hex[4..6], 16)?;

    Ok(Color::from_rgb(r, g, b))
}

fn wrap_text_at_char_limit(text: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return text.to_string();
    }

    let mut out = String::with_capacity(text.len() + text.len() / max_chars + 1);
    let mut count = 0usize;

    for ch in text.chars() {
        if ch == '\n' {
            count = 0;
            out.push(ch);
            continue;
        }

        if count >= max_chars {
            out.push('\n');
            count = 0;
        }

        out.push(ch);
        count += 1;
    }

    out
}

