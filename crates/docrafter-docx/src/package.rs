//! OOXML ZIP package assembly.

use std::io::{Cursor, Write};

use docrafter_core::{Error, Result};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::comments::{build_comments_xml, DocxComment};
use crate::numbering::NUMBERING_XML;
use crate::styles::{SETTINGS_XML, STYLES_XML};
use docrafter_office::Image;

/// Image part prepared for embedding.
pub struct ImageRef {
    /// Relationship id referenced by `r:embed`.
    pub rel_id: String,
    /// Zip entry path (`word/media/imageN.ext`).
    pub media_path: String,
    /// Raw bytes.
    pub data: Vec<u8>,
    /// PNG or JPEG content type override.
    pub content_type: &'static str,
}

/// Collect image parts and assign relationship ids.
pub fn prepare_images(images: &[Image], first_rel_id: u32) -> Vec<ImageRef> {
    images
        .iter()
        .enumerate()
        .map(|(i, image)| {
            let (ext, content_type) = image_format(image.data());
            let media_path = format!("word/media/image{}.{}", i + 1, ext);
            let rel_id = format!("rId{}", first_rel_id + i as u32);
            ImageRef {
                rel_id,
                media_path,
                data: image.data().to_vec(),
                content_type,
            }
        })
        .collect()
}

/// Pack a complete `.docx` file.
pub fn pack_docx(
    document_xml: &str,
    include_numbering: bool,
    images: &[ImageRef],
    comments: &[DocxComment],
) -> Result<Vec<u8>> {
    let content_types = build_content_types(include_numbering, images, !comments.is_empty());
    let document_rels = build_document_rels(include_numbering, images, !comments.is_empty());
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    let mut buffer = Cursor::new(Vec::new());
    {
        let mut zip = ZipWriter::new(&mut buffer);
        write_str(&mut zip, "[Content_Types].xml", &content_types, options)?;
        write_str(&mut zip, "_rels/.rels", ROOT_RELS, options)?;
        write_str(&mut zip, "word/document.xml", document_xml, options)?;
        write_str(&mut zip, "word/styles.xml", STYLES_XML, options)?;
        write_str(&mut zip, "word/settings.xml", SETTINGS_XML, options)?;
        write_str(
            &mut zip,
            "word/_rels/document.xml.rels",
            &document_rels,
            options,
        )?;
        write_str(&mut zip, "docProps/core.xml", CORE_PROPS, options)?;
        write_str(&mut zip, "docProps/app.xml", APP_PROPS, options)?;
        if include_numbering {
            write_str(&mut zip, "word/numbering.xml", NUMBERING_XML, options)?;
        }
        for image in images {
            write_bytes(&mut zip, &image.media_path, &image.data, options)?;
        }
        if !comments.is_empty() {
            write_str(
                &mut zip,
                "word/comments.xml",
                &build_comments_xml(comments),
                options,
            )?;
        }
        zip.finish()
            .map_err(|e| Error::Docx(format!("failed to finalize docx zip: {e}")))?;
    }
    Ok(buffer.into_inner())
}

const ROOT_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/>
<Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties" Target="docProps/app.xml"/>
</Relationships>
"#;

const CORE_PROPS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/" xmlns:dcmitype="http://purl.org/dc/dcmitype/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
<dc:creator>docrafter</dc:creator>
<cp:lastModifiedBy>docrafter</cp:lastModifiedBy>
</cp:coreProperties>
"#;

const APP_PROPS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties" xmlns:vt="http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes">
<Application>docrafter</Application>
</Properties>
"#;

fn build_content_types(
    include_numbering: bool,
    images: &[ImageRef],
    include_comments: bool,
) -> String {
    let mut out = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
<Default Extension="xml" ContentType="application/xml"/>
<Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
<Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>
<Override PartName="/word/settings.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.settings+xml"/>
<Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/>
<Override PartName="/docProps/app.xml" ContentType="application/vnd.openxmlformats-officedocument.extended-properties+xml"/>
"#,
    );
    if include_numbering {
        out.push_str(r#"<Override PartName="/word/numbering.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml"/>"#);
    }
    if include_comments {
        out.push_str(r#"<Override PartName="/word/comments.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml"/>"#);
    }
    for image in images {
        let ext = image.media_path.rsplit('.').next().unwrap_or("png");
        let default = match ext {
            "jpg" | "jpeg" => r#"<Default Extension="jpeg" ContentType="image/jpeg"/>"#,
            _ => r#"<Default Extension="png" ContentType="image/png"/>"#,
        };
        if !out.contains(default) {
            out.push_str(default);
        }
        out.push_str(&format!(
            r#"<Override PartName="/{}" ContentType="{}"/>"#,
            image.media_path, image.content_type
        ));
    }
    out.push_str("</Types>");
    out
}

fn build_document_rels(
    include_numbering: bool,
    images: &[ImageRef],
    include_comments: bool,
) -> String {
    let mut out = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
<Relationship Id="rIdSettings" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/settings" Target="settings.xml"/>
"#,
    );
    if include_numbering {
        out.push_str(
            r#"<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering" Target="numbering.xml"/>"#,
        );
    }
    for image in images {
        let target = image
            .media_path
            .strip_prefix("word/")
            .unwrap_or(&image.media_path);
        out.push_str(&format!(
            r#"<Relationship Id="{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="{}"/>"#,
            image.rel_id, target
        ));
    }
    if include_comments {
        out.push_str(
            r#"<Relationship Id="rIdComments" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments" Target="comments.xml"/>"#,
        );
    }
    out.push_str("</Relationships>");
    out
}

fn image_format(data: &[u8]) -> (&'static str, &'static str) {
    if data.starts_with(&[0xFF, 0xD8]) {
        ("jpeg", "image/jpeg")
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
        .map_err(|e| Error::Docx(format!("zip start {path}: {e}")))?;
    zip.write_all(data)
        .map_err(|e| Error::Docx(format!("zip write {path}: {e}")))?;
    Ok(())
}
