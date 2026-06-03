//! Text measurement and wrapping.

/// Approximate string width in points (Helvetica-like metrics).
#[must_use]
pub fn measure_text_width(text: &str, font_size: f32) -> f32 {
    text.chars().map(|ch| char_width(ch, font_size)).sum()
}

fn char_width(ch: char, font_size: f32) -> f32 {
    let factor = if ch.is_ascii() {
        match ch {
            ' ' => 0.28,
            'i' | 'l' | '!' | '.' | ',' => 0.28,
            'm' | 'w' | 'M' | 'W' => 0.72,
            _ => 0.52,
        }
    } else {
        0.62
    };
    font_size * factor
}

/// Wrap `text` into lines that fit within `max_width` (points).
#[must_use]
pub fn wrap_text(
    text: &str,
    max_width: f32,
    font_size: f32,
    measurer: Option<&dyn docrafter_font::TextMeasurer>,
    bold: bool,
) -> Vec<String> {
    if text.is_empty() {
        return vec![String::new()];
    }
    if max_width <= 0.0 {
        return vec![text.to_string()];
    }

    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        if paragraph.is_empty() {
            lines.push(String::new());
            continue;
        }
        let mut current = String::new();
        let mut current_width = 0.0;
        for word in paragraph.split_whitespace() {
            let word_width = if let Some(m) = measurer {
                m.measure(word, font_size, bold)
            } else {
                measure_text_width(word, font_size)
            };
            let space_width = if current.is_empty() {
                0.0
            } else if let Some(m) = measurer {
                m.measure(" ", font_size, bold)
            } else {
                char_width(' ', font_size)
            };
            if current.is_empty() {
                if word_width <= max_width {
                    current.push_str(word);
                    current_width = word_width;
                } else {
                    push_split_word(&mut lines, word, max_width, font_size);
                }
            } else if current_width + space_width + word_width <= max_width {
                current.push(' ');
                current.push_str(word);
                current_width += space_width + word_width;
            } else {
                lines.push(current);
                current = String::new();
                current_width = 0.0;
                if word_width <= max_width {
                    current.push_str(word);
                    current_width = word_width;
                } else {
                    push_split_word(&mut lines, word, max_width, font_size);
                }
            }
        }
        if !current.is_empty() {
            lines.push(current);
        }
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn push_split_word(lines: &mut Vec<String>, word: &str, max_width: f32, font_size: f32) {
    let mut chunk = String::new();
    let mut width = 0.0;
    for ch in word.chars() {
        let ch_w = char_width(ch, font_size);
        if width + ch_w > max_width && !chunk.is_empty() {
            lines.push(chunk);
            chunk = String::new();
            width = 0.0;
        }
        chunk.push(ch);
        width += ch_w;
    }
    if !chunk.is_empty() {
        lines.push(chunk);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_splits_long_line() {
        let lines = wrap_text(
            "The quick brown fox jumps over the lazy dog",
            120.0,
            12.0,
            None,
            false,
        );
        assert!(lines.len() > 1);
    }

    #[test]
    fn measure_respects_cyrillic() {
        let w = measure_text_width("Привет", 12.0);
        assert!(w > measure_text_width("Hi", 12.0));
    }
}
