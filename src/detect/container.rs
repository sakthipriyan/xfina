use serde::{Deserialize, Serialize};

/// The physical file format a statement arrives in, read from the leading
/// bytes rather than the extension.
///
/// The extension is not a reliable discriminator: institutions ship `.xls`
/// files that are really xlsx (a zip), `.txt` files that are really
/// spreadsheic exports, and CSVs with a UTF-8 BOM. Sniffing the container
/// first is what lets detection rule out most parsers without opening the
/// file, and what turns "Zip error: could not find EOCD" into "this is a PDF,
/// and no PDF parser claims it".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Container {
    /// Legacy OLE2 compound file — a real `.xls`.
    Ole2,
    /// Zip archive, which for our purposes means xlsx.
    Zip,
    Pdf,
    /// Anything that decodes as UTF-8: CSV, TSV, plain text.
    Text,
    /// Binary of a kind we do not recognise.
    Unknown,
}

impl std::fmt::Display for Container {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

const OLE2_MAGIC: [u8; 8] = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];
const UTF8_BOM: [u8; 3] = [0xEF, 0xBB, 0xBF];

impl Container {
    pub const fn as_str(self) -> &'static str {
        match self {
            Container::Ole2 => "ole2",
            Container::Zip => "zip",
            Container::Pdf => "pdf",
            Container::Text => "text",
            Container::Unknown => "unknown",
        }
    }
}

/// Identifies a file's container from its leading bytes.
///
/// Only the local-file-header signature counts as a zip: an archive that
/// begins with the end-of-central-directory or spanning marker has no first
/// entry to read, so no spreadsheet parser could use it anyway.
pub fn sniff(bytes: &[u8]) -> Container {
    if bytes.starts_with(&OLE2_MAGIC) {
        return Container::Ole2;
    }
    if bytes.starts_with(b"PK\x03\x04") {
        return Container::Zip;
    }
    if bytes.starts_with(b"%PDF") {
        return Container::Pdf;
    }
    if is_text(bytes) {
        return Container::Text;
    }
    Container::Unknown
}

/// Text means "decodes as UTF-8 and is not laced with control bytes".
///
/// Only the head is examined: a statement is identified by its first lines,
/// and a multi-megabyte CSV should not cost a full scan to classify. The
/// prefix is trimmed back to a character boundary so a multi-byte character
/// straddling the cut is not mistaken for invalid UTF-8.
fn is_text(bytes: &[u8]) -> bool {
    const HEAD: usize = 8192;
    let body = bytes.strip_prefix(&UTF8_BOM).unwrap_or(bytes);
    if body.is_empty() {
        return false;
    }
    let head = &body[..body.len().min(HEAD)];
    let head = match std::str::from_utf8(head) {
        Ok(text) => text,
        // A multi-byte character straddling the cut reports an unexpected end
        // of input rather than bad data; keep what decoded and judge on that.
        Err(e) if e.error_len().is_none() => {
            std::str::from_utf8(&head[..e.valid_up_to()]).unwrap_or("")
        }
        Err(_) => return false,
    };
    !head
        .bytes()
        .any(|b| matches!(b, 0x00..=0x08 | 0x0B | 0x0C | 0x0E..=0x1F))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_container_from_the_magic_bytes() {
        assert_eq!(sniff(&OLE2_MAGIC), Container::Ole2);
        assert_eq!(sniff(b"PK\x03\x04rest of an xlsx"), Container::Zip);
        assert_eq!(sniff(b"%PDF-1.4\n"), Container::Pdf);
        assert_eq!(sniff(b"Date,Narration,Amount\n"), Container::Text);
    }

    #[test]
    fn a_bom_does_not_hide_a_csv() {
        assert_eq!(sniff(b"\xEF\xBB\xBFStatement,Header\n"), Container::Text);
    }

    #[test]
    fn binary_that_matches_nothing_is_unknown() {
        assert_eq!(sniff(&[0x00, 0x01, 0x02, 0x03]), Container::Unknown);
        assert_eq!(sniff(&[]), Container::Unknown);
    }

    #[test]
    fn an_extension_never_gets_a_vote() {
        // An ICICI credit-card export named .xls is really a zip; the bytes
        // are the only thing consulted.
        assert_eq!(sniff(b"PK\x03\x04"), Container::Zip);
    }

    #[test]
    fn a_long_csv_cut_mid_character_is_still_text() {
        // The head is capped at 8 KiB; a multi-byte character landing on that
        // boundary must not read as binary.
        let mut bytes = b"Date,Narration\n".to_vec();
        while bytes.len() < 8191 {
            bytes.push(b'a');
        }
        bytes.extend_from_slice("\u{20B9}1234".as_bytes()); // rupee sign spans the cut
        assert_eq!(sniff(&bytes), Container::Text);
    }

    #[test]
    fn an_ole2_body_is_not_mistaken_for_text() {
        let mut bytes = OLE2_MAGIC.to_vec();
        bytes.extend_from_slice(&[0u8; 64]);
        assert_eq!(sniff(&bytes), Container::Ole2);
    }
}
