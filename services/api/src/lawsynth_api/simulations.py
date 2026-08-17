"""The ``simulate`` action: a sub-resource of ``worlds``.

A simulation is not a stored resource -- it is a ``POST`` action on a world
(``/v1/worlds/{id}/simulate``) that returns a trajectory.  This module owns the
recognition of that action and a typed pass-through to the domain simulation
validator, so the transport layer and the ``worlds`` resource share one
definition of what a simulate request is.  It emits no streaming event: the
domain records ``worlds.simulated`` in its own journal, not on the SSE contract.
"""

from __future__ import annotations

from typing import Sequence

from lawsynth_server.simulations import validate_simulation_spec

ACTION = "simulate"


def is_simulate(parts: Sequence[str]) -> bool:
    """True when ``parts`` addresses a world's simulate action."""

    return len(parts) == 3 and parts[0] == "worlds" and parts[2] == ACTION


def classify(method: str, parts: Sequence[str]) -> str | None:
    """Return the telemetry label for a simulate action, or ``None``."""

    if is_simulate(parts) and method == "POST":
        return "worlds.simulate"
    return None


def normalize_request(spec: object) -> dict[str, object]:
    """Validate and normalize a simulation request via the domain validator.

    Exposed as a typed helper so callers get the exact horizon/step/method rules
    the domain enforces without importing the server package directly.
    """

    return validate_simulation_spec(spec)
