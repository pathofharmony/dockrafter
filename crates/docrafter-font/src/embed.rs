//! Embed TrueType fonts into a `pdf-writer` document.

use std::fmt::Write;

use docrafter_core::{Error, Result};
use pdf_writer::types::{CidFontType, FontFlags, SystemInfo};
use pdf_writer::{Finish, Name, Pdf, Rect, Ref, Str};
use ttf_parser::{Face, GlyphId};

/// PDF resource name for regular text.
pub const FONT_REGULAR: Name = Name(b"F1");
/// PDF resource name for bold text.
pub const FONT_BOLD: Name = Name(b"F2");

const SYSTEM_INFO: SystemInfo = SystemInfo {
    registry: Str(b"Adobe"),
    ordering: Str(b"Identity"),
    supplement: 0,
};

/// Parsed TrueType face with metrics helpers.
#[derive(Debug, Clone)]
pub struct ParsedFace {
    /// Raw font bytes (owned for stable parsing).
    pub data: Vec<u8>,
    /// Units per em from the `head` table.
    pub units_per_em: f32,
    /// Ascender in font units.
    pub ascent: f32,
    /// Descender in font units (negative).
    pub descent: f32,
    /// Number of glyphs.
    pub glyph_count: u16,
}

impl ParsedFace {
    /// Parse TTF/OTF bytes.
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        let face =
            Face::parse(bytes, 0).map_err(|_| Error::Font("invalid TrueType font data".into()))?;
        let units_per_em = f32::from(face.units_per_em());
        let _bbox = face.global_bounding_box();
        Ok(Self {
            data: bytes.to_vec(),
            units_per_em,
            ascent: face.ascender() as f32,
            descent: face.descender() as f32,
            glyph_count: face.number_of_glyphs(),
        })
    }

    fn with_face<R>(&self, f: impl FnOnce(&Face<'_>) -> R) -> R {
        let face = Face::parse(&self.data, 0).expect("font already validated");
        f(&face)
    }

    /// Horizontal advance for a character in font units.
    #[must_use]
    pub fn advance_width(&self, ch: char) -> Option<f32> {
        self.with_face(|face| {
            let gid = face.glyph_index(ch)?;
            let advance = face.glyph_hor_advance(GlyphId(gid.0))?;
            Some(advance as f32)
        })
    }

    /// Resolve a glyph id back to a Unicode character (first match in font cmap).
    #[must_use]
    pub fn char_from_glyph_id(&self, gid: u16) -> Option<char> {
        self.with_face(|face| {
            for code in 0u32..=0x10_FFFF {
                let ch = char::from_u32(code)?;
                if face.glyph_index(ch).map(|g| g.0) == Some(gid) {
                    return Some(ch);
                }
            }
            None
        })
    }

    /// Map characters to CID bytes (big-endian glyph indices).
    pub fn encode_cid(&self, text: &str) -> Vec<u8> {
        self.with_face(|face| {
            let mut out = Vec::with_capacity(text.len() * 2);
            for ch in text.chars() {
                let gid = face.glyph_index(ch).map(|g| g.0).unwrap_or(0);
                out.extend_from_slice(&gid.to_be_bytes());
            }
            out
        })
    }
}

/// References for one embedded font family member.
#[derive(Debug, Clone)]
pub struct EmbeddedFont {
    /// Type0 font object id (used in page resources).
    pub type0_ref: Ref,
    /// Parsed metrics source.
    pub parsed: ParsedFace,
}

/// Regular + bold embedded fonts for a document.
#[derive(Debug, Clone)]
pub struct FontBundle {
    /// Regular face.
    pub regular: EmbeddedFont,
    /// Bold face.
    pub bold: EmbeddedFont,
}

impl FontBundle {
    /// Embed default DejaVu Sans regular and bold.
    pub fn dejavu_sans(pdf: &mut Pdf, next_ref: &mut dyn FnMut() -> Ref) -> Result<Self> {
        Self::from_bytes(
            pdf,
            next_ref,
            crate::dejavu_sans_regular_bytes(),
            crate::dejavu_sans_bold_bytes(),
        )
    }

    /// Embed custom TTF byte slices.
    pub fn from_bytes(
        pdf: &mut Pdf,
        next_ref: &mut dyn FnMut() -> Ref,
        regular: &[u8],
        bold: &[u8],
    ) -> Result<Self> {
        let regular_parsed = ParsedFace::parse(regular)?;
        let bold_parsed = ParsedFace::parse(bold)?;
        let regular_type0 = embed_truetype(pdf, next_ref, &regular_parsed, Name(b"DejaVuSans"))?;
        let bold_type0 = embed_truetype(pdf, next_ref, &bold_parsed, Name(b"DejaVuSans-Bold"))?;
        Ok(Self {
            regular: EmbeddedFont {
                type0_ref: regular_type0,
                parsed: regular_parsed,
            },
            bold: EmbeddedFont {
                type0_ref: bold_type0,
                parsed: bold_parsed,
            },
        })
    }

    /// Resource name for bold vs regular.
    #[must_use]
    pub const fn resource_name(bold: bool) -> Name<'static> {
        if bold {
            FONT_BOLD
        } else {
            FONT_REGULAR
        }
    }
}

/// Write Type0 + CIDFont + FontDescriptor + streams; returns Type0 reference.
pub fn embed_truetype(
    pdf: &mut Pdf,
    next_ref: &mut dyn FnMut() -> Ref,
    parsed: &ParsedFace,
    base_font: Name,
) -> Result<Ref> {
    let type0_ref = next_ref();
    let cid_ref = next_ref();
    let desc_ref = next_ref();
    let font_file_ref = next_ref();
    let cmap_ref = next_ref();

    let cmap_bytes = build_to_unicode_cmap(parsed, base_font)?;
    pdf.stream(cmap_ref, &cmap_bytes);

    pdf.stream(font_file_ref, &parsed.data);

    parsed.with_face(|face| {
        let units_per_em = f32::from(face.units_per_em());
        let scale = 1000.0 / units_per_em;
        let bb = face.global_bounding_box();
        let bbox = Rect::new(
            f32::from(bb.x_min) * scale,
            f32::from(bb.y_min) * scale,
            f32::from(bb.x_max) * scale,
            f32::from(bb.y_max) * scale,
        );

        let mut widths = Vec::with_capacity(parsed.glyph_count as usize);
        for gid in 0..parsed.glyph_count {
            let w = face.glyph_hor_advance(GlyphId(gid)).unwrap_or(0) as f32 * scale;
            widths.push(w);
        }

        let mut desc = pdf.font_descriptor(desc_ref);
        desc.name(base_font);
        desc.flags(FontFlags::NON_SYMBOLIC);
        desc.bbox(bbox);
        desc.italic_angle(0.0);
        desc.ascent(face.ascender() as f32 * scale);
        desc.descent(face.descender() as f32 * scale);
        desc.cap_height(face.ascender() as f32 * scale * 0.7);
        desc.stem_v(80.0);
        desc.font_file2(font_file_ref);
        desc.finish();

        let mut cid = pdf.cid_font(cid_ref);
        cid.subtype(CidFontType::Type2);
        cid.base_font(base_font);
        cid.system_info(SYSTEM_INFO);
        cid.font_descriptor(desc_ref);
        cid.cid_to_gid_map_predefined(Name(b"Identity"));
        cid.widths().consecutive(0, widths).finish();
        cid.finish();

        let mut type0 = pdf.type0_font(type0_ref);
        type0.base_font(base_font);
        type0.encoding_predefined(Name(b"Identity-H"));
        type0.descendant_font(cid_ref);
        type0.to_unicode(cmap_ref);
        type0.finish();
    });

    Ok(type0_ref)
}

/// Build a ToUnicode CMap stream compatible with `lopdf` text extraction.
///
/// `pdf_writer::UnicodeCmap` emits `/CMapType 0` and PostScript-style `CIDSystemInfo`;
/// `lopdf` requires `/CMapType 2` and `<< >>` dictionaries (see lopdf `cmap_parser.rs`).
fn build_to_unicode_cmap(parsed: &ParsedFace, base_font: Name) -> Result<Vec<u8>> {
    let mut pairs = Vec::new();
    parsed.with_face(|face| {
        for sub in face.tables().cmap.into_iter().flat_map(|c| c.subtables) {
            if sub.is_unicode() {
                sub.codepoints(|cp| {
                    if let (Some(ch), Some(gid)) = (char::from_u32(cp), sub.glyph_index(cp)) {
                        pairs.push((gid.0, ch));
                    }
                });
                break;
            }
        }
    });
    pairs.sort_by_key(|(gid, _)| *gid);
    pairs.dedup_by_key(|(gid, _)| *gid);

    let cmap_name = std::str::from_utf8(base_font.0)
        .map_err(|_| Error::Font("invalid font name in ToUnicode CMap".into()))?;

    Ok(encode_lopdf_to_unicode_cmap(cmap_name, &pairs))
}

fn encode_lopdf_to_unicode_cmap(cmap_name: &str, pairs: &[(u16, char)]) -> Vec<u8> {
    let mut out = String::from(
        "/CIDInit /ProcSet findresource begin\n\
12 dict begin\n\
begincmap\n\
/CIDSystemInfo << /Registry (Adobe) /Ordering (Identity) /Supplement 0 >> def\n",
    );
    out.push_str(&format!("/CMapName /{cmap_name} def\n"));
    out.push_str(
        "/CMapType 2 def\n\
1 begincodespacerange\n\
<0000> <FFFF>\n\
endcodespacerange\n",
    );

    for chunk in pairs.chunks(100) {
        let _ = writeln!(out, "{} beginbfchar", chunk.len());
        for &(gid, ch) in chunk {
            let target = encode_cmap_target(ch);
            let _ = writeln!(out, "<{gid:04X}> {target}");
        }
        out.push_str("endbfchar\n");
    }

    out.push_str(
        "endcmap\n\
CMapName currentdict /CMap defineresource pop\n\
end\n\
end\n",
    );
    out.into_bytes()
}

fn encode_cmap_target(ch: char) -> String {
    let mut buf = [0u16; 2];
    let units = ch.encode_utf16(&mut buf);
    if units.is_empty() {
        return "<0000>".to_string();
    }
    if units.len() == 1 {
        return format!("<{:04X}>", units[0]);
    }
    let hex: String = units.iter().map(|u| format!("{u:04X}")).collect();
    format!("<{hex}>")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_unicode_cmap_has_lopdf_required_headers() {
        let parsed = ParsedFace::parse(crate::dejavu_sans_regular_bytes()).unwrap();
        let bytes = build_to_unicode_cmap(&parsed, Name(b"DejaVuSans")).unwrap();
        let text = std::str::from_utf8(&bytes).unwrap();
        assert!(text.contains("/CMapType 2 def"));
        assert!(text.contains("/CIDSystemInfo <<"));
        assert!(!text.contains("/CMapVersion"));
    }

    #[test]
    fn parses_dejavu_and_encodes_cyrillic() {
        let parsed = ParsedFace::parse(crate::dejavu_sans_regular_bytes()).unwrap();
        let bytes = parsed.encode_cid("Привет");
        assert_eq!(bytes.len(), "Привет".chars().count() * 2);
        assert!(parsed.advance_width('П').unwrap() > 0.0);
    }
}
