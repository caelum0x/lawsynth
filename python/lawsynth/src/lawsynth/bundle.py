"""File-oriented wrappers for native `.lsworld` bundles."""

from os import PathLike


def save(world: object, path: str | PathLike[str]) -> None:
    """Persist a native World using its deterministic bundle encoding."""
    world.save(str(path))


def load(path: str | PathLike[str]):
    """Load a native continuous World from a validated bundle."""
    from ._native import World
    return World.load(str(path))
