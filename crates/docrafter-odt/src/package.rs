//! ODF ZIP package (mimetype first, uncompressed — required by LibreOffice).

use std::io::{Cursor, Write};

use docrafter_core::{Error, Result};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use docrafter_office::Image;

use crate::styles::STYLES_XML;

/// ODF package media type.
pub const MIMETYPE: &[u8] = b"application/vnd.oasis.opendocument.text";

/// Embedded image reference in `content.xml`.
pub struct ImageRef {
    /// `draw:name` attribute.
    pub draw_name: String,
    /// `xlink:href` (e.g. `Pictures/image1.png`).
    pub href: String,
    /// Zip path.
    pub media_path: String,
    /// Raw bytes.
    pub data: Vec<u8>,
}

/// Prepare image parts for `Pictures/`.
pub fn prepare_images(images: &[Image]) -> Vec<ImageRef> {
    images
        .iter()
        .enumerate()
        .map(|(i, image)| {
            let (ext, _) = image_format(image.data());
            let media_path = format!("Pictures/image{}.{}", i + 1, ext);
            ImageRef {
                draw_name: format!("Image{}", i + 1),
                href: media_path.clone(),
                media_path,
                data: image.data().to_vec(),
            }
        })
        .collect()
}

const META: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-meta xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0" xmlns:meta="urn:oasis:names:tc:opendocument:xmlns:meta:1.0" office:version="1.2">
<office:meta><meta:generator>docrafter</meta:generator></office:meta>
</office:document-meta>
"#;

/// Pack a complete `.odt` file.
pub fn pack_odt(content_xml: &str, images: &[ImageRef]) -> Result<Vec<u8>> {
    let manifest = build_manifest(images);
    let stored = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    let deflated =
        SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    let mut buffer = Cursor::new(Vec::new());
    {
        let mut zip = ZipWriter::new(&mut buffer);
        write_bytes(&mut zip, "mimetype", MIMETYPE, stored)?;
        write_str(&mut zip, "META-INF/manifest.xml", &manifest, deflated)?;
        write_str(&mut zip, "content.xml", content_xml, deflated)?;
        write_str(&mut zip, "styles.xml", STYLES_XML, deflated)?;
        write_str(&mut zip, "meta.xml", META, deflated)?;
        for image in images {
            write_bytes(&mut zip, &image.media_path, &image.data, deflated)?;
        }
        zip.finish()
            .map_err(|e| Error::Odt(format!("failed to finalize odt zip: {e}")))?;
    }
    Ok(buffer.into_inner())
}

fn build_manifest(images: &[ImageRef]) -> String {
    let mut out = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<manifest:manifest xmlns:manifest="urn:oasis:names:tc:opendocument:xmlns:manifest:1.0" manifest:version="1.2">
<manifest:file-entry manifest:full-path="/" manifest:version="1.2" manifest:media-type="application/vnd.oasis.opendocument.text"/>
<manifest:file-entry manifest:full-path="content.xml" manifest:media-type="text/xml"/>
<manifest:file-entry manifest:full-path="styles.xml" manifest:media-type="text/xml"/>
<manifest:file-entry manifest:full-path="meta.xml" manifest:media-type="text/xml"/>
"#,
    );
    for image in images {
        let media = if image.media_path.ends_with(".jpg") || image.media_path.ends_with(".jpeg") {
            "image/jpeg"
        } else {
            "image/png"
        };
        out.push_str(&format!(
            r#"<manifest:file-entry manifest:full-path="{}" manifest:media-type="{media}"/>"#,
            image.media_path
        ));
    }
    out.push_str("</manifest:manifest>");
    out
}

fn image_format(data: &[u8]) -> (&'static str, &'static str) {
    if data.starts_with(&[0xFF, 0xD8]) {
        ("jpg", "image/jpeg")
    } else {
        ("png", "image/png")
    }
}

fn write_str<W: Write + std::io::Seek>(
    zip: &mut ZipWriter<W>,
    path: &str,
    data: &str,
    options: SimpleFileOptions,
) -> Result<()> {
    write_bytes(zip, path, data.as_bytes(), options)
}

fn write_bytes<W: Write + std::io::Seek>(
    zip: &mut ZipWriter<W>,
    path: &str,
    data: &[u8],
    options: SimpleFileOptions,
) -> Result<()> {
    zip.start_file(path, options)
        .map_err(|e| Error::Odt(format!("zip start {path}: {e}")))?;
    zip.write_all(data)
        .map_err(|e| Error::Odt(format!("zip write {path}: {e}")))?;
    Ok(())
}
