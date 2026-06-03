//! Flow layout into pages.

use docrafter_core::Style;

use crate::config::LayoutConfig;
use crate::input::{FlowInput, ImageInput, ParagraphInput, SpacerInput, TableInput};
use crate::table_layout::{compute_column_widths, paginate_table, slice_height, table_row_height};
use crate::wrap_runs::{align_styled_line, wrap_styled_runs};

/// One styled text fragment on a line.
#[derive(Debug, Clone)]
pub struct TextSegmentPlacement {
    /// Text to draw.
    pub text: String,
    /// Character style.
    pub style: Style,
    /// X baseline origin (points).
    pub x: f32,
}

/// A positioned element ready for PDF rendering.
#[derive(Debug, Clone)]
pub enum LayoutPlacement {
    /// Text line with one or more styled runs.
    TextLine {
        /// Segments left-to-right.
        segments: Vec<TextSegmentPlacement>,
        /// Baseline Y (points, from bottom).
        y: f32,
    },
    /// Vertical gap (advances cursor only; no drawing).
    Spacer {
        /// Height in points.
        height: f32,
    },
    /// Table block.
    Table {
        /// Column headers.
        columns: Vec<String>,
        /// Data rows.
        rows: Vec<Vec<String>>,
        /// Table style.
        style: docrafter_core::TableStyle,
        /// Per-column widths (points).
        column_widths: Vec<f32>,
        /// Left edge.
        x: f32,
        /// Top edge (from bottom).
        y: f32,
        /// Total width.
        width: f32,
        /// Total height.
        height: f32,
    },
    /// Image draw rectangle.
    Image {
        /// Image payload.
        input: ImageInput,
        /// Left edge.
        x: f32,
        /// Bottom edge.
        y: f32,
        /// Width.
        width: f32,
        /// Height.
        height: f32,
    },
}

/// One laid-out page.
#[derive(Debug, Clone, Default)]
pub struct LayoutPage {
    /// Elements in paint order.
    pub placements: Vec<LayoutPlacement>,
}

/// Lay out flow items into one or more pages.
#[must_use]
pub fn layout_flow(config: LayoutConfig<'_>, items: &[FlowInput]) -> Vec<LayoutPage> {
    let margins = config.margins;
    let page_size = config.page_size;
    let measurer = config.measurer;
    let content_width = config.content_width();
    let mut pages = vec![LayoutPage::default()];
    let (_, y0, _, y1) = page_size.media_box();
    let mut cursor_y = y1 - margins.top;

    for item in items {
        match item {
            FlowInput::PageBreak => {
                new_page(&mut pages, y1, margins.top, &mut cursor_y);
            }
            FlowInput::Spacer(SpacerInput { height }) => {
                ensure_space(&mut pages, &mut cursor_y, y0, y1, margins, *height);
                pages
                    .last_mut()
                    .expect("page")
                    .placements
                    .push(LayoutPlacement::Spacer { height: *height });
                cursor_y -= *height;
            }
            FlowInput::Paragraph(paragraph) => {
                layout_paragraph(
                    paragraph,
                    &mut pages,
                    &mut cursor_y,
                    y0,
                    y1,
                    margins,
                    measurer,
                    content_width,
                );
            }
            FlowInput::List(list) => {
                for (i, item) in list.items.iter().enumerate() {
                    let numbered =
                        ParagraphInput::single(format!("{}. {item}", i + 1), Style::new());
                    layout_paragraph(
                        &numbered,
                        &mut pages,
                        &mut cursor_y,
                        y0,
                        y1,
                        margins,
                        measurer,
                        content_width,
                    );
                }
            }
            FlowInput::Table(table) => {
                layout_table(
                    &mut pages,
                    &mut cursor_y,
                    y0,
                    y1,
                    margins,
                    measurer,
                    table,
                    content_width,
                );
            }
            FlowInput::Image(image) => {
                let (w, h) = image_dimensions(image);
                ensure_space(&mut pages, &mut cursor_y, y0, y1, margins, h);
                let x = margins.left;
                let y = cursor_y - h;
                pages
                    .last_mut()
                    .expect("page")
                    .placements
                    .push(LayoutPlacement::Image {
                        input: image.clone(),
                        x,
                        y,
                        width: w,
                        height: h,
                    });
                cursor_y -= h + 8.0;
            }
        }
    }

    if pages.len() > 1 && pages.last().is_some_and(|p| p.placements.is_empty()) {
        pages.pop();
    }
    pages
}

fn new_page(pages: &mut Vec<LayoutPage>, y1: f32, top_margin: f32, cursor_y: &mut f32) {
    pages.push(LayoutPage::default());
    *cursor_y = y1 - top_margin;
}

fn ensure_space(
    pages: &mut Vec<LayoutPage>,
    cursor_y: &mut f32,
    y0: f32,
    y1: f32,
    margins: crate::config::LayoutMargins,
    needed: f32,
) {
    if *cursor_y - needed < y0 + margins.bottom {
        new_page(pages, y1, margins.top, cursor_y);
    }
}

#[allow(clippy::too_many_arguments)]
fn layout_table(
    pages: &mut Vec<LayoutPage>,
    cursor_y: &mut f32,
    y0: f32,
    y1: f32,
    margins: crate::config::LayoutMargins,
    measurer: Option<&dyn docrafter_font::TextMeasurer>,
    table: &TableInput,
    content_width: f32,
) {
    let font_size = table.style.effective_font_size();
    let column_widths = if table.column_widths.is_empty() {
        compute_column_widths(table, content_width, font_size, measurer)
    } else {
        table.column_widths.clone()
    };
    let row_height = table_row_height(&table.style);
    let available = *cursor_y - (y0 + margins.bottom);
    let slices = paginate_table(
        table,
        column_widths,
        row_height,
        available.max(row_height),
        table.repeat_header_on_new_page,
    );

    for slice in slices {
        let h = slice_height(&slice, row_height);
        ensure_space(pages, cursor_y, y0, y1, margins, h);
        let width: f32 = slice.column_widths.iter().sum();
        pages
            .last_mut()
            .expect("page")
            .placements
            .push(LayoutPlacement::Table {
                columns: slice.columns,
                rows: slice.rows,
                style: slice.style,
                column_widths: slice.column_widths,
                x: margins.left,
                y: *cursor_y,
                width,
                height: h,
            });
        *cursor_y -= h + 12.0;
    }
}

#[allow(clippy::too_many_arguments)]
fn layout_paragraph(
    paragraph: &ParagraphInput,
    pages: &mut Vec<LayoutPage>,
    cursor_y: &mut f32,
    y0: f32,
    y1: f32,
    margins: crate::config::LayoutMargins,
    measurer: Option<&dyn docrafter_font::TextMeasurer>,
    content_width: f32,
) {
    let align = paragraph.paragraph_style.effective_align();
    let mut lines = wrap_styled_runs(
        &paragraph.runs,
        &paragraph.paragraph_style,
        content_width,
        measurer,
    );
    for line in &mut lines {
        align_styled_line(line, margins.left, content_width, align, measurer);
        ensure_space(pages, cursor_y, y0, y1, margins, line.line_height);
        let segments = line
            .segments
            .iter()
            .map(|s| TextSegmentPlacement {
                text: s.text.clone(),
                style: s.style.clone(),
                x: s.x,
            })
            .collect();
        pages
            .last_mut()
            .expect("page")
            .placements
            .push(LayoutPlacement::TextLine {
                segments,
                y: *cursor_y,
            });
        *cursor_y -= line.line_height;
    }
}

fn image_dimensions(image: &ImageInput) -> (f32, f32) {
    (image.width.unwrap_or(120.0), image.height.unwrap_or(80.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::ParagraphInput;

    #[test]
    fn page_break_creates_second_page() {
        let items = vec![
            FlowInput::Paragraph(ParagraphInput::single("Page one", Style::new())),
            FlowInput::PageBreak,
            FlowInput::Paragraph(ParagraphInput::single("Page two", Style::new())),
        ];
        let pages = layout_flow(LayoutConfig::a4(), &items);
        assert_eq!(pages.len(), 2);
    }

    #[test]
    fn long_paragraph_wraps_to_multiple_lines() {
        let items = vec![FlowInput::Paragraph(ParagraphInput::single(
            "word ".repeat(80),
            Style::new().font_size(12.0),
        ))];
        let pages = layout_flow(LayoutConfig::a4(), &items);
        let text_lines: usize = pages[0]
            .placements
            .iter()
            .filter(|p| matches!(p, LayoutPlacement::TextLine { .. }))
            .count();
        assert!(text_lines > 3);
    }
}
