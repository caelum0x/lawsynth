#!/usr/bin/env python3
"""End-to-end LawSynth decision loop: discover → scenarios → dashboard.

Run it (from the repository root)::

    PYTHONPATH="python/lawsynth/src:python/lawsynth-notebook/src" \
        python3 python/lawsynth/examples/scenarios_and_dashboard.py

It generates a deterministic damped-oscillator series, discovers the governing
laws, defines a few what-if scenarios, prints the scenario-comparison table
(per-scenario final state + divergence from the baseline), renders a single
cohesive HTML dashboard to a file, and confirms — with the standard-library
``HTMLParser`` — that the written document is valid, self-contained HTML with
inline SVG. Everything is deterministic and offline.
"""

from __future__ import annotations

import csv
import tempfile
from html.parser import HTMLParser
from pathlib import Path

import lawsynth


def _write_synthetic_csv(path: Path) -> None:
    """A damped harmonic oscillator written as a 2-D first-order system.

    dx/dt = v
    dv/dt = -k·x - c·v         (k = 1.0 spring, c = 0.3 damping)
    """
    k, c = 1.0, 0.3
    dt, steps = 0.02, 900
    x, v = 1.0, 0.0
    rows = []
    for i in range(steps):
        rows.append((i * dt, x, v))

        def deriv(x_: float, v_: float) -> tuple[float, float]:
            return v_, -k * x_ - c * v_

        k1x, k1v = deriv(x, v)
        k2x, k2v = deriv(x + 0.5 * dt * k1x, v + 0.5 * dt * k1v)
        k3x, k3v = deriv(x + 0.5 * dt * k2x, v + 0.5 * dt * k2v)
        k4x, k4v = deriv(x + dt * k3x, v + dt * k3v)
        x += dt / 6 * (k1x + 2 * k2x + 2 * k3x + k4x)
        v += dt / 6 * (k1v + 2 * k2v + 2 * k3v + k4v)

    with path.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.writer(handle)
        writer.writerow(["time", "x", "v"])
        writer.writerows((f"{t:.6f}", f"{xv:.10f}", f"{vv:.10f}") for t, xv, vv in rows)


class _Validator(HTMLParser):
    """A tolerant HTML well-formedness check with element/tag accounting."""

    _VOID = {"area", "base", "br", "col", "embed", "hr", "img", "input",
             "link", "meta", "param", "source", "track", "wbr", "rect",
             "line", "polyline", "path", "circle", "use", "stop"}

    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.stack: list[str] = []
        self.tag_counts: dict[str, int] = {}
        self.max_depth = 0

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        self.tag_counts[tag] = self.tag_counts.get(tag, 0) + 1
        if tag not in self._VOID:
            self.stack.append(tag)
            self.max_depth = max(self.max_depth, len(self.stack))

    def handle_startendtag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        self.tag_counts[tag] = self.tag_counts.get(tag, 0) + 1

    def handle_endtag(self, tag: str) -> None:
        if tag in self._VOID:
            return
        # Pop to the matching open tag; tolerate implicit closes.
        if tag in self.stack:
            while self.stack and self.stack.pop() != tag:
                pass


def _validate_html(document: str) -> _Validator:
    validator = _Validator()
    validator.feed(document)
    validator.close()
    if validator.stack:
        raise AssertionError(f"unbalanced HTML; still-open tags: {validator.stack}")
    for required in ("html", "body", "section", "svg", "table"):
        if validator.tag_counts.get(required, 0) < 1:
            raise AssertionError(f"expected at least one <{required}> element")
    return validator


def main() -> None:
    workdir = Path(tempfile.mkdtemp(prefix="lawsynth_scenarios_"))
    csv_path = workdir / "oscillator.csv"
    _write_synthetic_csv(csv_path)
    print(f"synthetic observations written to {csv_path}\n")

    # 1. Observe -> discover the executable world.
    study = lawsynth.Study.from_csv(csv_path, time="time", state=["x", "v"], name="damped_oscillator")
    study.discover(threshold=0.05)
    print("discovered laws:")
    for law in study.explain().laws:
        print(f"  {law.readable}")
    print()

    # 2. Define what-if scenarios (initial-condition overrides vs. baseline).
    (study
        .add_scenario("shock", interventions={"x": 2.0})
        .add_scenario("mitigated", interventions={"x": 0.5, "v": -0.2})
        .add_scenario("kicked", interventions={"v": 1.5}))

    # 3. Compare — baseline is the implicit no-intervention run.
    comparison = study.compare_scenarios(horizon=20.0)
    print("=" * 72)
    print("SCENARIO COMPARISON (final state + divergence from baseline)")
    print("=" * 72)
    print(comparison.table())
    print("=" * 72, "\n")

    # ScenarioComparison renders its own self-contained HTML (multi-series SVG).
    comparison_html = comparison._repr_html_()
    assert "<svg" in comparison_html, "comparison view must embed inline SVG"
    print(f"ScenarioComparison HTML: {len(comparison_html):,} bytes, "
          f"SVG charts: {comparison_html.count('<svg')}\n")

    # 4. Render one cohesive dashboard (composes every view + the scenarios).
    dashboard = study.dashboard()
    document = dashboard.to_document()
    dashboard_path = workdir / "dashboard.html"
    dashboard_path.write_text(document, encoding="utf-8")

    validator = _validate_html(document)
    size_kb = dashboard_path.stat().st_size / 1024
    assert size_kb >= 2.0, f"dashboard is suspiciously small: {size_kb:.1f} KB"
    print(f"dashboard written : {dashboard_path}")
    print(f"dashboard size    : {size_kb:.1f} KB")
    print(f"valid HTML        : yes (max nesting depth {validator.max_depth})")
    print("inline SVG charts : "
          f"{validator.tag_counts.get('svg', 0)}, "
          f"sections: {validator.tag_counts.get('section', 0)}, "
          f"tables: {validator.tag_counts.get('table', 0)}")

    # 5. The decision: which scenario diverges most from the do-nothing baseline?
    ranked = sorted(
        comparison.order, key=lambda label: comparison.distance(label), reverse=True
    )
    print(f"\nlargest divergence from baseline: {ranked[0]!r} "
          f"(‖Δ‖ = {comparison.distance(ranked[0]):.4g})")
    print("decision loop complete.")


if __name__ == "__main__":
    main()
