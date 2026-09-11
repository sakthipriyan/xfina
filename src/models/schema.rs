use serde::{Deserialize, Serialize};

/// Which JSON shape a parsed account is rendered into.
///
/// This is the *output* selector, and is deliberately distinct from
/// `Format`, which names the *input* — the parser that read the file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Schema {
    /// Xfina's own shape: the ReBIT tree plus our `xfina` extension objects,
    /// with every date rendered as a UNIX timestamp.
    #[default]
    Xfina,
    /// Strict ReBIT / Sahamati AA output: the `xfina` extensions are stripped
    /// and the fields listed in `date_only_paths` stay `YYYY-MM-DD`.
    Rebit,
}

impl Schema {
    /// Parses a schema name, defaulting to [`Schema::Xfina`] for anything
    /// unrecognised — matching how the bindings have always treated this.
    pub fn from_name(name: Option<&str>) -> Self {
        match name {
            Some(s) if s.eq_ignore_ascii_case("rebit") => Schema::Rebit,
            _ => Schema::Xfina,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Schema::Xfina => "xfina",
            Schema::Rebit => "rebit",
        }
    }
}
