//! Page layout configuration.

use docrafter_core::PageSize;
use docrafter_font::TextMeasurer;

/// Page margins in points.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutMargins {
    /// Left margin.
    pub left: f32,
    /// Right margin.
    pub right: f32,
    /// Top margin.
    pub top: f32,
    /// Bottom margin.
    pub bottom: f32,
}

impl Default for LayoutMargins {
    fn default() -> Self {
        Self {
            left: 50.0,
            right: 50.0,
            top: 72.0,
            bottom: 72.0,
        }
    }
}

impl LayoutMargins {
    /// Standard document margins.
    #[must_use]
    pub fn standard() -> Self {
        Self::default()
    }
}

/// Layout settings for a document.
pub struct LayoutConfig<'a> {
    /// Paper size.
    pub page_size: PageSize,
    /// Content margins.
    pub margins: LayoutMargins,
    /// Optional embedded font metrics for measuring text.
    pub measurer: Option<&'a dyn TextMeasurer>,
}

impl LayoutConfig<'_> {
    /// A4 with standard margins.
    #[must_use]
    pub fn a4() -> LayoutConfig<'static> {
        LayoutConfig {
            page_size: PageSize::a4(),
            margins: LayoutMargins::standard(),
            measurer: None,
        }
    }

    /// Content area width in points.
    #[must_use]
    pub fn content_width(self) -> f32 {
        let (x0, _, x1, _) = self.page_size.media_box();
        x1 - x0 - self.margins.left - self.margins.right
    }

    /// Content area height in points.
    #[must_use]
    pub fn content_height(self) -> f32 {
        let (_, y0, _, y1) = self.page_size.media_box();
        y1 - y0 - self.margins.top - self.margins.bottom
    }
}
