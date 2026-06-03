//! ODT feature parity with DOCX.

use docrafter::odt::{Image, List, OdtBlock, OdtDocument, Paragraph, Table};
use docrafter::prelude::Style;
use docrafter_testing::assert_odt_structure;
use std::path::PathBuf;
use zip::ZipArchive;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn zip_has(bytes: &[u8], entry: &str) -> bool {
    let mut zip = ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
    let found = zip.by_name(entry).is_ok();
    found
}

#[test]
fn odt_has_odf_package_parts() {
    let bytes = OdtDocument::new()
        .push(Paragraph::new("ODF"))
        .to_bytes()
        .unwrap();
    assert!(zip_has(&bytes, "mimetype"));
    assert!(zip_has(&bytes, "META-INF/manifest.xml"));
    assert!(zip_has(&bytes, "content.xml"));
    assert!(zip_has(&bytes, "styles.xml"));
}

#[test]
fn odt_multi_run_and_table_roundtrip() {
    let mut doc = OdtDocument::new();
    doc.push(Paragraph::new("A ").run("B", Style::new().bold()));
    doc.push_table(Table::professional().columns(["X", "Y"]).row(["1", "2"]));
    let loaded = OdtDocument::from_bytes(&doc.to_bytes().unwrap()).unwrap();
    assert_eq!(loaded.blocks().len(), 2);
    assert!(matches!(&loaded.blocks()[1], OdtBlock::Table(t) if t.columns == ["X", "Y"]));
}

#[test]
fn odt_list_and_image() {
    let mut doc = OdtDocument::new();
    doc.push_list(List::new().item("One").item("Two"));
    doc.push_image(
        Image::from_path(fixture("logo.png"))
            .unwrap()
            .size(40.0, 40.0),
    );
    let bytes = doc.to_bytes().unwrap();
    assert_odt_structure(&bytes, &["One", "Two"]);
    assert!(zip_has(&bytes, "Pictures/image1.png"));
}
