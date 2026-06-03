//! Minimal PDF example (Phase 0).

use docrafter::prelude::*;

fn main() -> Result<()> {
    let mut doc = PdfDocument::new();
    doc.push(Paragraph::new("Hello, docrafter!"));
    doc.save("hello.pdf")?;
    println!("Wrote hello.pdf ({} paragraphs)", doc.len());
    Ok(())
}
