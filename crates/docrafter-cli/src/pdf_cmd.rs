//! `docrafter pdf …` subcommands.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use docrafter::pdf::{EncryptOptions, PdfReader, WatermarkOptions};
#[cfg(feature = "ocr")]
use docrafter::pdf::{OcrOptions, TextExtractMode};
use docrafter::Result;

use crate::util::{open_pdf, parse_page_list, parse_rotation};

pub fn merge(output: &Path, inputs: &[PathBuf]) -> Result<()> {
    if inputs.is_empty() {
        return Err(docrafter_core::Error::InvalidInput(
            "merge requires at least one input PDF".into(),
        ));
    }
    let mut doc = PdfReader::open(&inputs[0])?;
    for path in &inputs[1..] {
        doc.merge(&PdfReader::open(path)?)?;
    }
    doc.save(output)
}

pub fn text(input: &Path, output: Option<&Path>, ocr: bool, password: Option<&str>) -> Result<()> {
    if ocr {
        ocr_text(input, output, password)
    } else {
        let reader = open_pdf(input, password)?;
        write_text_output(output, &reader.extract_text()?)
    }
}

#[cfg(feature = "ocr")]
fn ocr_text(input: &Path, output: Option<&Path>, password: Option<&str>) -> Result<()> {
    let reader = open_pdf(input, password)?;
    let text = reader.extract_text_mode(TextExtractMode::Ocr(OcrOptions::default()))?;
    write_text_output(output, &text)
}

#[cfg(not(feature = "ocr"))]
fn ocr_text(_input: &Path, _output: Option<&Path>, _password: Option<&str>) -> Result<()> {
    Err(docrafter_core::Error::InvalidInput(
        "OCR requires the `ocr` feature (default in docrafter-cli)".into(),
    ))
}

pub fn text_batch(
    inputs: &[PathBuf],
    output_dir: &Path,
    ocr: bool,
    password: Option<&str>,
) -> Result<()> {
    if inputs.is_empty() {
        return Err(docrafter_core::Error::InvalidInput(
            "text-batch requires at least one input PDF".into(),
        ));
    }
    fs::create_dir_all(output_dir).map_err(|e| docrafter_core::Error::io(output_dir, e))?;
    for input in inputs {
        let stem = input.file_stem().and_then(|s| s.to_str()).unwrap_or("page");
        let out_path = output_dir.join(format!("{stem}.txt"));
        text(input, Some(&out_path), ocr, password)?;
        eprintln!("{} → {}", input.display(), out_path.display());
    }
    Ok(())
}

#[cfg(feature = "ocr")]
pub fn render(
    input: &Path,
    output: &Path,
    page: u32,
    all_pages: bool,
    dpi: u32,
    password: Option<&str>,
) -> Result<()> {
    use docrafter::pdf::{render_all_pages_rgba, render_page_rgba};

    let reader = open_pdf(input, password)?;
    let bytes = reader.to_bytes()?;
    let dpi = dpi.max(72) as f32;

    let pages = if all_pages {
        render_all_pages_rgba(&bytes, dpi)?
    } else {
        if page == 0 {
            return Err(docrafter_core::Error::InvalidInput(
                "page numbers are 1-based".into(),
            ));
        }
        vec![render_page_rgba(&bytes, page - 1, dpi)?]
    };

    if all_pages {
        fs::create_dir_all(output).map_err(|e| docrafter_core::Error::io(output, e))?;
    }

    for rendered in pages {
        let path = if all_pages {
            output.join(format!("page_{:03}.png", rendered.index + 1))
        } else {
            output.to_path_buf()
        };
        save_rgba_png(&path, rendered.width, rendered.height, &rendered.rgba)?;
        eprintln!("{}", path.display());
    }
    Ok(())
}

#[cfg(feature = "ocr")]
fn save_rgba_png(path: &Path, width: u32, height: u32, rgba: &[u8]) -> Result<()> {
    use image::{ImageBuffer, Rgba};

    let img: ImageBuffer<Rgba<u8>, _> = ImageBuffer::from_raw(width, height, rgba.to_vec())
        .ok_or_else(|| {
            docrafter_core::Error::InvalidInput("invalid RGBA dimensions for PNG".into())
        })?;
    img.save(path)
        .map_err(|e| docrafter_core::Error::io(path, std::io::Error::other(e)))
}

#[cfg(not(feature = "ocr"))]
pub fn render(
    _input: &Path,
    _output: &Path,
    _page: u32,
    _all_pages: bool,
    _dpi: u32,
    _password: Option<&str>,
) -> Result<()> {
    Err(docrafter_core::Error::InvalidInput(
        "pdf render requires the `ocr` feature (pulls in docrafter-pdf-render)".into(),
    ))
}

pub fn add_text_field(
    input: &Path,
    output: &Path,
    page: u32,
    name: &str,
    value: &str,
    rect: &str,
    password: Option<&str>,
) -> Result<()> {
    let mut parts = Vec::new();
    for segment in rect.split(',') {
        let v = segment.trim().parse::<f32>().map_err(|_| {
            docrafter_core::Error::InvalidInput(
                "rect must be four comma-separated numbers: left,bottom,right,top".into(),
            )
        })?;
        parts.push(v);
    }
    if parts.len() != 4 {
        return Err(docrafter_core::Error::InvalidInput(
            "rect must have exactly four values: left,bottom,right,top".into(),
        ));
    }
    let pdf_rect = [parts[0], parts[1], parts[2], parts[3]];
    let mut reader = open_pdf(input, password)?;
    reader.add_text_field(page, pdf_rect, name, value)?;
    reader.save(output)
}

pub fn list_fields(input: &Path, password: Option<&str>) -> Result<()> {
    let reader = open_pdf(input, password)?;
    let fields = reader.text_fields()?;
    if fields.is_empty() {
        println!("(no text fields)");
        return Ok(());
    }
    for field in fields {
        println!(
            "page {} name={} value={} [{:.1},{:.1},{:.1},{:.1}]",
            field.page,
            field.name,
            field.value,
            field.rect[0],
            field.rect[1],
            field.rect[2],
            field.rect[3]
        );
    }
    Ok(())
}

pub fn add_link(
    input: &Path,
    output: &Path,
    page: u32,
    uri: &str,
    rect: &str,
    password: Option<&str>,
) -> Result<()> {
    let mut parts = Vec::new();
    for segment in rect.split(',') {
        let value = segment.trim().parse::<f32>().map_err(|_| {
            docrafter_core::Error::InvalidInput(
                "rect must be four comma-separated numbers: left,bottom,right,top".into(),
            )
        })?;
        parts.push(value);
    }
    if parts.len() != 4 {
        return Err(docrafter_core::Error::InvalidInput(
            "rect must have exactly four values: left,bottom,right,top".into(),
        ));
    }
    let pdf_rect = [parts[0], parts[1], parts[2], parts[3]];
    let mut reader = open_pdf(input, password)?;
    reader.add_link(page, pdf_rect, uri)?;
    reader.save(output)
}

pub fn links(input: &Path, password: Option<&str>) -> Result<()> {
    let reader = open_pdf(input, password)?;
    let links = reader.links()?;
    if links.is_empty() {
        println!("(no URI links)");
        return Ok(());
    }
    for link in links {
        println!(
            "page {} [{:.1},{:.1},{:.1},{:.1}] {}",
            link.page, link.rect[0], link.rect[1], link.rect[2], link.rect[3], link.uri
        );
    }
    Ok(())
}

pub fn pages(input: &Path, password: Option<&str>) -> Result<()> {
    let reader = open_pdf(input, password)?;
    println!("{}", reader.page_count());
    Ok(())
}

pub fn rotate(
    input: &Path,
    output: &Path,
    degrees: u16,
    pages: Option<&str>,
    password: Option<&str>,
) -> Result<()> {
    let rotation = parse_rotation(degrees)?;
    let page_nums = pages.map(parse_page_list).transpose()?;
    let page_refs = page_nums.as_deref();

    let mut reader = open_pdf(input, password)?;
    reader.rotate(page_refs, rotation)?;
    reader.save(output)
}

pub fn split(input: &Path, output_dir: &Path, password: Option<&str>) -> Result<()> {
    let reader = open_pdf(input, password)?;
    let stem = input.file_stem().and_then(|s| s.to_str()).unwrap_or("page");
    fs::create_dir_all(output_dir).map_err(|e| docrafter_core::Error::io(output_dir, e))?;

    let parts = reader.split()?;
    for (idx, mut part) in parts.into_iter().enumerate() {
        let name = format!("{stem}_page_{:03}.pdf", idx + 1);
        let path = output_dir.join(name);
        part.save(&path)?;
    }
    Ok(())
}

pub fn watermark(
    input: &Path,
    output: &Path,
    text: &str,
    pages: Option<&str>,
    password: Option<&str>,
) -> Result<()> {
    let page_nums = pages.map(parse_page_list).transpose()?;
    let page_refs = page_nums.as_deref();

    let mut reader = open_pdf(input, password)?;
    let options = WatermarkOptions {
        text: text.to_string(),
        ..WatermarkOptions::default()
    };
    reader.add_watermark(page_refs, &options)?;
    reader.save(output)
}

pub fn extract(input: &Path, output: &Path, pages: &str, password: Option<&str>) -> Result<()> {
    let page_list = crate::util::parse_page_list(pages)?;
    let mut reader = open_pdf(input, password)?;
    reader.extract_pages(&page_list)?;
    reader.save(output)
}

pub fn set_metadata(
    input: &Path,
    output: &Path,
    title: Option<String>,
    author: Option<String>,
    subject: Option<String>,
    password: Option<&str>,
) -> Result<()> {
    let mut reader = open_pdf(input, password)?;
    let mut meta = reader.metadata();
    if let Some(t) = title {
        meta.title = Some(t);
    }
    if let Some(a) = author {
        meta.author = Some(a);
    }
    if let Some(s) = subject {
        meta.subject = Some(s);
    }
    reader.set_metadata(&meta)?;
    reader.save(output)
}

pub fn bookmark(
    input: &Path,
    output: &Path,
    title: &str,
    page: u32,
    password: Option<&str>,
) -> Result<()> {
    let mut reader = open_pdf(input, password)?;
    reader.add_bookmark(title, page, None)?;
    reader.save(output)
}

pub fn info(input: &Path, password: Option<&str>) -> Result<()> {
    let reader = open_pdf(input, password)?;
    let meta = reader.metadata();
    println!("pages: {}", reader.page_count());
    if reader.is_encrypted() {
        println!("encrypted: yes");
    }
    if let Some(t) = &meta.title {
        println!("title: {t}");
    }
    if let Some(a) = &meta.author {
        println!("author: {a}");
    }
    if let Some(s) = &meta.subject {
        println!("subject: {s}");
    }
    Ok(())
}

pub fn replace_text(
    input: &Path,
    output: &Path,
    from: &str,
    to: &str,
    page: u32,
    all_pages: bool,
    password: Option<&str>,
) -> Result<()> {
    let mut reader = open_pdf(input, password)?;
    if all_pages {
        reader.replace_text_all(from, to)?;
    } else {
        reader.replace_text(page, from, to)?;
    }
    reader.save(output)
}

pub fn compress(input: &Path, output: &Path, password: Option<&str>) -> Result<()> {
    let mut reader = open_pdf(input, password)?;
    reader.compress();
    reader.save(output)
}

pub fn encrypt_pdf(
    input: &Path,
    output: &Path,
    password: &str,
    owner_password: Option<&str>,
) -> Result<()> {
    let mut reader = open_pdf(input, None)?;
    let opts = EncryptOptions {
        user_password: password.to_string(),
        owner_password: owner_password.map(str::to_string),
        ..EncryptOptions::default()
    };
    reader.encrypt(&opts)?;
    reader.save(output)
}

fn write_text_output(path: Option<&Path>, text: &str) -> Result<()> {
    match path {
        Some(p) => fs::write(p, text).map_err(|e| docrafter_core::Error::io(p, e)),
        None => {
            let mut out = io::stdout().lock();
            out.write_all(text.as_bytes())
                .map_err(docrafter_core::Error::IoPlain)?;
            if !text.ends_with('\n') {
                out.write_all(b"\n")
                    .map_err(docrafter_core::Error::IoPlain)?;
            }
            Ok(())
        }
    }
}
