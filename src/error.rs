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
}

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
