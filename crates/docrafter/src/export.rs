//! Multi-format export: one [`OfficeDocument`] → PDF, DOCX, or ODT.

use std::path::Path;

use docrafter_core::{Error, PageSize, Result};
use docrafter_docx::DocxDocument;
use docrafter_layout::LayoutMargins;
use docrafter_odt::OdtDocument;
use docrafter_office::{OfficeBlock, OfficeDocument};
use docrafter_pdf_write::{PageHeaderFooter, PdfDocument, Table as PdfTable};

/// Target file format for [`export_bytes`] / [`export_save`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// PDF (flow layout).
    Pdf,
    /// Office Open XML Word.
    Docx,
    /// ODT (OpenDocument Text, LibreOffice).
    Odt,
}

impl OutputFormat {
    /// Guess format from a file extension (`pdf`, `docx`, `odt`).
    #[must_use]
    pub fn from_path(path: &Path) -> Option<Self> {
        match path.extension()?.to_str()? {
            "pdf" => Some(Self::Pdf),
            "docx" => Some(Self::Docx),
            "odt" => Some(Self::Odt),
            _ => None,
        }
    }
}

/// PDF layout options for [`export_bytes`] / [`pdf_from_office`].
#[derive(Debug, Clone)]
pub struct PdfExportOptions {
    /// Paper size.
    pub page_size: PageSize,
    /// Content margins in points.
    pub margins: LayoutMargins,
    /// Optional running header/footer (`{page}` / `{pages}` in footer).
    pub header_footer: Option<PageHeaderFooter>,
}

impl Default for PdfExportOptions {
    fn default() -> Self {
        Self {
            page_size: PageSize::a4(),
            margins: LayoutMargins::standard(),
            header_footer: None,
        }
    }
}

/// Serialize an office document to bytes.
pub fn export_bytes(doc: &OfficeDocument, format: OutputFormat) -> Result<Vec<u8>> {
    export_bytes_with_pdf_options(doc, format, &PdfExportOptions::default())
}

/// Serialize with explicit PDF layout (ignored for DOCX/ODT).
pub fn export_bytes_with_pdf_options(
    doc: &OfficeDocument,
    format: OutputFormat,
    pdf_options: &PdfExportOptions,
) -> Result<Vec<u8>> {
    match format {
        OutputFormat::Pdf => pdf_from_office_with_options(doc, pdf_options).to_bytes(),
        OutputFormat::Docx => DocxDocument::from_blocks(doc.blocks().to_vec()).to_bytes(),
        OutputFormat::Odt => OdtDocument::from_blocks(doc.blocks().to_vec()).to_bytes(),
    }
}

/// Write using an explicit format.
pub fn export_save(
    doc: &OfficeDocument,
    path: impl AsRef<Path>,
    format: OutputFormat,
) -> Result<()> {
    let path = path.as_ref();
    let bytes = export_bytes(doc, format)?;
    std::fs::write(path, &bytes).map_err(|source| Error::io(path, source))?;
    Ok(())
}

/// Write PDF, DOCX, and ODT with the same base path (e.g. `report` → `report.pdf`, …).
pub fn export_bundle(doc: &OfficeDocument, stem: impl AsRef<Path>) -> Result<()> {
    let stem = stem.as_ref();
    export_save_auto(doc, stem.with_extension("pdf"))?;
    export_save_auto(doc, stem.with_extension("docx"))?;
    export_save_auto(doc, stem.with_extension("odt"))?;
    Ok(())
}

/// Write using the path extension (`.pdf`, `.docx`, `.odt`).
pub fn export_save_auto(doc: &OfficeDocument, path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    let format = OutputFormat::from_path(path).ok_or_else(|| {
        Error::InvalidInput(format!("unsupported export extension: {}", path.display()))
    })?;
    export_save(doc, path, format)
}

/// Build a PDF from office blocks (full multi-run fidelity).
#[must_use]
pub fn pdf_from_office(doc: &OfficeDocument) -> PdfDocument {
    pdf_from_office_with_options(doc, &PdfExportOptions::default())
}

/// Build a PDF with explicit page size and margins.
#[must_use]
pub fn pdf_from_office_with_options(
    doc: &OfficeDocument,
    options: &PdfExportOptions,
) -> PdfDocument {
    let mut pdf = PdfDocument::with_page_size(options.page_size).with_margins(options.margins);
    if let Some(hf) = &options.header_footer {
        pdf = pdf.with_header_footer(hf.clone());
    }
    for block in doc.blocks() {
        match block {
            OfficeBlock::Paragraph(p) => {
                pdf.push(p.clone());
            }
            OfficeBlock::Table(t) => {
                pdf.push(table_to_pdf(t));
            }
            OfficeBlock::Image(img) => {
                pdf.push(img.clone());
            }
            OfficeBlock::List(list) => {
                pdf.push(list.clone());
            }
        }
    }
    pdf
}

fn table_to_pdf(t: &docrafter_office::Table) -> PdfTable {
    let mut table = PdfTable::new().style(t.style.clone());
    if t.repeat_header_on_new_page {
        table = table.repeat_header_on_new_page(true);
    }
    if !t.column_widths.is_empty() {
        table = table.column_widths(t.column_widths.clone());
    }
    if !t.columns.is_empty() {
        table = table.columns(t.columns.clone());
    }
    for row in &t.rows {
        table = table.row(row.clone());
    }
    table
}
