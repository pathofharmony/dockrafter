//! Encrypt PDF on save (PDF 1.4 revision 2, 40-bit RC4 — compatible with `lopdf` decrypt).

use docrafter_core::{Error, Result};
use lopdf::{Dictionary, Document, Object, ObjectId};
const PAD_BYTES: [u8; 32] = [
    0x28, 0xBF, 0x4E, 0x5E, 0x4E, 0x75, 0x8A, 0x41, 0x64, 0x00, 0x4E, 0x56, 0xFF, 0xFA, 0x01, 0x08,
    0x2E, 0x2E, 0x00, 0xB6, 0xD0, 0x68, 0x3E, 0x80, 0x2F, 0x0C, 0xA9, 0xFE, 0x64, 0x53, 0x69, 0x7A,
];

/// Password protection applied before [`Document::save`](lopdf::Document::save).
#[derive(Debug, Clone)]
pub struct EncryptOptions {
    /// Password required to open the document.
    pub user_password: String,
    /// Owner password (defaults to `user_password` when unset).
    pub owner_password: Option<String>,
    /// Permission flags (`/P`); default allows print and copy.
    pub permissions: i32,
}

impl Default for EncryptOptions {
    fn default() -> Self {
        Self {
            user_password: String::new(),
            owner_password: None,
            permissions: -3904,
        }
    }
}

impl EncryptOptions {
    /// New options with user password.
    #[must_use]
    pub fn user(password: impl Into<String>) -> Self {
        Self {
            user_password: password.into(),
            ..Self::default()
        }
    }
}

/// Apply standard security handler (revision 2) and encrypt strings/streams in place.
pub fn encrypt_document(doc: &mut Document, options: &EncryptOptions) -> Result<()> {
    if options.user_password.is_empty() {
        return Err(Error::InvalidInput(
            "encryption password must not be empty".into(),
        ));
    }
    if doc.is_encrypted() {
        return Err(Error::Pdf("document is already encrypted".into()));
    }

    let owner_password = options
        .owner_password
        .as_deref()
        .unwrap_or(options.user_password.as_str());

    let file_id: Vec<u8> = (0..16).map(|i| ((i * 37 + 91) % 256) as u8).collect();
    let o_value = compute_o_value(owner_password, 2)?;

    let encrypt_id = doc.new_object_id();
    let mut encrypt_dict = Dictionary::new();
    encrypt_dict.set("Filter", Object::Name(b"Standard".to_vec()));
    encrypt_dict.set("V", Object::Integer(1));
    encrypt_dict.set("R", Object::Integer(2));
    encrypt_dict.set("Length", Object::Integer(40));
    encrypt_dict.set("O", Object::String(o_value, lopdf::StringFormat::Literal));
    encrypt_dict.set("P", Object::Integer(i64::from(options.permissions)));
    doc.objects
        .insert(encrypt_id, Object::Dictionary(encrypt_dict));

    doc.trailer.set(
        "ID",
        Object::Array(vec![
            Object::String(file_id.clone(), lopdf::StringFormat::Literal),
            Object::String(file_id, lopdf::StringFormat::Literal),
        ]),
    );
    doc.trailer.set("Encrypt", Object::Reference(encrypt_id));

    let key = lopdf::encryption::get_encryption_key(doc, options.user_password.as_str(), false)
        .map_err(|e| Error::Pdf(format!("derive encryption key: {e}")))?;
    let u_value = rc4_encrypt(&key, &PAD_BYTES);
    if let Some(Object::Dictionary(dict)) = doc.objects.get_mut(&encrypt_id) {
        dict.set("U", Object::String(u_value, lopdf::StringFormat::Literal));
    }

    let ids: Vec<ObjectId> = doc.objects.keys().copied().collect();
    for id in ids {
        if id == encrypt_id {
            continue;
        }
        let obj = doc
            .objects
            .get_mut(&id)
            .ok_or_else(|| Error::Pdf("missing object".into()))?;
        encrypt_object(&key, id, obj)?;
    }
    Ok(())
}

fn pad_password(password: &str) -> [u8; 32] {
    let pad = PAD_BYTES;
    let mut out = [0u8; 32];
    let len = password.len().min(32);
    out[..len].copy_from_slice(&password.as_bytes()[..len]);
    out[len..].copy_from_slice(&pad[len..]);
    out
}

fn compute_o_value(owner_password: &str, revision: i64) -> Result<Vec<u8>> {
    let mut digest = md5::compute(pad_password(owner_password)).to_vec();
    if revision >= 3 {
        for _ in 0..50 {
            digest = md5::compute(&digest).to_vec();
        }
    }
    let key_len = 5;
    let rc4_key = &digest[..key_len];
    Ok(rc4_encrypt(rc4_key, &pad_password(owner_password)))
}

fn encrypt_object(key: &[u8], obj_id: ObjectId, obj: &mut Object) -> Result<()> {
    match obj {
        Object::String(content, format) => {
            let encrypted = encrypt_bytes(key, obj_id, content);
            *content = encrypted;
            *format = lopdf::StringFormat::Hexadecimal;
        }
        Object::Stream(stream) => {
            let encrypted = encrypt_bytes(key, obj_id, &stream.content);
            stream.content = encrypted;
        }
        _ => {}
    }
    Ok(())
}

fn encrypt_bytes(key: &[u8], obj_id: ObjectId, plain: &[u8]) -> Vec<u8> {
    let rc4_key = object_rc4_key(key, obj_id);
    rc4_encrypt(&rc4_key, plain)
}

fn object_rc4_key(key: &[u8], obj_id: ObjectId) -> Vec<u8> {
    let mut builder = Vec::with_capacity(key.len() + 5);
    builder.extend_from_slice(key);
    builder.extend_from_slice(&obj_id.0.to_le_bytes()[..3]);
    builder.extend_from_slice(&obj_id.1.to_le_bytes()[..2]);
    let key_len = (key.len() + 5).min(16);
    md5::compute(&builder)[..key_len].to_vec()
}

fn rc4_encrypt(key: &[u8], data: &[u8]) -> Vec<u8> {
    assert!(!key.is_empty() && key.len() <= 256);
    let mut state = [0_u8; 256];
    for (i, v) in state.iter_mut().enumerate() {
        *v = i as u8;
    }
    let mut j = 0_u8;
    for i in 0..256 {
        j = j.wrapping_add(state[i]).wrapping_add(key[i % key.len()]);
        state.swap(i, j as usize);
    }
    let mut i = 0_u8;
    let mut j = 0_u8;
    let mut out = Vec::with_capacity(data.len());
    for &byte in data {
        i = i.wrapping_add(1);
        j = j.wrapping_add(state[i as usize]);
        state.swap(i as usize, j as usize);
        let k = state[(state[i as usize].wrapping_add(state[j as usize])) as usize];
        out.push(byte ^ k);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use lopdf::encryption::get_encryption_key;
    use lopdf::Document;

    #[test]
    fn encrypt_roundtrip_with_lopdf() {
        let mut doc = Document::new();
        let mut dict = Dictionary::new();
        dict.set(
            "Title",
            Object::String(b"Secret".to_vec(), lopdf::StringFormat::Literal),
        );
        doc.trailer.set("Info", Object::Dictionary(dict));

        encrypt_document(&mut doc, &EncryptOptions::user("test")).unwrap();
        assert!(doc.is_encrypted());
        assert!(get_encryption_key(&doc, "test", true).is_ok());

        let mut buf = Vec::new();
        doc.save_to(&mut buf).unwrap();
        let loaded = Document::load_mem(&buf).unwrap();
        assert!(loaded.is_encrypted());
        let mut decrypted = loaded;
        decrypted
            .decrypt("test")
            .expect("decrypt with user password");
        let info = decrypted.trailer.get(b"Info").unwrap();
        assert!(matches!(info, Object::Dictionary(_)));
    }
}
