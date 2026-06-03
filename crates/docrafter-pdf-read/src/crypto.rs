//! Encrypted PDF support (decrypt with password).

use docrafter_core::{Error, Result};
use lopdf::Document;

/// Load bytes and decrypt when the file is encrypted.
pub fn load_mem_decrypt(bytes: &[u8], password: &str) -> Result<Document> {
    let mut doc = Document::load_mem(bytes).map_err(|e| Error::Pdf(e.to_string()))?;
    decrypt_if_needed(&mut doc, password)?;
    Ok(doc)
}

/// Load from path and decrypt when needed.
pub fn load_decrypt(path: &std::path::Path, password: &str) -> Result<Document> {
    let mut doc = Document::load(path)
        .map_err(|e| Error::Pdf(format!("failed to load {}: {e}", path.display())))?;
    decrypt_if_needed(&mut doc, password)?;
    Ok(doc)
}

/// Whether the document uses encryption.
#[must_use]
pub fn is_encrypted(doc: &Document) -> bool {
    doc.is_encrypted()
}

fn decrypt_if_needed(doc: &mut Document, password: &str) -> Result<()> {
    if doc.is_encrypted() {
        doc.decrypt(password)
            .map_err(|e| Error::Pdf(format!("decryption failed: {e}")))?;
    }
    Ok(())
}
