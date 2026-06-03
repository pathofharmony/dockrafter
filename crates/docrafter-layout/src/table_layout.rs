//! Table column sizing and row pagination.

use docrafter_font::TextMeasurer;

use crate::input::TableInput;

/// One slice of a table rendered on a single page.
#[derive(Debug, Clone)]
pub struct TableSlice {
    /// Column headers (empty when not drawn on this slice).
    pub columns: Vec<String>,
    /// Rows for this page slice.
    pub rows: Vec<Vec<String>>,
    /// Column widths in points.
    pub column_widths: Vec<f32>,
    /// Style copy.
    pub style: docrafter_core::TableStyle,
}

/// Compute per-column widths from content (proportional fill to `total_width`).
#[must_use]
pub fn compute_column_widths(
    table: &TableInput,
    total_width: f32,
    font_size: f32,
    measurer: Option<&dyn TextMeasurer>,
) -> Vec<f32> {
    let cols = table
        .columns
        .len()
        .max(table.rows.iter().map(|r| r.len()).max().unwrap_or(1));
    let pad = table.style.padding_pt() * 2.0;
    let mut widths = vec![0.0_f32; cols];

    let measure = |text: &str, bold: bool| -> f32 {
        if let Some(m) = measurer {
            m.measure(text, font_size, bold) + pad
        } else {
            text.len() as f32 * font_size * 0.52 + pad
        }
    };

    for (i, title) in table.columns.iter().enumerate() {
        if i < cols {
            widths[i] = widths[i].max(measure(title, table.style.header_bold()));
        }
    }
    for row in &table.rows {
        for (i, cell) in row.iter().enumerate() {
            if i < cols {
                widths[i] = widths[i].max(measure(cell, false));
            }
        }
    }

    let min_w = font_size * 2.0;
    for w in &mut widths {
        *w = w.max(min_w);
    }

    let sum: f32 = widths.iter().sum();
    if sum > total_width && sum > 0.0 {
        let scale = total_width / sum;
        widths.iter_mut().for_each(|w| *w *= scale);
    } else if sum < total_width && sum > 0.0 {
        let extra = total_width - sum;
        for w in &mut widths {
            *w += extra * (*w / sum);
        }
    } else if sum == 0.0 && cols > 0 {
        widths.fill(total_width / cols as f32);
    }

    widths
}

/// Split table rows across pages with optional header repeat.
#[must_use]
pub fn paginate_table(
    table: &TableInput,
    column_widths: Vec<f32>,
    row_height: f32,
    available_height: f32,
    repeat_header: bool,
) -> Vec<TableSlice> {
    let has_header = !table.columns.is_empty();
    let header_h = if has_header { row_height } else { 0.0 };
    let mut slices = Vec::new();
    let mut row_idx = 0usize;
    let mut first = true;

    while row_idx < table.rows.len() || (first && has_header) {
        first = false;
        let show_header = has_header && (slices.is_empty() || repeat_header);
        let mut budget = available_height;
        if show_header {
            budget -= header_h;
        }
        let mut chunk_rows = Vec::new();

        while row_idx < table.rows.len() && budget >= row_height {
            chunk_rows.push(table.rows[row_idx].clone());
            budget -= row_height;
            row_idx += 1;
        }

        if chunk_rows.is_empty() && !show_header {
            break;
        }

        slices.push(TableSlice {
            columns: if show_header {
                table.columns.clone()
            } else {
                Vec::new()
            },
            rows: chunk_rows,
            column_widths: column_widths.clone(),
            style: table.style.clone(),
        });

        if row_idx >= table.rows.len() {
            break;
        }
    }

    if slices.is_empty() {
        slices.push(TableSlice {
            columns: table.columns.clone(),
            rows: Vec::new(),
            column_widths,
            style: table.style.clone(),
        });
    }

    slices
}

/// Height of one table row in points.
#[must_use]
pub fn table_row_height(style: &docrafter_core::TableStyle) -> f32 {
    style.effective_font_size() * 1.8 + style.padding_pt()
}

/// Total table block height for a slice.
#[must_use]
pub fn slice_height(slice: &TableSlice, row_height: f32) -> f32 {
    let rows = slice.rows.len() + usize::from(!slice.columns.is_empty());
    rows as f32 * row_height + slice.style.padding_pt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use docrafter_core::TableStyle;

    #[test]
    fn paginate_repeats_header_on_second_page() {
        let table = TableInput {
            columns: vec!["A".into()],
            rows: (0..30).map(|i| vec![format!("row{i}")]).collect(),
            style: TableStyle::new(),
            column_widths: vec![100.0],
            repeat_header_on_new_page: true,
        };
        let slices = paginate_table(&table, vec![100.0], 20.0, 50.0, true);
        assert!(slices.len() > 1);
        assert!(!slices[1].columns.is_empty());
    }

    #[test]
    fn column_widths_scale_to_content() {
        let table = TableInput {
            columns: vec!["Short".into(), "Much longer header".into()],
            rows: vec![vec!["a".into(), "b".into()]],
            style: TableStyle::new(),
            column_widths: Vec::new(),
            repeat_header_on_new_page: false,
        };
        let w = compute_column_widths(&table, 400.0, 10.0, None);
        assert_eq!(w.len(), 2);
        assert!(w[1] > w[0]);
    }
}
