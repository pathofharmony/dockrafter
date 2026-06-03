//! Parse existing DOCX packages.

use std::collections::HashMap;
use std::io::{Cursor, Read};

use docrafter_core::{Color, Error, Result, Style};
use quick_xml::events::Event;
use quick_xml::Reader;
use zip::ZipArchive;

use crate::comments::{parse_comments_xml, DocxComment};
use crate::styles::style_from_paragraph_id;
use crate::DocxBlock;
use docrafter_office::{Image, List, Paragraph, Table, TextRun};

/// Loaded OOXML parts needed for parsing.
pub struct DocxArchive {
    /// Raw `word/document.xml`.
    pub document_xml: String,
    /// Relationship id → target path (e.g. `media/image1.png`).
    pub rels: HashMap<String, String>,
    /// Path inside zip → bytes (`word/media/image1.png`).
    pub media: HashMap<String, Vec<u8>>,
    /// Raw `word/comments.xml` when present.
    pub comments_xml: Option<String>,
}

/// Load package parts from `.docx` bytes.
pub fn load_archive(bytes: &[u8]) -> Result<DocxArchive> {
    let cursor = Cursor::new(bytes);
    let mut zip =
        ZipArchive::new(cursor).map_err(|e| Error::Docx(format!("invalid docx zip: {e}")))?;

    let document_xml = read_zip_text(&mut zip, "word/document.xml")?;
    let rels_xml = read_zip_text(&mut zip, "word/_rels/document.xml.rels").unwrap_or_default();
    let rels = parse_relationships(&rels_xml);
    let comments_xml = read_zip_text_optional(&mut zip, "word/comments.xml");

    let mut media = HashMap::new();
    for i in 0..zip.len() {
        let mut file = zip
            .by_index(i)
            .map_err(|e| Error::Docx(format!("zip entry {i}: {e}")))?;
        let name = file.name().to_string();
        if name.starts_with("word/media/") {
            let mut data = Vec::new();
            file.read_to_end(&mut data)
                .map_err(|e| Error::Docx(format!("read zip entry: {e}")))?;
            media.insert(name, data);
        }
    }

    Ok(DocxArchive {
        document_xml,
        rels,
        media,
        comments_xml,
    })
}

/// Parse review comments from a loaded archive.
pub fn parse_comments(archive: &DocxArchive) -> Result<Vec<DocxComment>> {
    match &archive.comments_xml {
        Some(xml) => parse_comments_xml(xml),
        None => Ok(Vec::new()),
    }
}

/// Parse body blocks in document order.
pub fn parse_body(archive: &DocxArchive) -> Result<Vec<DocxBlock>> {
    let mut reader = Reader::from_str(&archive.document_xml);
    reader.config_mut().trim_text(false);

    let mut blocks = Vec::new();
    let mut parser = Parser::new(archive);

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => parser.on_start(&e)?,
            Ok(Event::End(e)) => parser.on_end(&e, &mut blocks)?,
            Ok(Event::Empty(e)) => parser.on_empty(&e)?,
            Ok(Event::Text(e)) => parser.on_text(&e.unescape().map_err(xml_err)?),
            Ok(Event::Eof) => break,
            Err(e) => return Err(xml_err(e)),
            _ => {}
        }
    }

    if blocks.is_empty() {
        return Err(Error::Docx("document has no body content".into()));
    }
    Ok(blocks)
}

struct Parser<'a> {
    archive: &'a DocxArchive,
    tbl_depth: u32,
    tr_open: bool,
    tc_open: bool,
    cell_has_shd: bool,
    table_rows: Vec<TableRowParsed>,
    current_row: TableRowParsed,
    in_paragraph: bool,
    in_run: bool,
    in_text: bool,
    p_style: Style,
    runs: Vec<TextRun>,
    run_text: String,
    run_style: Style,
    pending_image_rid: Option<String>,
    para_is_list: bool,
}

#[derive(Default)]
struct TableRowParsed {
    cells: Vec<String>,
    header_cells: bool,
}

impl<'a> Parser<'a> {
    fn new(archive: &'a DocxArchive) -> Self {
        Self {
            archive,
            tbl_depth: 0,
            tr_open: false,
            tc_open: false,
            cell_has_shd: false,
            table_rows: Vec::new(),
            current_row: TableRowParsed::default(),
            in_paragraph: false,
            in_run: false,
            in_text: false,
            p_style: Style::new(),
            runs: Vec::new(),
            run_text: String::new(),
            run_style: Style::new(),
            pending_image_rid: None,
            para_is_list: false,
        }
    }

    fn on_start(&mut self, e: &quick_xml::events::BytesStart) -> Result<()> {
        match e.local_name().as_ref() {
            b"tbl" => self.tbl_depth += 1,
            b"tr" if self.tbl_depth > 0 => {
                self.tr_open = true;
                self.current_row = TableRowParsed::default();
            }
            b"tc" if self.tr_open => {
                self.tc_open = true;
                self.cell_has_shd = false;
                self.current_row.cells.push(String::new());
            }
            b"shd" if self.tc_open => self.cell_has_shd = true,
            b"numPr" if self.in_paragraph => self.para_is_list = true,
            b"p" if self.tc_open || self.tbl_depth == 0 => self.start_paragraph(),
            b"r" if self.in_paragraph => {
                self.in_run = true;
                self.run_text.clear();
                self.run_style = Style::new();
            }
            b"t" if self.in_run => self.in_text = true,
            b"blip" => {
                if let Some(rid) = attr_value(e, b"embed") {
                    self.pending_image_rid = Some(rid);
                }
            }
            b"pStyle" => {
                if let Some(id) = attr_value(e, b"val") {
                    self.p_style = style_from_paragraph_id(&id);
                }
            }
            b"jc" => {
                if let Some(val) = attr_value(e, b"val") {
                    self.p_style = self.p_style.clone().align(parse_align(&val));
                }
            }
            b"b" if self.in_run => self.run_style = self.run_style.clone().bold(),
            b"i" if self.in_run => self.run_style = self.run_style.clone().italic(),
            b"u" if self.in_run => {
                let val = attr_value(e, b"val");
                if val.as_deref() != Some("none") {
                    self.run_style = self.run_style.clone().underline();
                }
            }
            b"strike" if self.in_run => self.run_style = self.run_style.clone().strikethrough(),
            b"vertAlign" if self.in_run => {
                if let Some(val) = attr_value(e, b"val") {
                    match val.as_str() {
                        "superscript" => self.run_style = self.run_style.clone().superscript(),
                        "subscript" => self.run_style = self.run_style.clone().subscript(),
                        _ => {}
                    }
                }
            }
            b"sz" if self.in_run => {
                if let Some(val) = attr_value(e, b"val") {
                    if let Ok(half) = val.parse::<f32>() {
                        self.run_style = self.run_style.clone().font_size(half / 2.0);
                    }
                }
            }
            b"color" if self.in_run => {
                if let Some(val) = attr_value(e, b"val") {
                    if let Ok(color) = Color::from_hex(&val) {
                        self.run_style = self.run_style.clone().color_value(color);
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn on_end(
        &mut self,
        e: &quick_xml::events::BytesEnd,
        blocks: &mut Vec<DocxBlock>,
    ) -> Result<()> {
        match e.local_name().as_ref() {
            b"t" => self.in_text = false,
            b"r" => {
                if self.in_run {
                    self.flush_run();
                    self.in_run = false;
                }
            }
            b"p" => {
                if self.in_paragraph {
                    if self.tc_open {
                        self.end_cell_paragraph();
                    } else {
                        self.flush_paragraph(blocks)?;
                    }
                }
            }
            b"tc" if self.tc_open => {
                self.tc_open = false;
                if self.cell_has_shd {
                    self.current_row.header_cells = true;
                }
            }
            b"tr" if self.tr_open => {
                self.tr_open = false;
                self.table_rows.push(std::mem::take(&mut self.current_row));
            }
            b"tbl" => {
                self.tbl_depth = self.tbl_depth.saturating_sub(1);
                if self.tbl_depth == 0 {
                    blocks.push(DocxBlock::Table(self.finish_table()));
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn on_empty(&mut self, e: &quick_xml::events::BytesStart) -> Result<()> {
        self.on_start(e)
    }

    fn on_text(&mut self, text: &str) {
        if self.in_text && self.in_run {
            self.run_text.push_str(text);
        }
    }

    fn start_paragraph(&mut self) {
        self.in_paragraph = true;
        self.p_style = Style::new();
        self.para_is_list = false;
        self.runs.clear();
        self.run_text.clear();
        self.run_style = Style::new();
    }

    fn end_cell_paragraph(&mut self) {
        self.flush_run();
        let text = self.runs.iter().map(|r| r.text()).collect::<String>();
        if let Some(cell) = self.current_row.cells.last_mut() {
            if !cell.is_empty() && !text.is_empty() {
                cell.push(' ');
            }
            cell.push_str(&text);
        } else {
            self.current_row.cells.push(text);
        }
        self.runs.clear();
        self.in_paragraph = false;
    }

    fn flush_run(&mut self) {
        if !self.run_text.is_empty() {
            self.runs.push(
                TextRun::new(std::mem::take(&mut self.run_text)).style(self.run_style.clone()),
            );
        }
    }

    fn flush_paragraph(&mut self, blocks: &mut Vec<DocxBlock>) -> Result<()> {
        self.in_paragraph = false;
        if let Some(rid) = self.pending_image_rid.take() {
            if let Some(image) = self.load_image(&rid)? {
                blocks.push(DocxBlock::Image(image));
                return Ok(());
            }
        }
        self.flush_run();
        if self.runs.is_empty() {
            return Ok(());
        }
        if self.tbl_depth > 0 {
            return Ok(());
        }
        if self.para_is_list {
            let text = self.runs.iter().map(|r| r.text()).collect::<String>();
            self.runs.clear();
            if let Some(DocxBlock::List(list)) = blocks.last_mut() {
                list.push_item(text);
            } else {
                blocks.push(DocxBlock::List(List::new().item(text)));
            }
            return Ok(());
        }
        let mut runs = std::mem::take(&mut self.runs);
        if runs.len() == 1 {
            let style = self.p_style.clone();
            runs[0] = TextRun::new(runs[0].text()).style(style);
        }
        blocks.push(DocxBlock::Paragraph(Paragraph::from_runs(
            self.p_style.clone(),
            runs,
        )));
        Ok(())
    }

    fn finish_table(&self) -> Table {
        let mut columns = Vec::new();
        let mut rows = Vec::new();
        let mut style = docrafter_core::TableStyle::default();
        for (i, row) in self.table_rows.iter().enumerate() {
            if i == 0 && row.header_cells {
                columns = row.cells.clone();
                style = docrafter_core::TableStyle::professional();
            } else {
                rows.push(row.cells.clone());
            }
        }
        if columns.is_empty() && !self.table_rows.is_empty() {
            rows = self.table_rows.iter().map(|r| r.cells.clone()).collect();
        }
        Table {
            columns,
            rows,
            style,
            column_widths: Vec::new(),
            repeat_header_on_new_page: false,
        }
    }

    fn load_image(&self, rid: &str) -> Result<Option<Image>> {
        let Some(target) = self.archive.rels.get(rid) else {
            return Ok(None);
        };
        let path = format!("word/{}", target.trim_start_matches('/'));
        let Some(data) = self
            .archive
            .media
            .get(&path)
            .or_else(|| self.archive.media.get(target))
        else {
            return Ok(None);
        };
        Ok(Some(Image::from_bytes(data.clone())))
    }
}

fn parse_relationships(xml: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if xml.is_empty() {
        return map;
    }
    let mut reader = Reader::from_str(xml);
    loop {
        match reader.read_event() {
            Ok(Event::Empty(e)) | Ok(Event::Start(e)) => {
                if e.local_name().as_ref() == b"Relationship" {
                    let id = attr_value(&e, b"Id");
                    let target = attr_value(&e, b"Target");
                    if let (Some(id), Some(target)) = (id, target) {
                        map.insert(id, target);
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }
    map
}

fn attr_value(e: &quick_xml::events::BytesStart, name: &[u8]) -> Option<String> {
    e.attributes()
        .filter_map(|a| a.ok())
        .find(|a| a.key.local_name().as_ref() == name)
        .map(|a| String::from_utf8_lossy(&a.value).into_owned())
}

fn parse_align(val: &str) -> docrafter_core::Alignment {
    use docrafter_core::Alignment;
    match val {
        "center" => Alignment::Center,
        "right" | "end" => Alignment::End,
        "both" | "justify" => Alignment::Justify,
        _ => Alignment::Start,
    }
}

fn read_zip_text(zip: &mut ZipArchive<Cursor<&[u8]>>, name: &str) -> Result<String> {
    let mut file = zip
        .by_name(name)
        .map_err(|_| Error::Docx(format!("missing {name}")))?;
    let mut s = String::new();
    file.read_to_string(&mut s)
        .map_err(|e| Error::Docx(format!("read {name}: {e}")))?;
    Ok(s)
}

fn read_zip_text_optional(zip: &mut ZipArchive<Cursor<&[u8]>>, name: &str) -> Option<String> {
    let mut file = zip.by_name(name).ok()?;
    let mut s = String::new();
    file.read_to_string(&mut s).ok()?;
    Some(s)
}

fn xml_err(e: impl std::fmt::Display) -> Error {
    Error::Docx(format!("XML parse error: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::package::pack_docx;
    use crate::write::build_document_xml;
    use crate::DocxBlock;
    use docrafter_office::{Paragraph, Table};

    #[test]
    fn roundtrip_paragraph_and_table_order() {
        let blocks = vec![
            DocxBlock::Paragraph(Paragraph::new("Before")),
            DocxBlock::Table(Table::professional().columns(["A", "B"]).row(["1", "2"])),
            DocxBlock::Paragraph(Paragraph::new("After")),
        ];
        let xml = build_document_xml(&blocks, &[], &[]);
        let docx = pack_docx(&xml, true, &[], &[]).unwrap();
        let archive = load_archive(&docx).unwrap();
        let parsed = parse_body(&archive).unwrap();
        assert_eq!(parsed.len(), 3);
        assert!(matches!(&parsed[0], DocxBlock::Paragraph(p) if p.text() == "Before"));
        assert!(matches!(&parsed[2], DocxBlock::Paragraph(p) if p.text() == "After"));
    }
}
