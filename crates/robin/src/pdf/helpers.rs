use lopdf::{ Object };
use crate::pdf::types::{ TextSpan };
use regex::Regex;

pub fn is_numeric(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_numeric())
}

pub fn most_common_y_spacing(spans: &[TextSpan]) -> f32 {
    if spans.len() < 2 {
        return 0.0;
    }

    let mut spacing_counts: std::collections::HashMap<i64, usize> = std::collections::HashMap::new();

    let mut prev_y = spans[0].y;
    let mut prev_page = spans[0].page;

    for span in &spans[1..] {
        if span.y != prev_y && span.page == prev_page {
            let spacing = (prev_y - span.y).abs();
            let key = (spacing * 1000.0).round() as i64;
            *spacing_counts.entry(key).or_insert(0) += 1;
        }
        prev_y = span.y;
        prev_page = span.page;
    }

    round_two_decimal(spacing_counts
        .into_iter()
        .max_by_key(|&(_, count)| count)
        .map(|(key, _)| key as f32 / 1000.0)
        .unwrap_or(0.0))
}

pub fn normalize_text(text: &str) -> String {
    let brackets_re = Regex::new(r"\[[^\]]*\]").unwrap();
    let extra_spaces_re = Regex::new(r" {2,}").unwrap();
    let url_re = Regex::new(r"https?://\S+|www\.\S+").unwrap();

    let cleaned = url_re.replace_all(&text, "");
    let cleaned = brackets_re.replace_all(&cleaned, "");
    let cleaned = cleaned
        .replace("- ", "")
        .replace(" , ", ", ")
        // Fix ligatures
        .replace('\u{2}', "fi")
        .replace('\u{3}', "fl")
        .replace('\u{4}', "ff")
        .replace('\u{5}', "ffi")
        .replace('\u{6}', "ffl");

    cleaned
        .lines()
        .map(|line| extra_spaces_re.replace_all(line, " ").to_string())
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

pub fn obj_to_f32(obj: &Object) -> f32 {
    match obj {
        Object::Real(v) => *v,
        Object::Integer(v) => *v as f32,
        _ => 0.0,
    }
}

pub fn round_two_decimal(x: f32) -> f32 {
    (x * 100.0).round() / 100.0
}