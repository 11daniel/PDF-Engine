use std::{collections::HashMap, sync::Arc};

use color::Color;
use lopdf::Document;
use serde::{Deserialize, Serialize};

use crate::error::{AppError, BoxedError, GenericError};

pub mod acroform;
pub mod color;
pub mod font;
pub mod image;
pub mod link;
pub mod pool;
pub mod text;

pub type UserVariables = HashMap<String, UserVariableValue>;

pub struct PdfGenerateOptions {
    pub document: Arc<Document>,
    pub variables: Vec<PdfVariable>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableOptions {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub page: usize,
    pub field: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextAlignment {
    #[serde(rename = "left")]
    Left,
    #[serde(rename = "center")]
    Center,
    #[serde(rename = "right")]
    Right,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum VerticalAlign {
    Top,
    Middle,
    Bottom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextVariable {
    #[serde(flatten)]
    pub variable: VariableOptions,
    pub font_size: Option<f32>,
    pub align_h: Option<TextAlignment>,
    pub align_v: Option<VerticalAlign>,
    pub color: Option<Color>,
    pub wrap: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageVariable {
    #[serde(flatten)]
    pub variable: VariableOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PdfVariable {
    #[serde(rename = "text")]
    Text(TextVariable),

    #[serde(rename = "signature")]
    Signature(TextVariable),

    #[serde(rename = "image")]
    Image(ImageVariable),
}

pub struct PdfVariableList(pub Vec<PdfVariable>);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UserVariableValue {
    Text(String),
    Image(String),
}

pub fn get_page_media_box(
    document: &Document,
    page: &(u32, u16),
) -> Result<(f32, f32, f32, f32), BoxedError> {
    let page_object = document.get_object(page.to_owned())?;
    let media_box = page_object.as_dict()?.get(b"MediaBox")?.as_array()?;

    if media_box.len() != 4 {
        return Err(GenericError("Invalid media box".into()).into());
    }

    Ok((
        media_box.first().unwrap().as_float()?,
        media_box.get(1).unwrap().as_float()?,
        media_box.get(2).unwrap().as_float()?,
        media_box.get(3).unwrap().as_float()?,
    ))
}
