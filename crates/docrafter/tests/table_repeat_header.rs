//! Table header repeat across pages.

use docrafter::prelude::*;
use docrafter_testing::assert_pdf_structure;

#[test]
fn table_repeats_header_on_page_two() {
    let mut doc = PdfDocument::new();
    let mut table = Table::professional().columns(["Name", "Score"]);
    for i in 0..60 {
        table = table.row([format!("Person {i}"), format!("{}", i * 10)]);
    }
    doc.push(table);

    let bytes = doc.to_bytes().unwrap();
    assert_pdf_structure(&bytes, 2, &["Name", "Person 0"]);
    // Header label appears on continuation page when repeat is enabled.
    let name_hits = bytes.windows(4).filter(|w| w == b"Name").count();
    assert!(
        name_hits >= 2,
        "expected header text on multiple pages, got {name_hits}"
    );
}
