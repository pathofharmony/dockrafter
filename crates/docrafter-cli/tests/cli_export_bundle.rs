//! CLI `export --bundle` test.

use std::process::Command;

use docrafter::export::{export_save, OutputFormat};
use docrafter::office::{OfficeDocument, Paragraph};

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_docrafter"))
}

#[test]
fn export_bundle_writes_three_formats() {
    let dir = std::env::temp_dir().join("docrafter_cli_bundle");
    let _ = std::fs::create_dir_all(&dir);

    let docx = dir.join("src.docx");
    let stem = dir.join("report");

    let mut doc = OfficeDocument::new();
    doc.push(Paragraph::new("Bundle"));
    export_save(&doc, &docx, OutputFormat::Docx).unwrap();

    let status = bin()
        .args([
            "export",
            docx.to_str().unwrap(),
            "-o",
            stem.to_str().unwrap(),
            "--bundle",
        ])
        .status()
        .unwrap();
    assert!(status.success());
    assert!(dir.join("report.pdf").exists());
    assert!(dir.join("report.docx").exists());
    assert!(dir.join("report.odt").exists());

    let _ = std::fs::remove_dir_all(dir);
}
