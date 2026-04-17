mod helpers;
mod types;

use crate::{ RobinError };
use helpers::{ obj_to_f32 };
use lopdf::{ Document, Object };
use std::collections::{ BTreeMap };
use types::{ PDFBookmark, TextSpan };

#[derive(Debug)]
pub struct PDFDocument {
    pub bookmarks: BTreeMap<usize, Vec<PDFBookmark>>, // index determines position on page with multiple bookmarks
    pub text_spans: BTreeMap<usize, Vec<TextSpan>>
}

impl PDFDocument {
    /// Uses lopdf's Document struct to extract necessary data for .m4b conversion
    pub fn new(
        doc: &Document
    ) -> Result<Self, RobinError> {
        let bookmarks = Self::extract_pdf_bookmarks(doc)?;
        let mut text_spans = BTreeMap::new();
        
        // Extract text spans for each page
        for (page_num, _) in doc.get_pages() {
            let page_text_spans = Self::extract_page_text_spans(doc, page_num);
            text_spans.insert(page_num as usize, page_text_spans);
        }
        
        Ok(Self {
            bookmarks,
            text_spans
        })
    }
    
    fn extract_pdf_bookmarks(
        doc: &Document
    ) -> Result<BTreeMap<usize, Vec<PDFBookmark>>, RobinError> {
        let mut bookmarks: BTreeMap<usize, Vec<PDFBookmark>> = BTreeMap::new();
        let toc = match doc.get_toc() {
            Ok(toc) => toc,
            Err(lopdf::Error::DictKey(_)) => return Ok(BTreeMap::new()), // no outline
            Err(e) => return Err(e.into()),
        };
        
        for entry in toc.toc {
            bookmarks
                .entry(entry.page)
                .or_insert_with(Vec::new)
                .push(PDFBookmark {
                    title: entry.title,
                    page: entry.page
                });
        }
    
        Ok(bookmarks)
    }
    
    fn extract_page_text_spans(
        doc: &Document,
        page_num: u32
    ) -> Vec<TextSpan> {
        let mut spans = Vec::new();
    
        let pages = doc.get_pages();
        let page_id = match pages.get(&page_num) {
            Some(id) => *id,
            None => return spans,
        };
    
        let content = match doc.get_and_decode_page_content(page_id) {
            Ok(c) => c,
            Err(_) => return spans,
        };
    
        let mut x: f32 = 0.0;
        let mut y: f32 = 0.0;
        let mut font_name: String = String::new();
        let mut font_size: f32 = 12.0;
        // Text matrix position offsets
        let mut tx: f32 = 0.0;
        let mut ty: f32 = 0.0;
        let mut leading: f32 = 0.0;
    
        for op in &content.operations {
            match op.operator.as_str() {
                // Set font and size: Tf fontname size
                "Tf" => {
                    if let (Some(name), Some(size)) = (op.operands.get(0), op.operands.get(1)) {
                        font_name = match name {
                            Object::Name(bytes) => String::from_utf8_lossy(bytes).to_string(),
                            _ => String::new(),
                        };
                        font_size = match size {
                            Object::Real(s) => *s,
                            Object::Integer(s) => *s as f32,
                            _ => font_size,
                        };
                    }
                }
                // Set text matrix: Tm a b c d e f  (e=x, f=y)
                "Tm" => {
                    if let (Some(e), Some(f)) = (op.operands.get(4), op.operands.get(5)) {
                        x = obj_to_f32(e);
                        y = obj_to_f32(f);
                        tx = 0.0;
                        ty = 0.0;
                    }
                }
                // Move text position: Td tx ty
                "Td" | "TD" => {
                    if let (Some(dx), Some(dy)) = (op.operands.get(0), op.operands.get(1)) {
                        tx += obj_to_f32(dx);
                        ty += obj_to_f32(dy);
                        if op.operator == "TD" {
                            leading = -obj_to_f32(op.operands.get(1).unwrap());
                        }
                    }
                }
                // Move to next line using leading: T*
                "T*" => {
                    ty -= leading;
                }
                // Show string: Tj
                "Tj" => {
                    if let Some(Object::String(bytes, _)) = op.operands.get(0) {
                        let text = String::from_utf8_lossy(bytes).to_string();
                        spans.push(TextSpan {
                            text,
                            x: x + tx,
                            y: y + ty,
                            font_name: font_name.clone(),
                            font_size,
                            page: page_num,
                        });
                    }
                }
                // Show string array: TJ (handles kerning arrays)
                "TJ" => {
                    if let Some(Object::Array(arr)) = op.operands.get(0) {
                        let mut text = String::new();
                        for obj in arr {
                            match obj {
                                Object::String(bytes, _) => {
                                    text.push_str(&String::from_utf8_lossy(bytes));
                                }
                                Object::Integer(offset) => {
                                    // Large negative offset = word space
                                    if *offset < -100 {
                                        text.push(' ');
                                    }
                                }
                                Object::Real(offset) => {
                                    if *offset < -100.0 {
                                        text.push(' ');
                                    }
                                }
                                _ => {}
                            }
                        }
                
                        if !text.is_empty() {
                            spans.push(TextSpan {
                                text,
                                x: x + tx,
                                y: y + ty,
                                font_name: font_name.clone(),
                                font_size,
                                page: page_num,
                            });
                        }
                    }
                }
                // Move to next line and show string
                "'" => {
                    ty -= leading;
                    if let Some(Object::String(bytes, _)) = op.operands.get(0) {
                        let text = String::from_utf8_lossy(bytes).to_string();
                        spans.push(TextSpan {
                            text,
                            x: x + tx,
                            y: y + ty,
                            font_name: font_name.clone(),
                            font_size,
                            page: page_num,
                        });
                    }
                }
                _ => {}
            }
        }
    
        spans
    }
}
