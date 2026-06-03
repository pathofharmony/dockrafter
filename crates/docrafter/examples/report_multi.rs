//! Monthly report exported to PDF, DOCX, and ODT from one model.

use docrafter::office::{OfficeDocument, Paragraph, Table};
use docrafter::prelude::*;
use docrafter_core::Color;

fn main() -> Result<()> {
    let mut doc = OfficeDocument::new();

    doc.push(
        Paragraph::new("Отчёт за май")
            .align(Alignment::Center)
            .style(Style::heading1().color_value(Color::from_hex("#1e40af")?)),
    );

    doc.push_table(
        Table::professional()
            .columns(["Сотрудник", "Проекты", "Часы"])
            .row(["Анна", "CRM, Dashboard", "142"])
            .row(["Игорь", "API", "98"]),
    );

    export_save_auto(&doc, "report.pdf")?;
    export_save_auto(&doc, "report.docx")?;
    export_save_auto(&doc, "report.odt")?;

    println!("Wrote report.pdf, report.docx, report.odt");
    Ok(())
}
