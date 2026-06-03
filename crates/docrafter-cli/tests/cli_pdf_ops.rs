//! CLI tests for rotate, split, watermark.

use std::process::Command;

use docrafter::pdf::{Paragraph, PdfDocument};

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_docrafter"))
}

#[test]
fn pdf_rotate_and_watermark() {
    let dir = std::env::temp_dir().join("docrafter_cli_pdf_ops");
    let _ = std::fs::create_dir_all(&dir);

    let input = dir.join("in.pdf");
    let rotated = dir.join("rot.pdf");
    let marked = dir.join("out.pdf");

    let mut doc = PdfDocument::new();
    doc.push(Paragraph::new("rotate me"));
    doc.save(&input).unwrap();

    let status = bin()
        .args([
            "pdf",
            "rotate",
            input.to_str().unwrap(),
            "-o",
            rotated.to_str().unwrap(),
            "--angle",
            "90",
        ])
        .status()
        .unwrap();
    assert!(status.success());

    let status = bin()
        .args([
            "pdf",
            "watermark",
            rotated.to_str().unwrap(),
            "-o",
            marked.to_str().unwrap(),
            "--text",
            "TEST",
        ])
        .status()
        .unwrap();
    assert!(status.success());
    assert!(marked.exists());

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn pdf_split_writes_pages() {
    let dir = std::env::temp_dir().join("docrafter_cli_split");
    let _ = std::fs::create_dir_all(&dir);

    let input = dir.join("two.pdf");
    let out_dir = dir.join("pages");

    let mut doc = PdfDocument::new();
    doc.push(Paragraph::new("one"));
    doc.push(docrafter::pdf::PageBreak);
    doc.push(Paragraph::new("two"));
    doc.save(&input).unwrap();

    let status = bin()
        .args([
            "pdf",
            "split",
            input.to_str().unwrap(),
            "-o",
            out_dir.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(status.success());
    assert!(out_dir.join("two_page_001.pdf").exists());
    assert!(out_dir.join("two_page_002.pdf").exists());

    let _ = std::fs::remove_dir_all(dir);
}
