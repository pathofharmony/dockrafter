//! `content.xml` generation.

use docrafter_core::{Alignment, TableStyle};
use quick_xml::escape::escape;

use docrafter_office::OfficeBlock;
use docrafter_office::{Image, List, Paragraph, Table, TextRun};

use crate::package::ImageRef;
use crate::styles::paragraph_style_name;

const HEADER: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-content xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0" xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0" xmlns:table="urn:oasis:names:tc:opendocument:xmlns:table:1.0" xmlns:draw="urn:oasis:names:tc:opendocument:xmlns:drawing:1.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:fo="urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0" xmlns:svg="urn:oasis:names:tc:opendocument:xmlns:svg-compatible:1.0" xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0" office:version="1.2">
<office:body><office:text>
"#;

const FOOTER: &str = r#"
</office:text></office:body></office:document-content>
"#;

/// Build `content.xml` body.
pub fn build_content_xml(blocks: &[OfficeBlock], images: &[ImageRef]) -> String {
    let mut body = String::from(HEADER);
    let mut image_idx = 0;
    for block in blocks {
        match block {
            OfficeBlock::Paragraph(p) => body.push_str(&paragraph_xml(p)),
            OfficeBlock::Table(t) => body.push_str(&table_xml(t)),
            OfficeBlock::Image(img) => {
                if let Some(reference) = images.get(image_idx) {
                    body.push_str(&image_xml(img, reference));
                    image_idx += 1;
                }
            }
            OfficeBlock::List(list) => body.push_str(&list_xml(list)),
        }
    }
    body.push_str(FOOTER);
    body
}

fn paragraph_xml(paragraph: &Paragraph) -> String {
    let p_style = paragraph.paragraph_style();
    let name = paragraph_style_name(p_style);
    let align = fo_align(p_style.effective_align());
    let mut out = format!(r#"<text:p text:style-name="{name}""#);
    if let Some(align) = align {
        out.push_str(&format!(r#" fo:text-align="{align}""#));
    }
    out.push('>');
    for run in paragraph.runs() {
        out.push_str(&run_xml(run));
    }
    if paragraph.runs().is_empty() {
        out.push_str("<text:span/>");
    }
    out.push_str("</text:p>");
    out
}

fn run_xml(run: &TextRun) -> String {
    let style = run.resolved_style();
    let mut attrs = String::new();
    if style.is_bold() {
        attrs.push_str(r#" fo:font-weight="bold""#);
    }
    if style.is_italic() {
        attrs.push_str(r#" fo:font-style="italic""#);
    }
    if style.is_underline() {
        attrs.push_str(r#" style:text-underline-style="solid" style:text-underline-type="single""#);
    }
    if style.is_strikethrough() {
        attrs.push_str(
            r#" style:text-line-through-style="solid" style:text-line-through-type="single""#,
        );
    }
    match style.vertical_align() {
        docrafter_core::VerticalAlign::Superscript => {
            attrs.push_str(r#" style:text-position="super 58%""#);
        }
        docrafter_core::VerticalAlign::Subscript => {
            attrs.push_str(r#" style:text-position="sub 58%""#);
        }
        docrafter_core::VerticalAlign::Baseline => {}
    }
    let color = style.effective_color();
    attrs.push_str(&format!(
        " fo:color=\"#{:02x}{:02x}{:02x}\"",
        color.r(),
        color.g(),
        color.b()
    ));
    let pt = style.effective_font_size();
    attrs.push_str(&format!(r#" fo:font-size="{pt}pt""#));
    format!(
        r#"<text:span{attrs}>{text}</text:span>"#,
        text = escape_text(run.text())
    )
}

fn list_xml(list: &List) -> String {
    let mut out = String::from(r#"<text:list text:style-name="List1">"#);
    for item in list.items() {
        out.push_str(&format!(
            r#"<text:list-item><text:p text:style-name="Standard">{}</text:p></text:list-item>"#,
            escape_text(item)
        ));
    }
    out.push_str("</text:list>");
    out
}

fn table_xml(table: &Table) -> String {
    let cols = table
        .columns
        .len()
        .max(table.rows.first().map(|r| r.len()).unwrap_or(1));
    let cols = cols.max(1);
    let mut out = String::from(r#"<table:table table:name="Table1">"#);
    if !table.columns.is_empty() {
        out.push_str(&table_row_xml(&table.columns, true, &table.style));
    }
    for row in &table.rows {
        let cells: Vec<String> = (0..cols)
            .map(|i| row.get(i).cloned().unwrap_or_default())
            .collect();
        out.push_str(&table_row_xml(&cells, false, &table.style));
    }
    out.push_str("</table:table>");
    out
}

fn table_row_xml(cells: &[String], header: bool, _style: &TableStyle) -> String {
    let mut out = String::from("<table:table-row>");
    for cell in cells {
        let style = if header {
            r#" table:style-name="TableHeader""#
        } else {
            ""
        };
        out.push_str(&format!(
            r#"<table:table-cell{style} office:value-type="string"><text:p text:style-name="Standard">{}</text:p></table:table-cell>"#,
            escape_text(cell)
        ));
    }
    out.push_str("</table:table-row>");
    out
}

fn image_xml(image: &Image, reference: &ImageRef) -> String {
    let (w_pt, h_pt) = image_dimensions(image);
    let w = pt_to_cm(w_pt);
    let h = pt_to_cm(h_pt);
    format!(
        r#"<text:p><draw:frame draw:name="{name}" text:anchor-type="as-char" svg:width="{w}cm" svg:height="{h}cm"><draw:image xlink:href="{href}" xlink:type="simple" xlink:show="embed"/></draw:frame></text:p>"#,
        name = reference.draw_name,
        href = reference.href
    )
}

fn image_dimensions(image: &Image) -> (f32, f32) {
    if let (Some(w), Some(h)) = (image.width_pt(), image.height_pt()) {
        return (w, h);
    }
    if let Ok(dynamic) = image::load_from_memory(image.data()) {
        let w = dynamic.width() as f32 * 72.0 / 96.0;
        let h = dynamic.height() as f32 * 72.0 / 96.0;
        return (w.max(1.0), h.max(1.0));
    }
    (120.0, 80.0)
}

fn pt_to_cm(pt: f32) -> f32 {
    pt / 72.0 * 2.54
}

fn fo_align(align: Alignment) -> Option<&'static str> {
    match align {
        Alignment::Start => None,
        Alignment::Center => Some("center"),
        Alignment::End => Some("end"),
        Alignment::Justify => Some("justify"),
        _ => None,
    }
}

fn escape_text(text: &str) -> String {
    escape(text).into_owned()
}
