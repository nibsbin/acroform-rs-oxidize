use std::fmt;
use std::io;

/// PDF processing errors
///
/// This type wraps errors from oxidize-pdf and provides additional
/// error types specific to form manipulation.
#[derive(Debug)]
pub enum PdfError {
    /// Error parsing or reading PDF
    ParseError(String),
    /// Missing required entry in a PDF dictionary
    MissingEntry {
        typ: &'static str,
        field: String,
    },
    /// I/O error
    IoError(io::Error),
    /// Other errors
    Other(String),
}

impl fmt::Display for PdfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PdfError::ParseError(msg) => write!(f, "PDF parse error: {}", msg),
            PdfError::MissingEntry { typ, field } => {
                write!(f, "Missing required field '{}' in {} dictionary", field, typ)
            }
            PdfError::IoError(e) => write!(f, "I/O error: {}", e),
            PdfError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for PdfError {}

impl From<io::Error> for PdfError {
    fn from(err: io::Error) -> Self {
        PdfError::IoError(err)
    }
}

// Convert from oxidize-pdf errors
impl From<oxidize_pdf::error::PdfError> for PdfError {
    fn from(err: oxidize_pdf::error::PdfError) -> Self {
        PdfError::ParseError(err.to_string())
    }
}

impl From<oxidize_pdf::parser::ParseError> for PdfError {
    fn from(err: oxidize_pdf::parser::ParseError) -> Self {
        PdfError::ParseError(err.to_string())
    }
}
