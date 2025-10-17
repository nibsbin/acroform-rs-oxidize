# AcroForm Oxidize Implementation Plan

## Overview

This document outlines the plan to reimplement the `acroform` crate using the `oxidize-pdf@1.6.0` crate from crates.io instead of the forked `pdf` crate. The goal is to achieve **API parity** with the existing `acroform-rs-old/acroform` implementation while leveraging the modern, actively maintained oxidize-pdf library.

## Current Implementation Analysis

### Existing API Surface (acroform-rs-old/acroform)

The current implementation provides a minimal, high-level API for PDF form manipulation:

#### Core Types

1. **`AcroFormDocument`** - Main entry point for form operations
   ```rust
   pub struct AcroFormDocument {
       file: CachedFile<Vec<u8>>,
   }
   ```

2. **`FormField`** - High-level representation of a form field
   ```rust
   pub struct FormField {
       pub name: String,                      // Fully qualified name
       pub field_type: FieldType,             // Text, Button, Choice, Signature
       pub current_value: Option<FieldValue>, // Current value
       pub default_value: Option<FieldValue>, // Default value (DV)
       pub flags: u32,                        // Field flags
       pub tooltip: Option<String>,           // Alternate name (TU)
   }
   ```

3. **`FieldValue`** - Typed field values
   ```rust
   pub enum FieldValue {
       Text(String),      // Text fields
       Boolean(bool),     // Checkboxes
       Choice(String),    // Radio buttons, dropdowns
       Integer(i32),      // Numeric fields
   }
   ```

#### Public API Methods

**Loading PDFs:**
- `AcroFormDocument::from_pdf(path: impl AsRef<Path>) -> Result<Self, PdfError>`
- `AcroFormDocument::from_bytes(data: Vec<u8>) -> Result<Self, PdfError>`

**Reading Fields:**
- `fields(&self) -> Result<Vec<FormField>, PdfError>`

**Filling Forms:**
- `fill(&mut self, values: HashMap<String, FieldValue>) -> Result<Vec<u8>, PdfError>`
- `fill_and_save(&mut self, values: HashMap<String, FieldValue>, output: impl AsRef<Path>) -> Result<(), PdfError>`

#### Extension Traits (Internal)

1. **`FieldDictionaryExt`**
   - `get_full_name(&self, resolver: &impl Resolve) -> Result<String, PdfError>`
   - `traverse_field_refs(&self, resolver: &impl Resolve) -> Result<Vec<RcRef<FieldDictionary>>, PdfError>`

2. **`InteractiveFormDictionaryExt`**
   - `all_fields(&self, resolver: &impl Resolve) -> Result<Vec<RcRef<FieldDictionary>>, PdfError>`
   - `find_field_by_name(&self, name: &str, resolver: &impl Resolve) -> Result<Option<RcRef<FieldDictionary>>, PdfError>`

### Current Dependencies

The implementation depends on the forked `pdf` crate (`acroform-pdf`), which provides:
- `pdf::file::{CachedFile, FileOptions}` - File handling with in-memory cache
- `pdf::object::{FieldDictionary, FieldType, InteractiveFormDictionary, RcRef, Updater, Annot}` - PDF object types
- `pdf::primitive::{Primitive, PdfString, Dictionary}` - Low-level PDF primitives
- `pdf::error::PdfError` - Error handling

### Key Behaviors

1. **Field Name Resolution**: Automatically constructs fully qualified hierarchical field names (e.g., "parent.child.field")
2. **Field Traversal**: Recursively walks field hierarchies to find all terminal (fillable) fields
3. **Value Encoding**: Converts string values to UTF-16BE with BOM for proper PDF text encoding
4. **Dual Update Strategy**: Updates both field dictionaries AND page annotations to ensure compatibility
5. **NeedAppearances Flag**: Relies on PDF viewers to regenerate appearances (no appearance stream generation)
6. **In-Memory Operations**: Supports both file-based and fully in-memory workflows

## Oxidize-PDF Analysis

### Available Modules (v1.6.0)

The `oxidize-pdf` crate provides comprehensive PDF functionality:

#### Parsing/Reading Module (`parser`)
- `parser::PdfReader` - Opens and reads PDF files
- `parser::PdfDocument` - High-level document interface
- `parser::PdfObject` - Low-level PDF objects
- `parser::ParsedPage` - Page representation

#### Forms Module (`forms`)
- **Form Creation** (for generating new forms):
  - `AcroForm` - Interactive form dictionary
  - `FormManager` - High-level form creation API
  - `TextField`, `CheckBox`, `RadioButton`, `ComboBox`, `ListBox`, `PushButton` - Field types
  - `Widget`, `FormField` - Field and widget representations
  - `FieldOptions`, `FieldFlags` - Configuration options

- **Appearance Generation**:
  - `AppearanceGenerator`, `AppearanceStream`, `AppearanceDictionary`
  - Various appearance generators for different field types

#### Objects Module (`objects`)
- `Dictionary` - PDF dictionary representation
- `Object` - Generic PDF object enum
- `ObjectReference` - Object reference handling

#### Document Module (`document`)
- `Document` - Main document type for generation
- `Page` - Page handling

### Capabilities Assessment

**✅ Available in oxidize-pdf:**
- PDF parsing and reading (via `parser` module)
- Form structure representation (`forms::AcroForm`, `forms::FormField`)
- Low-level object access (`objects` module)
- Dictionary manipulation
- Document writing/saving

**❓ Needs Investigation:**
- Reading existing form field values from parsed PDFs
- Updating field values in existing PDFs
- Field hierarchy traversal for existing forms
- Annotation handling and updates
- In-memory PDF modification (may need custom implementation)

**❌ Potential Gaps:**
- Direct equivalents to `CachedFile<Vec<u8>>` with update capabilities
- `Updater` trait for in-place object modification
- `RcRef<T>` reference-counted indirect object references
- `Resolve` trait for dereferencing PDF objects

## Implementation Strategy

### Phase 1: Core Infrastructure

Create the foundational types and conversion utilities:

1. **Error Handling**
   - Map `oxidize_pdf::error::Result` to our `PdfError`
   - Maintain API compatibility with existing error types

2. **Internal Adapters**
   - Create adapter layer between oxidize-pdf's object model and our needs
   - Implement reference resolution strategy
   - Handle in-memory PDF buffering and modification

3. **PDF File Handling**
   - Implement wrapper around `parser::PdfDocument` that supports updates
   - Create in-memory modification capability
   - Support round-trip: read → modify → write

### Phase 2: Field Reading

Implement field discovery and value extraction:

1. **Field Traversal**
   - Parse AcroForm dictionary from parsed PDF
   - Recursively traverse field hierarchies
   - Build fully qualified field names
   - Identify terminal (fillable) fields

2. **Value Extraction**
   - Read field values from PDF objects
   - Convert PDF primitives to `FieldValue` enum
   - Extract field metadata (type, flags, tooltip, default value)

3. **Public API**
   - Implement `AcroFormDocument::from_pdf()`
   - Implement `AcroFormDocument::from_bytes()`
   - Implement `fields()` method

### Phase 3: Field Writing

Implement form filling capabilities:

1. **Value Encoding**
   - Convert `FieldValue` to PDF primitives
   - Implement UTF-16BE encoding for text values
   - Handle boolean, choice, and integer values

2. **Object Updates**
   - Locate field objects by name
   - Update field value (V key)
   - Update page annotations if needed
   - Maintain consistency between field and widget annotations

3. **PDF Regeneration**
   - Serialize updated document
   - Generate valid PDF with updated field values
   - Implement `fill()` and `fill_and_save()` methods

### Phase 4: Testing & Validation

Comprehensive testing to ensure API parity:

1. **Unit Tests**
   - Field name resolution
   - Value type conversions
   - Primitive encoding/decoding

2. **Integration Tests**
   - Load and list fields
   - Fill and save forms
   - Round-trip in-memory operations
   - Edge cases (nonexistent fields, complex hierarchies)

3. **Compatibility Tests**
   - Test with existing test PDFs from acroform_files/
   - Verify output PDFs open correctly in PDF viewers
   - Validate field values persist after saving

### Phase 5: Documentation & Examples

1. **API Documentation**
   - Document all public types and methods
   - Include usage examples
   - Migration guide from old implementation

2. **Examples**
   - Port `simple_fill.rs` example
   - Port `in_memory_fill.rs` example
   - Add new examples demonstrating capabilities

## Implementation Challenges & Solutions

### Challenge 1: Object Reference Resolution

**Problem**: The forked `pdf` crate uses `RcRef<T>` and `Resolve` trait for indirect object references. oxidize-pdf may use a different model.

**Solution**:
- Investigate oxidize-pdf's object reference model (`ObjectReference`, object lookup APIs)
- Create adapter layer if needed to provide similar resolution semantics
- May need to maintain internal index/cache of resolved objects

### Challenge 2: In-Memory PDF Modification

**Problem**: The current implementation uses `CachedFile<Vec<u8>>` with an `Updater` trait for in-place modifications. oxidize-pdf may not support this pattern.

**Solution**:
- Option A: Parse entire document, modify object tree, regenerate PDF
- Option B: Implement custom document wrapper that tracks modifications
- Option C: Use oxidize-pdf's `Document` type if it supports loading existing PDFs
- **Recommended**: Start with Option A (full regeneration) as it's simpler and aligns with "non-incremental" design goal

### Challenge 3: Annotation Updates

**Problem**: Current implementation updates both field dictionaries AND page annotations to ensure compatibility.

**Solution**:
- Understand oxidize-pdf's page annotation model
- Implement annotation lookup by field name
- Update both field and annotation in lockstep
- Test with various PDF viewers to ensure compatibility

### Challenge 4: Field Hierarchy Traversal

**Problem**: Need to recursively traverse field `/Kids` arrays to find all terminal fields.

**Solution**:
- Access field dictionaries from parsed AcroForm
- Implement recursive traversal algorithm
- Handle both terminal fields (with FT) and intermediate nodes
- Build fully qualified names during traversal

### Challenge 5: UTF-16BE Encoding

**Problem**: PDF text fields require UTF-16BE encoding with BOM for international characters.

**Solution**:
- Reuse existing encoding logic from `FieldValue::to_primitive()`
- Ensure oxidize-pdf's string types can handle raw byte sequences
- Test with international characters to verify correct encoding

## API Compatibility Matrix

| Feature | Current API | oxidize-pdf Mapping | Status |
|---------|------------|---------------------|--------|
| Load from file | `AcroFormDocument::from_pdf(path)` | `parser::PdfReader::open(path)` | ✅ Direct |
| Load from bytes | `AcroFormDocument::from_bytes(data)` | `parser::PdfReader::from_bytes(data)` | ✅ Direct |
| List fields | `doc.fields()` | Custom traversal of AcroForm | ⚠️ Custom |
| Fill fields | `doc.fill(values)` | Custom implementation | ⚠️ Custom |
| Save to file | `doc.fill_and_save(values, path)` | Document serialization + file I/O | ⚠️ Custom |
| Field types | `FieldType` enum | `forms::FieldType` or custom | ✅ Available |
| Field values | `FieldValue` enum | Custom implementation | ⚠️ Custom |
| Error handling | `PdfError` | `oxidize_pdf::error::Error` | ✅ Map |

**Legend:**
- ✅ Direct mapping available
- ⚠️ Requires custom implementation
- ❌ Not available, needs workaround

## Project Structure

```
acroform-oxidize/
├── Cargo.toml                 # Package manifest
├── src/
│   ├── lib.rs                # Public API exports, documentation
│   ├── api.rs                # AcroFormDocument implementation
│   ├── field.rs              # Field traversal and utilities
│   ├── value.rs              # FieldValue type and conversions
│   ├── error.rs              # Error type and conversions
│   └── internal/             # Internal implementation details
│       ├── mod.rs
│       ├── resolver.rs       # Object reference resolution
│       ├── updater.rs        # PDF modification logic
│       └── encoder.rs        # Value encoding utilities
├── examples/
│   ├── simple_fill.rs        # Basic form filling example
│   └── in_memory_fill.rs     # In-memory operations example
└── tests/
    ├── integration_test.rs   # Integration tests
    ├── field_test.rs         # Field traversal tests
    └── value_test.rs         # Value encoding tests
```

## Dependencies

```toml
[dependencies]
oxidize-pdf = "=1.6.0"

[dev-dependencies]
# Test dependencies as needed
```

## Testing Strategy

### Test Assets

Reuse existing test PDFs from `acroform_files/`:
- `af8.pdf` - PDF with text field for basic testing
- `af8_error.pdf` - Edge case testing
- Additional PDFs as needed for comprehensive coverage

### Test Coverage

1. **Field Discovery**
   - Single field PDFs
   - Multiple fields
   - Nested field hierarchies
   - Fields with various types (text, checkbox, choice)

2. **Value Operations**
   - Read existing values
   - Update values
   - Round-trip preservation
   - International characters (UTF-16BE)

3. **Edge Cases**
   - Nonexistent fields
   - Empty PDFs
   - PDFs without forms
   - Corrupted or malformed PDFs

4. **Performance**
   - Large forms (100+ fields)
   - Multiple round-trips
   - Memory usage for in-memory operations

## Success Criteria

The implementation will be considered complete when:

1. ✅ All public APIs from acroform-rs-old/acroform are implemented
2. ✅ All existing tests pass with the new implementation
3. ✅ Test PDFs can be loaded, read, filled, and saved successfully
4. ✅ Output PDFs display correctly in multiple PDF viewers (Adobe Reader, Chrome, Firefox)
5. ✅ Examples run without modification
6. ✅ Documentation is complete and accurate
7. ✅ No dependency on forked `pdf` crate
8. ✅ Code follows Rust best practices and is well-documented

## Migration Path

For users of the existing acroform crate:

1. **Import Changes**
   ```rust
   // Old
   use acroform::{AcroFormDocument, FieldValue};
   
   // New (same - no changes required!)
   use acroform::{AcroFormDocument, FieldValue};
   ```

2. **Code Changes**
   - **None required** - API remains identical
   - Error types may have different internal structure but same interface

3. **Dependency Changes**
   ```toml
   # Old
   [dependencies]
   acroform = { path = "acroform-rs-old/acroform" }
   
   # New
   [dependencies]
   acroform = { path = "acroform-oxidize" }
   # or from crates.io when published
   acroform = "1.0.0"
   ```

## Timeline Estimate

- **Phase 1** (Core Infrastructure): 2-3 days
- **Phase 2** (Field Reading): 2-3 days
- **Phase 3** (Field Writing): 3-4 days
- **Phase 4** (Testing & Validation): 2-3 days
- **Phase 5** (Documentation & Examples): 1-2 days

**Total: 10-15 days** for a complete, production-ready implementation with comprehensive testing and documentation.

## Open Questions

1. **Object Modification Strategy**: Does oxidize-pdf support incremental updates, or do we need full regeneration?
   - **Investigation needed**: Review `oxidize_pdf::writer` module and document serialization APIs

2. **Field Value Reading**: How to extract field values from parsed PDF objects?
   - **Investigation needed**: Test with sample PDFs to understand object structure

3. **Annotation Access**: How are page annotations represented and accessed?
   - **Investigation needed**: Review page parsing APIs and annotation structures

4. **Reference Resolution**: What's the oxidize-pdf equivalent of `Resolve` trait?
   - **Investigation needed**: Understand object reference model

5. **Memory Efficiency**: Can we achieve similar performance to the cached file approach?
   - **Investigation needed**: Profile memory usage and optimize if needed

## Next Steps

1. ✅ **Create this planning document** (ACROFORM_OXIDIZE_IMPL.md)
2. ⬜ **Prototype Phase 1**: Create basic project structure and test oxidize-pdf APIs
3. ⬜ **Spike Phase 2**: Implement basic field reading to validate approach
4. ⬜ **Decision Point**: Confirm implementation strategy based on prototype findings
5. ⬜ **Full Implementation**: Execute phases 2-5 based on validated approach

## References

- **oxidize-pdf Documentation**: https://docs.rs/oxidize-pdf/1.6.0
- **oxidize-pdf Repository**: https://github.com/bzsanti/oxidizePdf
- **PDF Specification**: ISO 32000-1 (PDF 1.7)
- **Current Implementation**: `acroform-rs-old/acroform/`
- **Test Assets**: `acroform-rs-old/acroform_files/`
