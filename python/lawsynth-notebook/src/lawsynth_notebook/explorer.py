"""``explore()`` — the rich interactive entry point for a discovered world.

:func:`explore` adapts a live LawSynth object (a ``Study`` or
``DiscoveryResult`` post-discovery, or a bare native ``World``) into a
:class:`~lawsynth_notebook.widget.WorldExplorerWidget`. :func:`enable_explore`
attaches an ``.explore()`` method onto those SDK/native classes so a data
scientist can write ``study.discover().explore()`` and get the live widget.

Everything needed for interaction is baked into the returned widget's bundle;
this module only *builds* it (using the public SDK surface — ``equations``,
``states``, ``simulate``) and never assumes a running Jupyter kernel.
"""

from __future__ import annotations

from collections.abc import Mapping
from typing import Any

from .errors import ArtifactValidationError
from .explorer_payload import build_payload
from .widget import WorldExplorerWidget

__all__ = ["explore", "enable_explore"]


def _equations(source: Any) -> dict[str, str]:
    equations = getattr(source, "equations", None)
    if callable(equations):
        return {str(k): str(v) for k, v in dict(equations()).items()}
    if isinstance(equations, Mapping):
        return {str(k): str(v) for k, v in equations.items()}
    world = getattr(source, "world", None)
    if world is not None and callable(getattr(world, "equations", None)):
        return {str(k): str(v) for k, v in dict(world.equations()).items()}
    raise ArtifactValidationError("source does not expose discoverable equations")


def _baseline(source: Any, states: list[str]) -> tuple[dict[str, float], float, float, float] | None:
    """Extract initial state and time bounds from a Study/DiscoveryResult baseline.

    Uses the object's own ``simulate()`` (native, over the observed window) so
    the widget opens on the same trajectory the SDK would produce.
    """
    if not hasattr(source, "explain") or not callable(getattr(source, "simulate", None)):
        return None
    try:
        traj = source.simulate()
    except Exception:  # native/optional path unavailable — fall back to defaults
        return None
    time = list(traj.time)
    if not time:
        return None
    initial = {state: float(traj.values[state][0]) for state in states if state in traj.values}
    start, end = float(time[0]), float(time[-1])
    step = float(time[1] - time[0]) if len(time) > 1 else (end - start) / 100.0
    return initial, start, end, step


def explore(
    source: Any,
    *,
    initial: Mapping[str, float] | None = None,
    start: float | None = None,
    end: float | None = None,
    step: float | None = None,
    method: str = "rk4",
    theme: str = "light",
    name: str | None = None,
) -> WorldExplorerWidget:
    """Build an interactive :class:`WorldExplorerWidget` for a discovered world."""
    equations = _equations(source)
    states = list(getattr(source, "states", None) or sorted(equations))
    states = [str(state) for state in states]

    baseline = _baseline(source, states)
    if baseline is not None:
        base_initial, base_start, base_end, base_step = baseline
    else:
        base_initial, base_start, base_end, base_step = ({state: 1.0 for state in states}, 0.0, 10.0, 0.1)

    resolved_initial = {**base_initial, **{k: float(v) for k, v in (initial or {}).items()}}
    for state in states:
        resolved_initial.setdefault(state, 0.0)

    payload = build_payload(
        name=name or str(getattr(source, "name", "world")),
        states=states,
        equations=equations,
        initial=resolved_initial,
        start=base_start if start is None else float(start),
        end=base_end if end is None else float(end),
        step=base_step if step is None else float(step),
        method=method,
    )
    return WorldExplorerWidget(payload=payload, theme=theme)


def enable_explore() -> None:
    """Attach ``.explore()`` to the LawSynth SDK/native world classes.

    Best-effort and idempotent, mirroring ``lawsynth.study.enable_rich_display``:
    a missing SDK or native extension simply means the method is not attached.
    """
    def _method(self: Any, **kwargs: Any) -> WorldExplorerWidget:
        return explore(self, **kwargs)

    targets: list[Any] = []
    try:
        from lawsynth import study as _study

        targets.extend([_study.Study, _study.DiscoveryResult])
    except Exception:  # pragma: no cover - SDK optional
        pass
    try:
        from lawsynth import _native

        targets.append(_native.World)
    except Exception:  # pragma: no cover - native optional
        pass
    for target in targets:
        try:
            if getattr(target, "explore", None) is None:
                target.explore = _method  # type: ignore[attr-defined]
        except (AttributeError, TypeError):  # pragma: no cover - defensive
            pass
