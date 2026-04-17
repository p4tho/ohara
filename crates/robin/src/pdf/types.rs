#[derive(Debug)]
pub struct PDFBookmark {
    pub title: String,
    pub page: usize
}

#[derive(Debug)]
pub struct TextSpan {
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub font_name: String,
    pub font_size: f32,
    pub page: u32,
}