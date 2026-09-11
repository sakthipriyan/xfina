use thiserror::Error;

#[derive(Error, Debug)]
pub enum XfinaError {
    #[error("I/O Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid Format: {0}")]
    InvalidFormat(String),

    #[error("Parse Error: {0}")]
    ParseError(String),

    #[error("Password required to parse this document")]
    PasswordRequired,

    #[error("Incorrect password provided")]
    IncorrectPassword,

    #[error("Feature not supported: {0}")]
    Unsupported(String),

    /// The file was read successfully, but no compiled-in parser recognised it.
    ///
    /// Distinct from [`XfinaError::InvalidFormat`], which one parser returns to
    /// say "not mine"; this is the answer after every candidate has said so.
    #[error("Unrecognized statement format ({container} file)")]
    UnrecognizedFormat {
        container: crate::detect::Container,
        filename: Option<String>,
    },

    /// A known format whose Cargo feature is not enabled in this build.
    #[error("Format '{0}' is not enabled in this build")]
    FormatNotEnabled(&'static str),
}

impl XfinaError {
    /// A stable, machine-readable tag for this failure.
    ///
    /// Bindings surface this so a caller can branch on the kind of failure --
    /// most importantly, telling "this needs a password" apart from "this file
    /// is not something we can read" -- without matching on message text.
    pub const fn kind(&self) -> &'static str {
        match self {
            XfinaError::Io(_) => "io",
            XfinaError::InvalidFormat(_) => "invalid_format",
            XfinaError::ParseError(_) => "parse_error",
            XfinaError::PasswordRequired => "password_required",
            XfinaError::IncorrectPassword => "incorrect_password",
            XfinaError::Unsupported(_) => "unsupported",
            XfinaError::UnrecognizedFormat { .. } => "unrecognized_format",
            XfinaError::FormatNotEnabled(_) => "format_not_enabled",
        }
    }

    /// Whether detection should move on to the next candidate.
    ///
    /// `InvalidFormat` means "this is not my format" and is expected while
    /// probing. Everything else -- a damaged file, a missing password -- is
    /// worth reporting rather than papering over with another parser's guess.
    pub const fn is_wrong_format(&self) -> bool {
        matches!(self, XfinaError::InvalidFormat(_))
    }
}

/// Bare strings become `ParseError`, the right default deep inside a parser
/// that has already committed to a format. A failure that means "this file is
/// not mine" must say so explicitly with [`XfinaError::InvalidFormat`] --
/// detection relies on the distinction.
impl From<String> for XfinaError {
    fn from(s: String) -> Self {
        XfinaError::ParseError(s)
    }
}

impl From<&str> for XfinaError {
    fn from(s: &str) -> Self {
        XfinaError::ParseError(s.to_string())
    }
}
