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
2. ✅ **Prototype Phase 1**: Create basic project structure and test oxidize-pdf APIs
3. 🔄 **Spike Phase 2**: Implement basic field reading to validate approach
4. ⬜ **Decision Point**: Confirm implementation strategy based on prototype findings
5. ⬜ **Full Implementation**: Execute phases 2-5 based on validated approach

## Implementation Findings (Steps 2-4)

### Step 2: Prototype Phase 1 - Basic Project Structure ✅

**Created:**
- Basic Cargo project with `oxidize-pdf = "=1.6.0"` dependency
- Module structure: `lib.rs`, `api.rs`, `field.rs`, `value.rs`, `error.rs`
- Error type conversions from oxidize-pdf errors
- Basic API stubs for `AcroFormDocument`

**Key Findings:**
1. **oxidize-pdf Error Types**: The library uses two error types:
   - `oxidize_pdf::error::PdfError` - Main error type
   - `oxidize_pdf::parser::ParseError` - Parser-specific errors
   - Both needed conversion implementations to our `PdfError`

2. **PDF Loading**: Successfully implemented using:
   ```rust
   let cursor = Cursor::new(data);
   let reader = PdfReader::new(cursor)?;
   let document = PdfDocument::new(reader);
   ```

3. **Basic Information Access**: Can retrieve:
   - PDF version
   - Page count
   - Document metadata

### Step 3: Spike Phase 2 - Field Reading Investigation 🔄

**Status:** Initial exploration complete, implementation strategy identified.

**Critical Finding - API Limitations:**

The `oxidize-pdf` library (v1.6.0) is primarily designed for **PDF generation** with basic **parsing capabilities for text extraction**. However, it has significant limitations for **form field manipulation**:

1. **Parser Module Access**: 
   - The `PdfReader` and `PdfDocument` types provide high-level document access
   - Internal object structure (dictionaries, references) is not fully exposed in public API
   - The `catalog()` method exists on `PdfReader` but returns `&PdfDictionary`
   - Dictionary access methods are limited or internal

2. **Forms Module Focus**:
   - The `forms` module in oxidize-pdf is focused on **creating new forms**
   - Types like `AcroForm`, `FormField`, `FormManager` are for generation, not reading
   - No clear API for reading existing form field values from parsed PDFs

3. **Object Model Differences**:
   - oxidize-pdf uses `PdfObject`, `PdfDictionary`, `PdfArray` types
   - Reference resolution through `document.get_object(obj_num, gen_num)`
   - Different from the forked `pdf` crate's `RcRef<T>` and `Resolve` trait

### Step 4: Decision Point - Implementation Strategy Recommendation 🎯

**CRITICAL ASSESSMENT:**

After prototyping with oxidize-pdf v1.6.0, I've identified that while the library can **parse PDFs**, it lacks the necessary **high-level API for reading form fields** from existing PDFs. The library is excellent for:
- ✅ PDF generation
- ✅ Text extraction
- ✅ Basic parsing
- ❌ Form field reading/manipulation

**RECOMMENDED PATH FORWARD:**

Given the constraints, there are three viable approaches:

#### Option A: Extend oxidize-pdf with Low-Level Object Access (RECOMMENDED)
**Approach:** 
- Use oxidize-pdf for basic PDF parsing and structure navigation
- Implement custom form field extraction by directly accessing PDF object streams
- Build field traversal logic on top of `PdfReader.get_object()` and `PdfDocument.resolve()`

**Pros:**
- Leverages oxidize-pdf's robust PDF parsing
- Full control over form field extraction
- Can achieve API parity with old implementation
- Modern, maintained library as foundation

**Cons:**
- More implementation work (custom field traversal)
- Need to understand PDF specification deeply
- May need to work with internal/private API details

**Estimated Effort:** 7-10 days

#### Option B: Fork/Patch oxidize-pdf with Form Reading Support
**Approach:**
- Contribute form field reading capabilities to oxidize-pdf
- Create PR with public API for accessing form fields
- Use patched version until merged

**Pros:**
- Benefits the wider community
- Clean, well-designed API
- Leverages oxidize-pdf expertise

**Cons:**
- Dependency on maintainer approval
- Longer timeline (community contribution process)
- May not align with library's design goals

**Estimated Effort:** 10-15 days + upstream contribution time

#### Option C: Continue with Forked pdf Crate
**Approach:**
- Keep using `acroform-rs-old/pdf` (the forked crate)
- Focus on other improvements instead

**Pros:**
- Known working solution
- No implementation risk
- Immediate availability

**Cons:**
- ❌ Doesn't achieve migration goal
- ❌ Maintains dependency on forked/unmaintained code
- ❌ No benefit from oxidize-pdf improvements

**Estimated Effort:** 0 days (no change)

### Recommended Implementation Strategy: Option A

**Detailed Plan:**

1. **Phase 1: Low-Level Object Access (2 days)**
   - Implement wrapper methods to access `PdfReader.catalog()`
   - Create utility functions for dictionary traversal
   - Build reference resolution helpers

2. **Phase 2: Field Discovery (3 days)**
   - Parse AcroForm dictionary from catalog
   - Traverse Fields array
   - Recursively process field hierarchies (Kids)
   - Build fully qualified field names

3. **Phase 3: Value Extraction (2 days)**
   - Extract field properties (T, FT, V, DV, Ff, TU)
   - Convert PDF objects to `FieldValue` enum
   - Map field types to `FieldType` enum
   - Handle special cases (nested fields, annotations)

4. **Phase 4: Form Filling (2-3 days)**
   - Implement document cloning/modification strategy
   - Update field values in PDF object tree
   - Generate updated PDF with modified fields
   - Handle UTF-16BE encoding for text values

5. **Phase 5: Testing & Refinement (2 days)**
   - Test with sample PDFs from `acroform_files/`
   - Validate API parity with old implementation
   - Performance testing
   - Documentation

**Total Estimated Effort:** 11-12 days

### Key Technical Insights

1. **oxidize-pdf v1.6.0 Capabilities:**
   - ✅ Robust PDF parsing with corruption recovery
   - ✅ Access to document structure (catalog, pages, objects)
   - ✅ Object reference resolution
   - ⚠️ Limited public API for dictionary/object manipulation
   - ❌ No high-level form field reading API

2. **Implementation Challenges:**
   - Need to work with lower-level object APIs
   - Dictionary access may require workarounds
   - Field value encoding/decoding needs custom implementation
   - Form filling requires PDF regeneration (no incremental updates)

3. **Success Criteria:**
   - Can load PDFs with oxidize-pdf ✅
   - Can parse basic document info ✅
   - Can access internal objects ⚠️ (needs custom implementation)
   - Can achieve API parity ⚠️ (feasible but requires work)

### Conclusion

The migration to oxidize-pdf v1.6.0 is **feasible** but will require **custom implementation** of form field reading/writing on top of the library's parsing foundation. The library provides excellent PDF parsing infrastructure but doesn't have ready-made form manipulation APIs.

**Recommendation:** Proceed with Option A - implement form field extraction using oxidize-pdf's low-level object access, supplemented with custom traversal logic. This achieves the migration goal while leveraging a modern, maintained library.

## References

- **oxidize-pdf Documentation**: https://docs.rs/oxidize-pdf/1.6.0
- **oxidize-pdf Repository**: https://github.com/bzsanti/oxidizePdf
- **PDF Specification**: ISO 32000-1 (PDF 1.7)
- **Current Implementation**: `acroform-rs-old/acroform/`
- **Test Assets**: `acroform-rs-old/acroform_files/`
- **Prototype Code**: `acroform-oxidize/` (created during investigation)
