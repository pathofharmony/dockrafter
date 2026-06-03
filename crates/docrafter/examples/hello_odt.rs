//! Minimal ODT example (LibreOffice / OpenDocument).

use docrafter::odt::{OdtDocument, Paragraph};
use docrafter::prelude::Style;

fn main() -> docrafter::Result<()> {
    let mut doc = OdtDocument::new();
    doc.push(Paragraph::new("Hello, LibreOffice!").style(Style::new().font_size(14.0)));
    doc.save("hello.odt")?;
    eprintln!("Wrote hello.odt — open with LibreOffice Writer");
    Ok(())
}
