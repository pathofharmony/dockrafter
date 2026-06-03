//! List numbering definitions (`word/numbering.xml`).

/// Decimal list numbering used by [`docrafter_office::List`].
pub const NUMBERING_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:numbering xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:abstractNum w:abstractNumId="0">
<w:lvl w:ilvl="0">
<w:start w:val="1"/>
<w:numFmt w:val="decimal"/>
<w:lvlText w:val="%1."/>
<w:lvlJc w:val="left"/>
<w:pPr><w:ind w:left="720" w:hanging="360"/></w:pPr>
</w:lvl>
</w:abstractNum>
<w:num w:numId="1">
<w:abstractNumId w:val="0"/>
</w:num>
</w:numbering>
"#;

/// `w:numId` for built-in decimal lists.
pub const LIST_NUM_ID: u32 = 1;
