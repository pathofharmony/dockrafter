//! WordprocessingML XML generation.

use docrafter_core::{Alignment, Style, TableStyle};
use quick_xml::escape::escape;

use crate::comments::{comment_range_markers, DocxComment};
use crate::numbering::LIST_NUM_ID;
use crate::package::ImageRef;
use crate::styles::paragraph_style_id;
use crate::DocxBlock;
use docrafter_office::{Image, Paragraph, Table, TextRun};

const HEADER: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture">
<w:body>
"#;

const FOOTER: &str = r#"
</w:body>
</w:document>
"#;

/// Build `word/document.xml` preserving block order.
pub fn build_document_xml(
    blocks: &[DocxBlock],
    images: &[ImageRef],
    comments: &[DocxComment],
) -> String {
    let mut body = String::from(HEADER);
    let mut image_idx = 0;
    let mut comments_applied = comments.is_empty();
    for block in blocks {
        match block {
            DocxBlock::Paragraph(p) => {
                let mut xml = paragraph_xml(p);
                if !comments_applied {
                    if let Some(pos) = xml.rfind("</w:pPr>") {
                        let end = pos + "</w:pPr>".len();
                        let markers: String = comments
                            .iter()
                            .map(|c| comment_range_markers(c.id))
                            .collect();
                        xml.insert_str(end, &markers);
                    }
                    comments_applied = true;
                }
                body.push_str(&xml);
            }
            DocxBlock::Table(t) => body.push_str(&table_xml(t)),
            DocxBlock::Image(img) => {
                if let Some(reference) = images.get(image_idx) {
                    body.push_str(&image_xml(img, reference));
                    image_idx += 1;
                }
            }
            DocxBlock::List(list) => {
                for item in list.items() {
                    body.push_str(&list_item_xml(item));
                }
            }
        }
    }
    body.push_str(FOOTER);
    body
}

fn paragraph_xml(paragraph: &Paragraph) -> String {
    let p_style = paragraph.paragraph_style();
    let mut out = String::from("<w:p><w:pPr>");
    if let Some(jc) = alignment_val(p_style.effective_align()) {
        out.push_str(&format!(r#"<w:jc w:val="{jc}"/>"#));
    }
    if let Some(id) = paragraph_style_id(p_style) {
        out.push_str(&format!(r#"<w:pStyle w:val="{id}"/>"#));
    }
    out.push_str("</w:pPr>");
    for run in paragraph.runs() {
        out.push_str(&run_xml(run));
    }
    if paragraph.runs().is_empty() {
        out.push_str(r#"<w:r><w:t></w:t></w:r>"#);
    }
    out.push_str("</w:p>");
    out
}

fn run_xml(run: &TextRun) -> String {
    let style = run.resolved_style();
    let mut out = String::from("<w:r><w:rPr>");
    out.push_str(&run_properties_xml(style));
    out.push_str("</w:rPr>");
    out.push_str(&format!(
        r#"<w:t xml:space="preserve">{}</w:t>"#,
        escape_text(run.text())
    ));
    out.push_str("</w:r>");
    out
}

fn list_item_xml(text: &str) -> String {
    format!(
        r#"<w:p><w:pPr><w:numPr><w:ilvl w:val="0"/><w:numId w:val="{LIST_NUM_ID}"/></w:numPr></w:pPr><w:r><w:t xml:space="preserve">{}</w:t></w:r></w:p>"#,
        escape_text(text)
    )
}

fn table_xml(table: &Table) -> String {
    let cols = table
        .columns
        .len()
        .max(table.rows.first().map(|r| r.len()).unwrap_or(1));
    let cols = cols.max(1);
    let col_w = (9000 / cols).max(1200);
    let mut out =
        String::from("<w:tbl><w:tblPr><w:tblW w:w=\"0\" w:type=\"auto\"/></w:tblPr><w:tblGrid>");
    for _ in 0..cols {
        out.push_str(&format!(r#"<w:gridCol w:w="{col_w}"/>"#));
    }
    out.push_str("</w:tblGrid>");

    if !table.columns.is_empty() {
        out.push_str(&table_row_xml(&table.columns, true, &table.style));
    }
    for row in &table.rows {
        let cells: Vec<String> = (0..cols)
            .map(|i| row.get(i).cloned().unwrap_or_default())
            .collect();
        out.push_str(&table_row_xml(&cells, false, &table.style));
    }
    out.push_str("</w:tbl>");
    out
}

fn table_row_xml(cells: &[String], header: bool, style: &TableStyle) -> String {
    let mut out = String::from("<w:tr>");
    for cell in cells {
        out.push_str("<w:tc><w:tcPr>");
        if header {
            let bg = style.effective_header_bg();
            let fill = format!("{:02X}{:02X}{:02X}", bg.r(), bg.g(), bg.b());
            out.push_str(&format!(
                r#"<w:shd w:val="clear" w:color="auto" w:fill="{fill}"/>"#
            ));
        }
        out.push_str(r#"</w:tcPr><w:p><w:r><w:rPr>"#);
        if header {
            out.push_str("<w:b/>");
        }
        out.push_str(&format!(
            r#"</w:rPr><w:t xml:space="preserve">{}</w:t></w:r></w:p></w:tc>"#,
            escape_text(cell)
        ));
    }
    out.push_str("</w:tr>");
    out
}

fn image_xml(image: &Image, reference: &ImageRef) -> String {
    let (width_pt, height_pt) = image_dimensions(image);
    let cx = pt_to_emu(width_pt);
    let cy = pt_to_emu(height_pt);
    format!(
        r#"<w:p><w:r><w:drawing><wp:inline distT="0" distB="0" distL="0" distR="0"><wp:extent cx="{cx}" cy="{cy}"/><wp:docPr id="1" name="Picture"/><a:graphic><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:pic><pic:nvPicPr><pic:cNvPr id="0" name=""/><pic:cNvPicPr/></pic:nvPicPr><pic:blipFill><a:blip r:embed="{rid}"/><a:stretch><a:fillRect/></a:stretch></pic:blipFill><pic:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="{cx}" cy="{cy}"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></pic:spPr></pic:pic></a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p>"#,
        rid = reference.rel_id
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

fn pt_to_emu(pt: f32) -> i64 {
    (pt * 12700.0).round() as i64
}

fn run_properties_xml(style: &Style) -> String {
    let mut out = String::new();
    if style.is_bold() {
        out.push_str("<w:b/>");
    }
    if style.is_italic() {
        out.push_str("<w:i/>");
    }
    if style.is_underline() {
        out.push_str(r#"<w:u w:val="single"/>"#);
    }
    if style.is_strikethrough() {
        out.push_str("<w:strike/>");
    }
    match style.vertical_align() {
        docrafter_core::VerticalAlign::Superscript => {
            out.push_str(r#"<w:vertAlign w:val="superscript"/>"#);
        }
        docrafter_core::VerticalAlign::Subscript => {
            out.push_str(r#"<w:vertAlign w:val="subscript"/>"#);
        }
        docrafter_core::VerticalAlign::Baseline => {}
    }
    let half_points = (style.effective_font_size() * 2.0).round() as u32;
    out.push_str(&format!(r#"<w:sz w:val="{half_points}"/>"#));
    let color = style.effective_color();
    out.push_str(&format!(
        r#"<w:color w:val="{:02X}{:02X}{:02X}"/>"#,
        color.r(),
        color.g(),
        color.b()
    ));
    out
}

fn alignment_val(align: Alignment) -> Option<&'static str> {
    match align {
        Alignment::Start => None,
        Alignment::Center => Some("center"),
        Alignment::End => Some("right"),
        Alignment::Justify => Some("both"),
        _ => None,
    }
}

fn escape_text(text: &str) -> String {
    escape(text).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DocxBlock;
    use docrafter_core::Style;

    #[test]
    fn document_xml_contains_cyrillic_and_pstyle() {
        let blocks = vec![DocxBlock::Paragraph(
            Paragraph::new("Отчёт за май").style(Style::heading1()),
        )];
        let xml = build_document_xml(&blocks, &[], &[]);
        assert!(xml.contains("Отчёт за май"));
        assert!(xml.contains(r#"w:val="Heading1""#));
    }

    #[test]
    fn multi_run_paragraph_xml() {
        let p = Paragraph::new("Hello ").run("world", Style::new().bold());
        let xml = build_document_xml(&[DocxBlock::Paragraph(p)], &[], &[]);
        assert!(xml.matches("<w:r>").count() >= 2);
        assert!(xml.contains("<w:b/>"));
    }

    #[test]
    fn underline_run_xml() {
        let p = Paragraph::new("x").run("u", Style::new().underline());
        let xml = build_document_xml(&[DocxBlock::Paragraph(p)], &[], &[]);
        assert!(xml.contains(r#"<w:u w:val="single"/>"#));
    }
}
