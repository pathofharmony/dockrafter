//! CLI batch convert.

use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_docrafter"))
}

#[test]
fn batch_html_to_pdf() {
    let spec = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/sample.html");
    let out_dir = std::env::temp_dir().join("docrafter_cli_batch");
    let _ = std::fs::remove_dir_all(&out_dir);
    let _ = std::fs::create_dir_all(&out_dir);

    let status = bin()
        .args([
            "batch",
            spec,
            "-o",
            out_dir.to_str().unwrap(),
            "--to",
            "pdf",
        ])
        .status()
        .unwrap();
    assert!(status.success());

    let pdf = out_dir.join("sample.pdf");
    assert!(pdf.is_file());

    let _ = std::fs::remove_dir_all(out_dir);
}
