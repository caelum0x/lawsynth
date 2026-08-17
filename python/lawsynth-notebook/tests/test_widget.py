"""Tests for the notebook widgets, including the interactive World Explorer.

The embedded JavaScript cannot run in this headless environment, so these tests
validate the Python side and the emitted bundle rigorously: the world -> JS
payload serialization is exact (parameterised laws reproduce the originals),
the mimebundle is valid, self-contained HTML (balanced tags via stdlib
``HTMLParser``) carrying the integrator JS, the world-as-JSON payload, a slider
per parameter and an ``<svg>``, and it references no external URL.
"""

from __future__ import annotations

from html.parser import HTMLParser

import pytest

from lawsynth_notebook.display import render_json
from lawsynth_notebook.explorer import explore
from lawsynth_notebook.explorer_math import evaluate, integrate, parse_expression
from lawsynth_notebook.explorer_payload import build_payload, flatten_terms
from lawsynth_notebook.widget import NotebookWidget, WorldExplorerWidget

# A representative discovered Lotka-Volterra world in the exact string format the
# native engine emits (coefficients baked into the arithmetic). This keeps the
# core bundle/serialization tests independent of the compiled extension.
LOTKA_EQUATIONS = {
    "x": "((1.04472645743551062e0*x)+(-3.95330420201189414e-1*(x*y)))",
    "y": "((-4.13098911450876627e-1*y)+(1.02763237084625800e-1*(x*y)))",
}
LOTKA_INITIAL = {"x": 1.5, "y": 1.0}

_HTML_VOID = {"area", "base", "br", "col", "embed", "hr", "img", "input",
              "link", "meta", "param", "source", "track", "wbr"}


class _BalanceChecker(HTMLParser):
    """Assert every non-void element is properly opened and closed.

    ``HTMLParser`` treats ``<script>``/``<style>`` bodies as CDATA, so the
    JavaScript (with its ``<``/``>`` operators) is never mis-parsed as markup.
    """

    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.stack: list[str] = []
        self.errors: list[str] = []

    def handle_starttag(self, tag: str, attrs: object) -> None:
        if tag not in _HTML_VOID:
            self.stack.append(tag)

    def handle_startendtag(self, tag: str, attrs: object) -> None:
        # self-closing element (e.g. <input .../>, <rect .../>) — no nesting.
        return

    def handle_endtag(self, tag: str) -> None:
        if tag in _HTML_VOID:
            return
        if not self.stack:
            self.errors.append(f"closing </{tag}> with empty stack")
        elif self.stack[-1] != tag:
            self.errors.append(f"mismatched </{tag}>, open is <{self.stack[-1]}>")
        else:
            self.stack.pop()


def _lotka_widget() -> WorldExplorerWidget:
    payload = build_payload(
        name="lotka",
        states=["x", "y"],
        equations=LOTKA_EQUATIONS,
        initial=LOTKA_INITIAL,
        start=0.0,
        end=6.0,
        step=0.05,
    )
    return WorldExplorerWidget(payload=payload, theme="light")


# --------------------------------------------------------------------------- #
# Existing NotebookWidget contract                                             #
# --------------------------------------------------------------------------- #


def test_widget_delegates_ipython_protocol():
    assert "text/html" in NotebookWidget(render_json("x", {"a": 1}))._repr_mimebundle_()


# --------------------------------------------------------------------------- #
# Payload serialization                                                        #
# --------------------------------------------------------------------------- #


def test_flatten_terms_factors_coefficients():
    terms = flatten_terms(LOTKA_EQUATIONS["x"])
    assert terms[0][0] == pytest.approx(1.04472645743551062)
    assert terms[0][1] == ("x",)
    assert terms[1][0] == pytest.approx(-3.95330420201189414e-1)
    assert terms[1][1] == ("x", "y")


def test_build_payload_exposes_one_parameter_per_term():
    payload = build_payload(
        name="lotka", states=["x", "y"], equations=LOTKA_EQUATIONS,
        initial=LOTKA_INITIAL, start=0.0, end=6.0, step=0.05,
    )
    assert payload["states"] == ["x", "y"]
    assert payload["initial"] == {"x": 1.5, "y": 1.0}
    assert payload["time"] == {"start": 0.0, "end": 6.0, "step": 0.05}
    # two additive terms per state -> four parameters.
    assert len(payload["parameters"]) == 4
    ids = [p["id"] for p in payload["parameters"]]
    assert ids == ["k_x_0", "k_x_1", "k_y_0", "k_y_1"]
    for param in payload["parameters"]:
        assert param["min"] < param["value"] < param["max"]
        assert param["step"] > 0
    # laws reference parameter ids, never the raw coefficients.
    assert payload["laws"]["x"] == "(k_x_0*(x))+(k_x_1*(x*y))"
    assert "1.0447" not in payload["laws"]["x"]


def test_parameterised_laws_reproduce_original_expressions():
    payload = build_payload(
        name="lotka", states=["x", "y"], equations=LOTKA_EQUATIONS,
        initial=LOTKA_INITIAL, start=0.0, end=6.0, step=0.05,
    )
    params = {p["id"]: p["value"] for p in payload["parameters"]}
    scope = {"x": 1.3, "y": 0.7, "t": 0.0}
    for state in ("x", "y"):
        original = evaluate(parse_expression(LOTKA_EQUATIONS[state]), scope)
        rebuilt = evaluate(parse_expression(payload["laws"][state]), {**params, **scope})
        assert rebuilt == pytest.approx(original, rel=1e-12, abs=1e-12)


def test_evaluator_supports_unary_functions_and_pow():
    node = parse_expression("exp(0) + sin(0) + 2**3 + abs(neg(4))")
    assert evaluate(node, {}) == pytest.approx(1 + 0 + 8 + 4)


def test_integrator_is_deterministic_and_matches_rk4_reference():
    # Linear decay dx/dt = -x has closed form x(t) = x0 * e^{-t}.
    result = integrate(["x"], {"x": "(-1*x)"}, {}, {"x": 1.0}, start=0.0, end=1.0, step=0.001)
    import math

    assert result["values"]["x"][-1] == pytest.approx(math.exp(-1.0), rel=1e-4)
    again = integrate(["x"], {"x": "(-1*x)"}, {}, {"x": 1.0}, start=0.0, end=1.0, step=0.001)
    assert again == result  # deterministic, offline


# --------------------------------------------------------------------------- #
# Self-contained interactive bundle                                            #
# --------------------------------------------------------------------------- #


def test_mimebundle_is_produced_with_all_representations():
    mimebundle = _lotka_widget()._repr_mimebundle_()
    assert set(mimebundle) == {"text/html", "application/json", "text/plain"}
    assert mimebundle["text/html"].startswith("<section")


def test_bundle_is_valid_balanced_html():
    html = _lotka_widget().html()
    checker = _BalanceChecker()
    checker.feed(html)
    assert checker.errors == []
    assert checker.stack == []


def test_bundle_embeds_integrator_and_payload():
    html = _lotka_widget().html()
    # embedded expression evaluator + integrator.
    for marker in ("function tokenize(", "function parse(", "function evalNode(", "function integrate("):
        assert marker in html
    # the world-as-JSON payload travels inside the bundle.
    assert "var PAYLOAD=" in html
    assert '"laws"' in html and '"k_x_0"' in html
    assert '<svg' in html


def test_bundle_has_a_slider_for_every_parameter():
    widget = _lotka_widget()
    html = widget.html()
    assert html.count('type="range"') == len(widget.payload["parameters"])
    for param in widget.payload["parameters"]:
        assert f'data-id="{param["id"]}"' in html
    # play/reset controls for scenario playback.
    assert 'class="ls-play"' in html and 'class="ls-reset"' in html


def test_bundle_references_no_external_urls():
    html = _lotka_widget().html()
    lowered = html.lower()
    for needle in ("http://", "https://", "//cdn", "<link", "<script src", "src=\"http", "import "):
        assert needle not in lowered


def test_prerendered_svg_contains_a_real_trajectory():
    # The server-side SVG must contain polylines (one per state) so a trajectory
    # is visible even if the notebook front-end strips inline scripts.
    html = _lotka_widget().html()
    assert html.count("<polyline") >= 2


def test_dark_theme_uses_dark_palette_tokens():
    payload = build_payload(
        name="lotka", states=["x", "y"], equations=LOTKA_EQUATIONS,
        initial=LOTKA_INITIAL, start=0.0, end=6.0, step=0.05,
    )
    html = WorldExplorerWidget(payload=payload, theme="dark").html()
    assert "#111827" in html  # dark background token


# --------------------------------------------------------------------------- #
# explore() over a real discovered world (requires the native extension)       #
# --------------------------------------------------------------------------- #


def _real_discovery():
    lawsynth = pytest.importorskip("lawsynth")
    pytest.importorskip("lawsynth._native")
    from lawsynth.study import Study

    times = [i * 0.05 for i in range(200)]
    xs, ys = [], []
    xi, yi, dt = 1.5, 1.0, 0.05
    for _ in range(200):
        xs.append(xi)
        ys.append(yi)
        dx = 1.1 * xi - 0.4 * xi * yi
        dy = -0.4 * yi + 0.1 * xi * yi
        xi += dx * dt
        yi += dy * dt
    study = Study.from_columns(times, {"x": xs, "y": ys}, state=["x", "y"], name="lotka")
    return study.discover(polynomial_degree=2, threshold=0.05)


def test_explore_builds_widget_for_a_real_discovered_world():
    result = _real_discovery()
    widget = explore(result)
    assert isinstance(widget, WorldExplorerWidget)
    assert widget.payload["states"] == ["x", "y"]
    assert widget.payload["parameters"], "discovered world should expose tunable term weights"
    # initial conditions come from the observed baseline (first sample).
    assert widget.payload["initial"]["x"] == pytest.approx(1.5)
    html = widget.html()
    _BalanceChecker().feed(html)
    assert '<svg' in html and 'var PAYLOAD=' in html
    assert "http://" not in html and "https://" not in html


def test_enable_explore_attaches_method():
    from lawsynth_notebook import enable_explore

    enable_explore()
    result = _real_discovery()
    assert callable(getattr(result, "explore", None))
    assert isinstance(result.explore(theme="dark"), WorldExplorerWidget)
