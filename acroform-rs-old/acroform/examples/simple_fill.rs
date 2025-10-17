use acroform::{AcroFormDocument, FieldValue};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load the test PDF
    let mut doc = AcroFormDocument::from_pdf("acroform_files/af8_error.pdf")?;
    
    println!("Fields in the PDF:");
    for field in doc.fields()? {
        println!("  Name: {}", field.name);
        println!("    Type: {:?}", field.field_type);
        println!("    Value: {:?}", field.current_value);
        println!("    Default Value: {:?}", field.default_value);
        if let Some(ref tooltip) = field.tooltip {
            println!("    Tooltip: {}", tooltip);
        }
        println!();
    }
    
    // Update the field value
    let mut values = HashMap::new();
    values.insert(
        "P[0].Page1[0].topmostSubform[0].MbrName[1]".to_string(),
        FieldValue::Text("NEW_VALUE".to_string()),
    );
    
    doc.fill_and_save(values, "/tmp/af8_filled.pdf")?;
    println!("Saved filled PDF to /tmp/af8_filled.pdf");
    
    Ok(())
}
