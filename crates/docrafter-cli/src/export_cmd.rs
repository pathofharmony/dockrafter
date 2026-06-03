//! `docrafter export …` and convert helpers.

use std::fs;
use std::path::Path;

use docrafter::export::{export_bundle, export_save, export_save_auto, OutputFormat};
use docrafter::html::html_to_office;
use docrafter::office::{self, OfficeDocument};
use docrafter::open_document::OpenDocument;
use docrafter::Result;

/// Load docx/odt/html into [`OfficeDocument`].
pub fn load_office(input: &Path) -> Result<OfficeDocument> {
    let ext = OpenDocument::extension_lower(input)?;
    match ext.as_str() {
        "html" | "htm" => {
            let html =
                fs::read_to_string(input).map_err(|e| docrafter_core::Error::io(input, e))?;
            html_to_office(&html)
        }
        "docx" | "odt" => office::open(input),
        other => Err(docrafter_core::Error::InvalidInput(format!(
            "export does not support .{other} (use docx, odt, or html)"
        ))),
    }
}

pub fn export_one(input: &Path, output: &Path) -> Result<()> {
    let doc = load_office(input)?;
    export_save_auto(&doc, output)
}

/// `export --bundle`: write `{stem}.pdf`, `{stem}.docx`, `{stem}.odt`.
pub fn export_with_bundle(input: &Path, output_stem: &Path) -> Result<()> {
    let doc = load_office(input)?;
    let stem = bundle_stem(output_stem);
    export_bundle(&doc, &stem)
}

pub fn convert(input: &Path, output: &Path) -> Result<()> {
    let in_ext = OpenDocument::extension_lower(input)?;
    let out_ext = OpenDocument::extension_lower(output)?;
    match in_ext.as_str() {
        "html" | "htm" | "docx" | "odt" => {
            let doc = load_office(input)?;
            if out_ext == "pdf" || out_ext == "docx" || out_ext == "odt" {
                let format = OutputFormat::from_path(output).ok_or_else(|| {
                    docrafter_core::Error::InvalidInput(format!(
                        "unsupported output extension .{out_ext}"
                    ))
                })?;
                export_save(&doc, output, format)
            } else {
                Err(docrafter_core::Error::InvalidInput(format!(
                    "unsupported output extension .{out_ext}"
                )))
            }
        }
        "pdf" if out_ext == "pdf" => {
            fs::copy(input, output).map_err(|e| docrafter_core::Error::io(output, e))?;
            Ok(())
        }
        "pdf" => Err(docrafter_core::Error::InvalidInput(format!(
            "cannot convert PDF to .{out_ext}"
        ))),
        other => Err(docrafter_core::Error::InvalidInput(format!(
            "unsupported input .{other}"
        ))),
    }
}

/// Convert many office/HTML inputs to one output format in a directory.
pub fn convert_batch(inputs: &[std::path::PathBuf], output_dir: &Path, to: &str) -> Result<()> {
    let ext = to.trim_start_matches('.').to_ascii_lowercase();
    if ext != "pdf" && ext != "docx" && ext != "odt" {
        return Err(docrafter_core::Error::InvalidInput(
            "batch --to must be pdf, docx, or odt".into(),
        ));
    }
    if inputs.is_empty() {
        return Err(docrafter_core::Error::InvalidInput(
            "batch requires at least one input file".into(),
        ));
    }
    fs::create_dir_all(output_dir).map_err(|e| docrafter_core::Error::io(output_dir, e))?;

    for input in inputs {
        let stem = input.file_stem().and_then(|s| s.to_str()).unwrap_or("out");
        let out_path = output_dir.join(format!("{stem}.{ext}"));
        convert(input, &out_path)?;
        eprintln!("{} → {}", input.display(), out_path.display());
    }
    Ok(())
}

/// Normalize `-o report` or `-o report.pdf` → stem path without extension.
fn bundle_stem(path: &Path) -> std::path::PathBuf {
    match path.extension().and_then(|e| e.to_str()) {
        Some("pdf" | "docx" | "odt") => path.with_extension(""),
        _ => path.to_path_buf(),
    }
}
