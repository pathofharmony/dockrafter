//! Smoke tests for the `docrafter` binary.

use std::process::Command;

use docrafter::pdf::{Paragraph, PdfDocument};

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_docrafter"))
}

#[test]
fn help_exits_zero() {
    let status = bin().arg("--help").status().unwrap();
    assert!(status.success());
}

#[test]
fn pdf_pages_on_generated_file() {
    let path = std::env::temp_dir().join("docrafter_cli_pages.pdf");
    let mut doc = PdfDocument::new();
    doc.push(Paragraph::new("cli"));
    doc.save(&path).unwrap();

    let output = bin()
        .args(["pdf", "pages", path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output.stderr);
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "1");

    let _ = std::fs::remove_file(path);
}
