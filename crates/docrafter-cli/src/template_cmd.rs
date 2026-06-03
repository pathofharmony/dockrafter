//! `docrafter template render` — JSON report spec → PDF / DOCX / ODT.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use docrafter::export::{export_bundle, export_save, OutputFormat};
use docrafter::prelude::*;
use docrafter::Result;
use docrafter_core::Error;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct TableSpec {
    columns: Vec<String>,
    rows: Vec<Vec<String>>,
    #[serde(default)]
    professional: bool,
}

#[derive(Debug, Deserialize)]
struct TemplateSpec {
    #[serde(default)]
    vars: HashMap<String, String>,
    title: Option<String>,
    #[serde(default)]
    paragraphs: Vec<String>,
    #[serde(default)]
    tables: Vec<TableSpec>,
}

/// Output format for a single file (not bundle).
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum TemplateFormat {
    Pdf,
    Docx,
    Odt,
}

pub fn render(
    spec_path: &Path,
    output: &Path,
    bundle: bool,
    format: Option<TemplateFormat>,
) -> Result<()> {
    let raw = fs::read_to_string(spec_path).map_err(|e| Error::io(spec_path, e))?;
    let spec: TemplateSpec = serde_json::from_str(&raw)
        .map_err(|e| Error::InvalidInput(format!("invalid template JSON: {e}")))?;

    let mut ctx = Context::new();
    for (k, v) in spec.vars {
        ctx = ctx.with(k, v);
    }

    let mut builder = ReportBuilder::new();
    if let Some(title) = spec.title {
        builder = builder.title(title);
    }
    for p in spec.paragraphs {
        builder = builder.paragraph(p);
    }
    for table in spec.tables {
        builder = if table.professional {
            builder.table_professional(table.columns, &table.rows)
        } else {
            builder.table(table.columns, &table.rows)
        };
    }

    let doc = builder.build(&ctx)?;

    if bundle {
        let stem = output.to_string_lossy();
        export_bundle(&doc, stem.as_ref())?;
        println!(
            "Wrote {stem}.pdf, {stem}.docx, {stem}.odt",
            stem = output.display()
        );
        return Ok(());
    }

    let fmt = format
        .ok_or_else(|| Error::InvalidInput("use --bundle or --format pdf|docx|odt".into()))?;
    let out_fmt = match fmt {
        TemplateFormat::Pdf => OutputFormat::Pdf,
        TemplateFormat::Docx => OutputFormat::Docx,
        TemplateFormat::Odt => OutputFormat::Odt,
    };
    export_save(&doc, output, out_fmt)?;
    println!("Wrote {}", output.display());
    Ok(())
}
