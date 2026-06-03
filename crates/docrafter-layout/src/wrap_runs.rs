//! Multi-run line breaking (reportlab / python-docx style).

use docrafter_core::{Alignment, Style};

use crate::input::TextRunInput;
use crate::wrap::measure_text_width;

/// One typeset fragment on a line.
#[derive(Debug, Clone)]
pub struct LineSegment {
    /// Visible text (word or chunk).
    pub text: String,
    /// Effective character style.
    pub style: Style,
    /// X position in points (set by layout).
    pub x: f32,
}

/// A fully broken line of styled segments.
#[derive(Debug, Clone)]
pub struct StyledLine {
    /// Fragments left-to-right.
    pub segments: Vec<LineSegment>,
    /// Line height in points.
    pub line_height: f32,
}

/// Merge paragraph-level and run-level styles.
#[must_use]
pub fn effective_run_style(paragraph_style: &Style, run_style: &Style) -> Style {
    let mut style = paragraph_style.clone();
    if run_style.is_bold() {
        style = style.bold();
    }
    if run_style.is_italic() {
        style = style.italic();
    }
    if run_style.is_underline() {
        style = style.underline();
    }
    if run_style.is_strikethrough() {
        style = style.strikethrough();
    }
    match run_style.vertical_align() {
        docrafter_core::VerticalAlign::Superscript => style = style.superscript(),
        docrafter_core::VerticalAlign::Subscript => style = style.subscript(),
        docrafter_core::VerticalAlign::Baseline => {}
    }
    if run_style.effective_font_size() != paragraph_style.effective_font_size() {
        style = style.font_size(run_style.effective_font_size());
    }
    if run_style.effective_color() != paragraph_style.effective_color() {
        style = style.color_value(run_style.effective_color());
    }
    style
}

struct WordPiece {
    text: String,
    style: Style,
}

fn measure_word(
    text: &str,
    style: &Style,
    measurer: Option<&dyn docrafter_font::TextMeasurer>,
) -> f32 {
    let font_size = style.effective_font_size();
    let bold = style.is_bold();
    if let Some(m) = measurer {
        m.measure(text, font_size, bold)
    } else {
        measure_text_width(text, font_size)
    }
}

fn space_width(style: &Style, measurer: Option<&dyn docrafter_font::TextMeasurer>) -> f32 {
    measure_word(" ", style, measurer)
}

fn collect_words(runs: &[TextRunInput], paragraph_style: &Style) -> Vec<WordPiece> {
    let mut words = Vec::new();
    for run in runs {
        let style = effective_run_style(paragraph_style, &run.style);
        if run.text.is_empty() {
            continue;
        }
        for (line_idx, line) in run.text.split('\n').enumerate() {
            if line_idx > 0 {
                words.push(WordPiece {
                    text: String::new(),
                    style: style.clone(),
                });
            }
            for word in line.split_whitespace() {
                words.push(WordPiece {
                    text: word.to_string(),
                    style: style.clone(),
                });
            }
        }
    }
    words
}

/// Break styled runs into lines that fit `max_width`.
#[must_use]
pub fn wrap_styled_runs(
    runs: &[TextRunInput],
    paragraph_style: &Style,
    max_width: f32,
    measurer: Option<&dyn docrafter_font::TextMeasurer>,
) -> Vec<StyledLine> {
    if max_width <= 0.0 {
        return vec![StyledLine {
            segments: runs
                .iter()
                .map(|r| LineSegment {
                    text: r.text.clone(),
                    style: effective_run_style(paragraph_style, &r.style),
                    x: 0.0,
                })
                .collect(),
            line_height: paragraph_style.effective_line_height(),
        }];
    }

    let words = collect_words(runs, paragraph_style);
    if words.is_empty() {
        return vec![StyledLine {
            segments: vec![],
            line_height: paragraph_style.effective_line_height(),
        }];
    }

    let mut lines: Vec<Vec<WordPiece>> = Vec::new();
    let mut current: Vec<WordPiece> = Vec::new();
    let mut current_width = 0.0;

    for piece in words {
        if piece.text.is_empty() {
            if !current.is_empty() {
                lines.push(current);
                current = Vec::new();
                current_width = 0.0;
            }
            lines.push(vec![piece]);
            continue;
        }

        let word_width = measure_word(&piece.text, &piece.style, measurer);
        let space_before = if current.is_empty() {
            0.0
        } else {
            space_width(&current[0].style, measurer)
        };

        if current.is_empty() {
            if word_width <= max_width {
                current.push(piece);
                current_width = word_width;
            } else {
                lines.push(vec![piece]);
            }
        } else if current_width + space_before + word_width <= max_width {
            current_width += space_before + word_width;
            current.push(piece);
        } else {
            lines.push(current);
            current = vec![piece];
            current_width = word_width;
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }

    lines
        .into_iter()
        .map(|line_pieces| {
            let line_height = line_pieces
                .iter()
                .map(|p| p.style.effective_line_height())
                .fold(paragraph_style.effective_line_height(), f32::max);
            let segments = line_pieces
                .into_iter()
                .map(|p| LineSegment {
                    text: p.text,
                    style: p.style,
                    x: 0.0,
                })
                .collect();
            StyledLine {
                segments,
                line_height,
            }
        })
        .collect()
}

/// Apply horizontal alignment to a broken line.
pub fn align_styled_line(
    line: &mut StyledLine,
    margin_left: f32,
    content_width: f32,
    align: Alignment,
    measurer: Option<&dyn docrafter_font::TextMeasurer>,
) {
    if line.segments.is_empty() {
        return;
    }
    let mut total = 0.0;
    for (i, seg) in line.segments.iter().enumerate() {
        if i > 0 {
            total += space_width(&seg.style, measurer);
        }
        total += measure_word(&seg.text, &seg.style, measurer);
    }
    let offset = match align {
        Alignment::Center => (content_width - total) / 2.0,
        Alignment::End => content_width - total,
        _ => 0.0,
    };
    let mut x = margin_left + offset;
    for seg in &mut line.segments {
        seg.x = x;
        x += measure_word(&seg.text, &seg.style, measurer);
        x += space_width(&seg.style, measurer);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use docrafter_core::Style;

    #[test]
    fn wraps_mixed_bold_runs() {
        let runs = vec![
            TextRunInput {
                text: "Hello ".into(),
                style: Style::new(),
            },
            TextRunInput {
                text: "world".into(),
                style: Style::new().bold(),
            },
        ];
        let lines = wrap_styled_runs(&runs, &Style::new(), 200.0, None);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].segments.len(), 2);
        assert!(lines[0].segments[1].style.is_bold());
    }
}
