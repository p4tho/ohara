use robin::pdf::{ PDFDocument };
use lopdf::{ Document };

mod pdf_document {
    use super::*;
    
    #[test]
    fn bookmarks_extracted_correctly() {
        let doc = Document::load("tests/data/multi-agent.pdf").expect("lopdf document loaded incorrectly");
        let pdf = PDFDocument::new(&doc).unwrap();
        let total_bookmarks: usize = pdf.bookmarks.values().map(|v| v.len()).sum();
        
        assert!(pdf.bookmarks.contains_key(&1));
        assert_eq!(pdf.bookmarks[&1][0].title, "Introduction");
        assert_eq!(total_bookmarks, 25);
    }
    
    #[test]
    fn no_outline_returns_empty() {
        let doc = Document::load("tests/data/smm-rootkit.pdf").expect("lopdf document loaded incorrectly");
        let pdf = PDFDocument::new(&doc).unwrap();
        
        assert!(pdf.bookmarks.is_empty());
    }
}
