"""Helpers for integrations intentionally excluded from the core install."""

from __future__ import annotations

import importlib
from types import ModuleType

from .errors import DependencyUnavailableError


def dependency(module: str, *, extra: str, connector: str) -> ModuleType:
    try:
        return importlib.import_module(module)
    except ImportError as exc:
        raise DependencyUnavailableError(module, extra=extra, connector=connector) from exc
