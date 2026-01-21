use std::{
    io::Cursor,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use aws_sdk_s3::primitives::ByteStream;
use axum::{
    Json,
    body::{Body, Bytes},
    extract::State,
    response::Response,
};
use crc32fast::hash;
use image::ImageFormat;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    error::{AppError, GenericError},
    fonts::{CURSIVE_REGULAR_BYTES, FontType, FontWeight},
    pdf::{
        PdfVariable,
        acroform::remove_acroforms,
        color::Color,
        font::{embed_ttf_font, get_most_used_font_size, reference_base_fonts},
        get_page_media_box,
        image::{DrawImageOptions, draw_image},
        link::{DrawLinkOptions, add_link},
        text::{DrawTextOptions, draw_text, draw_text_fit, draw_text_wrap, draw_text_wrap_fit},
    },
    pdf::{TextAlignment, VerticalAlign},
    state::AppState,
    util::get_file_url,
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratePdfRequest {
    template_url: String,
    variables: Vec<PdfVariable>,
    include_hash_in_header: Option<bool>,
    store_output: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratePdfResponse {
    template_name: String,
    template_hash: String,
    form_schema_hash: String,
    //form_schema_values_hash: String,
    download_link: String,
    error: Option<String>,
}

#[axum::debug_handler]
pub async fn generate_pdf(
    State(state): State<AppState>,
    Json(payload): Json<GeneratePdfRequest>,
) -> Result</*Json<GeneratePdfResponse>*/ Response<Body>, AppError> {
    let url = Url::parse(&payload.template_url)?;
    let filename = url
        .path_segments()
        .ok_or(GenericError("Cannot be a URL".into()))?
        .next_back()
        .or(Some("unknown.pdf"));

    let template = reqwest::get(&payload.template_url).await?.bytes().await?;

    log::trace!("template length: {}", template.len());

    let template_hash = hash(&template);
    let form_schema_hash = hash(serde_json::to_string(&payload.variables)?.as_bytes());
    let generated_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let mut document = lopdf::Document::load_mem(&template)?;
    remove_acroforms(&mut document);
    let most_used_font_size = get_most_used_font_size(&document);

    // get page refs
    let page_refs = document.get_pages();

    reference_base_fonts(&mut document)?;

    let mut has_signature_embedded = false;

    let page = document.page_iter().next().unwrap();

    for variable in &payload.variables {
        match variable {
            PdfVariable::Text(variable) => {
                let page_ref = page_refs
                    .get(&(variable.variable.page as u32))
                    .ok_or(GenericError("Page not found".into()))?;

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
                    align_v: variable.align_v.clone(),
                };

                // if variable.wrap.unwrap_or(false) {
                //     draw_text_wrap(&mut document, page_ref, opts)?;
                // } else {
                //     draw_text_fit(&mut document, page_ref, opts)?;
                // }
                //draw_text_wrap(&mut document, page_ref, opts)?; //use this for wrapping newline below (does not dynamically shrinks)
                //draw_text_fit(&mut document, page_ref, opts)?;
                draw_text_wrap_fit(&mut document, page_ref, opts)?; // use this for dynamic shrinking however not stable yet 
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
                    text_alignment: variable.align_h.clone().or(Some(TextAlignment::Center)), //TODO: make this option if you want hardcoded alignment -> variable.align_h.clone().or(Some(TextAlignment::Center)) or Some(TextAlignment::Center)
                    align_v: variable.align_v.clone().or(Some(VerticalAlign::Bottom)), //TODO: make this option if you want hardcoded alignment -> variable.align_v.clone().or(Some(VerticalAlign::Bottom)) or Some(VerticalAlign::Bottom),
                };
                draw_text_fit(
                    &mut document,
                    page_refs
                        .get(&(variable.variable.page as u32))
                        .ok_or(GenericError("Page not found".into()))?,
                    opts,
                )?;
            }
            PdfVariable::Image(variable) => {
                let image_req = reqwest::get(&variable.variable.value).await?;

                let image_mime = image_req
                    .headers()
                    .get("content-type")
                    .ok_or(GenericError("Missing Content-Type header".into()))?;
                let image_type = ImageFormat::from_mime_type(image_mime.to_str()?)
                    .ok_or(GenericError("Invalid MIME Type".into()))?;

                let image_contents = image_req.bytes().await?.to_vec();

                draw_image(
                    &mut document,
                    page_refs
                        .get(&(variable.variable.page as u32))
                        .ok_or(GenericError("Page not found".into()))?,
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
        };
    }

    for page in document.get_pages() {
        if payload.include_hash_in_header.unwrap_or(false) {
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
                    x: page_w - 350f32, //old 232f32
                    y: page_h - 20f32,  //old 20f32
                    w: 220f32,          //old 220f32
                    h: 12f32,           //old 9f32
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
                    x: page_w - 350f32, //old 232f32
                    y: page_h - 20f32,  //old 20f32
                    w: 220f32,          //old 220f32
                    h: 12f32,           //old 9f32
                },
            )?;
        }
    }

    document.compress();

    let mut out_document = Vec::new();
    let mut out_cursor = Cursor::new(&mut out_document);

    let mut doc_id = uuid::Uuid::new_v4().to_string();
    doc_id.push_str(".pdf");

    document.save_to(&mut out_cursor)?;

    // log::trace!("Uploading to S3...");

    // let s3_upload = state
    //     .s3_client
    //     .put_object()
    //     .bucket(&state.env.s3_bucket)
    //     .key(&doc_id)
    //     .body(ByteStream::from(out_document))
    //     .send()
    //     .await;

    // if let Err(err) = s3_upload {
    //     Err(GenericError(format!("Unable to upload to S3: {err:?}")).into())
    // } else {
    //     Ok(Json(GeneratePdfResponse {
    //         template_name: filename.unwrap_or("document.pdf").to_string(),
    //         template_hash: format!("{template_hash:x}"),
    //         form_schema_hash: format!("{form_schema_hash:x}"),
    //         download_link: get_file_url(&state.env, &doc_id, &state.env.s3_bucket),
    //         error: None,
    //     }))
    // }
    // Ok(Json(GeneratePdfResponse {
    //     template_name: filename.unwrap_or("document.pdf").to_string(),
    //     template_hash: format!("{template_hash:x}"),
    //     form_schema_hash: format!("{form_schema_hash:x}"),
    //     download_link: get_file_url(&state.env, &doc_id, &state.env.s3_bucket),
    //     error: None,
    // }))
    Ok(Response::builder()
        .status(200)
        .header("X-Template-Hash", format!("{template_hash:x}"))
        .header("X-Form-Schema-Hash", format!("{form_schema_hash:x}"))
        .header("X-Generated-Timestamp", generated_timestamp)
        .header("Content-Type", String::from("application/pdf"))
        .body(Body::from(out_document))?)
}

//#[axum::debug_handler]
//pub async fn mass_generate_pdf(Json(payload): Json<PdfMassGenerateOptions>) {}
