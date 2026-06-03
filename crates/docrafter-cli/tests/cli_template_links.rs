//! CLI: template render, pdf links.

use std::process::Command;

use docrafter::pdf::{Paragraph, PdfDocument, PdfReader};

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_docrafter"))
}

#[test]
fn template_render_bundle() {
    let spec = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/template-spec.json"
    );
    let out_dir = std::env::temp_dir().join("docrafter_cli_template");
    let _ = std::fs::create_dir_all(&out_dir);
    let stem = out_dir.join("cli_report");

    let status = bin()
        .args([
            "template",
            "render",
            spec,
            "-o",
            stem.to_str().unwrap(),
            "--bundle",
        ])
        .status()
        .unwrap();
    assert!(status.success());

    assert!(stem.with_extension("pdf").is_file());
    assert!(stem.with_extension("docx").is_file());
    assert!(stem.with_extension("odt").is_file());
}

#[test]
fn pdf_links_lists_uri() {
    let path = std::env::temp_dir().join("docrafter_cli_links.pdf");
    let mut doc = PdfDocument::new();
    doc.push(Paragraph::new("link test"));
    doc.save(&path).unwrap();

    let mut reader = PdfReader::open(&path).unwrap();
    reader
        .add_link(1, [72.0, 72.0, 200.0, 100.0], "https://example.com")
        .unwrap();
    reader.save(&path).unwrap();

    let output = bin()
        .args(["pdf", "links", path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("https://example.com"));

    let _ = std::fs::remove_file(path);
}
