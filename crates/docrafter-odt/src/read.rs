//! Parse OpenDocument `content.xml`.

use std::collections::HashMap;
use std::io::{Cursor, Read};

use docrafter_core::{Color, Error, Result, Style};
use quick_xml::events::Event;
use quick_xml::Reader;
use zip::ZipArchive;

use docrafter_office::OfficeBlock;
use docrafter_office::{Image, List, Paragraph, Table, TextRun};

use crate::styles::style_from_paragraph_name;

/// Loaded ODT parts.
pub struct OdtArchive {
    /// `content.xml` body.
    pub content_xml: String,
    /// `Pictures/image1.png` → bytes.
    pub pictures: HashMap<String, Vec<u8>>,
}

/// Load `.odt` bytes.
pub fn load_archive(bytes: &[u8]) -> Result<OdtArchive> {
    let cursor = Cursor::new(bytes);
    let mut zip =
        ZipArchive::new(cursor).map_err(|e| Error::Odt(format!("invalid odt zip: {e}")))?;
    let content_xml = read_zip_text(&mut zip, "content.xml")?;
    let mut pictures = HashMap::new();
    for i in 0..zip.len() {
        let mut file = zip
            .by_index(i)
            .map_err(|e| Error::Odt(format!("zip entry {i}: {e}")))?;
        let name = file.name().to_string();
        if name.starts_with("Pictures/") {
            let mut data = Vec::new();
            file.read_to_end(&mut data)
                .map_err(|e| Error::Odt(format!("read {name}: {e}")))?;
            pictures.insert(name, data);
        }
    }
    Ok(OdtArchive {
        content_xml,
        pictures,
    })
}

/// Parse body blocks in order.
pub fn parse_body(archive: &OdtArchive) -> Result<Vec<OfficeBlock>> {
    let mut reader = Reader::from_str(&archive.content_xml);
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
        return Err(Error::Odt("document has no body content".into()));
    }
    Ok(blocks)
}

#[derive(Clone)]
struct TableRowParsed {
    cells: Vec<String>,
    header: bool,
}

struct Parser<'a> {
    archive: &'a OdtArchive,
    in_list: bool,
    list_depth: u32,
    tbl_depth: u32,
    tr_open: bool,
    tc_open: bool,
    row_is_header: bool,
    table_rows: Vec<TableRowParsed>,
    current_row: Vec<String>,
    in_paragraph: bool,
    in_span: bool,
    p_style: Style,
    p_style_name: Option<String>,
    runs: Vec<TextRun>,
    run_text: String,
    run_style: Style,
    pending_href: Option<String>,
}

impl<'a> Parser<'a> {
    fn new(archive: &'a OdtArchive) -> Self {
        Self {
            archive,
            in_list: false,
            list_depth: 0,
            tbl_depth: 0,
            tr_open: false,
            tc_open: false,
            row_is_header: false,
            table_rows: Vec::new(),
            current_row: Vec::new(),
            in_paragraph: false,
            in_span: false,
            p_style: Style::new(),
            p_style_name: None,
            runs: Vec::new(),
            run_text: String::new(),
            run_style: Style::new(),
            pending_href: None,
        }
    }

    fn on_start(&mut self, e: &quick_xml::events::BytesStart) -> Result<()> {
        match e.local_name().as_ref() {
            b"list" => {
                self.in_list = true;
                self.list_depth += 1;
            }
            b"table" => {
                self.tbl_depth += 1;
            }
            b"table-row" => {
                self.tr_open = true;
                self.current_row.clear();
                self.row_is_header = false;
            }
            b"table-cell" => {
                self.tc_open = true;
                if attr_value(e, b"style-name").is_some_and(|s| s.contains("TableHeader")) {
                    self.row_is_header = true;
                }
                self.current_row.push(String::new());
            }
            b"p" if is_text_ns(e) => self.start_paragraph(e),
            b"span" if self.in_paragraph => {
                self.flush_run();
                self.in_span = true;
                self.run_text.clear();
                self.run_style = parse_span_attrs(e);
            }
            b"image" if is_draw_ns(e) => {
                if let Some(href) = attr_value(e, b"href") {
                    self.pending_href = Some(href);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn on_end(
        &mut self,
        e: &quick_xml::events::BytesEnd,
        blocks: &mut Vec<OfficeBlock>,
    ) -> Result<()> {
        match e.local_name().as_ref() {
            b"span" => {
                self.flush_run();
                self.in_span = false;
            }
            b"p" if self.in_paragraph => {
                if self.tc_open {
                    self.end_cell_paragraph();
                } else if self.in_list && self.list_depth > 0 {
                    self.flush_list_item(blocks)?;
                } else {
                    self.flush_paragraph(blocks)?;
                }
            }
            b"table-cell" => {
                self.tc_open = false;
            }
            b"table-row" => {
                self.tr_open = false;
                self.table_rows.push(TableRowParsed {
                    cells: std::mem::take(&mut self.current_row),
                    header: self.row_is_header,
                });
            }
            b"table" => {
                self.tbl_depth = self.tbl_depth.saturating_sub(1);
                if self.tbl_depth == 0 {
                    blocks.push(OfficeBlock::Table(self.finish_table()));
                    self.table_rows.clear();
                }
            }
            b"list" => {
                self.list_depth = self.list_depth.saturating_sub(1);
                if self.list_depth == 0 {
                    self.in_list = false;
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
        if self.in_paragraph {
            self.run_text.push_str(text);
        }
    }

    fn start_paragraph(&mut self, e: &quick_xml::events::BytesStart) {
        self.in_paragraph = true;
        self.p_style_name = attr_value(e, b"style-name");
        self.p_style = self
            .p_style_name
            .as_deref()
            .map(style_from_paragraph_name)
            .unwrap_or_default();
        self.runs.clear();
        self.run_text.clear();
        self.run_style = Style::new();
    }

    fn end_cell_paragraph(&mut self) {
        self.flush_run();
        let text = self.runs.iter().map(|r| r.text()).collect::<String>();
        if let Some(cell) = self.current_row.last_mut() {
            *cell = text;
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

    fn flush_paragraph(&mut self, blocks: &mut Vec<OfficeBlock>) -> Result<()> {
        self.in_paragraph = false;
        if let Some(href) = self.pending_href.take() {
            if let Some(image) = self.load_image(&href) {
                blocks.push(OfficeBlock::Image(image));
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
        let mut runs = std::mem::take(&mut self.runs);
        if runs.len() == 1 {
            let style = self.p_style.clone();
            runs[0] = TextRun::new(runs[0].text()).style(style);
        }
        blocks.push(OfficeBlock::Paragraph(Paragraph::from_runs(
            self.p_style.clone(),
            runs,
        )));
        Ok(())
    }

    fn flush_list_item(&mut self, blocks: &mut Vec<OfficeBlock>) -> Result<()> {
        self.in_paragraph = false;
        self.flush_run();
        let text = self.runs.iter().map(|r| r.text()).collect::<String>();
        self.runs.clear();
        if text.is_empty() {
            return Ok(());
        }
        if let Some(OfficeBlock::List(list)) = blocks.last_mut() {
            list.push_item(text);
        } else {
            blocks.push(OfficeBlock::List(List::new().item(text)));
        }
        Ok(())
    }

    fn finish_table(&self) -> Table {
        let mut columns = Vec::new();
        let mut rows = Vec::new();
        let mut style = docrafter_core::TableStyle::default();
        for row in &self.table_rows {
            if row.header {
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

    fn load_image(&self, href: &str) -> Option<Image> {
        let path = href.trim_start_matches('/');
        let data = self
            .archive
            .pictures
            .get(path)
            .or_else(|| self.archive.pictures.get(&format!("Pictures/{path}")))?;
        Some(Image::from_bytes(data.clone()))
    }
}

fn is_text_ns(e: &quick_xml::events::BytesStart) -> bool {
    String::from_utf8_lossy(e.name().as_ref()).contains("text:")
}

fn is_draw_ns(e: &quick_xml::events::BytesStart) -> bool {
    String::from_utf8_lossy(e.name().as_ref()).contains("draw:")
}

fn parse_span_attrs(e: &quick_xml::events::BytesStart) -> Style {
    let mut style = Style::new();
    if let Some(fw) = attr_value(e, b"font-weight") {
        if fw == "bold" {
            style = style.bold();
        }
    }
    if let Some(fs) = attr_value(e, b"font-style") {
        if fs == "italic" {
            style = style.italic();
        }
    }
    if let Some(color) = attr_value(e, b"color") {
        if let Ok(c) = parse_odf_color(&color) {
            style = style.color_value(c);
        }
    }
    if let Some(size) = attr_value(e, b"font-size") {
        if let Some(pt) = size.strip_suffix("pt").and_then(|s| s.parse().ok()) {
            style = style.font_size(pt);
        }
    }
    if attr_value(e, b"text-underline-style").is_some()
        || attr_value(e, b"text-underline-type").is_some()
    {
        style = style.underline();
    }
    if attr_value(e, b"text-line-through-style").is_some()
        || attr_value(e, b"text-line-through-type").is_some()
    {
        style = style.strikethrough();
    }
    if let Some(pos) = attr_value(e, b"text-position") {
        if pos.starts_with("super") {
            style = style.superscript();
        } else if pos.starts_with("sub") {
            style = style.subscript();
        }
    }
    style
}

fn parse_odf_color(raw: &str) -> Result<Color> {
    let hex = raw.trim_start_matches('#');
    Color::from_hex(hex)
}

fn attr_value(e: &quick_xml::events::BytesStart, local: &[u8]) -> Option<String> {
    e.attributes()
        .filter_map(|a| a.ok())
        .find(|a| a.key.local_name().as_ref() == local)
        .map(|a| String::from_utf8_lossy(&a.value).into_owned())
}

fn read_zip_text(zip: &mut ZipArchive<Cursor<&[u8]>>, name: &str) -> Result<String> {
    let mut file = zip
        .by_name(name)
        .map_err(|_| Error::Odt(format!("missing {name}")))?;
    let mut s = String::new();
    file.read_to_string(&mut s)
        .map_err(|e| Error::Odt(format!("read {name}: {e}")))?;
    Ok(s)
}

fn xml_err(e: impl std::fmt::Display) -> Error {
    Error::Odt(format!("XML parse error: {e}"))
}
