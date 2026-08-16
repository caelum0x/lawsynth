from pathlib import Path
import json

import pytest


@pytest.fixture
def fixture_root() -> Path:
    return Path(__file__).parents[1] / "fixtures"


def load_fixture(root: Path, name: str) -> dict:
    return json.loads((root / name / "sample.json").read_text())
