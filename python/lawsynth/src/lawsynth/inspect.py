"""Inspectable native world summaries without leaking implementation details."""


def world_summary(world: object) -> dict[str, object]:
    """Return stable equation text exposed by a native World."""
    equations = dict(world.equations())
    return {"equations": equations, "state_count": len(equations)}
