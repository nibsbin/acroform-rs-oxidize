/// Typed representation of field values
///
/// This enum represents the different types of values that can be stored in PDF form fields.
/// Each variant corresponds to a specific field type in the PDF specification.
#[derive(Debug, Clone, PartialEq)]
pub enum FieldValue {
    /// Text field value (used for text input fields)
    Text(String),
    /// Boolean value (used for checkboxes and radio buttons)
    Boolean(bool),
    /// Choice value (used for dropdown menus and radio button selections)
    Choice(String),
    /// Integer value (used for numeric fields)
    Integer(i32),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_field_value_variants() {
        let text = FieldValue::Text("hello".to_string());
        let boolean = FieldValue::Boolean(true);
        let choice = FieldValue::Choice("option1".to_string());
        let integer = FieldValue::Integer(42);
        
        assert!(matches!(text, FieldValue::Text(_)));
        assert!(matches!(boolean, FieldValue::Boolean(_)));
        assert!(matches!(choice, FieldValue::Choice(_)));
        assert!(matches!(integer, FieldValue::Integer(_)));
    }
}
