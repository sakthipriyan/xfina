use serde::Serialize;

use crate::decode::Decoded;

/// How strongly a parser claims a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Strength {
    /// Nothing in the file suggests this format.
    No,
    /// Plausible, but not conclusive -- worth trying if nothing stronger claims it.
    Weak,
    /// A marker only this format prints.
    Strong,
}

/// One parser's answer to "is this yours?".
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Claim {
    pub strength: Strength,
    /// Why, as a fixed reason code.
    ///
    /// Deliberately `&'static str` rather than `String`: a `Claim` is reported
    /// in `Detection`, which is serialized to callers and rendered in the web
    /// UI. Making it impossible to build from matched text means no narration,
    /// name or account number can ever leak out through a detection result.
    pub reason: &'static str,
}

impl Claim {
    pub const NO: Claim = Claim {
        strength: Strength::No,
        reason: "",
    };

    pub const fn strong(reason: &'static str) -> Claim {
        Claim {
            strength: Strength::Strong,
            reason,
        }
    }

    pub const fn weak(reason: &'static str) -> Claim {
        Claim {
            strength: Strength::Weak,
            reason,
        }
    }

    pub const fn is_match(&self) -> bool {
        !matches!(self.strength, Strength::No)
    }
}

/// A parser's content probe: cheap, read-only, and never fails.
pub type Probe = fn(&Decoded<'_>) -> Claim;

/// Convenience for the common probe shape: "does the head of this file carry
/// any of these markers?". Markers must be uppercase.
pub fn any_marker(haystack: &str, markers: &[&'static str]) -> bool {
    markers.iter().any(|m| haystack.contains(m))
}
