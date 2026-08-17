//! Hand-built, dependency-free inline SVG charts.
//!
//! Every chart is emitted as a self-contained `<svg>` fragment with no external
//! assets. Coordinates are computed deterministically from the input data.

use std::fmt::Write;

use crate::render::format_number;

/// Deterministic categorical palette (colour-blind friendly ordering).
const PALETTE: [&str; 8] =
    ["#2563eb", "#dc2626", "#059669", "#d97706", "#7c3aed", "#0891b2", "#db2777", "#65a30d"];

/// Returns the stable series colour for a given series index.
pub fn series_color(index: usize) -> &'static str {
    PALETTE[index % PALETTE.len()]
}

struct Bounds {
    min: f64,
    max: f64,
}

impl Bounds {
    fn of(values: impl Iterator<Item = f64>) -> Self {
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        for value in values.filter(|value| value.is_finite()) {
            min = min.min(value);
            max = max.max(value);
        }
        if !min.is_finite() || !max.is_finite() {
            min = 0.0;
            max = 1.0;
        }
        if (max - min).abs() < f64::EPSILON {
            // Pad a flat series so it renders as a centred horizontal line.
            let pad = if min.abs() < f64::EPSILON { 1.0 } else { min.abs() * 0.1 };
            min -= pad;
            max += pad;
        }
        Self { min, max }
    }

    fn normalize(&self, value: f64) -> f64 {
        (value - self.min) / (self.max - self.min)
    }
}

struct Frame {
    width: f64,
    height: f64,
    left: f64,
    right: f64,
    top: f64,
    bottom: f64,
}

impl Frame {
    fn new(width: f64, height: f64) -> Self {
        Self { width, height, left: 56.0, right: 16.0, top: 16.0, bottom: 40.0 }
    }

    fn plot_width(&self) -> f64 {
        self.width - self.left - self.right
    }

    fn plot_height(&self) -> f64 {
        self.height - self.top - self.bottom
    }

    fn x(&self, normalized: f64) -> f64 {
        self.left + normalized * self.plot_width()
    }

    fn y(&self, normalized: f64) -> f64 {
        // SVG y grows downward, so invert.
        self.top + (1.0 - normalized) * self.plot_height()
    }
}

/// Renders a multi-series line chart of state trajectories against time.
///
/// `series` is a slice of `(label, values)` pairs sharing the `time` axis.
pub fn line_chart(time: &[f64], series: &[(String, Vec<f64>)], width: f64, height: f64) -> String {
    let frame = Frame::new(width, height);
    let x_bounds = Bounds::of(time.iter().copied());
    let y_bounds = Bounds::of(series.iter().flat_map(|(_, values)| values.iter().copied()));

    let mut svg = String::new();
    open_svg(&mut svg, width, height);
    axes(&mut svg, &frame, &x_bounds, &y_bounds);

    for (index, (label, values)) in series.iter().enumerate() {
        let color = series_color(index);
        let mut points = String::new();
        for (position, value) in values.iter().enumerate() {
            let time_value = time.get(position).copied().unwrap_or(0.0);
            if !value.is_finite() {
                continue;
            }
            let x = frame.x(x_bounds.normalize(time_value));
            let y = frame.y(y_bounds.normalize(*value));
            if !points.is_empty() {
                points.push(' ');
            }
            let _ = write!(points, "{x:.2},{y:.2}");
        }
        let _ = writeln!(
            svg,
            "  <polyline fill=\"none\" stroke=\"{color}\" stroke-width=\"1.8\" points=\"{points}\" />"
        );
        legend_swatch(&mut svg, &frame, index, label, color);
    }

    axis_label(&mut svg, &frame, "time");
    svg.push_str("</svg>\n");
    svg
}

/// Renders a 2-D phase portrait of one state against another.
pub fn phase_portrait(
    x_label: &str,
    x_values: &[f64],
    y_label: &str,
    y_values: &[f64],
    width: f64,
    height: f64,
) -> String {
    let frame = Frame::new(width, height);
    let x_bounds = Bounds::of(x_values.iter().copied());
    let y_bounds = Bounds::of(y_values.iter().copied());

    let mut svg = String::new();
    open_svg(&mut svg, width, height);
    axes(&mut svg, &frame, &x_bounds, &y_bounds);

    let mut points = String::new();
    for (x_value, y_value) in x_values.iter().zip(y_values.iter()) {
        if !x_value.is_finite() || !y_value.is_finite() {
            continue;
        }
        let x = frame.x(x_bounds.normalize(*x_value));
        let y = frame.y(y_bounds.normalize(*y_value));
        if !points.is_empty() {
            points.push(' ');
        }
        let _ = write!(points, "{x:.2},{y:.2}");
    }
    let _ = writeln!(
        svg,
        "  <polyline fill=\"none\" stroke=\"{}\" stroke-width=\"1.6\" points=\"{points}\" />",
        series_color(4)
    );
    // Mark the trajectory start and end.
    if let (Some(first_x), Some(first_y)) = (x_values.first(), y_values.first()) {
        marker(&mut svg, &frame, &x_bounds, &y_bounds, *first_x, *first_y, "#059669");
    }
    if let (Some(last_x), Some(last_y)) = (x_values.last(), y_values.last()) {
        marker(&mut svg, &frame, &x_bounds, &y_bounds, *last_x, *last_y, "#dc2626");
    }

    axis_label(&mut svg, &frame, x_label);
    vertical_axis_label(&mut svg, &frame, y_label);
    svg.push_str("</svg>\n");
    svg
}

/// One state's observed samples and the simulated trajectory to compare against.
///
/// `observed` is aligned to the `obs_time` axis and `simulated` to the
/// `sim_time` axis passed to [`fit_overlay_chart`]; the two axes need not match.
#[derive(Clone, Debug, PartialEq)]
pub struct FitSeries {
    /// State label shown in the legend.
    pub label: String,
    /// Observed samples aligned to the observation time axis.
    pub observed: Vec<f64>,
    /// Simulated samples aligned to the simulation time axis.
    pub simulated: Vec<f64>,
}

/// Overlays simulated trajectories (solid lines) on observed samples (markers).
///
/// This is the "how well does the model fit?" view: each state's simulated line
/// is drawn over its observed scatter so systematic bias is visible at a glance.
pub fn fit_overlay_chart(
    sim_time: &[f64],
    obs_time: &[f64],
    series: &[FitSeries],
    width: f64,
    height: f64,
) -> String {
    let frame = Frame::new(width, height);
    let x_bounds = Bounds::of(sim_time.iter().chain(obs_time.iter()).copied());
    let y_bounds = Bounds::of(
        series
            .iter()
            .flat_map(|entry| entry.simulated.iter().chain(entry.observed.iter()).copied()),
    );

    let mut svg = String::new();
    open_svg(&mut svg, width, height);
    axes(&mut svg, &frame, &x_bounds, &y_bounds);

    for (index, entry) in series.iter().enumerate() {
        let color = series_color(index);
        // Simulated trajectory as a solid line.
        let mut points = String::new();
        for (position, value) in entry.simulated.iter().enumerate() {
            let time_value = sim_time.get(position).copied().unwrap_or(0.0);
            if !value.is_finite() {
                continue;
            }
            let x = frame.x(x_bounds.normalize(time_value));
            let y = frame.y(y_bounds.normalize(*value));
            if !points.is_empty() {
                points.push(' ');
            }
            let _ = write!(points, "{x:.2},{y:.2}");
        }
        let _ = writeln!(
            svg,
            "  <polyline fill=\"none\" stroke=\"{color}\" stroke-width=\"1.8\" points=\"{points}\" />"
        );
        // Observed samples as small hollow markers.
        for (position, value) in entry.observed.iter().enumerate() {
            let time_value = obs_time.get(position).copied().unwrap_or(0.0);
            if !value.is_finite() {
                continue;
            }
            let x = frame.x(x_bounds.normalize(time_value));
            let y = frame.y(y_bounds.normalize(*value));
            let _ = writeln!(
                svg,
                "  <circle cx=\"{x:.2}\" cy=\"{y:.2}\" r=\"2.1\" fill=\"#ffffff\" stroke=\"{color}\" stroke-width=\"1\" />"
            );
        }
        legend_swatch(&mut svg, &frame, index, &entry.label, color);
    }

    axis_label(&mut svg, &frame, "time");
    svg.push_str("</svg>\n");
    svg
}

/// Renders a residual strip: per-state `simulated - observed` against a zero line.
///
/// Residuals are drawn as vertical stems from the zero baseline, so both the
/// magnitude and the sign of the misfit are legible.
pub fn residual_strip(
    obs_time: &[f64],
    residuals: &[(String, Vec<f64>)],
    width: f64,
    height: f64,
) -> String {
    let frame = Frame::new(width, height);
    let x_bounds = Bounds::of(obs_time.iter().copied());
    // Symmetric y bounds centred on zero so the baseline sits in the middle.
    let extent = residuals
        .iter()
        .flat_map(|(_, values)| values.iter().copied())
        .filter(|value| value.is_finite())
        .fold(0.0_f64, |acc, value| acc.max(value.abs()));
    let extent = if extent > 0.0 { extent } else { 1.0 };
    let y_bounds = Bounds { min: -extent, max: extent };

    let mut svg = String::new();
    open_svg(&mut svg, width, height);
    axes(&mut svg, &frame, &x_bounds, &y_bounds);

    // Emphasised zero baseline.
    let zero_y = frame.y(y_bounds.normalize(0.0));
    let _ = writeln!(
        svg,
        "  <line x1=\"{:.1}\" y1=\"{zero_y:.1}\" x2=\"{:.1}\" y2=\"{zero_y:.1}\" stroke=\"#94a3b8\" stroke-width=\"1.2\" />",
        frame.x(0.0),
        frame.x(1.0)
    );

    for (index, (label, values)) in residuals.iter().enumerate() {
        let color = series_color(index);
        for (position, value) in values.iter().enumerate() {
            let time_value = obs_time.get(position).copied().unwrap_or(0.0);
            if !value.is_finite() {
                continue;
            }
            let x = frame.x(x_bounds.normalize(time_value));
            let y = frame.y(y_bounds.normalize(*value));
            let _ = writeln!(
                svg,
                "  <line x1=\"{x:.2}\" y1=\"{zero_y:.2}\" x2=\"{x:.2}\" y2=\"{y:.2}\" stroke=\"{color}\" stroke-width=\"1.1\" />"
            );
        }
        legend_swatch(&mut svg, &frame, index, &format!("{label} residual"), color);
    }

    axis_label(&mut svg, &frame, "time");
    svg.push_str("</svg>\n");
    svg
}

/// A contiguous regime span over sample indices `[start, end)`.
#[derive(Clone, Debug, PartialEq)]
pub struct RegimeSpan {
    /// Inclusive start sample index.
    pub start: usize,
    /// Exclusive end sample index.
    pub end: usize,
    /// Human-readable label for the regime (e.g. its mean level).
    pub label: String,
}

/// Renders a horizontal regime timeline coloured by segment, with change-point ticks.
///
/// `total` is the number of samples the spans partition; empty input yields an
/// empty (but still self-contained) SVG so callers can degrade gracefully.
pub fn regime_timeline(spans: &[RegimeSpan], total: usize, width: f64, height: f64) -> String {
    let frame = Frame::new(width, height);
    let mut svg = String::new();
    open_svg(&mut svg, width, height);
    if spans.is_empty() || total == 0 {
        svg.push_str("</svg>\n");
        return svg;
    }
    let bar_top = frame.top + 6.0;
    let bar_height = (frame.plot_height() - 24.0).max(18.0);
    let span = |index: usize| frame.left + (index as f64 / total as f64) * frame.plot_width();

    for (order, regime) in spans.iter().enumerate() {
        let x0 = span(regime.start);
        let x1 = span(regime.end.min(total));
        let color = series_color(order);
        let _ = writeln!(
            svg,
            "  <rect x=\"{x0:.2}\" y=\"{bar_top:.2}\" width=\"{:.2}\" height=\"{bar_height:.2}\" fill=\"{color}\" fill-opacity=\"0.55\" stroke=\"{color}\" stroke-width=\"1\" />",
            (x1 - x0).max(0.0)
        );
        let _ = writeln!(
            svg,
            "  <text x=\"{:.2}\" y=\"{:.2}\" font-size=\"10\" text-anchor=\"middle\" fill=\"#0f172a\">{}</text>",
            (x0 + x1) / 2.0,
            bar_top + bar_height / 2.0 + 3.0,
            escape_text(&regime.label)
        );
        // Change-point tick + sample index at each internal boundary.
        if order + 1 < spans.len() {
            let _ = writeln!(
                svg,
                "  <line x1=\"{x1:.2}\" y1=\"{:.2}\" x2=\"{x1:.2}\" y2=\"{:.2}\" stroke=\"#0f172a\" stroke-width=\"1.2\" />",
                bar_top - 4.0,
                bar_top + bar_height + 4.0
            );
            let _ = writeln!(
                svg,
                "  <text x=\"{x1:.2}\" y=\"{:.2}\" font-size=\"9\" text-anchor=\"middle\" fill=\"#475569\">t={}</text>",
                bar_top + bar_height + 16.0,
                regime.end
            );
        }
    }
    svg.push_str("</svg>\n");
    svg
}

/// Renders an uncertainty band: a filled polygon between `lower` and `upper`
/// with the `median` drawn as a line on top.
///
/// All three series share the `time` axis. Mismatched or empty inputs yield a
/// self-contained (possibly empty) SVG rather than panicking.
pub fn uncertainty_band_chart(
    time: &[f64],
    lower: &[f64],
    median: &[f64],
    upper: &[f64],
    label: &str,
    width: f64,
    height: f64,
) -> String {
    let frame = Frame::new(width, height);
    let x_bounds = Bounds::of(time.iter().copied());
    let y_bounds = Bounds::of(lower.iter().chain(median.iter()).chain(upper.iter()).copied());

    let mut svg = String::new();
    open_svg(&mut svg, width, height);
    axes(&mut svg, &frame, &x_bounds, &y_bounds);

    let count = time.len().min(lower.len()).min(upper.len());
    if count >= 2 {
        // Band polygon: upper edge left-to-right, then lower edge right-to-left.
        let mut polygon = String::new();
        for position in 0..count {
            let x = frame.x(x_bounds.normalize(time[position]));
            let y = frame.y(y_bounds.normalize(upper[position]));
            if !polygon.is_empty() {
                polygon.push(' ');
            }
            let _ = write!(polygon, "{x:.2},{y:.2}");
        }
        for position in (0..count).rev() {
            let x = frame.x(x_bounds.normalize(time[position]));
            let y = frame.y(y_bounds.normalize(lower[position]));
            let _ = write!(polygon, " {x:.2},{y:.2}");
        }
        let color = series_color(0);
        let _ = writeln!(
            svg,
            "  <polygon fill=\"{color}\" fill-opacity=\"0.18\" stroke=\"none\" points=\"{polygon}\" />"
        );
    }
    // Median line.
    let mut points = String::new();
    for (position, value) in median.iter().enumerate() {
        let time_value = time.get(position).copied().unwrap_or(0.0);
        if !value.is_finite() {
            continue;
        }
        let x = frame.x(x_bounds.normalize(time_value));
        let y = frame.y(y_bounds.normalize(*value));
        if !points.is_empty() {
            points.push(' ');
        }
        let _ = write!(points, "{x:.2},{y:.2}");
    }
    let _ = writeln!(
        svg,
        "  <polyline fill=\"none\" stroke=\"{}\" stroke-width=\"1.8\" points=\"{points}\" />",
        series_color(0)
    );
    legend_swatch(&mut svg, &frame, 0, label, series_color(0));

    axis_label(&mut svg, &frame, "time");
    svg.push_str("</svg>\n");
    svg
}

fn open_svg(svg: &mut String, width: f64, height: f64) {
    let _ = writeln!(
        svg,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {width} {height}\" width=\"{width}\" height=\"{height}\" role=\"img\">"
    );
    let _ = writeln!(
        svg,
        "  <rect x=\"0\" y=\"0\" width=\"{width}\" height=\"{height}\" fill=\"#ffffff\" />"
    );
}

fn axes(svg: &mut String, frame: &Frame, x_bounds: &Bounds, y_bounds: &Bounds) {
    let x0 = frame.x(0.0);
    let x1 = frame.x(1.0);
    let y0 = frame.y(0.0);
    let y1 = frame.y(1.0);
    // Plot border.
    let _ = writeln!(
        svg,
        "  <rect x=\"{x0:.1}\" y=\"{y1:.1}\" width=\"{:.1}\" height=\"{:.1}\" fill=\"#f8fafc\" stroke=\"#cbd5e1\" stroke-width=\"1\" />",
        x1 - x0,
        y0 - y1
    );
    // Horizontal gridlines with y tick labels.
    for step in 0..=4 {
        let fraction = step as f64 / 4.0;
        let y = frame.y(fraction);
        let value = y_bounds.min + fraction * (y_bounds.max - y_bounds.min);
        let _ = writeln!(
            svg,
            "  <line x1=\"{x0:.1}\" y1=\"{y:.1}\" x2=\"{x1:.1}\" y2=\"{y:.1}\" stroke=\"#e2e8f0\" stroke-width=\"1\" />"
        );
        let _ = writeln!(
            svg,
            "  <text x=\"{:.1}\" y=\"{:.1}\" font-size=\"10\" text-anchor=\"end\" fill=\"#475569\">{}</text>",
            x0 - 6.0,
            y + 3.0,
            format_number(value)
        );
    }
    // X tick labels at the two ends.
    for (fraction, anchor) in [(0.0, "start"), (1.0, "end")] {
        let x = frame.x(fraction);
        let value = x_bounds.min + fraction * (x_bounds.max - x_bounds.min);
        let _ = writeln!(
            svg,
            "  <text x=\"{x:.1}\" y=\"{:.1}\" font-size=\"10\" text-anchor=\"{anchor}\" fill=\"#475569\">{}</text>",
            frame.y(0.0) + 16.0,
            format_number(value)
        );
    }
}

fn marker(
    svg: &mut String,
    frame: &Frame,
    x_bounds: &Bounds,
    y_bounds: &Bounds,
    x_value: f64,
    y_value: f64,
    color: &str,
) {
    if !x_value.is_finite() || !y_value.is_finite() {
        return;
    }
    let x = frame.x(x_bounds.normalize(x_value));
    let y = frame.y(y_bounds.normalize(y_value));
    let _ = writeln!(svg, "  <circle cx=\"{x:.2}\" cy=\"{y:.2}\" r=\"3.2\" fill=\"{color}\" />");
}

fn legend_swatch(svg: &mut String, frame: &Frame, index: usize, label: &str, color: &str) {
    let x = frame.left + 8.0;
    let y = frame.top + 14.0 + index as f64 * 16.0;
    let _ = writeln!(
        svg,
        "  <rect x=\"{x:.1}\" y=\"{:.1}\" width=\"10\" height=\"10\" fill=\"{color}\" />",
        y - 9.0
    );
    let _ = writeln!(
        svg,
        "  <text x=\"{:.1}\" y=\"{y:.1}\" font-size=\"11\" fill=\"#0f172a\">{}</text>",
        x + 15.0,
        escape_text(label)
    );
}

fn axis_label(svg: &mut String, frame: &Frame, label: &str) {
    let _ = writeln!(
        svg,
        "  <text x=\"{:.1}\" y=\"{:.1}\" font-size=\"11\" text-anchor=\"middle\" fill=\"#0f172a\">{}</text>",
        frame.left + frame.plot_width() / 2.0,
        frame.height - 6.0,
        escape_text(label)
    );
}

fn vertical_axis_label(svg: &mut String, frame: &Frame, label: &str) {
    let x = 14.0;
    let y = frame.top + frame.plot_height() / 2.0;
    let _ = writeln!(
        svg,
        "  <text x=\"{x:.1}\" y=\"{y:.1}\" font-size=\"11\" text-anchor=\"middle\" fill=\"#0f172a\" transform=\"rotate(-90 {x:.1} {y:.1})\">{}</text>",
        escape_text(label)
    );
}

fn escape_text(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_chart_is_self_contained_svg() {
        let svg =
            line_chart(&[0.0, 1.0, 2.0], &[("x".to_owned(), vec![1.0, 0.5, 0.25])], 640.0, 320.0);
        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("polyline"));
        assert!(!svg.contains("http://") || svg.contains("www.w3.org"));
    }

    #[test]
    fn phase_portrait_marks_start_and_end() {
        let svg = phase_portrait("x", &[0.0, 1.0], "y", &[0.0, 1.0], 400.0, 400.0);
        assert!(svg.contains("circle"));
    }

    #[test]
    fn flat_series_still_renders() {
        let svg = line_chart(&[0.0, 1.0], &[("c".to_owned(), vec![3.0, 3.0])], 320.0, 200.0);
        assert!(svg.contains("polyline"));
    }

    #[test]
    fn fit_overlay_draws_line_and_markers() {
        let series = vec![FitSeries {
            label: "x".to_owned(),
            observed: vec![1.0, 0.6, 0.4],
            simulated: vec![1.0, 0.5, 0.25, 0.12],
        }];
        let svg = fit_overlay_chart(&[0.0, 1.0, 2.0, 3.0], &[0.0, 1.0, 2.0], &series, 640.0, 320.0);
        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("polyline"));
        assert!(svg.contains("<circle"));
    }

    #[test]
    fn residual_strip_has_zero_baseline_and_stems() {
        let svg = residual_strip(
            &[0.0, 1.0, 2.0],
            &[("x".to_owned(), vec![0.1, -0.2, 0.05])],
            640.0,
            160.0,
        );
        assert!(svg.contains("<line"));
        assert!(svg.contains("residual"));
    }

    #[test]
    fn regime_timeline_renders_segments_and_change_points() {
        let spans = vec![
            RegimeSpan { start: 0, end: 4, label: "0.5".to_owned() },
            RegimeSpan { start: 4, end: 10, label: "1.5".to_owned() },
        ];
        let svg = regime_timeline(&spans, 10, 720.0, 90.0);
        assert!(svg.contains("<rect"));
        assert!(svg.contains("t=4"));
    }

    #[test]
    fn empty_regime_timeline_degrades_gracefully() {
        let svg = regime_timeline(&[], 0, 720.0, 90.0);
        assert!(svg.starts_with("<svg"));
        assert!(svg.trim_end().ends_with("</svg>"));
    }

    #[test]
    fn uncertainty_band_draws_polygon_and_median() {
        let svg = uncertainty_band_chart(
            &[0.0, 1.0, 2.0],
            &[0.8, 0.4, 0.2],
            &[1.0, 0.5, 0.25],
            &[1.2, 0.6, 0.3],
            "x",
            640.0,
            320.0,
        );
        assert!(svg.contains("<polygon"));
        assert!(svg.contains("polyline"));
    }
}
