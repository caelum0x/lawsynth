"""Collect identically named example contracts without module collisions.

Every self-contained example intentionally uses the familiar
``test_example.py`` name.  The collector gives each file a deterministic,
path-derived import name so a normal ``pytest examples`` invocation executes
all of them instead of importing the first one twenty times.
"""
from __future__ import annotations

import hashlib
import importlib.util
import sys
from pathlib import Path

import pytest


class ExampleContractModule(pytest.Module):
    def _getobj(self):
        path = Path(str(self.path))
        digest = hashlib.sha256(str(path).encode("utf-8")).hexdigest()
        name = f"_lawsynth_example_contract_{digest}"
        spec = importlib.util.spec_from_file_location(name, path)
        if spec is None or spec.loader is None:
            raise ImportError(f"cannot load example contract {path}")
        module = importlib.util.module_from_spec(spec)
        sys.modules[name] = module
        spec.loader.exec_module(module)
        return module


def pytest_pycollect_makemodule(module_path: Path, parent: pytest.Collector):
    if module_path.name == "test_example.py":
        return ExampleContractModule.from_parent(parent, path=module_path)
    return None
