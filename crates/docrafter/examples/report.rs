//! Monthly report example (Phase 0.1).

use docrafter::prelude::*;

fn main() -> Result<()> {
    let mut doc = PdfDocument::new();

    doc.push(
        Paragraph::new("Отчёт за май")
            .align(Alignment::Center)
            .style(Style::heading1().color_value(Color::from_hex("#1e40af")?)),
    );

    doc.push(Spacer::new(Length::pt(12.0)));

    doc.push(
        Table::professional()
            .columns(["Сотрудник", "Проекты", "Часы"])
            .row(["Анна", "CRM, Dashboard", "142"])
            .row(["Игорь", "API", "98"]),
    );

    doc.save("report.pdf")?;
    println!("Wrote report.pdf");
    Ok(())
}
