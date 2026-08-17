#!/usr/bin/env python3
"""Model governance (P9) end-to-end: model card + lineage + audit log.

Run it (from the repository root)::

    PYTHONPATH=python/lawsynth/src python3 python/lawsynth/examples/governance_model_card.py

It generates a deterministic damped-oscillator series, discovers the governing
laws, then exercises the three P9 governance surfaces:

1. **Model card** — a standardized, self-contained HTML document bundling the
   recovered laws, their assumptions, in-window fit, out-of-sample skill (holdout
   validation + rolling-origin backtest), ensemble term stability, and an
   explicit "known limitations / not validated" section. Unmeasured fields are
   marked absent, never fabricated.
2. **Lineage** — a content-addressed chain (dataset -> discovery -> world ->
   evaluations -> report). ``verify_reproducible()`` re-runs the recorded dataset
   + config and asserts the same world revision hash.
3. **Audit log** — an append-only, tamper-evident hash chain. ``verify()`` is
   True for an intact log and False once an entry is altered.

Everything is deterministic and offline.
"""

from __future__ import annotations

import dataclasses
import tempfile
from pathlib import Path

import lawsynth
from lawsynth import AuditLog, Study


def _damped_oscillator(steps: int = 700, dt: float = 0.02) -> tuple[list[float], dict[str, list[float]]]:
    """A deterministic damped harmonic oscillator: x'' = -k·x - c·x'."""
    k, c = 1.0, 0.3
    x, v = 1.0, 0.0
    time: list[float] = []
    xs: list[float] = []
    vs: list[float] = []
    for i in range(steps):
        time.append(i * dt)
        xs.append(x)
        vs.append(v)

        def deriv(x_: float, v_: float) -> tuple[float, float]:
            return v_, -k * x_ - c * v_

        k1x, k1v = deriv(x, v)
        k2x, k2v = deriv(x + 0.5 * dt * k1x, v + 0.5 * dt * k1v)
        k3x, k3v = deriv(x + 0.5 * dt * k2x, v + 0.5 * dt * k2v)
        k4x, k4v = deriv(x + dt * k3x, v + dt * k3v)
        x += dt / 6 * (k1x + 2 * k2x + 2 * k3x + k4x)
        v += dt / 6 * (k1v + 2 * k2v + 2 * k3v + k4v)
    return time, {"x": xs, "v": vs}


def main() -> None:
    workdir = Path(tempfile.mkdtemp(prefix="lawsynth_governance_"))
    rule = "=" * 68

    # 1. Observe -> discover.
    time, columns = _damped_oscillator()
    study = Study.from_columns(time, columns, state=["x", "v"], name="damped_oscillator")
    study.discover(threshold=0.05)
    print(f"discovered {len(study.world.equations())} laws for '{study.name}'\n")

    # 2. Build the model card — validate + backtest + ensemble populated.
    card = study.model_card(holdout=0.25, origins=5, ensemble_n=10, ensemble_seed=0)
    print(rule)
    print(card.to_text())
    print(rule, "\n")

    card_path = card.save(workdir / "model_card.html")
    print(f"model card HTML : {card_path}  ({card_path.stat().st_size} bytes)")
    print(f"world revision  : {card.world_revision}")
    print(f"report hash     : {card.digest}\n")

    # 3. Lineage — content-addressed chain + reproducibility check.
    lineage = card.lineage
    print(rule)
    print(lineage.to_text())
    print(rule)
    print(f"verify_reproducible(): {lineage.verify_reproducible()}")
    lineage_path = workdir / "lineage.json"
    lineage_path.write_text(lineage.to_json(), encoding="utf-8")
    print(f"lineage exported: {lineage_path}\n")

    # 4. Audit log — append-only, tamper-evident.
    audit_path = workdir / "audit.jsonl"
    log = AuditLog(audit_path)
    log.append("alice", "submit", world=card.world_revision[:12])
    log.append("bob", "evaluate", holdout_r2=round(card.validation.mean_r_squared, 4))
    log.append("carol", "approve", report=card.digest[:12])
    print(rule)
    print(log.to_text())
    print(rule)
    print(f"log.verify(): {log.verify()}   (file: {AuditLog.verify_file(audit_path)})")

    # Tamper with a middle entry and show the chain break is detected.
    forged = dataclasses.replace(log.entries[1], action="silently-rejected")
    log._entries = (log.entries[0], forged, log.entries[2])  # simulate tampering
    print(f"log.verify() after tampering with entry #1: {log.verify()}")
    print("\ngovernance demo complete — deterministic, offline.")


if __name__ == "__main__":
    main()
