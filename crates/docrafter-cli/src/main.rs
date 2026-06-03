//! `docrafter` CLI — PDF and office document tools.

#![allow(clippy::multiple_crate_versions)] // docrafter → OCR/render trees

mod export_cmd;
mod pdf_cmd;
mod template_cmd;
mod util;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use docrafter::Result;

#[derive(Parser)]
#[command(
    name = "docrafter",
    version,
    about = "PDF and office documents (reportlab / pypdf / python-docx style)"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// PDF: merge, text, rotate, split, watermark, …
    Pdf {
        #[command(subcommand)]
        action: PdfAction,
    },
    /// Export office/HTML to PDF, DOCX, or ODT
    Export {
        input: PathBuf,
        /// Output file, or stem when using `--bundle`
        #[arg(short, long)]
        output: PathBuf,
        /// Write `{stem}.pdf`, `{stem}.docx`, and `{stem}.odt`
        #[arg(long)]
        bundle: bool,
    },
    /// Convert by extension (docx/odt/html → pdf/docx/odt)
    Convert {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Convert many files to the same format in an output directory
    Batch {
        #[arg(required = true)]
        inputs: Vec<PathBuf>,
        #[arg(short, long)]
        output: PathBuf,
        /// Target format: `pdf`, `docx`, or `odt`
        #[arg(long)]
        to: String,
    },
    /// HTML file → PDF / DOCX / ODT (output extension)
    Html {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
    },
    /// JSON report templates (see `examples/template-spec.json`)
    Template {
        #[command(subcommand)]
        action: TemplateAction,
    },
}

#[derive(Subcommand)]
enum TemplateAction {
    /// Build PDF / DOCX / ODT from a JSON spec
    Render {
        /// JSON spec: `vars`, `title`, `paragraphs`, `tables`
        spec: PathBuf,
        /// Output stem (`--bundle`) or file path (`--format`)
        #[arg(short, long)]
        output: PathBuf,
        #[arg(long)]
        bundle: bool,
        #[arg(long, value_enum)]
        format: Option<template_cmd::TemplateFormat>,
    },
}

#[derive(Subcommand)]
enum PdfAction {
    /// Concatenate PDFs into one file
    Merge {
        output: PathBuf,
        #[arg(required = true)]
        inputs: Vec<PathBuf>,
    },
    /// Extract text to stdout or a file
    Text {
        input: PathBuf,
        #[arg(short, long)]
        output: Option<PathBuf>,
        #[arg(long)]
        ocr: bool,
        #[arg(short, long)]
        password: Option<String>,
    },
    /// Print page count
    Pages {
        input: PathBuf,
        #[arg(short, long)]
        password: Option<String>,
    },
    /// Rotate pages (90°, 180°, or 270°)
    Rotate {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        /// Rotation in degrees: 90, 180, or 270
        #[arg(short, long, default_value = "90")]
        angle: u16,
        /// Comma-separated 1-based pages (default: all)
        #[arg(long)]
        pages: Option<String>,
        #[arg(short, long)]
        password: Option<String>,
    },
    /// Split into one PDF per page in a directory
    Split {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(short, long)]
        password: Option<String>,
    },
    /// Add a diagonal text watermark
    Watermark {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(short, long, default_value = "DRAFT")]
        text: String,
        #[arg(long)]
        pages: Option<String>,
        #[arg(short, long)]
        password: Option<String>,
    },
    /// Extract 1-based pages to a new PDF
    Extract {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        /// Comma-separated page numbers
        #[arg(short, long)]
        pages: String,
        #[arg(short, long)]
        password: Option<String>,
    },
    /// Set document Info metadata
    Metadata {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        author: Option<String>,
        #[arg(long)]
        subject: Option<String>,
        #[arg(short, long)]
        password: Option<String>,
    },
    /// Add a PDF bookmark (outline entry)
    Bookmark {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        title: String,
        /// 1-based destination page
        #[arg(short, long)]
        page: u32,
        #[arg(short, long)]
        password: Option<String>,
    },
    /// Encrypt PDF with a user password (revision 2 / 40-bit RC4)
    Encrypt {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(short, long)]
        password: String,
        #[arg(long)]
        owner_password: Option<String>,
    },
    /// Print page count and Info metadata
    Info {
        input: PathBuf,
        #[arg(short, long)]
        password: Option<String>,
    },
    /// Replace text on one page (docrafter PDFs or standard encodings)
    Replace {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        from: String,
        to: String,
        #[arg(short, long, default_value = "1")]
        page: u32,
        #[arg(long)]
        all_pages: bool,
        #[arg(short, long)]
        password: Option<String>,
    },
    /// Compress internal streams (smaller file)
    Compress {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(short, long)]
        password: Option<String>,
    },
    /// List URI links (page, rect, URL)
    Links {
        input: PathBuf,
        #[arg(short, long)]
        password: Option<String>,
    },
    /// Rasterize page(s) to PNG (requires OCR/render stack)
    Render {
        input: PathBuf,
        /// Output `.png` file or directory when using `--all`
        #[arg(short, long)]
        output: PathBuf,
        /// 1-based page (ignored with `--all`)
        #[arg(short, long, default_value = "1")]
        page: u32,
        #[arg(long)]
        all: bool,
        #[arg(long, default_value = "150")]
        dpi: u32,
        #[arg(short, long)]
        password: Option<String>,
    },
    /// List AcroForm text fields
    Fields {
        input: PathBuf,
        #[arg(short, long)]
        password: Option<String>,
    },
    /// Add an AcroForm text field
    AddField {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(short, long)]
        page: u32,
        name: String,
        #[arg(short, long, default_value = "")]
        value: String,
        #[arg(long, value_name = "RECT")]
        rect: String,
        #[arg(short, long)]
        password: Option<String>,
    },
    /// Add a clickable URI link on a page
    AddLink {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        /// 1-based page number
        #[arg(short, long)]
        page: u32,
        uri: String,
        /// Hotspot rectangle: `left,bottom,right,top` in PDF points
        #[arg(long, value_name = "RECT")]
        rect: String,
        #[arg(short, long)]
        password: Option<String>,
    },
    /// Extract text from many PDFs into `{output}/{stem}.txt`
    TextBatch {
        #[arg(required = true)]
        inputs: Vec<PathBuf>,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(long)]
        ocr: bool,
        #[arg(short, long)]
        password: Option<String>,
    },
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Pdf { action } => run_pdf(action),
        Command::Export {
            input,
            output,
            bundle,
        } => {
            if bundle {
                export_cmd::export_with_bundle(&input, &output)
            } else {
                export_cmd::export_one(&input, &output)
            }
        }
        Command::Convert { input, output } => export_cmd::convert(&input, &output),
        Command::Batch { inputs, output, to } => export_cmd::convert_batch(&inputs, &output, &to),
        Command::Html { input, output } => export_cmd::export_one(&input, &output),
        Command::Template { action } => match action {
            TemplateAction::Render {
                spec,
                output,
                bundle,
                format,
            } => template_cmd::render(&spec, &output, bundle, format),
        },
    }
}

fn run_pdf(action: PdfAction) -> Result<()> {
    match action {
        PdfAction::Merge { output, inputs } => pdf_cmd::merge(&output, &inputs),
        PdfAction::Text {
            input,
            output,
            ocr,
            password,
        } => pdf_cmd::text(&input, output.as_deref(), ocr, password.as_deref()),
        PdfAction::Pages { input, password } => pdf_cmd::pages(&input, password.as_deref()),
        PdfAction::Rotate {
            input,
            output,
            angle,
            pages,
            password,
        } => pdf_cmd::rotate(
            &input,
            &output,
            angle,
            pages.as_deref(),
            password.as_deref(),
        ),
        PdfAction::Split {
            input,
            output,
            password,
        } => pdf_cmd::split(&input, &output, password.as_deref()),
        PdfAction::Watermark {
            input,
            output,
            text,
            pages,
            password,
        } => pdf_cmd::watermark(
            &input,
            &output,
            &text,
            pages.as_deref(),
            password.as_deref(),
        ),
        PdfAction::Extract {
            input,
            output,
            pages,
            password,
        } => pdf_cmd::extract(&input, &output, &pages, password.as_deref()),
        PdfAction::Metadata {
            input,
            output,
            title,
            author,
            subject,
            password,
        } => pdf_cmd::set_metadata(&input, &output, title, author, subject, password.as_deref()),
        PdfAction::Bookmark {
            input,
            output,
            title,
            page,
            password,
        } => pdf_cmd::bookmark(&input, &output, &title, page, password.as_deref()),
        PdfAction::Encrypt {
            input,
            output,
            password,
            owner_password,
        } => pdf_cmd::encrypt_pdf(&input, &output, &password, owner_password.as_deref()),
        PdfAction::Info { input, password } => pdf_cmd::info(&input, password.as_deref()),
        PdfAction::Replace {
            input,
            output,
            from,
            to,
            page,
            all_pages,
            password,
        } => pdf_cmd::replace_text(
            &input,
            &output,
            &from,
            &to,
            page,
            all_pages,
            password.as_deref(),
        ),
        PdfAction::Compress {
            input,
            output,
            password,
        } => pdf_cmd::compress(&input, &output, password.as_deref()),
        PdfAction::Links { input, password } => pdf_cmd::links(&input, password.as_deref()),
        PdfAction::Render {
            input,
            output,
            page,
            all,
            dpi,
            password,
        } => pdf_cmd::render(&input, &output, page, all, dpi, password.as_deref()),
        PdfAction::Fields { input, password } => pdf_cmd::list_fields(&input, password.as_deref()),
        PdfAction::AddField {
            input,
            output,
            page,
            name,
            value,
            rect,
            password,
        } => pdf_cmd::add_text_field(
            &input,
            &output,
            page,
            &name,
            &value,
            &rect,
            password.as_deref(),
        ),
        PdfAction::AddLink {
            input,
            output,
            page,
            uri,
            rect,
            password,
        } => pdf_cmd::add_link(&input, &output, page, &uri, &rect, password.as_deref()),
        PdfAction::TextBatch {
            inputs,
            output,
            ocr,
            password,
        } => pdf_cmd::text_batch(&inputs, &output, ocr, password.as_deref()),
    }
}
