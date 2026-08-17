#!/usr/bin/env python3
"""Streaming / online discovery — detect a regime switch and re-discover the law.

Run it (from the repository root)::

    PYTHONPATH=python/lawsynth/src python3 python/lawsynth/examples/stream_discovery.py

A single series is processed *as if it streamed*: a window advances across the
time column, a current model is maintained, and the model is re-discovered only
on a **sustained** standardized-residual drift — a regime/law change — never on a
transient outlier. The series runs a coupled spiral for its first half and a
decoupled decay for its second; ``stream_discover`` finds the first system,
detects the switch, and produces a second model with a change record naming the
terms that differ. Deterministic and offline: replaying the identical series
yields byte-identical models and change records.

Honesty: this is not incremental learning — each model is re-discovered from
scratch over its triggering window (a batched re-run), not updated in place.
"""

from __future__ import annotations

import lawsynth


def _rk4(x, y, f, dt):
    k1 = f(x, y)
    k2 = f(x + 0.5 * dt * k1[0], y + 0.5 * dt * k1[1])
    k3 = f(x + 0.5 * dt * k2[0], y + 0.5 * dt * k2[1])
    k4 = f(x + dt * k3[0], y + dt * k3[1])
    return (
        x + dt / 6 * (k1[0] + 2 * k2[0] + 2 * k3[0] + k4[0]),
        y + dt / 6 * (k1[1] + 2 * k2[1] + 2 * k3[1] + k4[1]),
    )


def _regime_switch_series(dt=0.02, half=400):
    """A coupled spiral (regime A) then a decoupled decay (regime B)."""
    regime_a = lambda x, y: (-0.5 * x + 2.0 * y, -2.0 * x - 0.5 * y)  # noqa: E731
    regime_b = lambda x, y: (-1.5 * x, -0.3 * y)  # noqa: E731
    times, xs, ys = [], [], []
    t, x, y = 0.0, 1.0, 0.5
    for _ in range(half):
        times.append(t); xs.append(x); ys.append(y)
        x, y = _rk4(x, y, regime_a, dt); t += dt
    x, y = 1.0, 1.0  # reseed so regime B is excited, not already decayed
    for _ in range(half):
        times.append(t); xs.append(x); ys.append(y)
        x, y = _rk4(x, y, regime_b, dt); t += dt
    return times, xs, ys


def main() -> None:
    times, xs, ys = _regime_switch_series()
    switch_time = times[len(times) // 2]

    # A Study over the whole series; .stream() does not require a prior discover().
    study = lawsynth.Study.from_columns(
        times, {"x": xs, "y": ys}, state=["x", "y"], name="regime-switch"
    )
    history = study.stream(window=80, step=40, threshold=4.0, sustain=2)

    print(history.to_text())
    print()

    # There should be exactly two models: the initial spiral and the re-discovered
    # decay, with one change record naming the differing (removed) coupling terms.
    assert len(history.models) == 2, f"expected 2 models, got {len(history.models)}"
    assert len(history.changes) == 1, f"expected 1 change record, got {len(history.changes)}"

    change = history.changes[0]
    print(f"Regime change detected near t={switch_time:.2f}:")
    print(f"  prior world revision: {change.prior_revision[:12]}")
    print(f"  new world revision:   {change.new_revision[:12]}")
    print("  differing terms:")
    for term in change.diff:
        print(
            f"    d{term.target}/dt · {term.feature}: "
            f"{term.prior:+.3g} -> {term.new:+.3g} ({term.kind})"
        )
    # The cross-coupling terms (x depends on y, y depends on x) are removed.
    removed = {(t.target, t.feature) for t in change.diff if t.kind == "removed"}
    assert removed, "expected coupling terms to be removed across the regime change"

    # Determinism under replay: the identical series yields identical models and
    # change records, field-for-field.
    replay = lawsynth.stream_discover(
        study.dataset, time="time", state=["x", "y"], window=80, step=40, threshold=4.0, sustain=2
    )
    models_one = [m.to_dict() for m in history.models]
    models_two = [m.to_dict() for m in replay.models]
    changes_one = [c.to_dict() for c in history.changes]
    changes_two = [c.to_dict() for c in replay.changes]
    assert models_one == models_two, "replayed models diverged"
    assert changes_one == changes_two, "replayed change records diverged"
    print("\nreplay determinism: PASS (identical models + change records)")


if __name__ == "__main__":
    main()
