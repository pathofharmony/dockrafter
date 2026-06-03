//! `styles.xml` automatic styles for LibreOffice.

use docrafter_core::Style;

/// OpenDocument styles part.
pub const STYLES_XML: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<office:document-styles xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0" xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0" xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0" xmlns:fo="urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0" xmlns:table="urn:oasis:names:tc:opendocument:xmlns:table:1.0" office:version="1.2">
<office:styles>
<style:style style:name="Standard" style:family="paragraph" style:class="text">
<style:paragraph-properties fo:text-align="start"/>
<style:text-properties style:font-size="12pt"/>
</style:style>
<style:style style:name="Heading1" style:family="paragraph" style:parent-style-name="Standard">
<style:text-properties fo:font-weight="bold" style:font-size="18pt"/>
</style:style>
<style:style style:name="Heading2" style:family="paragraph" style:parent-style-name="Standard">
<style:text-properties fo:font-weight="bold" style:font-size="14pt"/>
</style:style>
<style:style style:name="TableHeader" style:family="table-cell">
<style:table-cell-properties fo:background-color="#e2e8f0" fo:border="0.5pt solid #1e293b"/>
<style:text-properties fo:font-weight="bold"/>
</style:style>
<style:style style:name="List1" style:family="list">
<style:list-style>
<text:list-level-style-number text:level="1" style:num-suffix="." text:display-outline-level="0">
<style:list-level-properties text:min-label-width="0.6cm"/>
</text:list-level-style-number>
</style:list-style>
</style:style>
</office:styles>
</office:document-styles>
"##;

/// Map [`Style`] to ODF paragraph style name.
#[must_use]
pub fn paragraph_style_name(style: &Style) -> &'static str {
    if style.is_bold() && approx_eq(style.effective_font_size(), 18.0) {
        return "Heading1";
    }
    if style.is_bold() && approx_eq(style.effective_font_size(), 14.0) {
        return "Heading2";
    }
    "Standard"
}

/// Reverse map from `text:style-name` on paragraphs.
#[must_use]
pub fn style_from_paragraph_name(name: &str) -> Style {
    match name {
        "Heading1" => Style::heading1(),
        "Heading2" => Style::heading2(),
        _ => Style::new(),
    }
}

fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.5
}
