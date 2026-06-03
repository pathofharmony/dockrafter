//! Image embedding via `Image::from_path`.

use docrafter::prelude::*;
use docrafter_testing::assert_pdf_structure;
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

#[test]
fn image_from_path_renders_in_pdf() {
    let logo = fixture("logo.png");
    let mut doc = PdfDocument::new();
    doc.push(Paragraph::new("Logo below"));
    doc.push(
        Image::from_path(&logo)
            .expect("fixture logo.png")
            .size(24.0, 24.0),
    );

    let bytes = doc.to_bytes().unwrap();
    assert_pdf_structure(&bytes, 1, &["Logo below"]);
    assert!(bytes.windows(4).any(|w| w == b"/Im1"));
}
