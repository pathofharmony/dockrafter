//! Testing utilities for golden PDF comparison and structural assertions.

#![deny(missing_docs)]

mod docx;
mod odt;
mod pdf;

pub use docx::{
    assert_docx_snapshot_file, assert_docx_structure, docx_fingerprint, normalize_docx_bytes,
    DocxStructure,
};
pub use odt::{
    assert_odt_snapshot_file, assert_odt_structure, normalize_odt_bytes, odt_fingerprint,
    OdtStructure,
};
pub use pdf::{
    assert_pdf_snapshot, assert_pdf_snapshot_file, assert_pdf_structure, normalize_pdf_bytes,
    pdf_fingerprint, PdfStructure,
};
