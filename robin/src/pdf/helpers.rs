use crate::{ RobinError };
use lopdf::{ Document, Object };

pub fn decode_pdf_bytes(bytes: &[u8]) -> String {
    // UTF-16BE: BOM 0xFE 0xFF
    if bytes.starts_with(&[0xFE, 0xFF]) {
        let utf16: Vec<u16> = bytes[2..]
            .chunks_exact(2)
            .map(|b| u16::from_be_bytes([b[0], b[1]]))
            .collect();
        return String::from_utf16_lossy(&utf16).to_owned();
    }

    // UTF-16LE: BOM 0xFF 0xFE
    if bytes.starts_with(&[0xFF, 0xFE]) {
        let utf16: Vec<u16> = bytes[2..]
            .chunks_exact(2)
            .map(|b| u16::from_le_bytes([b[0], b[1]]))
            .collect();
        return String::from_utf16_lossy(&utf16).to_owned();
    }

    // UTF-8: try first before falling back to PDFDocEncoding
    if let Ok(s) = std::str::from_utf8(bytes) {
        return s.to_owned();
    }

    // PDFDocEncoding fallback (ISO Latin-1 extended)
    bytes.iter().map(|&b| pdfdoc_to_char(b)).collect()
}

/// Maps PDFDocEncoding bytes to Unicode characters
pub fn pdfdoc_to_char(b: u8) -> char {
    match b {
        // PDFDocEncoding-specific range (0x80–0x9F differs from Latin-1)
        0x80 => '\u{02D8}', // BREVE
        0x81 => '\u{02C7}', // CARON
        0x82 => '\u{02C6}', // MODIFIER LETTER CIRCUMFLEX
        0x83 => '\u{02D9}', // DOT ABOVE
        0x84 => '\u{02DD}', // DOUBLE ACUTE ACCENT
        0x85 => '\u{02DB}', // OGONEK
        0x86 => '\u{02DA}', // RING ABOVE
        0x87 => '\u{02DC}', // SMALL TILDE
        0x88 => '\u{2014}', // EM DASH
        0x89 => '\u{2013}', // EN DASH
        0x8A => '\u{2018}', // LEFT SINGLE QUOTATION MARK
        0x8B => '\u{2019}', // RIGHT SINGLE QUOTATION MARK
        0x8C => '\u{201C}', // LEFT DOUBLE QUOTATION MARK
        0x8D => '\u{201D}', // RIGHT DOUBLE QUOTATION MARK
        0x8E => '\u{2022}', // BULLET
        0x8F => '\u{2026}', // HORIZONTAL ELLIPSIS
        0x90 => '\u{2020}', // DAGGER
        0x91 => '\u{2021}', // DOUBLE DAGGER
        0x92 => '\u{2030}', // PER MILLE SIGN
        0x93 => '\u{2022}', // BULLET (duplicate, often unused)
        0x94 => '\u{2014}', // EM DASH (duplicate, often unused)
        0x95 => '\u{0160}', // LATIN CAPITAL LETTER S WITH CARON
        0x96 => '\u{0161}', // LATIN SMALL LETTER S WITH CARON
        0x97 => '\u{0178}', // LATIN CAPITAL LETTER Y WITH DIAERESIS
        0x98 => '\u{017D}', // LATIN CAPITAL LETTER Z WITH CARON
        0x99 => '\u{017E}', // LATIN SMALL LETTER Z WITH CARON
        0x9A => '\u{0131}', // LATIN SMALL LETTER DOTLESS I
        0x9B => '\u{0142}', // LATIN SMALL LETTER L WITH STROKE
        0x9C => '\u{0152}', // LATIN CAPITAL LIGATURE OE
        0x9D => '\u{0153}', // LATIN SMALL LIGATURE OE
        0x9E => '\u{0192}', // LATIN SMALL LETTER F WITH HOOK
        0x9F => '\u{02C6}', // MODIFIER LETTER CIRCUMFLEX (duplicate)
        0xA0 => '\u{00A0}', // NO-BREAK SPACE
        // 0xA1–0xFF: identical to Latin-1 / Unicode code points
        _ => b as char,
    }
}

pub fn get_title_from_ref(doc: &Document, title_ref: &Object) -> Result<String, RobinError> {
    let title_obj = match title_ref {
        Object::Reference(obj_id) => doc.get_object(*obj_id)?,
        _ => &title_ref.clone(),
    };

    match title_obj {
        Object::String(bytes, _) => Ok(decode_pdf_bytes(&bytes)),
        _ => Err(RobinError::InvalidTitle),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod decode_pdf_bytes {
        use super::*;
        
        #[test]
        fn utf16be_basic_ascii() {
            // BOM + "Hello"
            let mut bytes = vec![0xFE, 0xFF];
            for c in "Hello".encode_utf16() {
                bytes.push((c >> 8) as u8);
                bytes.push((c & 0xFF) as u8);
            }
            assert_eq!(decode_pdf_bytes(&bytes), "Hello");
        }
        
        #[test]
        fn utf16be_unicode() {
            // BOM + "café"
            let mut bytes = vec![0xFE, 0xFF];
            for c in "café".encode_utf16() {
                bytes.push((c >> 8) as u8);
                bytes.push((c & 0xFF) as u8);
            }
            assert_eq!(decode_pdf_bytes(&bytes), "café");
        }
        
        #[test]
        fn utf16be_empty_after_bom() {
            let bytes = vec![0xFE, 0xFF];
            assert_eq!(decode_pdf_bytes(&bytes), "");
        }
        
        #[test]
        fn utf16be_chinese_characters() {
            // BOM + "中文"
            let mut bytes = vec![0xFE, 0xFF];
            for c in "中文".encode_utf16() {
                bytes.push((c >> 8) as u8);
                bytes.push((c & 0xFF) as u8);
            }
            assert_eq!(decode_pdf_bytes(&bytes), "中文");
        }
        
        // --- UTF-16LE ---
        
        #[test]
        fn utf16le_basic_ascii() {
            // BOM + "Hello"
            let mut bytes = vec![0xFF, 0xFE];
            for c in "Hello".encode_utf16() {
                bytes.push((c & 0xFF) as u8);
                bytes.push((c >> 8) as u8);
            }
            assert_eq!(decode_pdf_bytes(&bytes), "Hello");
        }
        
        #[test]
        fn utf16le_unicode() {
            // BOM + "café"
            let mut bytes = vec![0xFF, 0xFE];
            for c in "café".encode_utf16() {
                bytes.push((c & 0xFF) as u8);
                bytes.push((c >> 8) as u8);
            }
            assert_eq!(decode_pdf_bytes(&bytes), "café");
        }
        
        #[test]
        fn utf16le_empty_after_bom() {
            let bytes = vec![0xFF, 0xFE];
            assert_eq!(decode_pdf_bytes(&bytes), "");
        }
        
        // --- UTF-8 ---
        
        #[test]
        fn utf8_plain_ascii() {
            assert_eq!(decode_pdf_bytes(b"Chapter 1"), "Chapter 1");
        }
        
        #[test]
        fn utf8_multibyte() {
            assert_eq!(decode_pdf_bytes("café".as_bytes()), "café");
        }
        
        #[test]
        fn utf8_empty() {
            assert_eq!(decode_pdf_bytes(b""), "");
        }
        
        #[test]
        fn utf8_symbols() {
            assert_eq!(decode_pdf_bytes("© 2024".as_bytes()), "© 2024");
        }
        
        // --- PDFDocEncoding ---
        
        #[test]
        fn pdfdoc_plain_ascii() {
            // ASCII bytes with no valid UTF-8 high bytes — stays in PDFDoc path
            // Use a byte that's invalid UTF-8 to force the fallback
            let bytes: Vec<u8> = vec![0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x80]; // "Hello" + BREVE
            let result = decode_pdf_bytes(&bytes);
            assert!(result.starts_with("Hello"));
            assert!(result.contains('\u{02D8}')); // BREVE
        }
        
        #[test]
        fn pdfdoc_em_dash() {
            let bytes: Vec<u8> = vec![0x88];
            assert_eq!(decode_pdf_bytes(&bytes), "\u{2014}");
        }
        
        #[test]
        fn pdfdoc_en_dash() {
            let bytes: Vec<u8> = vec![0x89];
            assert_eq!(decode_pdf_bytes(&bytes), "\u{2013}");
        }
        
        #[test]
        fn pdfdoc_curly_quotes() {
            let bytes: Vec<u8> = vec![0x8C, 0x8D];
            assert_eq!(decode_pdf_bytes(&bytes), "\u{201C}\u{201D}");
        }
        
        #[test]
        fn pdfdoc_ellipsis() {
            let bytes: Vec<u8> = vec![0x8F];
            assert_eq!(decode_pdf_bytes(&bytes), "\u{2026}");
        }
        
        #[test]
        fn pdfdoc_s_with_caron() {
            let bytes: Vec<u8> = vec![0x95, 0x96]; // Š š
            assert_eq!(decode_pdf_bytes(&bytes), "\u{0160}\u{0161}");
        }
        
        #[test]
        fn pdfdoc_oe_ligatures() {
            let bytes: Vec<u8> = vec![0x9C, 0x9D]; // Œ œ
            assert_eq!(decode_pdf_bytes(&bytes), "\u{0152}\u{0153}");
        }
        
        #[test]
        fn pdfdoc_latin1_high_range() {
            let bytes: Vec<u8> = vec![0xE9]; // é in Latin-1
            assert_eq!(decode_pdf_bytes(&bytes), "é");
        }
        
        #[test]
        fn pdfdoc_no_break_space() {
            let bytes: Vec<u8> = vec![0xA0];
            assert_eq!(decode_pdf_bytes(&bytes), "\u{00A0}");
        }
        
        // --- Edge cases ---
        
        #[test]
        fn empty_input() {
            assert_eq!(decode_pdf_bytes(b""), "");
        }
        
        #[test]
        fn single_byte_ascii() {
            assert_eq!(decode_pdf_bytes(b"A"), "A");
        }
        
        #[test]
        fn just_fe_byte_no_second_bom_byte() {
            let bytes: Vec<u8> = vec![0xFE];
            let result = decode_pdf_bytes(&bytes);
            assert_eq!(result, "\u{00FE}");
        }
        
        #[test]
        fn utf16be_odd_trailing_byte_ignored() {
            let bytes = vec![0xFE, 0xFF, 0x00, 0x48, 0x00];
            let result = decode_pdf_bytes(&bytes);
            assert_eq!(result, "H");
        }
    }
}