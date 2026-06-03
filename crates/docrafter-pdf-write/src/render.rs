//! Layout → PDF rendering.

use docrafter_core::{Color, Error, PageSize, Result, Style, TableStyle};
use docrafter_font::{FontBundle, FONT_BOLD, FONT_REGULAR};
use docrafter_layout::{
    effective_run_style, layout_flow, FlowInput, ImageInput, LayoutConfig, LayoutMargins,
    LayoutPage, LayoutPlacement, ParagraphInput, SpacerInput, TableInput, TextRunInput,
};
use docrafter_office::Paragraph;
use image::GenericImageView;
use miniz_oxide::deflate::{compress_to_vec_zlib, CompressionLevel};
use pdf_writer::{Content, Filter, Finish, Name, Pdf, Rect, Ref, Str};

use crate::flow::FlowItem;
use crate::header_footer::{expand_page_template, PageHeaderFooter};

const IMAGE_NAMES: &[Name] = &[
    Name(b"Im1"),
    Name(b"Im2"),
    Name(b"Im3"),
    Name(b"Im4"),
    Name(b"Im5"),
    Name(b"Im6"),
    Name(b"Im7"),
    Name(b"Im8"),
];

/// Renders a flow document to PDF bytes.
pub struct PdfRenderer {
    page_size: PageSize,
    margins: LayoutMargins,
    header_footer: Option<PageHeaderFooter>,
    items: Vec<FlowItem>,
}

impl PdfRenderer {
    /// New renderer with default A4 layout.
    #[must_use]
    pub fn new(page_size: PageSize) -> Self {
        Self {
            page_size,
            margins: LayoutMargins::standard(),
            header_footer: None,
            items: Vec::new(),
        }
    }

    /// Override margins.
    #[must_use]
    pub fn with_margins(mut self, margins: LayoutMargins) -> Self {
        self.margins = margins;
        self
    }

    /// Running header/footer on every page.
    #[must_use]
    pub fn with_header_footer(mut self, hf: PageHeaderFooter) -> Self {
        self.header_footer = Some(hf);
        self
    }

    /// Queue a flow element.
    pub fn push(&mut self, item: impl Into<FlowItem>) {
        self.items.push(item.into());
    }

    /// Build PDF bytes.
    pub fn finish(self) -> Result<Vec<u8>> {
        let mut pdf = Pdf::new();
        let mut ids = IdGen::new();
        let mut next_ref = || ids.next();
        let fonts = FontBundle::dejavu_sans(&mut pdf, &mut next_ref)?;
        let config = LayoutConfig {
            page_size: self.page_size,
            margins: self.margins,
            measurer: Some(&fonts),
        };
        let inputs = self.to_flow_inputs();
        let pages = layout_flow(config, &inputs);
        if pages.is_empty() {
            return Err(Error::Pdf("no pages to render".into()));
        }
        write_pdf(
            pdf,
            &mut ids,
            self.page_size,
            self.margins,
            self.header_footer.as_ref(),
            &pages,
            &fonts,
        )
    }

    fn to_flow_inputs(&self) -> Vec<FlowInput> {
        self.items
            .iter()
            .map(|item| match item {
                FlowItem::Paragraph(p) => FlowInput::Paragraph(paragraph_input_from_office(p)),
                FlowItem::List(list) => FlowInput::List(docrafter_layout::ListInput {
                    items: list.items().to_vec(),
                }),
                FlowItem::Spacer(s) => FlowInput::Spacer(SpacerInput {
                    height: s.height_pt(),
                }),
                FlowItem::PageBreak(_) => FlowInput::PageBreak,
                FlowItem::Image(img) => FlowInput::Image(ImageInput {
                    data: img.data().to_vec(),
                    width: img.width_pt(),
                    height: img.height_pt(),
                }),
                FlowItem::Table(t) => FlowInput::Table(TableInput {
                    columns: t.columns.clone(),
                    rows: t.rows.clone(),
                    style: t.style.clone(),
                    column_widths: t.column_widths.clone(),
                    repeat_header_on_new_page: t.repeat_header_on_new_page,
                }),
            })
            .collect()
    }
}

struct IdGen(i32);

impl IdGen {
    fn new() -> Self {
        Self(2)
    }

    fn next(&mut self) -> Ref {
        self.0 += 1;
        Ref::new(self.0)
    }
}

struct ImageParts {
    id: Ref,
    encoded: Vec<u8>,
    filter: Filter,
    width: i32,
    height: i32,
    mask_id: Option<Ref>,
    mask_encoded: Option<Vec<u8>>,
}

struct BuiltPage {
    page_id: Ref,
    content_id: Ref,
    content: Vec<u8>,
    image_ids: Vec<Ref>,
    image_name_indices: Vec<usize>,
}

fn write_pdf(
    mut pdf: Pdf,
    ids: &mut IdGen,
    page_size: PageSize,
    margins: LayoutMargins,
    header_footer: Option<&PageHeaderFooter>,
    pages: &[LayoutPage],
    fonts: &FontBundle,
) -> Result<Vec<u8>> {
    let catalog_id = ids.next();
    let page_tree_id = ids.next();

    let (page_w, page_h) = {
        let b = page_size.media_box();
        (b.2, b.3)
    };

    let mut built = Vec::new();
    let mut global_image_idx = 0usize;

    let page_total = pages.len();

    for (page_idx, layout_page) in pages.iter().enumerate() {
        let page_id = ids.next();
        let content_id = ids.next();

        let mut content = Content::new();
        let mut page_image_ids = Vec::new();
        let mut page_image_names = Vec::new();

        for placement in &layout_page.placements {
            match placement {
                LayoutPlacement::TextLine { segments, y } => {
                    for seg in segments {
                        draw_text_line(&mut content, fonts, &seg.text, &seg.style, seg.x, *y);
                    }
                }
                LayoutPlacement::Spacer { .. } => {}
                LayoutPlacement::Table {
                    columns,
                    rows,
                    style,
                    column_widths,
                    x,
                    y,
                    width: _,
                    height,
                } => {
                    draw_table(
                        &mut content,
                        fonts,
                        columns,
                        rows,
                        style,
                        column_widths,
                        *x,
                        *y,
                        *height,
                    );
                }
                LayoutPlacement::Image {
                    input,
                    x,
                    y,
                    width,
                    height,
                } => {
                    let parts = embed_image(ids, input)?;
                    write_image_xobject(&mut pdf, &parts);
                    let name = IMAGE_NAMES
                        .get(global_image_idx)
                        .copied()
                        .ok_or_else(|| Error::Pdf("too many images in document".into()))?;
                    global_image_idx += 1;
                    page_image_ids.push(parts.id);
                    page_image_names.push(global_image_idx - 1);
                    draw_image(&mut content, name, *x, *y, *width, *height);
                }
            }
        }

        if let Some(hf) = header_footer.filter(|h| !h.is_empty()) {
            let hf_style = hf.draw_style();
            let page_num = page_idx + 1;
            if let Some(header) = &hf.header {
                let y = page_h - margins.top + hf.font_size;
                draw_text_line(&mut content, fonts, header, &hf_style, margins.left, y);
            }
            if let Some(footer_tpl) = &hf.footer {
                let footer = expand_page_template(footer_tpl, page_num, page_total);
                draw_text_line(
                    &mut content,
                    fonts,
                    &footer,
                    &hf_style,
                    margins.left,
                    margins.bottom - hf.font_size,
                );
            }
        }

        built.push(BuiltPage {
            page_id,
            content_id,
            content: content.finish(),
            image_ids: page_image_ids,
            image_name_indices: page_image_names,
        });
    }

    let page_refs: Vec<Ref> = built.iter().map(|p| p.page_id).collect();
    let page_count = page_refs.len() as i32;
    pdf.catalog(catalog_id).pages(page_tree_id);
    pdf.pages(page_tree_id).kids(page_refs).count(page_count);

    for page in &built {
        let mut pw = pdf.page(page.page_id);
        pw.parent(page_tree_id)
            .media_box(Rect::new(0.0, 0.0, page_w, page_h))
            .contents(page.content_id);
        {
            let mut resources = pw.resources();
            resources
                .fonts()
                .pair(FONT_REGULAR, fonts.regular.type0_ref)
                .pair(FONT_BOLD, fonts.bold.type0_ref);
            if !page.image_ids.is_empty() {
                let mut xo = resources.x_objects();
                for (i, id) in page.image_ids.iter().enumerate() {
                    let name = IMAGE_NAMES[page.image_name_indices[i]];
                    xo.pair(name, *id);
                }
            }
        }
        pw.finish();
    }

    for page in &built {
        pdf.stream(page.content_id, &page.content);
    }

    Ok(pdf.finish())
}

fn paragraph_input_from_office(p: &Paragraph) -> ParagraphInput {
    let paragraph_style = p.paragraph_style().clone();
    let runs = p
        .runs()
        .iter()
        .map(|r| TextRunInput {
            text: r.text().to_string(),
            style: effective_run_style(p.paragraph_style(), r.resolved_style()),
        })
        .collect();
    ParagraphInput {
        paragraph_style,
        runs,
    }
}

fn draw_text_line(
    content: &mut Content,
    fonts: &FontBundle,
    text: &str,
    style: &Style,
    x: f32,
    y: f32,
) {
    use docrafter_font::measure_text;

    let font_size = style.effective_font_size();
    let y = y + style.baseline_shift_pt();
    let (r, g, b) = style.effective_color().as_pdf_rgb();
    let bold = style.is_bold();
    let font_name = FontBundle::resource_name(bold);
    let face = if bold {
        &fonts.bold.parsed
    } else {
        &fonts.regular.parsed
    };
    let encoded = face.encode_cid(text);
    content.set_fill_rgb(r, g, b);
    content.begin_text();
    content.set_font(font_name, font_size);
    content.next_line(x, y);
    content.show(Str(&encoded));
    content.end_text();
    if !text.is_empty() && (style.is_underline() || style.is_strikethrough()) {
        let width = measure_text(face, text, font_size);
        content.set_stroke_rgb(r, g, b);
        content.set_line_width(0.75);
        if style.is_underline() {
            let underline_y = y - 1.5;
            content.move_to(x, underline_y);
            content.line_to(x + width, underline_y);
            content.stroke();
        }
        if style.is_strikethrough() {
            let strike_y = y + font_size * 0.35;
            content.move_to(x, strike_y);
            content.line_to(x + width, strike_y);
            content.stroke();
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_table(
    content: &mut Content,
    fonts: &FontBundle,
    columns: &[String],
    rows: &[Vec<String>],
    style: &TableStyle,
    column_widths: &[f32],
    x: f32,
    top_y: f32,
    height: f32,
) {
    let cols = column_widths.len().max(1);
    let width: f32 = column_widths.iter().sum();
    let col_offsets = column_offsets(x, column_widths);
    let row_count = rows.len() + usize::from(!columns.is_empty());
    if row_count == 0 {
        return;
    }
    let row_h = height / row_count as f32;
    let bottom_y = top_y - height;

    let (br, bg, bb) = Color::rgb(30, 41, 59).as_pdf_rgb();
    content.set_stroke_rgb(br, bg, bb);
    content.set_line_width(style.border_width());
    content.rect(x, bottom_y, width, height);
    content.stroke();

    if !columns.is_empty() {
        let (hr, hg, hb) = style.effective_header_bg().as_pdf_rgb();
        content.set_fill_rgb(hr, hg, hb);
        content.rect(x, top_y - row_h, width, row_h);
        content.fill_nonzero();
    }

    for &lx in col_offsets.iter().skip(1) {
        content.move_to(lx, bottom_y);
        content.line_to(lx, top_y);
        content.stroke();
    }

    for r in 1..row_count {
        let ly = top_y - row_h * r as f32;
        content.move_to(x, ly);
        content.line_to(x + width, ly);
        content.stroke();
    }

    let font_size = style.effective_font_size();
    let pad = style.padding_pt();

    let mut row_index = 0;
    if !columns.is_empty() {
        for (ci, cell) in columns.iter().enumerate() {
            let cx = col_offsets.get(ci).copied().unwrap_or(x) + pad;
            let cy = top_y - row_h + pad + font_size * 0.25;
            draw_text_line(
                content,
                fonts,
                cell,
                &Style::new().bold().font_size(font_size),
                cx,
                cy,
            );
        }
        row_index = 1;
    }

    for (ri, row) in rows.iter().enumerate() {
        let visual_row = row_index + ri;
        for (ci, cell) in row.iter().enumerate() {
            if ci >= cols {
                break;
            }
            let cx = col_offsets.get(ci).copied().unwrap_or(x) + pad;
            let cy = top_y - row_h * (visual_row as f32 + 1.0) + pad + font_size * 0.25;
            draw_text_line(
                content,
                fonts,
                cell,
                &Style::new().font_size(font_size),
                cx,
                cy,
            );
        }
    }
}

fn column_offsets(x: f32, column_widths: &[f32]) -> Vec<f32> {
    let mut offsets = Vec::with_capacity(column_widths.len());
    let mut cursor = x;
    for w in column_widths {
        offsets.push(cursor);
        cursor += *w;
    }
    offsets
}

fn draw_image(content: &mut Content, name: Name, x: f32, y: f32, w: f32, h: f32) {
    content.save_state();
    content.transform([w, 0.0, 0.0, h, x, y]);
    content.x_object(name);
    content.restore_state();
}

fn embed_image(ids: &mut IdGen, input: &ImageInput) -> Result<ImageParts> {
    let format = image::guess_format(&input.data)
        .map_err(|_| Error::Pdf("unsupported or unknown image format".into()))?;
    let dynamic = image::load_from_memory(&input.data)
        .map_err(|e| Error::Pdf(format!("decode image: {e}")))?;

    let (filter, encoded, mask) = match format {
        image::ImageFormat::Jpeg => (Filter::DctDecode, input.data.clone(), None),
        image::ImageFormat::Png => {
            let level = CompressionLevel::DefaultLevel as u8;
            let encoded = compress_to_vec_zlib(dynamic.to_rgb8().as_raw(), level);
            let mask = dynamic.color().has_alpha().then(|| {
                let alphas: Vec<_> = dynamic.pixels().map(|p| p.2 .0[3]).collect();
                compress_to_vec_zlib(&alphas, level)
            });
            (Filter::FlateDecode, encoded, mask)
        }
        _ => return Err(Error::Pdf("only PNG and JPEG images are supported".into())),
    };

    let image_id = ids.next();
    let mask_id = mask.as_ref().map(|_| ids.next());

    Ok(ImageParts {
        id: image_id,
        encoded,
        filter,
        width: dynamic.width() as i32,
        height: dynamic.height() as i32,
        mask_id,
        mask_encoded: mask,
    })
}

fn write_image_xobject(pdf: &mut Pdf, img: &ImageParts) {
    let mut xobj = pdf.image_xobject(img.id, &img.encoded);
    xobj.filter(img.filter);
    xobj.width(img.width);
    xobj.height(img.height);
    xobj.color_space().device_rgb();
    xobj.bits_per_component(8);
    if let Some(mask_id) = img.mask_id {
        xobj.s_mask(mask_id);
    }
    xobj.finish();

    if let (Some(mask_id), Some(mask_data)) = (img.mask_id, &img.mask_encoded) {
        let mut mask = pdf.image_xobject(mask_id, mask_data);
        mask.filter(img.filter);
        mask.width(img.width);
        mask.height(img.height);
        mask.color_space().device_gray();
        mask.bits_per_component(8);
        mask.finish();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flow::{PageBreak, Paragraph, Spacer, Table};
    use docrafter_core::TableStyle;
    use docrafter_testing::assert_pdf_structure;

    #[test]
    fn renders_table_and_page_break() {
        let mut r = PdfRenderer::new(PageSize::a4());
        r.push(Paragraph::new("Title"));
        r.push(
            Table::new()
                .columns(["A", "B"])
                .row(["1", "2"])
                .style(TableStyle::professional()),
        );
        r.push(Spacer::new(docrafter_core::Length::pt(24.0)));
        r.push(PageBreak);
        r.push(Paragraph::new("Page 2"));
        let bytes = r.finish().unwrap();
        assert_pdf_structure(&bytes, 2, &["Title", "A", "Page 2"]);
    }
}
