//! Brand theme: palette tokens + font stacks applied across every report.
//!
//! A [`Theme`] carries the LawSynth brand tokens (from `assets/brand/palette.json`)
//! and the three font *stacks* from `assets/brand/typography.md` (serif display,
//! sans interface, mono canonical). It is the single source of colour and type
//! for the HTML report, the comparison report, and the scenarios report, and it
//! colours the inline SVG charts (series, axes, gridlines, cards).
//!
//! Every field is a `'static` string slice so a `Theme` is `Copy` and can live
//! inside [`crate::ReportOptions`] with no allocation. [`Theme::default`] returns
//! the brand light theme.

/// Ordered categorical series palette, drawn from the brand tokens.
///
/// The accent leads (primary series); success and muted follow, then the
/// remaining semantic hues. This keeps a discovery's first trajectory in the
/// brand accent and secondary series legible against warm paper.
pub(crate) const BRAND_SERIES: &[&str] = &[
    "#b54b2a", // accent   — primary series
    "#2f6f4f", // success  — second series
    "#59635e", // muted    — third series
    "#3a5a78", // info
    "#b8822a", // warning
    "#a3341f", // danger
];

/// A self-contained visual theme: brand palette tokens plus font stacks.
///
/// Colours are CSS hex strings; fonts are CSS font-family *stacks* (no external
/// font files, so a report stays a single portable file).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Theme {
    /// Primary text / high-contrast ink.
    pub ink: &'static str,
    /// Default page background (warm paper).
    pub paper: &'static str,
    /// Raised surfaces: cards, panels, chart backgrounds.
    pub surface: &'static str,
    /// Borders, dividers, hairlines, gridlines.
    pub line: &'static str,
    /// Secondary / metadata text.
    pub muted: &'static str,
    /// Primary accent: header rule, active series, equation marker.
    pub accent: &'static str,
    /// Soft accent wash for highlights and selection.
    pub accent_soft: &'static str,
    /// Positive / succeeded state (also the "added" diff colour).
    pub success: &'static str,
    /// Caution state (also the "changed" diff colour).
    pub warning: &'static str,
    /// Error state (also the "removed" diff colour).
    pub danger: &'static str,
    /// Informational hue.
    pub info: &'static str,
    /// Serif display stack (wordmark, titles, headings).
    pub font_serif: &'static str,
    /// Sans interface stack (body copy, controls).
    pub font_sans: &'static str,
    /// Monospace stack (identifiers, equations, labels, kickers).
    pub font_mono: &'static str,
    /// Ordered categorical series palette for charts and legends.
    pub series: &'static [&'static str],
}

impl Theme {
    /// The LawSynth brand light theme (warm paper, dark ink, terracotta accent).
    pub const fn brand_light() -> Self {
        Self {
            ink: "#18201d",
            paper: "#f3f0e8",
            surface: "#fffdf7",
            line: "#c8c6ba",
            muted: "#59635e",
            accent: "#b54b2a",
            accent_soft: "#e5c3b4",
            success: "#2f6f4f",
            warning: "#b8822a",
            danger: "#a3341f",
            info: "#3a5a78",
            font_serif: "Georgia, \"Times New Roman\", serif",
            font_sans: "Inter, system-ui, sans-serif",
            font_mono: "ui-monospace, SFMono-Regular, \"SF Mono\", monospace",
            series: BRAND_SERIES,
        }
    }

    /// Returns the stable series colour for `index`, wrapping the palette.
    pub fn series_color(&self, index: usize) -> &str {
        self.series[index % self.series.len()]
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::brand_light()
    }
}

/// Builds the inline stylesheet for a report from a [`Theme`].
///
/// The sheet is self-contained (no `@import`, no external fonts): typography uses
/// the brand font stacks and every colour resolves to a brand token. Diff/state
/// classes (`.added`, `.removed`, `.changed`, `.up`, `.down`, …) are defined here
/// so the comparison and scenarios reports inherit the theme without inline CSS.
pub(crate) fn stylesheet(theme: &Theme) -> String {
    format!(
        "* {{ box-sizing: border-box; }}
body {{ margin: 0; background: {paper}; color: {ink};
  font-family: {sans}; font-size: 14px; line-height: 1.5;
  -webkit-font-smoothing: antialiased; text-rendering: optimizeLegibility; }}
main {{ max-width: 820px; margin: 0 auto; padding: 32px 20px 64px; }}
header {{ border-bottom: 2px solid {accent}; padding-bottom: 12px; margin-bottom: 8px; }}
h1 {{ font-family: {serif}; font-weight: 700; font-size: 30px; line-height: 1.1;
  letter-spacing: -0.01em; margin: 0 0 4px; color: {ink}; }}
h2 {{ font-family: {serif}; font-weight: 650; font-size: 19px; line-height: 1.25;
  margin: 0 0 12px; color: {ink}; }}
h3 {{ font-family: {serif}; font-weight: 650; font-size: 15px; line-height: 1.3;
  margin: 16px 0 6px; color: {ink}; }}
.subtitle {{ margin: 0; color: {muted}; font-family: {mono}; font-size: 11px;
  letter-spacing: 0.08em; text-transform: uppercase; }}
section {{ background: {surface}; border: 1px solid {line}; border-radius: 8px;
  padding: 20px 24px; margin-top: 20px; }}
.equations {{ display: flex; flex-direction: column; gap: 8px; }}
.equation {{ font-family: {mono}; font-size: 13px; line-height: 1.55; color: {ink};
  background: {paper}; border-left: 3px solid {accent};
  padding: 8px 12px; border-radius: 4px; overflow-x: auto; }}
table {{ width: 100%; border-collapse: collapse; font-size: 14px; }}
th, td {{ text-align: left; padding: 6px 10px; border-bottom: 1px solid {line}; }}
th {{ color: {muted}; font-family: {mono}; font-weight: 600; font-size: 11px;
  letter-spacing: 0.08em; text-transform: uppercase; }}
.mono {{ font-family: {mono}; }}
.muted {{ color: {muted}; font-size: 14px; }}
.chart {{ overflow-x: auto; }}
svg {{ max-width: 100%; height: auto; border-radius: 6px; }}
.added {{ color: {success}; font-weight: 600; }}
.removed {{ color: {danger}; font-weight: 600; }}
.changed {{ color: {warning}; font-weight: 600; }}
.neutral {{ color: {muted}; }}
.up {{ color: {success}; font-weight: 600; }}
.down {{ color: {danger}; font-weight: 600; }}
.flat {{ color: {muted}; }}
.equation-cell {{ font-family: {mono}; font-size: 13px; }}
@media (prefers-reduced-motion: reduce) {{
  * {{ animation: none !important; transition: none !important; }}
}}",
        paper = theme.paper,
        ink = theme.ink,
        sans = theme.font_sans,
        serif = theme.font_serif,
        mono = theme.font_mono,
        muted = theme.muted,
        accent = theme.accent,
        surface = theme.surface,
        line = theme.line,
        success = theme.success,
        danger = theme.danger,
        warning = theme.warning,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_brand_light() {
        let theme = Theme::default();
        assert_eq!(theme.accent, "#b54b2a");
        assert_eq!(theme.paper, "#f3f0e8");
        assert_eq!(theme.ink, "#18201d");
        assert!(theme.font_serif.contains("Georgia"));
    }

    #[test]
    fn series_colours_wrap_and_lead_with_accent() {
        let theme = Theme::default();
        assert_eq!(theme.series_color(0), "#b54b2a");
        assert_eq!(theme.series_color(1), "#2f6f4f");
        assert_eq!(theme.series_color(theme.series.len()), "#b54b2a");
    }

    #[test]
    fn stylesheet_carries_brand_tokens_and_fonts() {
        let sheet = stylesheet(&Theme::default());
        assert!(sheet.contains("#b54b2a"));
        assert!(sheet.contains("#f3f0e8"));
        assert!(sheet.contains("Georgia"));
        assert!(sheet.contains("ui-monospace"));
    }
}
