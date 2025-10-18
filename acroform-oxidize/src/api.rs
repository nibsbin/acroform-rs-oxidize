use crate::error::PdfError;
use crate::field::FormField;
use crate::value::FieldValue;
use oxidize_pdf::parser::{PdfDocument, PdfReader};
use std::collections::HashMap;
use std::io::Cursor;
use std::path::Path;

/// Main API for working with PDF forms
///
/// This struct provides the primary interface for loading PDF files,
/// reading form fields, and filling form values.
///
/// # Examples
///
/// ```no_run
/// use acroform_oxidize::{AcroFormDocument, FieldValue};
/// use std::collections::HashMap;
///
/// let mut doc = AcroFormDocument::from_pdf("form.pdf").unwrap();
///
/// // List all fields
/// for field in doc.fields().unwrap() {
///     println!("{}: {:?}", field.name, field.current_value);
/// }
///
/// // Fill fields
/// let mut values = HashMap::new();
/// values.insert("name".to_string(), FieldValue::Text("John".to_string()));
/// doc.fill_and_save(values, "filled.pdf").unwrap();
/// ```
pub struct AcroFormDocument {
    // Internal PDF representation
    data: Vec<u8>,
    // Parsed document for reading
    document: PdfDocument<Cursor<Vec<u8>>>,
}

impl AcroFormDocument {
    /// Load a PDF file from the given path
    ///
    /// Opens and parses a PDF file, preparing it for form field manipulation.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the PDF file to load
    ///
    /// # Errors
    ///
    /// Returns `PdfError` if the file cannot be opened or parsed.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use acroform_oxidize::AcroFormDocument;
    ///
    /// let doc = AcroFormDocument::from_pdf("form.pdf").unwrap();
    /// ```
    pub fn from_pdf(path: impl AsRef<Path>) -> Result<Self, PdfError> {
        let data = std::fs::read(path)?;
        Self::from_bytes(data)
    }
    
    /// Load a PDF from a byte vector
    ///
    /// Parses a PDF from an in-memory byte vector, preparing it for form field manipulation.
    ///
    /// # Arguments
    ///
    /// * `data` - A byte vector containing the PDF data
    ///
    /// # Errors
    ///
    /// Returns `PdfError` if the data cannot be parsed as a valid PDF.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use acroform_oxidize::AcroFormDocument;
    /// use std::fs;
    ///
    /// let data = fs::read("form.pdf").unwrap();
    /// let doc = AcroFormDocument::from_bytes(data).unwrap();
    /// ```
    pub fn from_bytes(data: Vec<u8>) -> Result<Self, PdfError> {
        // Parse the PDF using oxidize-pdf
        let cursor = Cursor::new(data.clone());
        let reader = PdfReader::new(cursor)?;
        let document = PdfDocument::new(reader);
        
        Ok(AcroFormDocument { data, document })
    }
    
    /// Get all form fields in the PDF
    ///
    /// Returns a vector of all fillable form fields in the document.
    /// Each field includes its name, type, current value, and flags.
    ///
    /// # Errors
    ///
    /// Returns `PdfError` if field information cannot be retrieved from the PDF.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use acroform_oxidize::AcroFormDocument;
    ///
    /// let doc = AcroFormDocument::from_pdf("form.pdf").unwrap();
    /// for field in doc.fields().unwrap() {
    ///     println!("Field: {} (type: {:?})", field.name, field.field_type);
    /// }
    /// ```
    pub fn fields(&self) -> Result<Vec<FormField>, PdfError> {
        // TODO: Implement field discovery
        Ok(Vec::new())
    }
    
    /// Fill form fields with provided values and return the PDF as a byte vector
    ///
    /// Updates the specified form fields with new values and returns the modified
    /// PDF as an in-memory byte vector. Fields not specified in the `values` map remain unchanged.
    ///
    /// # Arguments
    ///
    /// * `values` - A map from field names to their new values
    ///
    /// # Errors
    ///
    /// Returns `PdfError` if field updates cannot be applied.
    pub fn fill(
        &mut self,
        _values: HashMap<String, FieldValue>,
    ) -> Result<Vec<u8>, PdfError> {
        // TODO: Implement field filling
        Ok(self.data.clone())
    }
    
    /// Fill form fields with provided values and save to a new file
    ///
    /// Updates the specified form fields with new values and writes the modified
    /// PDF to the output path.
    ///
    /// # Arguments
    ///
    /// * `values` - A map from field names to their new values
    /// * `output` - Path where the filled PDF should be saved
    ///
    /// # Errors
    ///
    /// Returns `PdfError` if field updates cannot be applied or the file cannot be written.
    pub fn fill_and_save(
        &mut self,
        values: HashMap<String, FieldValue>,
        output: impl AsRef<Path>,
    ) -> Result<(), PdfError> {
        let bytes = self.fill(values)?;
        std::fs::write(output, bytes)?;
        Ok(())
    }
}
