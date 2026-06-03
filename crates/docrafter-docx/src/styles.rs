//! Built-in Word style definitions (`word/styles.xml`).

use docrafter_core::Style;

/// Minimal styles part required by Word.
pub const STYLES_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:docDefaults>
<w:rPrDefault><w:rPr><w:sz w:val="24"/><w:szCs w:val="24"/></w:rPr></w:rPrDefault>
<w:pPrDefault><w:pPr/></w:pPrDefault>
</w:docDefaults>
<w:style w:type="paragraph" w:default="1" w:styleId="Normal">
<w:name w:val="Normal"/>
<w:qFormat/>
</w:style>
<w:style w:type="paragraph" w:styleId="Heading1">
<w:name w:val="heading 1"/>
<w:basedOn w:val="Normal"/>
<w:qFormat/>
<w:pPr><w:jc w:val="left"/></w:pPr>
<w:rPr><w:b/><w:sz w:val="36"/><w:szCs w:val="36"/></w:rPr>
</w:style>
<w:style w:type="paragraph" w:styleId="Heading2">
<w:name w:val="heading 2"/>
<w:basedOn w:val="Normal"/>
<w:qFormat/>
<w:rPr><w:b/><w:sz w:val="28"/><w:szCs w:val="28"/></w:rPr>
</w:style>
</w:styles>
"#;

/// Minimal settings part (improves LibreOffice / Word compatibility).
pub const SETTINGS_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:settings xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:zoom w:percent="100"/>
<w:defaultTabStop w:val="720"/>
</w:settings>
"#;

/// Map a [`Style`] to a `w:pStyle` id when it matches a preset.
#[must_use]
pub fn paragraph_style_id(style: &Style) -> Option<&'static str> {
    if style.is_bold() && approx_eq(style.effective_font_size(), 18.0) {
        return Some("Heading1");
    }
    if style.is_bold() && approx_eq(style.effective_font_size(), 14.0) {
        return Some("Heading2");
    }
    None
}

/// Resolve preset from a `w:pStyle` value read from XML.
#[must_use]
pub fn style_from_paragraph_id(id: &str) -> Style {
    match id {
        "Heading1" => Style::heading1(),
        "Heading2" => Style::heading2(),
        _ => Style::new(),
    }
}

fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.5
}
