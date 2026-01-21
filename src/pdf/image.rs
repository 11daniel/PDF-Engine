use std::io::Write;

use flate2::{Compression, write::ZlibEncoder};
use image::ImageFormat;
use lopdf::{
    Dictionary, Document, Object, Stream,
    content::{Content, Operation},
};
use uuid::Uuid;

use crate::error::{BoxedError, GenericError};

use super::get_page_media_box;

pub struct DrawImageOptions<'a> {
    pub image_data: &'a Vec<u8>,
    pub image_type: ImageFormat,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

pub fn draw_image<'a>(
    document: &'a mut Document,
    page: &(u32, u16),
    options: DrawImageOptions<'a>,
) -> Result<(), BoxedError> {
    let (_, _, _page_w, page_h) = get_page_media_box(document, page)?;

    let mut resources_dict = document
        .get_or_create_resources(page.to_owned())?
        .as_dict_mut()?
        .clone();

    let mut image_data = options.image_data.clone();
    let mut image_is_jpg = true;

    let img = image::load_from_memory_with_format(&image_data, options.image_type)?;

    let image_width = img.width();
    let image_height = img.height();

    if options.image_type.to_mime_type() != "image/jpeg" {
        let img_vec = img.to_rgb8().into_raw();

        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
        encoder.write_all(img_vec.as_slice())?;

        let image_data_bytes = encoder.finish()?;
        image_data = image_data_bytes;

        image_is_jpg = false;
    }

    let image_stream = Stream::new(
        Dictionary::from_iter(vec![
            ("Type", Object::Name(b"XObject".to_vec())),
            ("Subtype", Object::Name(b"Image".to_vec())),
            ("Width", Object::Integer(image_width as i64)),
            ("Height", Object::Integer(image_height as i64)),
            ("ColorSpace", Object::Name(b"DeviceRGB".to_vec())),
            ("BitsPerComponent", Object::Integer(8)),
            (
                "Filter",
                Object::Name(if image_is_jpg {
                    b"DCTDecode".to_vec()
                } else {
                    b"FlateDecode".to_vec()
                }),
            ),
        ]),
        image_data,
    );

    let image_id = document.add_object(image_stream);

    let image_uuid = Uuid::now_v7().to_string().replace('-', "");

    // Get mutable access to the resources dictionary
    let xobj_dict = match resources_dict.get_mut(b"XObject") {
        Ok(obj) => obj.as_dict_mut()?,
        Err(_) => {
            // Create new XObject dictionary
            let mut xdict = Dictionary::new();
            xdict.set(image_uuid.clone(), image_id);
            resources_dict.set("XObject", Object::Dictionary(xdict));
            // Return a fresh mutable ref
            resources_dict.get_mut(b"XObject")?.as_dict_mut()?
        }
    };
    //let mut xobj_dict = resources_dict.get_mut(b"XObject")?.as_dict_mut()?;

    // Add the image to the XObject dictionary
    xobj_dict.set(image_uuid.clone(), image_id);

    let new_resources_id = document.add_object(Object::Dictionary(resources_dict.clone()));
    document
        .get_object_mut(page.to_owned())?
        .as_dict_mut()?
        .set("Resources", new_resources_id);

    // Create new content stream (draw image at x=100, y=400, in pixels/points)
    let content = Content {
        operations: vec![
            Operation::new("q", vec![]), // Save graphics state
            Operation::new(
                "cm",
                vec![
                    (options.w as f64).into(),
                    0.into(),
                    0.into(),
                    (options.h as f64).into(),
                    options.x.into(),
                    (page_h - options.y - options.h).into(), // Position (x, y)
                ],
            ),
            Operation::new("Do", vec![Object::Name(image_uuid.as_bytes().to_vec())]), // Draw image
            Operation::new("Q", vec![]), // Restore graphics state
        ],
    };

    let encoded_content = content.encode()?;
    document.add_page_contents(page.to_owned(), encoded_content)?;

    Ok(())
}
