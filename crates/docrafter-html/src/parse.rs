//! Minimal HTML parser (no browser engine).

use docrafter_core::{Alignment, Color, Error, Length, Style};
use docrafter_office::{List, OfficeDocument, Paragraph, Table};

/// Parse HTML fragment into an office document.
///
/// Supported tags: `p`, `div`, `span`, `blockquote`, `h1`–`h3`, `a`, `hr`, `b`/`strong`,
/// `i`/`em`, `u`, `s`/`del`/`strike`, `br`, `table`, `tr`, `th`/`td` (`colspan`), `ul`/`ol`,
/// `li`, inline `style` (`color`, `font-weight`, `font-style`, `font-size`, `text-align`,
/// `line-height`, `margin-top`, `text-decoration`).
pub fn html_to_office(html: &str) -> docrafter_core::Result<OfficeDocument> {
    let mut parser = Parser::new();
    for token in tokenize(html) {
        parser.apply(token);
    }
    Ok(parser.finish())
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Text(String),
    Open {
        name: String,
        style: Option<String>,
        href: Option<String>,
        colspan: Option<u32>,
        rowspan: Option<u32>,
    },
    Close(String),
}

struct Parser {
    doc: OfficeDocument,
    para_style: Style,
    run_style: Style,
    para: Option<Paragraph>,
    table: Option<Table>,
    row: Vec<String>,
    cell: String,
    in_cell: bool,
    row_is_header: bool,
    list: Option<List>,
    list_item: String,
    in_li: bool,
    cell_colspan: u32,
    cell_rowspan: u32,
    row_spans: Vec<u8>,
    table_col: usize,
}

impl Parser {
    fn new() -> Self {
        Self {
            doc: OfficeDocument::new(),
            para_style: Style::new(),
            run_style: Style::new(),
            para: None,
            table: None,
            row: Vec::new(),
            cell: String::new(),
            in_cell: false,
            row_is_header: false,
            list: None,
            list_item: String::new(),
            in_li: false,
            cell_colspan: 1,
            cell_rowspan: 1,
            row_spans: Vec::new(),
            table_col: 0,
        }
    }

    fn apply(&mut self, token: Token) {
        match token {
            Token::Text(t) => self.append_text(&decode_entities(&t)),
            Token::Open {
                name,
                style,
                href,
                colspan,
                rowspan,
            } => self.open_tag(&name, style.as_deref(), href.as_deref(), colspan, rowspan),
            Token::Close(name) => self.close_tag(&name),
        }
    }

    fn append_text(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        if self.in_cell {
            self.cell.push_str(text);
            return;
        }
        if self.in_li {
            self.list_item.push_str(text);
            return;
        }
        let style = self.para_style.clone();
        let run = self.run_style.clone();
        match &mut self.para {
            Some(p) => {
                let base = std::mem::replace(p, Paragraph::from_runs(style, vec![]));
                *p = base.run(text, run);
            }
            None => {
                self.para = Some(Paragraph::from_runs(style, vec![]).run(text, run));
            }
        }
    }

    fn open_tag(
        &mut self,
        name: &str,
        inline_style: Option<&str>,
        href: Option<&str>,
        colspan: Option<u32>,
        rowspan: Option<u32>,
    ) {
        match name {
            "a" => {
                if href.is_some() {
                    self.run_style = self.run_style.clone().color_value(Color::rgb(0, 0, 180));
                }
            }
            "hr" => {
                self.flush_para();
                self.para_style = Style::new().align(Alignment::Center);
                self.para = Some(
                    Paragraph::from_runs(self.para_style.clone(), vec![]).run("—", Style::new()),
                );
                self.flush_para();
            }
            "p" | "div" => {
                self.flush_para();
                self.para_style = Style::new();
            }
            "h1" => {
                self.flush_para();
                self.para_style = Style::heading1();
            }
            "h2" => {
                self.flush_para();
                self.para_style = Style::heading2();
            }
            "h3" => {
                self.flush_para();
                self.para_style = Style::heading2().font_size(12.0);
            }
            "blockquote" => {
                self.flush_para();
                self.para_style = Style::new().italic();
                self.run_style = Style::new().italic();
            }
            "span" => {}
            "b" | "strong" => self.run_style = self.run_style.clone().bold(),
            "i" | "em" => self.run_style = self.run_style.clone().italic(),
            "u" => self.run_style = self.run_style.clone().underline(),
            "s" | "del" | "strike" => self.run_style = self.run_style.clone().strikethrough(),
            "sup" => self.run_style = self.run_style.clone().superscript(),
            "sub" => self.run_style = self.run_style.clone().subscript(),
            "br" => self.flush_para(),
            "table" => {
                self.flush_para();
                self.table = Some(Table::new());
            }
            "tr" => self.begin_row(),
            "th" => {
                self.in_cell = true;
                self.row_is_header = true;
                self.cell_colspan = colspan.unwrap_or(1).max(1);
                self.cell_rowspan = rowspan.unwrap_or(1).max(1);
            }
            "td" => {
                self.in_cell = true;
                self.cell_colspan = colspan.unwrap_or(1).max(1);
                self.cell_rowspan = rowspan.unwrap_or(1).max(1);
            }
            "ul" | "ol" => {
                self.flush_para();
                self.list = Some(List::new());
            }
            "li" => {
                self.list_item.clear();
                self.in_li = true;
            }
            _ => {}
        }
        if let Some(css) = inline_style {
            self.apply_inline_style(css);
        }
    }

    fn close_tag(&mut self, name: &str) {
        match name {
            "p" | "div" | "h1" | "h2" | "h3" | "blockquote" => self.flush_para(),
            "a" => self.run_style = Style::new(),
            "b" | "strong" | "i" | "em" | "u" | "s" | "del" | "strike" | "sup" | "sub" => {
                self.run_style = Style::new();
            }
            "table" => self.flush_table(),
            "tr" => self.flush_row(),
            "th" | "td" => self.flush_cell(),
            "ul" | "ol" => {
                if let Some(list) = self.list.take().filter(|l| !l.items().is_empty()) {
                    self.doc.push_list(list);
                }
            }
            "li" if self.in_li => {
                if let Some(list) = self.list.as_mut().filter(|_| !self.list_item.is_empty()) {
                    list.push_item(&self.list_item);
                }
                self.list_item.clear();
                self.in_li = false;
            }
            _ => {}
        }
    }

    fn begin_row(&mut self) {
        self.row.clear();
        self.row_is_header = false;
        self.table_col = 0;
        while self.table_col < self.row_spans.len() {
            if self.row_spans[self.table_col] > 0 {
                self.row.push(String::new());
                self.row_spans[self.table_col] -= 1;
                self.table_col += 1;
            } else {
                break;
            }
        }
    }

    fn flush_cell(&mut self) {
        if self.in_cell {
            self.row.push(std::mem::take(&mut self.cell));
            let colspan = self.cell_colspan;
            for _ in 1..colspan {
                self.row.push(String::new());
            }
            if self.cell_rowspan > 1 {
                while self.row_spans.len() <= self.table_col {
                    self.row_spans.push(0);
                }
                self.row_spans[self.table_col] = self.cell_rowspan.saturating_sub(1).min(255) as u8;
            }
            self.table_col += colspan as usize;
            self.in_cell = false;
            self.cell_colspan = 1;
            self.cell_rowspan = 1;
        }
    }

    fn flush_para(&mut self) {
        if let Some(p) = self.para.take().filter(|p| !p.text().is_empty()) {
            self.doc.push(p);
        }
        self.para_style = Style::new();
        self.run_style = Style::new();
    }

    fn flush_row(&mut self) {
        self.flush_cell();
        if self.row.is_empty() {
            return;
        }
        if let Some(table) = self.table.as_mut() {
            if self.row_is_header && table.columns.is_empty() {
                table.columns = self.row.clone();
            } else {
                table.rows.push(self.row.clone());
            }
        }
        self.row.clear();
        self.row_is_header = false;
    }

    fn flush_table(&mut self) {
        self.flush_row();
        if let Some(table) = self
            .table
            .take()
            .filter(|t| !t.columns.is_empty() || !t.rows.is_empty())
        {
            self.doc.push_table(table);
        }
    }

    fn apply_inline_style(&mut self, css: &str) {
        for decl in css.split(';') {
            let Some((prop, value)) = decl.split_once(':') else {
                continue;
            };
            let prop = prop.trim().to_ascii_lowercase();
            let value = value.trim();
            match prop.as_str() {
                "color" => {
                    if let Ok(c) = parse_css_color(value) {
                        self.run_style = self.run_style.clone().color_value(c);
                    }
                }
                "font-weight" if value == "bold" || value == "700" || value == "800" => {
                    self.run_style = self.run_style.clone().bold();
                }
                "font-style" if value == "italic" => {
                    self.run_style = self.run_style.clone().italic();
                }
                "text-decoration" if value.contains("underline") => {
                    self.run_style = self.run_style.clone().underline();
                }
                "text-decoration" if value.contains("line-through") => {
                    self.run_style = self.run_style.clone().strikethrough();
                }
                "vertical-align" if value == "super" => {
                    self.run_style = self.run_style.clone().superscript();
                }
                "vertical-align" if value == "sub" => {
                    self.run_style = self.run_style.clone().subscript();
                }
                "font-size" => {
                    if let Ok(pt) = parse_font_size_pt(value) {
                        self.run_style = self.run_style.clone().font_size(pt);
                        self.para_style = self.para_style.clone().font_size(pt);
                    }
                }
                "text-align" => {
                    if let Some(align) = parse_text_align(value) {
                        self.para_style = self.para_style.clone().align(align);
                    }
                }
                "line-height" | "margin-top" => {
                    if let Some(lh) =
                        parse_line_height(value, self.para_style.effective_font_size())
                    {
                        self.para_style = self.para_style.clone().line_height(lh);
                    }
                }
                _ => {}
            }
        }
    }

    fn finish(mut self) -> OfficeDocument {
        self.flush_para();
        self.flush_table();
        if let Some(list) = self.list.take().filter(|l| !l.items().is_empty()) {
            self.doc.push_list(list);
        }
        self.doc
    }
}

fn parse_css_color(value: &str) -> docrafter_core::Result<Color> {
    let v = value.trim();
    if v.starts_with('#') || (v.len() == 6 && v.chars().all(|c| c.is_ascii_hexdigit())) {
        return Color::from_hex(v);
    }
    match v.to_ascii_lowercase().as_str() {
        "red" => Ok(Color::rgb(200, 0, 0)),
        "blue" => Ok(Color::rgb(0, 0, 200)),
        "green" => Ok(Color::rgb(0, 120, 0)),
        "gray" | "grey" => Ok(Color::rgb(100, 100, 100)),
        _ => Err(Error::InvalidInput(format!(
            "unsupported CSS color: {value}"
        ))),
    }
}

fn parse_line_height(value: &str, base_font_pt: f32) -> Option<Length> {
    let v = value.trim();
    if let Ok(mult) = v.parse::<f32>() {
        if mult > 0.0 {
            return Some(Length::pt(base_font_pt * mult));
        }
    }
    parse_length_pt(v).ok()
}

fn parse_length_pt(value: &str) -> docrafter_core::Result<Length> {
    let v = value.trim();
    if let Some(num) = v.strip_suffix("pt") {
        let pt: f32 = num
            .trim()
            .parse()
            .map_err(|_| Error::InvalidInput(format!("invalid length: {value}")))?;
        return Ok(Length::pt(pt));
    }
    if let Some(num) = v.strip_suffix("px") {
        let px: f32 = num
            .trim()
            .parse()
            .map_err(|_| Error::InvalidInput(format!("invalid length: {value}")))?;
        return Ok(Length::pt(px * 0.75));
    }
    Err(Error::InvalidInput(format!(
        "length must use pt, px, or a unitless multiplier, got {value}"
    )))
}

fn parse_text_align(value: &str) -> Option<Alignment> {
    match value.trim().to_ascii_lowercase().as_str() {
        "left" | "start" => Some(Alignment::Start),
        "center" | "centre" => Some(Alignment::Center),
        "right" | "end" => Some(Alignment::End),
        "justify" => Some(Alignment::Justify),
        _ => None,
    }
}

fn parse_font_size_pt(value: &str) -> docrafter_core::Result<f32> {
    let v = value.trim();
    if let Some(num) = v.strip_suffix("pt") {
        return num
            .trim()
            .parse::<f32>()
            .map_err(|_| Error::InvalidInput(format!("invalid font-size: {value}")));
    }
    if let Some(num) = v.strip_suffix("px") {
        let px: f32 = num
            .trim()
            .parse()
            .map_err(|_| Error::InvalidInput(format!("invalid font-size: {value}")))?;
        return Ok(px * 0.75);
    }
    Err(Error::InvalidInput(format!(
        "font-size must use pt or px, got {value}"
    )))
}

fn parse_quoted_attr(tag: &str, attr: &str) -> Option<String> {
    let lower = tag.to_ascii_lowercase();
    let needle = format!("{attr}=");
    let idx = lower.find(&needle)?;
    let quote = lower.as_bytes().get(idx + needle.len())?;
    let q = *quote as char;
    if q != '"' && q != '\'' {
        return None;
    }
    let rest = &tag[idx + needle.len() + 1..];
    let end = rest.find(q)?;
    Some(rest[..end].to_string())
}

fn parse_style_attr(tag: &str) -> Option<String> {
    parse_quoted_attr(tag, "style")
}

fn parse_span_attr(tag: &str, attr: &str) -> Option<u32> {
    let raw = parse_quoted_attr(tag, attr).or_else(|| {
        tag.split_whitespace().find_map(|part| {
            let prefix = format!("{attr}=");
            let v = part.strip_prefix(prefix.as_str())?;
            Some(v.trim_matches(|c| c == '"' || c == '\'').to_string())
        })
    })?;
    raw.parse().ok().filter(|&n| n >= 1)
}

fn parse_colspan(tag: &str) -> Option<u32> {
    parse_span_attr(tag, "colspan")
}

fn parse_rowspan(tag: &str) -> Option<u32> {
    parse_span_attr(tag, "rowspan")
}

fn decode_entities(s: &str) -> String {
    s.replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
}

fn tokenize(html: &str) -> Vec<Token> {
    let mut out = Vec::new();
    let mut i = 0;
    let bytes = html.as_bytes();
    while i < bytes.len() {
        if bytes[i] == b'<' {
            if let Some(end) = html[i..].find('>') {
                let tag = &html[i + 1..i + end];
                let close = tag.starts_with('/');
                let name = tag
                    .trim_start_matches('/')
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .trim_end_matches('/')
                    .to_ascii_lowercase();
                if !name.is_empty() {
                    if close {
                        out.push(Token::Close(name));
                    } else {
                        out.push(Token::Open {
                            name: name.clone(),
                            style: parse_style_attr(tag),
                            href: parse_quoted_attr(tag, "href"),
                            colspan: parse_colspan(tag),
                            rowspan: parse_rowspan(tag),
                        });
                        if name == "hr" {
                            out.push(Token::Close("hr".into()));
                        }
                    }
                }
                i += end + 1;
                continue;
            }
        }
        let start = i;
        while i < bytes.len() && bytes[i] != b'<' {
            i += 1;
        }
        if i > start {
            let text = html[start..i].trim();
            if !text.is_empty() {
                out.push(Token::Text(text.to_string()));
            }
        } else {
            i += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use docrafter_office::OfficeBlock;

    #[test]
    fn parses_heading_and_paragraph() {
        let doc = html_to_office("<h1>Title</h1><p>Hello <b>world</b></p>").unwrap();
        assert_eq!(doc.blocks().len(), 2);
        match &doc.blocks()[0] {
            OfficeBlock::Paragraph(p) => assert!(p.text().contains("Title")),
            _ => panic!("expected paragraph"),
        }
    }

    #[test]
    fn parses_inline_style() {
        let doc = html_to_office(r#"<p style="color: #ff0000; font-weight: bold">Hi</p>"#).unwrap();
        match &doc.blocks()[0] {
            OfficeBlock::Paragraph(p) => {
                let run = &p.runs()[0];
                assert!(run.resolved_style().is_bold());
            }
            _ => panic!("expected paragraph"),
        }
    }

    #[test]
    fn parses_text_align_on_paragraph() {
        let doc = html_to_office(r#"<p style="text-align: center">Mid</p>"#).unwrap();
        match &doc.blocks()[0] {
            OfficeBlock::Paragraph(p) => {
                assert_eq!(p.paragraph_style().effective_align(), Alignment::Center);
            }
            _ => panic!("expected paragraph"),
        }
    }

    #[test]
    fn parses_div_block() {
        let doc = html_to_office("<div>Block</div>").unwrap();
        assert_eq!(doc.blocks().len(), 1);
    }

    #[test]
    fn parses_anchor_link() {
        let doc = html_to_office(r#"<p>See <a href="https://example.com">docs</a></p>"#).unwrap();
        match &doc.blocks()[0] {
            OfficeBlock::Paragraph(p) => {
                assert!(p.text().contains("docs"));
                assert!(p.runs().iter().any(|r| r.text() == "docs"));
            }
            _ => panic!("expected paragraph"),
        }
    }

    #[test]
    fn parses_table_colspan() {
        let doc = html_to_office(
            "<table><tr><th colspan=\"2\">AB</th></tr><tr><td>1</td><td>2</td></tr></table>",
        )
        .unwrap();
        match &doc.blocks()[0] {
            OfficeBlock::Table(t) => {
                assert_eq!(t.columns.len(), 2);
                assert_eq!(t.columns[0], "AB");
                assert_eq!(t.columns[1], "");
            }
            _ => panic!("expected table"),
        }
    }

    #[test]
    fn parses_superscript() {
        let doc = html_to_office("<p>E=mc<sup>2</sup></p>").unwrap();
        match &doc.blocks()[0] {
            OfficeBlock::Paragraph(p) => {
                assert!(p.runs().iter().any(|r| r.resolved_style().is_superscript()));
            }
            _ => panic!("expected paragraph"),
        }
    }

    #[test]
    fn parses_table_rowspan() {
        let doc = html_to_office(
            "<table><tr><td rowspan=\"2\">A</td><td>1</td></tr><tr><td>2</td></tr></table>",
        )
        .unwrap();
        match &doc.blocks()[0] {
            OfficeBlock::Table(t) => {
                assert_eq!(t.rows.len(), 2);
                assert_eq!(t.rows[0][0], "A");
                assert_eq!(t.rows[1][0], "");
                assert_eq!(t.rows[1][1], "2");
            }
            _ => panic!("expected table"),
        }
    }

    #[test]
    fn parses_strikethrough() {
        let doc = html_to_office("<p><del>old</del></p>").unwrap();
        match &doc.blocks()[0] {
            OfficeBlock::Paragraph(p) => {
                assert!(p
                    .runs()
                    .iter()
                    .any(|r| r.resolved_style().is_strikethrough()));
            }
            _ => panic!("expected paragraph"),
        }
    }

    #[test]
    fn parses_underline() {
        let doc = html_to_office(
            r#"<p><u>line</u> and <span style="text-decoration: underline">x</span></p>"#,
        )
        .unwrap();
        match &doc.blocks()[0] {
            OfficeBlock::Paragraph(p) => {
                assert!(p.runs().iter().any(|r| r.resolved_style().is_underline()));
            }
            _ => panic!("expected paragraph"),
        }
    }

    #[test]
    fn parses_blockquote() {
        let doc = html_to_office("<blockquote>Note</blockquote>").unwrap();
        match &doc.blocks()[0] {
            OfficeBlock::Paragraph(p) => assert!(p.paragraph_style().is_italic()),
            _ => panic!("expected paragraph"),
        }
    }

    #[test]
    fn parses_line_height() {
        let doc = html_to_office(r#"<p style="line-height: 2">Tall</p>"#).unwrap();
        match &doc.blocks()[0] {
            OfficeBlock::Paragraph(p) => {
                assert!(p.paragraph_style().effective_line_height() >= 24.0);
            }
            _ => panic!("expected paragraph"),
        }
    }

    #[test]
    fn parses_hr() {
        let doc = html_to_office("<p>Above</p><hr><p>Below</p>").unwrap();
        assert_eq!(doc.blocks().len(), 3);
    }

    #[test]
    fn parses_table() {
        let doc = html_to_office(
            "<table><tr><th>A</th><th>B</th></tr><tr><td>1</td><td>2</td></tr></table>",
        )
        .unwrap();
        match &doc.blocks()[0] {
            OfficeBlock::Table(t) => {
                assert_eq!(t.columns, vec!["A", "B"]);
                assert_eq!(t.rows[0], vec!["1", "2"]);
            }
            _ => panic!("expected table"),
        }
    }
}
