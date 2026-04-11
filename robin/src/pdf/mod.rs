pub mod helpers;

use crate::{ RobinError };
use helpers::{
    get_title_from_ref
};
use lopdf::{
    Dictionary, Document, Object
};

/// Extract .pdf bookmarks to use as reference when sectioning .pdf text
pub fn get_pdf_bookmarks(doc: &Document) -> Result<Vec<String>, RobinError> {
    let mut bookmarks = Vec::new();
    let mut stack: Vec<Object> = Vec::new();

    let outline_ref = match doc.catalog()?.get(b"Outlines") {
        Ok(obj) => obj,
        Err(_) => return Ok(bookmarks),
    };

    // Initialize stack with first bookmark
    if let Object::Reference(obj_id) = outline_ref {
        let outlines_dict: &Dictionary = doc.get_object(*obj_id)?.as_dict()?;

        // Get .pdf reference (e.g. 6 0 R)
        if let Ok(first_ref) = outlines_dict.get(b"First") {
            stack.push(first_ref.clone());
        }
    }

    // Keep iterating until stack is empty (children bookmarks added before siblings)
    while let Some(current_ref) = stack.pop() {
        if let Object::Reference(bookmark_id) = current_ref {
            let bookmark_dict: &Dictionary = doc.get_object(bookmark_id)?.as_dict()?;

            // Extract title
            if let Ok(title_ref) = bookmark_dict.get(b"Title") {
                let title = get_title_from_ref(doc, title_ref)?;
                bookmarks.push(title);
            }

            // Push next sibling first (so it's processed after children)
            if let Ok(next_ref) = bookmark_dict.get(b"Next") {
                if !matches!(next_ref, Object::Null) {
                    stack.push(next_ref.clone());
                }
            }

            // Push first child (processed next)
            if let Ok(first_child) = bookmark_dict.get(b"First") {
                stack.push(first_child.clone());
            }
        }
    }

    Ok(bookmarks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lopdf::{Dictionary, Document, Object, ObjectId};
    
    /// Creates a minimal document with a real catalog and returns (doc, catalog_id)
    fn make_doc() -> (Document, ObjectId) {
        let mut doc = Document::new();
    
        let catalog_id = doc.add_object(Object::Dictionary({
            let mut d = Dictionary::new();
            d.set("Type", Object::Name(b"Catalog".to_vec()));
            d
        }));
    
        // lopdf uses "Root" in the trailer to locate the catalog
        doc.trailer.set("Root", Object::Reference(catalog_id));
    
        (doc, catalog_id)
    }
    
    /// Inserts a bookmark dict and returns its ObjectId
    fn insert_bookmark(
        doc: &mut Document,
        title: &str,
        first_child: Option<ObjectId>,
        next_sibling: Option<ObjectId>,
    ) -> ObjectId {
        let mut dict = Dictionary::new();
        dict.set(
            "Title",
            Object::String(title.as_bytes().to_vec(), lopdf::StringFormat::Literal),
        );
        if let Some(id) = first_child {
            dict.set("First", Object::Reference(id));
        }
        if let Some(id) = next_sibling {
            dict.set("Next", Object::Reference(id));
        }
        doc.add_object(Object::Dictionary(dict))
    }
    
    /// Attaches an Outlines tree (rooted at `first`) to the catalog
    fn attach_outlines(doc: &mut Document, catalog_id: ObjectId, first: ObjectId) {
        let outlines_id = doc.add_object(Object::Dictionary({
            let mut d = Dictionary::new();
            d.set("Type", Object::Name(b"Outlines".to_vec()));
            d.set("First", Object::Reference(first));
            d
        }));
    
        doc.objects
            .get_mut(&catalog_id)
            .unwrap()
            .as_dict_mut()
            .unwrap()
            .set("Outlines", Object::Reference(outlines_id));
    }
    
    mod get_pdf_bookmarks {
        use super::*;
        
        #[test]
        fn returns_empty_vec_when_no_outlines() {
            let (doc, _) = make_doc();
            let result = get_pdf_bookmarks(&doc).expect("should not error");
            assert!(result.is_empty());
        }
    
        #[test]
        fn single_bookmark() {
            let (mut doc, cat_id) = make_doc();
            let bm = insert_bookmark(&mut doc, "Chapter 1", None, None);
            attach_outlines(&mut doc, cat_id, bm);
    
            let bookmarks = get_pdf_bookmarks(&doc).expect("should not error");
            assert_eq!(bookmarks, vec!["Chapter 1"]);
        }
    
        #[test]
        fn multiple_top_level_siblings() {
            let (mut doc, cat_id) = make_doc();
            let bm3 = insert_bookmark(&mut doc, "Chapter 3", None, None);
            let bm2 = insert_bookmark(&mut doc, "Chapter 2", None, Some(bm3));
            let bm1 = insert_bookmark(&mut doc, "Chapter 1", None, Some(bm2));
            attach_outlines(&mut doc, cat_id, bm1);
    
            let bookmarks = get_pdf_bookmarks(&doc).expect("should not error");
            assert_eq!(bookmarks, vec!["Chapter 1", "Chapter 2", "Chapter 3"]);
        }
    
        #[test]
        fn parent_with_child() {
            let (mut doc, cat_id) = make_doc();
            let child = insert_bookmark(&mut doc, "Section 1.1", None, None);
            let parent = insert_bookmark(&mut doc, "Chapter 1", Some(child), None);
            attach_outlines(&mut doc, cat_id, parent);
    
            let bookmarks = get_pdf_bookmarks(&doc).expect("should not error");
            assert_eq!(bookmarks, vec!["Chapter 1", "Section 1.1"]);
        }
    
        #[test]
        fn siblings_each_with_child() {
            let (mut doc, cat_id) = make_doc();
            let child2 = insert_bookmark(&mut doc, "Section 2.1", None, None);
            let child1 = insert_bookmark(&mut doc, "Section 1.1", None, None);
            let bm2 = insert_bookmark(&mut doc, "Chapter 2", Some(child2), None);
            let bm1 = insert_bookmark(&mut doc, "Chapter 1", Some(child1), Some(bm2));
            attach_outlines(&mut doc, cat_id, bm1);
    
            let bookmarks = get_pdf_bookmarks(&doc).expect("should not error");
            assert_eq!(
                bookmarks,
                vec!["Chapter 1", "Section 1.1", "Chapter 2", "Section 2.1"]
            );
        }
    
        #[test]
        fn three_levels_deep() {
            let (mut doc, cat_id) = make_doc();
            let grandchild = insert_bookmark(&mut doc, "Sub-section 1.1.1", None, None);
            let child = insert_bookmark(&mut doc, "Section 1.1", Some(grandchild), None);
            let root = insert_bookmark(&mut doc, "Chapter 1", Some(child), None);
            attach_outlines(&mut doc, cat_id, root);
    
            let bookmarks = get_pdf_bookmarks(&doc).expect("should not error");
            assert_eq!(
                bookmarks,
                vec!["Chapter 1", "Section 1.1", "Sub-section 1.1.1"]
            );
        }
    
        #[test]
        fn utf16be_encoded_title() {
            let (mut doc, cat_id) = make_doc();
    
            // UTF-16BE BOM + "Hi" (U+0048, U+0069)
            let utf16be_bytes: Vec<u8> = vec![0xFE, 0xFF, 0x00, 0x48, 0x00, 0x69];
            let mut dict = Dictionary::new();
            dict.set(
                "Title",
                Object::String(utf16be_bytes, lopdf::StringFormat::Literal),
            );
            let bm_id = doc.add_object(Object::Dictionary(dict));
            attach_outlines(&mut doc, cat_id, bm_id);
    
            let bookmarks = get_pdf_bookmarks(&doc).expect("should not error");
            assert_eq!(bookmarks, vec!["Hi"]);
        }
    
        #[test]
        fn bookmark_without_title_is_skipped() {
            let (mut doc, cat_id) = make_doc();
            let bm2 = insert_bookmark(&mut doc, "Chapter 2", None, None);
    
            // No Title key, but has a Next pointing to bm2
            let mut dict = Dictionary::new();
            dict.set("Next", Object::Reference(bm2));
            let no_title_id = doc.add_object(Object::Dictionary(dict));
            attach_outlines(&mut doc, cat_id, no_title_id);
    
            let bookmarks = get_pdf_bookmarks(&doc).expect("should not error");
            assert_eq!(bookmarks, vec!["Chapter 2"]);
        }
    
        #[test]
        fn null_next_ref_stops_traversal() {
            let (mut doc, cat_id) = make_doc();
    
            let mut dict = Dictionary::new();
            dict.set(
                "Title",
                Object::String(b"Only Bookmark".to_vec(), lopdf::StringFormat::Literal),
            );
            dict.set("Next", Object::Null);
            let bm_id = doc.add_object(Object::Dictionary(dict));
            attach_outlines(&mut doc, cat_id, bm_id);
    
            let bookmarks = get_pdf_bookmarks(&doc).expect("should not error");
            assert_eq!(bookmarks, vec!["Only Bookmark"]);
        }
    }
}