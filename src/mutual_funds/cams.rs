//! CAMS PDF reading. The spatial extraction lives in the shared decode layer;
//! what is specific to CAMS is which glyphs it must ignore.

pub use crate::decode::pdf::CharItem;
use crate::decode::Decoded;

use crate::models::validation::ParseResult;

use crate::models::request::ParseRequest;

pub fn parse_cams_pdf(
    input: ParseRequest<'_>,
) -> Result<ParseResult<crate::models::MutualFundsAccount>, crate::error::XfinaError> {
    let decoded = Decoded::new(&input);
    parse_decoded(&decoded, &input)
}

/// Parses from an already-decoded input, so detection can probe and
/// parse against one read of the file.
pub(crate) fn parse_decoded(
    decoded: &Decoded<'_>,
    input: &ParseRequest<'_>,
) -> Result<ParseResult<crate::models::MutualFundsAccount>, crate::error::XfinaError> {
    let filename = input.filename;
    let pages = decoded.pdf()?.pages()?;

    // A CAS names its registrar and its folios. A PDF with neither is not one,
    // and would otherwise come back as an account with no folios at all.
    if !has_cas_markers(pages) {
        return Err(crate::error::XfinaError::InvalidFormat(
            "Not a mutual fund consolidated account statement".to_string(),
        ));
    }

    let mut all_pages_lines = Vec::new();
    for page in pages {
        // CAMS stamps a document-generation watermark down the page margin as
        // vertical text. Left in, one of its glyphs occasionally lands within
        // the y-tolerance of an unrelated content line and fuses onto it,
        // corrupting text matches -- an AMC heading being the case that bit.
        // Everything rotated goes before lines are grouped.
        let upright: Vec<CharItem> = page.iter().filter(|c| c.upright).cloned().collect();
        let lines = super::layout::group_into_lines(&upright, 2.0); // 2.0 pt tolerance
        all_pages_lines.push(lines);
    }

    super::cas::parse_cas_lines(all_pages_lines, filename)
}

/// Markers a consolidated account statement carries on its opening pages --
/// the registrar that produced it, or the folio structure it is built around.
fn has_cas_markers(pages: &[Vec<CharItem>]) -> bool {
    const MARKERS: [&str; 5] = [
        "CAMS",
        "KFINTECH",
        "KARVY",
        "CONSOLIDATED ACCOUNT STATEMENT",
        "FOLIO NO",
    ];
    let text: String = pages
        .iter()
        .take(2)
        .flat_map(|page| page.iter().map(|c| c.text.as_str()))
        .collect::<String>()
        .to_uppercase();
    MARKERS.iter().any(|m| text.contains(m))
}

/// A consolidated account statement names its registrar and its folios.
pub(crate) fn probe(dec: &crate::decode::Decoded<'_>) -> crate::detect::Claim {
    use crate::detect::{probe::any_marker, Claim};
    let Ok(doc) = dec.pdf() else {
        return Claim::NO;
    };
    let text = doc.page1_text().to_uppercase();
    if any_marker(&text, &["CONSOLIDATED ACCOUNT STATEMENT", "CAMSCASWS"]) {
        return Claim::strong("cas-title");
    }
    if any_marker(&text, &["CAMS", "KFINTECH", "KARVY"]) && text.contains("FOLIO") {
        return Claim::weak("cas-registrar");
    }
    Claim::NO
}
