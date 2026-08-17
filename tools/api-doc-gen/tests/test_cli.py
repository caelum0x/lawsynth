"""Tests for the API documentation generator.

The scanner is exercised on a representative Rust snippet and, when present, the
real ``crates/lawsynth-api-types`` crate.  Every surface renderer is checked for
well-formed, deterministic output.  No external toolchain is required.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))

import openapi  # noqa: E402
import python  # noqa: E402
import rust  # noqa: E402
import typescript  # noqa: E402
from rust import scan_source  # noqa: E402

REPO_ROOT = Path(__file__).resolve().parents[3]
CRATE = REPO_ROOT / "crates" / "lawsynth-api-types"

SAMPLE = """
pub struct ProjectId(String);

pub enum RunStatus {
    Queued,
    Running,
    Succeeded,
}

pub struct RunSummary {
    pub id: RunId,
    pub project_id: ProjectId,
    pub status: RunStatus,
    pub created_at_ms: u64,
    pub finished_at_ms: Option<u64>,
}

pub struct Project {
    pub id: ProjectId,
    pub display_name: String,
    pub created_at_ms: u64,
}
"""


@pytest.fixture
def schema():
    return scan_source(SAMPLE)


def test_rust_markdown(schema) -> None:
    out = rust.render(schema)
    assert "# Rust API types" in out
    assert "### `RunSummary`" in out
    assert "`Option<u64>`" in out
    assert "| `ProjectId` | `String` |" in out


def test_python_markdown(schema) -> None:
    out = python.render(schema)
    assert "# Python API types" in out
    assert "class RunStatus(str, Enum)" in out
    assert "`int | None`" in out


def test_typescript_markdown(schema) -> None:
    out = typescript.render(schema)
    assert "# TypeScript API types" in out
    assert 'type RunStatus = "Queued" | "Running" | "Succeeded"' in out
    assert "`finished_at_ms?`" in out


def test_openapi_document(schema) -> None:
    document = json.loads(openapi.render(schema))
    assert document["openapi"] == "3.1.0"
    schemas = document["components"]["schemas"]
    assert schemas["RunStatus"]["enum"] == ["Queued", "Running", "Succeeded"]
    run_summary = schemas["RunSummary"]
    assert run_summary["type"] == "object"
    # Optional fields are omitted from `required`.
    assert "finished_at_ms" not in run_summary["required"]
    assert "created_at_ms" in run_summary["required"]
    # A struct-typed field references its component schema.
    assert run_summary["properties"]["status"] == {"$ref": "#/components/schemas/RunStatus"}
    # An illustrative path exists for the Project resource.
    assert "/projects" in document["paths"]


def test_openapi_is_deterministic(schema) -> None:
    assert openapi.render(schema) == openapi.render(scan_source(SAMPLE))


@pytest.mark.skipif(not CRATE.is_dir(), reason="api-types crate not present")
def test_real_crate_all_surfaces() -> None:
    from rust import scan_crate

    schema = scan_crate(CRATE)
    assert {"ProjectId", "RunStatus", "SimulationRequest"} <= schema.type_names
    document = json.loads(openapi.render(schema))
    assert "SimulationRequest" in document["components"]["schemas"]
    for renderer in (rust.render, python.render, typescript.render):
        assert renderer(schema).strip()
