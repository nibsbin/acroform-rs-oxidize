# acroform-rs High-Level Plan

## Overview

`acroform-rs` is a minimal, auditable PDF form manipulation library forked from `pdf-rs`. It focuses on logical correctness and compatibility with standard PDF viewers for appearance regeneration, not on visual rendering or incremental updates.

## Goals

- New `acroform` crate that depends on forked code
- Simple three-step API: `from_pdf`, `fields`, and `fill_and_save`
- Enable updating field values (`/V` and `/AS` entries) of both AcroForm fields and widget annotations
- Add traversal and serialization utilities for AcroForm dictionaries
- Implement basic non-incremental writer for generating new PDFs
- Maintain KISS principle: opinionated and minimal

## Current State

The repository already has:
- PDF parsing and reading (`pdf/src/file.rs`, `pdf/src/parser/`)
- Object model including `Catalog`, `InteractiveFormDictionary`, `FieldDictionary` (`pdf/src/object/types.rs`)
- Basic write capability with `File::save_to()` and `Storage::save()` (`pdf/src/file.rs`)
- Field update example in `examples/src/bin/form.rs`
- Serialization traits: `Object`, `ObjectWrite`

## Architecture

### 1. Core PDF Infrastructure (Forked Code - DO NOT MODIFY)
- **pdf/src/file.rs**: `File`, `Storage` - PDF file management
- **pdf/src/object/types.rs**: Type definitions for PDF objects
- **pdf/src/primitive.rs**: Low-level PDF primitives
- **pdf/src/xref.rs**: Cross-reference table management

**Strategy**: Use these modules as-is through their public APIs. Do not modify forked code.

### 2. AcroForm Domain Layer (New Separate Crate)

Create a new `acroform/` crate that depends on the `pdf` crate. This approach:
- Keeps all acroform-specific code separate from forked pdf-rs code
- Makes it easy to merge upstream pdf-rs updates
- Provides clean separation of concerns

#### Field Traversal Utilities
Location: `acroform/src/field.rs` (new file in new crate)

```rust
// Extension traits to add acroform functionality without modifying forked code
pub trait FieldDictionaryExt {
    fn traverse_fields(&self, resolver: &impl Resolve) -> Vec<&FieldDictionary>;
    fn get_full_name(&self, resolver: &impl Resolve) -> String;
}

impl FieldDictionaryExt for FieldDictionary {
    // Implementation using only public APIs from pdf crate
}

pub trait InteractiveFormDictionaryExt {
    fn all_fields(&self, resolver: &impl Resolve) -> Vec<&FieldDictionary>;
    fn find_field_by_name(&self, name: &str, resolver: &impl Resolve) -> Option<&FieldDictionary>;
}

impl InteractiveFormDictionaryExt for InteractiveFormDictionary {
    // Implementation using only public APIs from pdf crate
}
```

#### Field Value Updates
Handle field updates in the acroform crate using the existing public APIs from the pdf crate.

### 3. High-Level API (New Separate Crate)
Location: `acroform/src/api.rs` and `acroform/src/lib.rs`

```rust
pub struct AcroFormDocument {
    file: File<Vec<u8>, OC, SC, L>,
}

impl AcroFormDocument {
    // Step 1: Load PDF
    pub fn from_pdf(path: impl AsRef<Path>) -> Result<Self>
    
    // Step 2: Get fields
    pub fn fields(&self) -> Vec<FormField>  // High-level field representation
    
    // Step 3: Fill and save
    pub fn fill_and_save(&mut self, values: HashMap<String, FieldValue>, output: impl AsRef<Path>) -> Result<()>
}

pub struct FormField {
    pub name: String,
    pub field_type: FieldType,
    pub current_value: Option<FieldValue>,
    pub flags: u32,
    // Reference for updating
    reference: PlainRef,
}

pub enum FieldValue {
    Text(String),
    Boolean(bool),     // For checkboxes
    Choice(String),    // For radio buttons and dropdowns
    Integer(i32),
}
```

### 4. Non-Incremental Writer (Already Present in Forked Code)
The existing `Storage::save()` already implements non-incremental writing:
- Collects all modified objects in `changes: HashMap<ObjNr, (Primitive, GenNr)>`
- Writes all objects sequentially
- Creates new xref table
- Writes new trailer

**Key**: Set `/NeedAppearances true` in the AcroForm dictionary so PDF viewers regenerate appearances.

### 5. Necessary Modifications to Forked Code (If Any)

**IMPORTANT**: We will attempt to implement everything using extension traits and wrapper types first. Only if absolutely necessary will we modify forked code.

## Implementation Steps

### Phase 1: Field Traversal (New Acroform Crate)
1. Create new `acroform/` crate with `Cargo.toml` depending on `pdf` crate
2. Create extension traits for `FieldDictionary` and `InteractiveFormDictionary`
3. Implement field traversal methods using only public APIs from pdf crate
4. Implement full name resolution (concatenating parent `/T` values with periods)
5. Write unit tests for traversal in acroform crate
6. **Check if `/AS` field exists in `FieldDictionary`** - document findings

### Phase 2: Field Update Helpers (Acroform Crate)
1. Create safe update methods in acroform crate wrapping pdf crate APIs
2. Handle type conversions (String → PdfString, bool → Name, etc.)
3. Validate field types match update operations
4. Write unit tests for updates

### Phase 3: High-Level API (Acroform Crate)
1. Create `AcroFormDocument` struct in acroform crate wrapping `File` from pdf crate
2. Implement `from_pdf()` using existing `FileOptions::cached().open()` from pdf crate
3. Implement `fields()` returning high-level `FormField` structs
4. Implement `fill_and_save()`:
   - Update field values using `File::update()` from pdf crate
   - Set `/NeedAppearances` to `true` in AcroForm dictionary
   - Call `File::save_to()` from pdf crate
5. Add comprehensive integration tests with real PDF forms

### Phase 4: Documentation and Examples
1. Update README with API usage examples showing the acroform crate
2. Create example program demonstrating the three-step API using acroform crate
3. Document field types and their value formats
4. Add notes about viewer compatibility (NeedAppearances flag)
5. Document any modifications made to forked code (if any)

## Key Design Decisions

### 1. NeedAppearances Strategy
**Decision**: Set `/NeedAppearances true` instead of generating appearance streams.
**Rationale**: 
- Appearance generation requires font metrics, text layout, color management
- Standard PDF viewers already do this correctly
- Keeps library minimal and focused
- Reduces attack surface

### 2. Non-Incremental Writing Only
**Decision**: Only support full PDF rewrite, not incremental updates.
**Rationale**:
- Simpler implementation
- Smaller code surface for security audits
- Sufficient for most form-filling use cases
- Existing `Storage::save()` already supports this

### 3. Value Type Safety
**Decision**: Provide enum-based API for field values.
**Rationale**:
- Type safety at API level
- Prevents mismatched value types
- Clear documentation of supported types

### 4. Field Hierarchy Flattening
**Decision**: Present flat list of fields to user, handle hierarchy internally.
**Rationale**:
- Simpler API
- Most use cases work with flat field names
- Full names naturally disambiguate nested fields

## Testing Strategy

1. **Unit Tests**: Field traversal, value conversion, type checking
2. **Integration Tests**: Load sample PDFs, fill fields, verify output
3. **Real-World PDFs**: Test with IRS forms, W-9, job applications
4. **Viewer Testing**: Open generated PDFs in multiple viewers
5. **Example Files**: `acroform_files/af8.pdf` contains a PDF with a single field filled out with "OLD_VALUE". Try to change this to "NEW_VALUE" using the library. Use `qpdf --stream-data=uncompress <input.pdf> <output.txt>` to inspect the raw PDF content before and after.

## Success Criteria

1. ✅ Can load PDF with AcroForm
2. ✅ Can list all fillable fields with names and types
3. ✅ Can update text field values
4. ✅ Can update checkbox/radio button states
5. ✅ Can save modified PDF that opens correctly in standard viewers
6. ✅ Generated PDFs show updated values when opened

## Non-Goals

- PDF rendering or visual preview
- Incremental updates (linearized PDFs)
- Appearance stream generation
- Digital signature creation/validation
- XFA form support
- Interactive JavaScript evaluation
- PDF creation from scratch (only modification)
- Form field creation/deletion (only value updates)

**Note on Forked Code**: The `pdf/` directory contains forked code from `pdf-rs`. To maintain the ability to merge future updates from the upstream repository, we will:
- **Avoid modifying forked files wherever possible**
- **Create new modules in a separate `acroform/` crate** for all acroform-specific functionality
- **Only modify forked files if absolutely essential** (e.g., adding a missing field to an existing struct)
- **Document all modifications to forked files explicitly** in this plan and in code comments

## Dependencies

Current dependencies are acceptable. No new major dependencies needed:
- **`acroform-pdf` crate** (forked from pdf-rs, located in `pdf/` directory, published to crates.io) - base functionality
- **`acroform` crate** (new, located in `acroform/` directory) - depends on `acroform-pdf` crate
- Standard Rust std library
- Existing dependencies for PDF parsing (already vetted)