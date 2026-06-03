//! Minimal DOCX example (Phase 0.3).

use docrafter::docx::{DocxDocument, Paragraph};
use docrafter::prelude::Style;

fn main() -> docrafter::Result<()> {
    let mut doc = DocxDocument::new();
    doc.push(Paragraph::new("Hello, docrafter!").style(Style::new().font_size(14.0)));
    doc.save("hello.docx")?;
    eprintln!("Wrote hello.docx");
    Ok(())
}
