"""Model governance (P9): the standardized, self-contained **model card**.

A model card bundles everything a reviewer needs to judge a recovered law system
*honestly*:

* the recovered law system and the **assumptions** it is contingent on
  (continuity, the feature library, causal caveats);
* in-window **fit quality**;
* **out-of-sample skill** — a holdout :func:`~lawsynth.validation.validate` and a
  rolling-origin :func:`~lawsynth.backtesting.backtest`;
* **ensemble term-stability** — robust vs unstable terms
  (:meth:`~lawsynth.study.Study.discover_ensemble`);
* an explicit **known limitations / not validated** section.

:func:`model_card` orchestrates the *real* SDK calls and assembles a
:class:`ModelCard`. A section whose input was not produced (an evaluation that was
disabled, or that could not run on a short series) is marked **absent**, never
fabricated. The card renders as branded, dependency-free HTML reusing the SDK
report generator (:mod:`lawsynth.report`).
"""

from __future__ import annotations

from dataclasses import dataclass
from html import escape
from math import isfinite
from os import PathLike
from pathlib import Path
from typing import Mapping, Sequence

from . import report as _report
from ._content import SHORT, content_digest, world_hash
from .errors import LawSynthError, NativeError, ValidationError

__all__ = ["ModelCard", "model_card"]

# Absent-field marker: an em dash, matching the report crate's convention.
_ABSENT = "—"


def _fmt(value: float | None, spec: str = ".4g") -> str:
    """Format a measured number, or the absent marker when unmeasured."""
    if value is None or (isinstance(value, float) and not isfinite(value)):
        return _ABSENT
    return format(value, spec)


# --------------------------------------------------------------------------- #
# ModelCard                                                                    #
# --------------------------------------------------------------------------- #


@dataclass(frozen=True, slots=True)
class ModelCard:
    """An assembled, self-contained governance document for a recovered world.

    Every evaluation-backed section is optional: ``None`` means *not measured*
    and is rendered as an explicit placeholder. The card is immutable; render it
    with :meth:`_repr_html_` / :meth:`save`, or export it via :meth:`to_dict`.
    """

    name: str
    world_revision: str
    engine_version: str
    laws_readable: tuple[str, ...]
    equations: Mapping[str, str]
    assumptions: tuple[str, ...]
    fit: Mapping[str, Mapping[str, float]]
    validation: object | None            # lawsynth.validation.Validation | None
    backtest: object | None              # lawsynth.backtesting.Backtest | None
    ensemble: object | None              # lawsynth.ensemble.Ensemble | None
    monitor: object | None               # lawsynth.monitor.MonitorReport | None
    limitations: tuple[str, ...]
    lineage: object | None               # lawsynth.lineage.Lineage | None
    preview_time: tuple[float, ...]
    preview_values: Mapping[str, tuple[float, ...]]

    # -- structural digest (the report hash recorded in lineage) ------------ #

    @property
    def digest(self) -> str:
        """Content digest over the card's measured sections (its report hash)."""
        payload = {
            "name": self.name,
            "world_revision": self.world_revision,
            "equations": {k: v for k, v in sorted(self.equations.items())},
            "assumptions": list(self.assumptions),
            "fit": {k: dict(v) for k, v in sorted(self.fit.items())},
            "validation": self.validation.to_dict() if self.validation is not None else None,
            "backtest": self.backtest.to_dict() if self.backtest is not None else None,
            "ensemble": self.ensemble.to_dict() if self.ensemble is not None else None,
            "monitor": self.monitor.to_dict() if self.monitor is not None else None,
            "limitations": list(self.limitations),
        }
        return content_digest(payload)

    # -- serialisation ------------------------------------------------------ #

    def to_dict(self) -> dict[str, object]:
        return {
            "name": self.name,
            "world_revision": self.world_revision,
            "engine_version": self.engine_version,
            "report_hash": self.digest,
            "laws": list(self.laws_readable),
            "equations": {k: v for k, v in sorted(self.equations.items())},
            "assumptions": list(self.assumptions),
            "fit": {k: dict(v) for k, v in sorted(self.fit.items())},
            "out_of_sample": {
                "holdout_validation": self.validation.to_dict() if self.validation is not None else None,
                "rolling_origin_backtest": self.backtest.to_dict() if self.backtest is not None else None,
            },
            "ensemble_stability": self.ensemble.to_dict() if self.ensemble is not None else None,
            "monitoring": self.monitor.to_dict() if self.monitor is not None else None,
            "limitations": list(self.limitations),
            "lineage": self.lineage.to_dict() if self.lineage is not None else None,
        }

    def to_text(self) -> str:
        lines = [
            f"MODEL CARD — {self.name}",
            f"  world revision {self.world_revision[:SHORT]} · engine {self.engine_version}",
            f"  report hash    {self.digest[:SHORT]}",
            "",
            "Recovered law system:",
        ]
        lines.extend(f"  {law}" for law in self.laws_readable)
        lines.append("")
        lines.append("Assumptions this model is contingent on:")
        lines.extend(f"  - {item}" for item in self.assumptions)
        lines.append("")
        lines.append("Fit quality (in-window):")
        for state, metrics in sorted(self.fit.items()):
            lines.append(
                f"  {state}: R² = {_fmt(metrics.get('r_squared'), '.4f')}, "
                f"RMSE = {_fmt(metrics.get('rmse'))}"
            )
        lines.append("")
        lines.append("Out-of-sample skill — holdout validation:")
        if self.validation is not None:
            lines.append(
                f"  {self.validation.verdict}; mean R² = "
                f"{self.validation.mean_r_squared:.4f} "
                f"({self.validation.test_samples} held-out samples)"
            )
        else:
            lines.append("  Not measured — no holdout validation was run.")
        lines.append("")
        lines.append("Out-of-sample skill — rolling-origin backtest:")
        if self.backtest is not None:
            lines.append(
                f"  {self.backtest.verdict}; mean R² = "
                f"{self.backtest.mean_r_squared:.4f}; "
                f"{len(self.backtest.origins)} origins, horizon {self.backtest.horizon}"
            )
        else:
            lines.append("  Not measured — no backtest was run.")
        lines.append("")
        lines.append("Ensemble term stability:")
        if self.ensemble is not None:
            robust = self.ensemble.robust_terms()
            lines.append(
                f"  {len(robust)} of {len(self.ensemble.terms)} terms robust "
                f"across {self.ensemble.members} members "
                f"({', '.join(f'{t.target}<-{t.feature}' for t in robust) or 'none'})"
            )
        else:
            lines.append("  Not measured — no ensemble was run.")
        lines.append("")
        lines.append("Known limitations / not validated:")
        lines.extend(f"  - {item}" for item in self.limitations)
        return "\n".join(lines)

    def __str__(self) -> str:
        return self.to_text()

    def __repr__(self) -> str:
        measured = [
            name
            for name, value in (
                ("validation", self.validation),
                ("backtest", self.backtest),
                ("ensemble", self.ensemble),
                ("monitor", self.monitor),
            )
            if value is not None
        ]
        return f"ModelCard(name={self.name!r}, world={self.world_revision[:SHORT]!r}, measured={measured})"

    # -- HTML view ---------------------------------------------------------- #

    def to_html(self, *, theme: str = "light") -> str:
        return _render_model_card_html(self, theme=theme)

    def _repr_html_(self) -> str:
        return self.to_html()

    def save(self, path: str | PathLike[str], *, theme: str = "light") -> Path:
        """Write the model card as a self-contained HTML document."""
        target = Path(path)
        if target.suffix.lower() not in {".html", ".htm"}:
            raise ValidationError("model card path must end in .html or .htm")
        target.write_text(self.to_html(theme=theme), encoding="utf-8")
        return target


# --------------------------------------------------------------------------- #
# HTML rendering — reuses the SDK report generator primitives                  #
# --------------------------------------------------------------------------- #


def _section(colors: Mapping[str, str], heading: str, body: str) -> str:
    return (
        f'<section style="background:{colors["bg"]};border:1px solid {colors["border"]};'
        f'border-radius:10px;padding:16px 18px;margin:14px 0">'
        f'<h2 style="margin:0 0 10px;font-size:16px;color:{colors["accent"]}">{escape(heading)}</h2>'
        f"{body}</section>"
    )


def _not_measured(colors: Mapping[str, str], why: str) -> str:
    return f'<p style="color:{colors["muted"]};margin:0">Not measured — {escape(why)}.</p>'


def _accuracy_table(colors: Mapping[str, str], per_state: Sequence[tuple[str, float, float, float]]) -> str:
    head = "".join(
        f'<th style="padding:6px 10px;text-align:{"left" if i == 0 else "right"};'
        f'color:{colors["muted"]};border-bottom:1px solid {colors["border"]};'
        f'font-size:11px;letter-spacing:0.06em;text-transform:uppercase">{escape(h)}</th>'
        for i, h in enumerate(["state", "RMSE", "MAE", "R²"])
    )
    rows = "".join(
        "<tr>"
        f'<td style="padding:6px 10px;font-weight:600">{escape(state)}</td>'
        f'<td style="padding:6px 10px;text-align:right;font-family:ui-monospace,monospace">{_fmt(rmse)}</td>'
        f'<td style="padding:6px 10px;text-align:right;font-family:ui-monospace,monospace">{_fmt(mae)}</td>'
        f'<td style="padding:6px 10px;text-align:right;font-family:ui-monospace,monospace">{_fmt(r2, ".4f")}</td>'
        "</tr>"
        for state, rmse, mae, r2 in per_state
    )
    return (
        f'<table style="border-collapse:collapse;width:100%;color:{colors["fg"]}">'
        f"<thead><tr>{head}</tr></thead><tbody>{rows}</tbody></table>"
    )


def _validation_body(colors: Mapping[str, str], validation: object | None) -> str:
    if validation is None:
        return _not_measured(colors, "no holdout validation was run")
    rows = [
        (s, validation.rmse[s], validation.mae[s], validation.r_squared[s])
        for s in validation.states
    ]
    note = (
        f'<p style="margin:0 0 10px;color:{colors["muted"]}">Model re-fit on '
        f'{validation.train_samples} leading sample(s), scored on the held-out final '
        f'{validation.test_samples} ({validation.holdout_fraction * 100:.0f}% holdout). '
        f'<b>{escape(validation.verdict)}</b> — mean R² {validation.mean_r_squared:.3f}.</p>'
    )
    return note + _accuracy_table(colors, rows)


def _backtest_body(colors: Mapping[str, str], backtest: object | None, theme: str) -> str:
    if backtest is None:
        return _not_measured(colors, "no rolling-origin backtest was run")
    rows = [
        (s, backtest.rmse[s], backtest.mae[s], backtest.r_squared[s]) for s in backtest.states
    ]
    decay = backtest.decay
    decay_text = f"{decay:.2f}×" if isfinite(decay) else _ABSENT
    note = (
        f'<p style="margin:0 0 10px;color:{colors["muted"]}">Walk-forward from '
        f'{len(backtest.origins)} origin(s), horizon {backtest.horizon} step(s). '
        f'<b>{escape(backtest.verdict)}</b> — mean R² {backtest.mean_r_squared:.3f}; '
        f'error grows {decay_text} from lead 1 to {backtest.horizon}.</p>'
    )
    chart = backtest._skill_chart(theme=theme)
    return note + chart + _accuracy_table(colors, rows)


def _ensemble_body(colors: Mapping[str, str], ensemble: object | None) -> str:
    if ensemble is None:
        return _not_measured(colors, "no ensemble was run")
    head = "".join(
        f'<th style="padding:6px 10px;text-align:{"left" if i < 2 else "right"};'
        f'color:{colors["muted"]};border-bottom:1px solid {colors["border"]};'
        f'font-size:11px;letter-spacing:0.06em;text-transform:uppercase">{escape(h)}</th>'
        for i, h in enumerate(["target", "term", "selection", "mean", "std", "stability"])
    )
    rows = []
    for term in ensemble.terms:
        if term.robust:
            badge = '<span style="color:#2f6f4f;font-weight:600">robust</span>'
        elif term.selection_frequency < 0.6:
            badge = '<span style="color:#a3341f;font-weight:600">unstable</span>'
        else:
            badge = '<span style="color:#b8822a">borderline</span>'
        rows.append(
            "<tr>"
            f'<td style="padding:6px 10px;font-weight:600">{escape(term.target)}</td>'
            f'<td style="padding:6px 10px;font-family:ui-monospace,monospace">{escape(term.feature)}</td>'
            f'<td style="padding:6px 10px;text-align:right">{term.selection_frequency * 100:.0f}%</td>'
            f'<td style="padding:6px 10px;text-align:right;font-family:ui-monospace,monospace">{term.mean:.4g}</td>'
            f'<td style="padding:6px 10px;text-align:right;font-family:ui-monospace,monospace">{term.std:.4g}</td>'
            f'<td style="padding:6px 10px;text-align:right">{badge}</td>'
            "</tr>"
        )
    robust = ensemble.robust_terms()
    note = (
        f'<p style="margin:0 0 10px;color:{colors["muted"]}">{ensemble.members} bootstrap '
        f'member(s); {len(robust)} of {len(ensemble.terms)} term(s) robust. A term flagged '
        '<b style="color:#a3341f">unstable</b> should not be trusted as structure.</p>'
    )
    table = (
        f'<table style="border-collapse:collapse;width:100%;color:{colors["fg"]}">'
        f"<thead><tr>{head}</tr></thead><tbody>{''.join(rows)}</tbody></table>"
    )
    return note + table


def _monitor_body(colors: Mapping[str, str], monitor: object | None) -> str:
    if monitor is None:
        return _not_measured(colors, "no fresh data was provided to monitor")
    color = "#2f6f4f" if monitor.in_control else "#a3341f"
    return (
        f'<p style="margin:0"><span style="color:{color};font-weight:700">'
        f'{escape(monitor.verdict)}</span> <span style="color:{colors["muted"]}">· '
        f'threshold {monitor.threshold:g}σ · {len(monitor.time)} fresh samples · '
        f'{len(monitor.anomalies)} anomaly flag(s)</span></p>'
    )


def _lineage_body(colors: Mapping[str, str], lineage: object | None) -> str:
    if lineage is None:
        return _not_measured(colors, "no lineage was captured")
    rows = "".join(
        f'<tr><td style="padding:4px 10px;color:{colors["muted"]}">{escape(link.kind)}</td>'
        f'<td style="padding:4px 10px;font-family:ui-monospace,monospace">{escape(link.digest[:16])}</td>'
        f'<td style="padding:4px 10px;font-family:ui-monospace,monospace;color:{colors["muted"]}">'
        f'{escape(link.parent[:16])}</td></tr>'
        for link in lineage.links
    )
    world_rev = lineage.world_revision or _ABSENT
    valid = "valid" if lineage.verify_chain() else "BROKEN"
    return (
        f'<p style="margin:0 0 8px;color:{colors["muted"]}">Content-addressed chain '
        f'({len(lineage.links)} links, {valid}); world revision '
        f'<span style="font-family:ui-monospace,monospace">{escape(world_rev[:16])}</span>.</p>'
        f'<table style="border-collapse:collapse">'
        f'<thead><tr>'
        + "".join(
            f'<th style="padding:4px 10px;text-align:left;color:{colors["muted"]};'
            f'font-size:11px;text-transform:uppercase">{escape(h)}</th>'
            for h in ["link", "digest", "parent"]
        )
        + f"</tr></thead><tbody>{rows}</tbody></table>"
    )


def _render_model_card_html(card: ModelCard, *, theme: str = "light") -> str:
    colors = _report._theme(theme)
    trajectory_chart = _report.svg_line_chart(
        card.preview_time, dict(card.preview_values),
        width=760, height=360, title="Default forward trajectory", theme=theme,
    )
    laws_list = "".join(
        f'<li style="margin:4px 0;font-family:ui-monospace,monospace">{escape(law)}</li>'
        for law in card.laws_readable
    )
    assume_list = "".join(f"<li>{escape(item)}</li>" for item in card.assumptions)
    limit_list = "".join(f"<li>{escape(item)}</li>" for item in card.limitations)
    fit_rows = [
        (state, metrics.get("rmse"), None, metrics.get("r_squared"))
        for state, metrics in sorted(card.fit.items())
    ]
    fit_head = "".join(
        f'<th style="padding:6px 10px;text-align:{"left" if i == 0 else "right"};'
        f'color:{colors["muted"]};border-bottom:1px solid {colors["border"]};'
        f'font-size:11px;letter-spacing:0.06em;text-transform:uppercase">{escape(h)}</th>'
        for i, h in enumerate(["state", "R²", "RMSE"])
    )
    fit_table = (
        f'<table style="border-collapse:collapse;width:100%;color:{colors["fg"]}">'
        f"<thead><tr>{fit_head}</tr></thead><tbody>"
        + "".join(
            "<tr>"
            f'<td style="padding:6px 10px;font-weight:600">{escape(state)}</td>'
            f'<td style="padding:6px 10px;text-align:right;font-family:ui-monospace,monospace">{_fmt(metrics.get("r_squared"), ".4f")}</td>'
            f'<td style="padding:6px 10px;text-align:right;font-family:ui-monospace,monospace">{_fmt(metrics.get("rmse"))}</td>'
            "</tr>"
            for state, metrics in sorted(card.fit.items())
        )
        + "</tbody></table>"
    )

    body = "".join([
        f'<h1 style="font-size:22px;margin:0 0 4px">Model card — {escape(card.name)}</h1>',
        f'<p style="color:{colors["muted"]};margin:0 0 8px;font-family:ui-monospace,monospace">'
        f'world {escape(card.world_revision[:SHORT])} · engine {escape(card.engine_version)} · '
        f'report {escape(card.digest[:SHORT])}</p>',
        _section(colors, "Recovered law system",
                 _report.equations_table_html(dict(card.equations), theme)
                 + f'<ul style="margin:10px 0 0;padding-left:18px">{laws_list}</ul>'),
        _section(colors, "Assumptions this model is contingent on",
                 f'<ul style="margin:0;padding-left:18px">{assume_list}</ul>'),
        _section(colors, "Fit quality (in-window)", fit_table),
        _section(colors, "Out-of-sample skill — holdout validation",
                 _validation_body(colors, card.validation)),
        _section(colors, "Out-of-sample skill — rolling-origin backtest",
                 _backtest_body(colors, card.backtest, theme)),
        _section(colors, "Ensemble term stability", _ensemble_body(colors, card.ensemble)),
        _section(colors, "Monitoring against fresh data", _monitor_body(colors, card.monitor)),
        _section(colors, "Known limitations / not validated",
                 f'<ul style="margin:0;padding-left:18px">{limit_list}</ul>'),
        _section(colors, "Default forward trajectory", trajectory_chart),
        _section(colors, "Lineage", _lineage_body(colors, card.lineage)),
    ])
    _ = fit_rows  # retained for parity with the accuracy-table shape
    return (
        '<!doctype html><html lang="en"><head><meta charset="utf-8">'
        '<meta name="viewport" content="width=device-width,initial-scale=1">'
        f"<title>Model card — {escape(card.name)}</title></head>"
        f'<body style="background:{colors["grid"]};color:{colors["fg"]};'
        'font:14px/1.5 system-ui,sans-serif;margin:0;padding:24px">'
        f'<main style="max-width:880px;margin:0 auto">{body}</main></body></html>'
    )


# --------------------------------------------------------------------------- #
# Orchestration — assemble a card from the real SDK evaluations                #
# --------------------------------------------------------------------------- #


def _default_limitations(study: object) -> list[str]:
    span = (float(study.dataset.time[0]), float(study.dataset.time[-1]))  # type: ignore[attr-defined]
    return [
        f"Extrapolation beyond the observed window t ∈ [{span[0]:.4g}, {span[1]:.4g}] "
        "is not validated; forecasts outside it are unsupported.",
        "Continuity is assumed: the model presumes smooth first-order dynamics; "
        "regime changes, discontinuities and stochastic shocks are not represented.",
        "Terms are associational structure fit to observed trajectories — no causal "
        "claim is established by discovery alone.",
        "Skill is measured only over the feature library supplied to discovery; "
        "dynamics requiring absent features cannot be recovered.",
    ]


def model_card(
    study: object,
    *,
    name: str | None = None,
    holdout: float = 0.25,
    origins: int = 5,
    ensemble_n: int = 12,
    ensemble_fraction: float = 0.8,
    ensemble_seed: int = 0,
    monitor: object | None = None,
    run_validate: bool = True,
    run_backtest: bool = True,
    run_ensemble: bool = True,
) -> ModelCard:
    """Assemble a :class:`ModelCard` for a *discovered* ``study``.

    Runs the real SDK evaluations (``explain`` + holdout ``validate`` +
    rolling-origin ``backtest`` + ``discover_ensemble`` + optional ``monitor``)
    and assembles them into a card. Any evaluation that is disabled, or that
    cannot run (e.g. a series too short for a holdout), is honestly recorded as
    **absent** rather than fabricated. The study's captured lineage is extended
    with evaluation and report links.
    """
    world = study.world  # raises if discover() has not run  # type: ignore[attr-defined]
    explanation = study.explain()  # type: ignore[attr-defined]
    card_name = name or f"{study.name}"  # type: ignore[attr-defined]

    validation = None
    if run_validate:
        try:
            validation = study.validate(holdout=holdout)  # type: ignore[attr-defined]
        except (ValidationError, NativeError):
            validation = None

    backtest_result = None
    if run_backtest:
        try:
            backtest_result = study.backtest(origins=origins)  # type: ignore[attr-defined]
        except (ValidationError, NativeError):
            backtest_result = None

    ensemble_result = None
    if run_ensemble:
        try:
            ensemble_result = study.discover_ensemble(  # type: ignore[attr-defined]
                n=ensemble_n, fraction=ensemble_fraction, seed=ensemble_seed
            )
        except (ValidationError, NativeError):
            ensemble_result = None

    monitor_report = None
    if monitor is not None:
        try:
            monitor_report = study.monitor(monitor)  # type: ignore[attr-defined]
        except (ValidationError, NativeError):
            monitor_report = None

    preview = study.simulate()  # type: ignore[attr-defined]

    card = ModelCard(
        name=card_name,
        world_revision=world_hash(world),
        engine_version=_engine_version(),
        laws_readable=tuple(law.readable for law in explanation.laws),
        equations={law.target: law.expression for law in explanation.laws},
        assumptions=tuple(explanation.assumptions),
        fit=explanation.fit,
        validation=validation,
        backtest=backtest_result,
        ensemble=ensemble_result,
        monitor=monitor_report,
        limitations=tuple(_default_limitations(study)),
        lineage=None,
        preview_time=tuple(preview.time),
        preview_values={k: tuple(v) for k, v in preview.values.items()},
    )

    # Extend the study's captured lineage with evaluation + report links, then
    # fold the finished chain back into the (immutable) card.
    lineage = getattr(study, "lineage", None)
    if lineage is not None:
        if validation is not None:
            lineage = lineage.record_evaluation("validate", {
                "mean_r_squared": validation.mean_r_squared,
                "verdict": validation.verdict,
            })
        if backtest_result is not None:
            lineage = lineage.record_evaluation("backtest", {
                "mean_r_squared": backtest_result.mean_r_squared,
                "verdict": backtest_result.verdict,
            })
        if ensemble_result is not None:
            lineage = lineage.record_evaluation("ensemble", {
                "members": ensemble_result.members,
                "robust_terms": len(ensemble_result.robust_terms()),
            })
        lineage = lineage.record_report(card.digest, kind="model_card")

    from dataclasses import replace

    return replace(card, lineage=lineage)


def _engine_version() -> str:
    from ._version import __version__

    return __version__
