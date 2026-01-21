use lopdf::{Dictionary, Document, Object};

use crate::{error::BoxedError, pdf::get_page_media_box};

#[derive(Debug)]
pub struct DrawLinkOptions<'a> {
    pub link: &'a str,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

pub fn add_link<'a>(
    document: &'a mut Document,
    page: &(u32, u16),
    options: DrawLinkOptions<'a>,
) -> Result<(), BoxedError> {
    let (_, _, _page_w, page_h) = get_page_media_box(document, page)?;

    let rect = vec![
        Object::Real(options.x),                      // x1
        Object::Real(page_h - options.y - options.h), // y1
        Object::Real(options.x + options.w),          // x2
        Object::Real(page_h - options.y),             // y2
    ];

    let annotation = Dictionary::from_iter(vec![
        ("Type", Object::Name("Annot".into())),
        ("Subtype", Object::Name("Link".into())),
        ("Rect", Object::Array(rect)),
        (
            "Border",
            Object::Array(vec![
                Object::Integer(0),
                Object::Integer(0),
                Object::Integer(0),
            ]),
        ), // no border
        (
            "A",
            Object::Dictionary(Dictionary::from_iter(vec![
                ("S", Object::Name("URI".into())),
                (
                    "URI",
                    Object::String(options.link.into(), lopdf::StringFormat::Literal),
                ),
            ])),
        ),
    ]);

    let annotation_id = document.add_object(annotation);

    let page_dict = document.get_object_mut(*page)?.as_dict_mut().unwrap();

    match page_dict.get_mut(b"Annots") {
        Ok(annots_obj) => {
            if let Ok(ref_id) = annots_obj.as_reference() {
                // Case 1: Annots is an indirect reference to an array
                let arr = document.get_object_mut(ref_id)?.as_array_mut()?;
                arr.push(Object::Reference(annotation_id));
            } else if let Ok(arr) = annots_obj.as_array_mut() {
                // Case 2: Annots is a direct array
                arr.push(Object::Reference(annotation_id));
            } else {
                // Case 3: Annots exists but isn't an array or reference — overwrite
                page_dict.set(
                    "Annots",
                    Object::Array(vec![Object::Reference(annotation_id)]),
                );
            }
        }
        Err(_) => {
            // Case 4: No /Annots key yet — create new one
            page_dict.set(
                "Annots",
                Object::Array(vec![Object::Reference(annotation_id)]),
            );
        }
    }

    Ok(())
}
