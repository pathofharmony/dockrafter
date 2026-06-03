//! DOCX Phase 0.3: tables, runs, images, lists, styles roundtrip.

use docrafter::docx::{DocxBlock, DocxDocument, Image, List, Paragraph, Table};
use docrafter::prelude::Style;
use docrafter_testing::assert_docx_structure;
use std::path::PathBuf;
use zip::ZipArchive;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn zip_contains(bytes: &[u8], entry: &str) -> bool {
    let mut zip = ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
    let found = zip.by_name(entry).is_ok();
    found
}

#[test]
fn docx_includes_styles_and_numbering_parts() {
    let mut doc = DocxDocument::new();
    doc.push_list(List::new().item("One").item("Two"));
    let bytes = doc.to_bytes().unwrap();
    assert!(zip_contains(&bytes, "word/styles.xml"));
    assert!(zip_contains(&bytes, "word/numbering.xml"));
}

#[test]
fn comments_roundtrip_via_open() {
    let mut doc = DocxDocument::new();
    doc.push(Paragraph::new("Annotated"));
    doc.add_comment("Reviewer", "Fix typo");
    let bytes = doc.to_bytes().unwrap();
    let loaded = DocxDocument::from_bytes(&bytes).unwrap();
    assert_eq!(loaded.comments().len(), 1);
    assert_eq!(loaded.comments()[0].author, "Reviewer");
    assert_eq!(loaded.comments()[0].text, "Fix typo");
    assert!(zip_contains(&bytes, "word/comments.xml"));
}

#[test]
fn multi_run_roundtrip() {
    let mut doc = DocxDocument::new();
    doc.push(
        Paragraph::new("Plain ")
            .run("bold", Style::new().bold())
            .run(" tail", Style::new().italic()),
    );
    let loaded = DocxDocument::from_bytes(&doc.to_bytes().unwrap()).unwrap();
    let DocxBlock::Paragraph(p) = &loaded.blocks()[0] else {
        panic!("expected paragraph");
    };
    assert_eq!(p.text(), "Plain bold tail");
    assert_eq!(p.runs().len(), 3);
}

#[test]
fn table_and_paragraph_order_roundtrip() {
    let mut doc = DocxDocument::new();
    doc.push(Paragraph::new("Intro"));
    doc.push_table(Table::professional().columns(["X", "Y"]).row(["1", "2"]));
    doc.push(Paragraph::new("Outro"));
    let loaded = DocxDocument::from_bytes(&doc.to_bytes().unwrap()).unwrap();
    assert_eq!(loaded.blocks().len(), 3);
    assert!(matches!(&loaded.blocks()[1], DocxBlock::Table(_)));
}

#[test]
fn image_embedded_in_docx() {
    let path = fixture("logo.png");
    let mut doc = DocxDocument::new();
    doc.push_image(Image::from_path(&path).unwrap().size(48.0, 48.0));
    let bytes = doc.to_bytes().unwrap();
    assert!(zip_contains(&bytes, "word/media/image1.png"));
    let loaded = DocxDocument::from_bytes(&bytes).unwrap();
    assert!(matches!(&loaded.blocks()[0], DocxBlock::Image(img) if !img.data().is_empty()));
}

#[test]
fn list_roundtrip() {
    let mut doc = DocxDocument::new();
    doc.push_list(List::new().item("Alpha").item("Beta"));
    let bytes = doc.to_bytes().unwrap();
    assert_docx_structure(&bytes, &["Alpha", "Beta"]);
    let loaded = DocxDocument::from_bytes(&bytes).unwrap();
    match &loaded.blocks()[0] {
        DocxBlock::List(list) => assert_eq!(list.items(), &["Alpha", "Beta"]),
        _ => panic!("expected list"),
    }
}
