use crate::value::FieldValue;

/// Field type enumeration
///
/// Represents the different types of form fields in PDF documents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldType {
    /// Text input field
    Text,
    /// Button (checkbox, radio button, or push button)
    Button,
    /// Choice field (dropdown or list)
    Choice,
    /// Signature field
    Signature,
}

/// High-level representation of a form field
///
/// This struct contains all the information needed to understand and manipulate
/// a PDF form field, including its name, type, current value, and flags.
#[derive(Debug, Clone)]
pub struct FormField {
    /// The fully qualified name of the field (e.g., "parent.child.field")
    pub name: String,
    /// The type of the field (e.g., Text, Button, Choice)
    pub field_type: FieldType,
    /// The current value of the field, if any
    pub current_value: Option<FieldValue>,
    /// The default value of the field (DV entry in PDF specification), if any
    pub default_value: Option<FieldValue>,
    /// Field flags as defined in the PDF specification
    pub flags: u32,
    /// The tooltip/alternate name of the field (TU entry in PDF specification)
    pub tooltip: Option<String>,
}
