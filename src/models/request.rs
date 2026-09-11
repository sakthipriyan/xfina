#[derive(Debug, Clone)]
pub struct ParseRequest<'a> {
    pub content: &'a [u8],
    pub password: Option<&'a str>,
    pub filename: Option<&'a str>,
    pub modified_timestamp: Option<i64>, // UNIX timestamp in seconds
    /// Parse as this format instead of detecting one.
    ///
    /// Having the override ride on the request is what keeps `parse` a single
    /// entry point: there is no separate "parse as" function for every surface
    /// to expose and keep in step.
    pub format: Option<crate::detect::Format>,
}

impl<'a> ParseRequest<'a> {
    pub fn new(content: &'a [u8]) -> Self {
        Self {
            content,
            password: None,
            filename: None,
            modified_timestamp: None,
            format: None,
        }
    }

    pub fn with_password(mut self, password: Option<&'a str>) -> Self {
        self.password = password;
        self
    }

    pub fn with_filename(mut self, filename: Option<&'a str>) -> Self {
        self.filename = filename;
        self
    }

    pub fn with_modified_timestamp(mut self, timestamp: Option<i64>) -> Self {
        self.modified_timestamp = timestamp;
        self
    }

    /// Skip detection and use this format.
    pub fn with_format(mut self, format: Option<crate::detect::Format>) -> Self {
        self.format = format;
        self
    }
}
