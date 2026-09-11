//! SBI's PDF reading, which is now just the shared decode layer.
//!
//! This module used to carry its own copy of the spatial output device; the
//! only thing that ever distinguished it from the CAMS copy was that CAMS
//! dropped rotated glyphs. That difference is now a flag on each glyph, so
//! both read the same extraction.

pub(crate) use crate::decode::pdf::CharItem;
