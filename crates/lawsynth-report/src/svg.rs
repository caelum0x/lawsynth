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
}
