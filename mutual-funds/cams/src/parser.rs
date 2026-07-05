use pdf_extract::{OutputDev, OutputError, Transform, Document, MediaBox};

#[derive(Debug, Clone)]
pub struct CharItem {
    pub text: String,
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}

pub struct SpatialOutputDev {
    pub pages: Vec<Vec<CharItem>>,
    current_page: Vec<CharItem>,
    flip_ctm: Transform,
}

impl SpatialOutputDev {
    pub fn new() -> Self {
        Self {
            pages: Vec::new(),
            current_page: Vec::new(),
            flip_ctm: Transform::default(),
        }
    }
}

impl OutputDev for SpatialOutputDev {
    fn begin_page(&mut self, _page_num: u32, media_box: &MediaBox, _: Option<(f64, f64, f64, f64)>) -> Result<(), OutputError> {
        self.current_page.clear();
        self.flip_ctm = Transform::row_major(1., 0., 0., -1., 0., media_box.ury - media_box.lly);
        Ok(())
    }

    fn end_page(&mut self) -> Result<(), OutputError> {
        self.pages.push(self.current_page.clone());
        Ok(())
    }

    fn output_character(&mut self, trm: &Transform, width: f64, _spacing: f64, font_size: f64, char: &str) -> Result<(), OutputError> {
        let position = trm.post_transform(&self.flip_ctm);
        let x = position.m31;
        let y = position.m32;
        
        let scaled_w = trm.m11 * width * font_size; 
        let scaled_h = trm.m22 * font_size;

        self.current_page.push(CharItem {
            text: char.to_string(),
            x0: x,
            y0: y,
            x1: x + scaled_w.abs(),
            y1: y + scaled_h.abs(),
        });
        Ok(())
    }

    fn begin_word(&mut self) -> Result<(), OutputError> { Ok(()) }
    fn end_word(&mut self) -> Result<(), OutputError> { Ok(()) }
    fn end_line(&mut self) -> Result<(), OutputError> { Ok(()) }
}

pub fn extract_spatial_pages(bytes: &[u8], password: Option<&str>) -> Result<Vec<Vec<CharItem>>, String> {
    let mut doc = Document::load_mem(bytes).map_err(|e| format!("Failed to load PDF: {:?}", e))?;
    if let Some(pw) = password {
        doc.decrypt(pw).map_err(|e| format!("Failed to decrypt: {:?}", e))?;
    } else if doc.is_encrypted() {
        return Err("PDF is encrypted, password required".to_string());
    }
    
    let mut out = SpatialOutputDev::new();
    pdf_extract::output_doc(&doc, &mut out).map_err(|e| format!("Extraction failed: {:?}", e))?;
    Ok(out.pages)
}
