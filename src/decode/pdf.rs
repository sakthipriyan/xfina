use pdf_extract::{Document, MediaBox, OutputDev, OutputError, Transform};
use std::cell::OnceCell;

use crate::decode::DecodeError;

/// One laid-out glyph and the box it occupies on the page.
#[derive(Debug, Clone)]
pub struct CharItem {
    pub text: String,
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
    /// Whether the glyph sits on the page the normal way up.
    ///
    /// Rotated glyphs have an off-diagonal text matrix (`m11`/`m22` near zero,
    /// `m12`/`m21` near ±1). CAMS stamps a generation watermark as vertical
    /// text down the margin, and a stray watermark glyph landing within the
    /// y-tolerance of a content line silently fuses onto it. Recording the
    /// orientation rather than discarding the glyph lets each parser decide:
    /// CAMS filters these out, SBI keeps everything.
    pub upright: bool,
}

pub struct SpatialOutputDev {
    pub pages: Vec<Vec<CharItem>>,
    current_page: Vec<CharItem>,
    flip_ctm: Transform,
}

impl Default for SpatialOutputDev {
    fn default() -> Self {
        Self::new()
    }
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
    fn begin_page(
        &mut self,
        _page_num: u32,
        media_box: &MediaBox,
        _: Option<(f64, f64, f64, f64)>,
    ) -> Result<(), OutputError> {
        self.current_page.clear();
        self.flip_ctm = Transform::row_major(1., 0., 0., -1., 0., media_box.ury - media_box.lly);
        Ok(())
    }

    fn end_page(&mut self) -> Result<(), OutputError> {
        self.pages.push(self.current_page.clone());
        Ok(())
    }

    fn output_character(
        &mut self,
        trm: &Transform,
        width: f64,
        _spacing: f64,
        font_size: f64,
        char: &str,
    ) -> Result<(), OutputError> {
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
            upright: trm.m12.abs() <= 0.5 && trm.m21.abs() <= 0.5,
        });
        Ok(())
    }

    fn begin_word(&mut self) -> Result<(), OutputError> {
        Ok(())
    }
    fn end_word(&mut self) -> Result<(), OutputError> {
        Ok(())
    }
    fn end_line(&mut self) -> Result<(), OutputError> {
        Ok(())
    }
}

/// A PDF opened and decrypted once, with its expensive derivations cached.
///
/// Loading the document is the costly half of reading a PDF, and detection
/// would otherwise pay it once per candidate parser. `page1_text` is the cheap
/// view probes use to recognise an institution; `pages` is the full spatial
/// layout a parser needs.
pub struct PdfDoc {
    doc: Document,
    pages: OnceCell<Result<Vec<Vec<CharItem>>, DecodeError>>,
    page1_text: OnceCell<String>,
}

impl PdfDoc {
    pub fn open(bytes: &[u8], password: Option<&str>) -> Result<Self, DecodeError> {
        let mut doc = Document::load_mem(bytes)
            .map_err(|e| DecodeError::NotThisContainer(format!("Failed to load PDF: {:?}", e)))?;
        if let Some(pw) = password {
            doc.decrypt(pw)
                .map_err(|_| DecodeError::IncorrectPassword)?;
        } else if doc.is_encrypted() {
            return Err(DecodeError::PasswordRequired);
        }
        Ok(Self {
            doc,
            pages: OnceCell::new(),
            page1_text: OnceCell::new(),
        })
    }

    /// Every glyph on every page, with its position and orientation.
    pub fn pages(&self) -> Result<&[Vec<CharItem>], DecodeError> {
        self.pages
            .get_or_init(|| {
                let mut out = SpatialOutputDev::new();
                // The document opened, so a failure here is damage rather than
                // a wrong guess about the format.
                pdf_extract::output_doc(&self.doc, &mut out)
                    .map_err(|e| DecodeError::Damaged(format!("Extraction failed: {:?}", e)))?;
                Ok(out.pages)
            })
            .as_deref()
            .map_err(Clone::clone)
    }

    /// Plain text of page 1, for probing.
    ///
    /// An institution identifies itself on the first page, so this is all a
    /// probe needs and it avoids laying out a forty-page statement to answer
    /// "is this yours?". Returns an empty string if the page cannot be laid
    /// out -- a probe should decline, not fail the whole parse.
    pub fn page1_text(&self) -> &str {
        self.page1_text.get_or_init(|| {
            let mut text = String::new();
            {
                let mut out = pdf_extract::PlainTextOutput::new(&mut text);
                if pdf_extract::output_doc_page(&self.doc, &mut out, 1).is_err() {
                    return String::new();
                }
            }
            text
        })
    }
}
