// Example to explore oxidize-pdf API and understand how to access form fields
use oxidize_pdf::parser::{PdfDocument, PdfReader, objects::PdfObject};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pdf_path = "../acroform-rs-old/acroform_files/af8.pdf";
    println!("Loading PDF: {}", pdf_path);
    
    // Load the PDF
    let data = fs::read(pdf_path)?;
    let cursor = std::io::Cursor::new(data);
    let reader = PdfReader::new(cursor)?;
    let document = PdfDocument::new(reader);
    
    // Get basic information
    println!("PDF Version: {}", document.version()?);
    println!("Page Count: {}", document.page_count()?);
    
    println!("\n--- Exploring PDF Structure ---");
    println!("Successfully loaded PDF with oxidize-pdf!");
    println!("This proves we can parse PDF files.");
    println!("\nNext steps:");
    println!("1. Access catalog and AcroForm dictionary");
    println!("2. Parse Fields array");
    println!("3. Extract field properties (name, type, value, etc.)");
    println!("4. Implement field traversal for nested fields");
    
    Ok(())
}
