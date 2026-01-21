use lopdf::{Document, Object};

pub fn remove_acroforms(document: &mut Document) {
    let pages = document.get_pages();

    if let Ok(root) = document.trailer.get(b"Root")
        && let Ok(root_ref) = root.as_reference()
        && let Ok(Object::Dictionary(catalog)) = document.get_object_mut(root_ref)
    {
        catalog.remove(b"AcroForm");
    }

    for page in pages {
        if let Ok(Object::Dictionary(page_dict)) = document.get_object_mut(page.1) {
            page_dict.remove(b"Annots");
        }
    }

    document.prune_objects();
}
