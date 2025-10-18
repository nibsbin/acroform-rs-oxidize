use acroform_oxidize::AcroFormDocument;

#[test]
fn test_load_pdf() {
    let pdf_path = "../acroform-rs-old/acroform_files/af8.pdf";
    let doc = AcroFormDocument::from_pdf(pdf_path);
    assert!(doc.is_ok(), "Failed to load PDF: {:?}", doc.err());
}

#[test]
fn test_list_fields() {
    let pdf_path = "../acroform-rs-old/acroform_files/af8.pdf";
    let doc = AcroFormDocument::from_pdf(pdf_path).unwrap();
    let fields = doc.fields();
    
    // For now, just check that it doesn't crash
    // We'll implement proper field reading later
    assert!(fields.is_ok());
}
