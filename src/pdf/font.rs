use std::collections::BTreeMap;

use image::EncodableLayout;
use lopdf::{Dictionary, Document, Object, Stream, content::Content, dictionary};

use crate::{error::BoxedError, fonts::CURSIVE_REGULAR};

fn create_font(document: &mut Document, font_name: &str) -> (u32, u16) {
    let font_id = document.new_object_id();
    let font_dict = dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => font_name.to_string()
    };
    document
        .objects
        .insert(font_id, Object::Dictionary(font_dict));

    font_id
}

pub fn reference_base_fonts(document: &mut Document) -> Result<(), BoxedError> {
    let helvetica_font_id = create_font(document, "Helvetica");
    let courier_font_id = create_font(document, "Courier");
    let times_font_id = create_font(document, "Times New Roman");

    for (_, page_id) in document.get_pages() {
        let mut resources_dict = {
            document
                .get_or_create_resources(page_id.to_owned())?
                .as_dict_mut()?
                .clone()
        };

        let font_entry = resources_dict.get_mut(b"Font");

        let font_obj: &mut Object = if let Ok(entry) = font_entry {
            entry
        } else {
            resources_dict.set(b"Font", Object::Dictionary(Dictionary::new()));
            resources_dict.get_mut(b"Font")?
        };
        let font_dict = font_obj.as_dict_mut()?;

        font_dict.set("pdf-Serif", times_font_id);
        font_dict.set("pdf-SansSerif", helvetica_font_id);
        font_dict.set("pdf-Mono", courier_font_id);
        document
            .get_object_mut(page_id)?
            .as_dict_mut()?
            .set("Resources", resources_dict.clone());
    }

    Ok(())
}

pub fn get_most_used_font_size(document: &Document) -> f32 {
    let mut current_size = 11f32;
    let mut current_scale = 1f32;
    let mut size_map: BTreeMap<String, usize> = BTreeMap::new();
    for page in document.page_iter() {
        for content in document.get_page_contents(page) {
            if let Ok(content_obj) = document.get_object(content)
                && let Ok(stream) = content_obj.as_stream()
                && let Ok(decompressed) = stream.get_plain_content()
                && let Ok(content) = Content::decode(decompressed.as_bytes())
            {
                for operation in content.operations {
                    if operation.operator == "Tf" && operation.operands.len() > 0 {
                        current_size = operation
                            .operands
                            .get(1)
                            .unwrap_or(&operation.operands[0])
                            .as_float()
                            .unwrap_or(current_size);
                    }

                    if operation.operator == "Tj" && operation.operands.len() == 1 {
                        if let Some(operand) = operation.operands.first() {
                            let str = operand.as_str().unwrap_or(&[]);
                            let size = (current_size * current_scale).to_string();
                            let len = str.len();
                            size_map
                                .entry(size)
                                .and_modify(|x| *x += len)
                                .or_insert(len);
                        }
                    }
                    if operation.operator == "TJ"
                        && operation.operands.len() == 1
                        && let Some(operand) = operation.operands.first()
                        && let Ok(operand_array) = operand.as_array()
                    {
                        for el in operand_array {
                            let str = el.as_str().unwrap_or(&[]);
                            let size = (current_size * current_scale).to_string();
                            let len = str.len();
                            size_map
                                .entry(size)
                                .and_modify(|x| *x += len)
                                .or_insert(len);
                        }
                    }
                    if operation.operator == "Tm" && operation.operands.len() == 6 {
                        current_scale = operation.operands[3].as_float().unwrap_or(1f32);
                    }
                }
            }
        }
    }

    size_map
        .iter()
        .max_by_key(|(_, val)| **val)
        .unwrap_or((&"11.0".to_string(), &0usize))
        .0
        .parse::<f32>()
        .unwrap_or(11.0f32)
}

pub fn embed_ttf_font(
    document: &mut Document,
    font_name: &str,
    font_reference_id: &str,
    font_data: &'static [u8],
) -> Result<(), BoxedError> {
    let font_stream_id = document.new_object_id();
    let font_stream = Stream::new(
        lopdf::dictionary! {
            "Length" => font_data.len() as i64,
            "Length1" => font_data.len() as i64, // TrueType needs Length1
        },
        font_data.to_vec(),
    );
    document
        .objects
        .insert(font_stream_id, Object::Stream(font_stream));

    let font_descriptor_id = document.new_object_id();
    document.objects.insert(
        font_descriptor_id,
        Object::Dictionary(lopdf::dictionary! {
            "Type" => "FontDescriptor",
            "FontName" => lopdf::Object::Name(font_name.as_bytes().to_vec()),
            "Flags" => 32,
            "Ascent" => 800,   // dummy values, should come from font metrics
            "Descent" => -450,
            "CapHeight" => 577,
            "ItalicAngle" => 0,
            "CapHeight" => 800,
            "AvgWidth" => 419,
            "MaxWidth" => 1728,
            "FontWeight" => 400,
            "StemV" => 41,
            "XHeight" => 250,
            "FontBBox" => lopdf::Object::Array(vec![
                lopdf::Object::Integer(-464),
                lopdf::Object::Integer(-450),
                lopdf::Object::Integer(1264),
                lopdf::Object::Integer(800),
            ]),
            "FontFile2" => font_stream_id,
        }),
    );

    let font_id = document.new_object_id();
    document.objects.insert(
        font_id,
        Object::Dictionary(lopdf::dictionary! {
            "Type" => "Font",
            "Subtype" => "TrueType",
            "Name" => format!("{font_reference_id}"),
            "BaseFont" => lopdf::Object::Name(font_name.as_bytes().to_vec()),
            "FontDescriptor" => font_descriptor_id,
            "Encoding" => "WinAnsiEncoding",
            "FirstChar" => 32,
            "LastChar" => 126,
            "Widths" => lopdf::Object::Array(vec![
                lopdf::Object::Integer(169),
                lopdf::Object::Integer(295),
                lopdf::Object::Integer(168),
                lopdf::Object::Integer(579),
                lopdf::Object::Integer(313),
                lopdf::Object::Integer(632),
                lopdf::Object::Integer(608),
                lopdf::Object::Integer(106),
                lopdf::Object::Integer(268),
                lopdf::Object::Integer(507),
                lopdf::Object::Integer(507),
                lopdf::Object::Integer(326),
                lopdf::Object::Integer(129),
                lopdf::Object::Integer(249),
                lopdf::Object::Integer(149),
                lopdf::Object::Integer(377),
                lopdf::Object::Integer(368),
                lopdf::Object::Integer(243),
                lopdf::Object::Integer(331),
                lopdf::Object::Integer(314),
                lopdf::Object::Integer(403),
                lopdf::Object::Integer(331),
                lopdf::Object::Integer(336),
                lopdf::Object::Integer(310),
                lopdf::Object::Integer(351),
                lopdf::Object::Integer(336),
                lopdf::Object::Integer(154),
                lopdf::Object::Integer(154),
                lopdf::Object::Integer(293),
                lopdf::Object::Integer(344),
                lopdf::Object::Integer(293),
                lopdf::Object::Integer(434),
                lopdf::Object::Integer(707),
                lopdf::Object::Integer(768),
                lopdf::Object::Integer(562),
                lopdf::Object::Integer(527),
                lopdf::Object::Integer(626),
                lopdf::Object::Integer(478),
                lopdf::Object::Integer(582),
                lopdf::Object::Integer(576),
                lopdf::Object::Integer(770),
                lopdf::Object::Integer(365),
                lopdf::Object::Integer(203),
                lopdf::Object::Integer(560),
                lopdf::Object::Integer(547),
                lopdf::Object::Integer(830),
                lopdf::Object::Integer(721),
                lopdf::Object::Integer(594),
                lopdf::Object::Integer(541),
                lopdf::Object::Integer(599),
                lopdf::Object::Integer(602),
                lopdf::Object::Integer(451),
                lopdf::Object::Integer(432),
                lopdf::Object::Integer(611),
                lopdf::Object::Integer(538),
                lopdf::Object::Integer(892),
                lopdf::Object::Integer(542),
                lopdf::Object::Integer(452),
                lopdf::Object::Integer(453),
                lopdf::Object::Integer(362),
                lopdf::Object::Integer(498),
                lopdf::Object::Integer(397),
                lopdf::Object::Integer(313),
                lopdf::Object::Integer(846),
                lopdf::Object::Integer(223),
                lopdf::Object::Integer(294),
                lopdf::Object::Integer(269),
                lopdf::Object::Integer(238),
                lopdf::Object::Integer(294),
                lopdf::Object::Integer(249),
                lopdf::Object::Integer(182),
                lopdf::Object::Integer(265),
                lopdf::Object::Integer(294),
                lopdf::Object::Integer(192),
                lopdf::Object::Integer(135),
                lopdf::Object::Integer(324),
                lopdf::Object::Integer(214),
                lopdf::Object::Integer(470),
                lopdf::Object::Integer(330),
                lopdf::Object::Integer(265),
                lopdf::Object::Integer(255),
                lopdf::Object::Integer(279),
                lopdf::Object::Integer(272),
                lopdf::Object::Integer(232),
                lopdf::Object::Integer(198),
                lopdf::Object::Integer(393),
                lopdf::Object::Integer(294),
                lopdf::Object::Integer(500),
                lopdf::Object::Integer(299),
                lopdf::Object::Integer(344),
                lopdf::Object::Integer(234),
                lopdf::Object::Integer(354),
                lopdf::Object::Integer(364),
                lopdf::Object::Integer(411),
                lopdf::Object::Integer(277),
            ])
        }),
    );

    for (_, page_id) in document.get_pages() {
        let mut resources_dict = {
            document
                .get_or_create_resources(page_id.to_owned())?
                .as_dict_mut()?
                .clone()
        };

        let font_entry = resources_dict.get_mut(b"Font");

        let font_obj: &mut Object = if let Ok(entry) = font_entry {
            entry
        } else {
            resources_dict.set(b"Font", Object::Dictionary(Dictionary::new()));
            resources_dict.get_mut(b"Font")?
        };
        let font_dict = font_obj.as_dict_mut()?;

        font_dict.set(font_reference_id, font_id);
        document
            .get_object_mut(page_id)?
            .as_dict_mut()?
            .set("Resources", resources_dict.clone());
    }
    Ok(())
}
