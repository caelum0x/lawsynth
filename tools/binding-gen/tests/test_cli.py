"""Tests for the binding generator.

The scanner is exercised against a representative Rust snippet and against the
real ``crates/lawsynth-api-types`` crate when it is present, then each generator
is checked for deterministic, well-formed output.  No external toolchain is
required.
"""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))

import protobuf  # noqa: E402
import python  # noqa: E402
import typescript  # noqa: E402
from rust import parse_type, render, scan_source  # noqa: E402

REPO_ROOT = Path(__file__).resolve().parents[3]
CRATE = REPO_ROOT / "crates" / "lawsynth-api-types"

SAMPLE = """
pub struct ProjectId(String);

#[derive(Clone, Debug, Eq, PartialEq)]
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
    pub outputs: Vec<String>,
}
"""


@pytest.fixture
def schema():
    return scan_source(SAMPLE)


def test_parse_type_variants() -> None:
    assert parse_type("u64").kind == "primitive"
    assert parse_type("Option<u64>").kind == "optional"
    assert parse_type("Vec<String>").kind == "list"
    assert parse_type("ProjectId").kind == "named"


def test_scan_source_extracts_surface(schema) -> None:
    assert [enum.name for enum in schema.enums] == ["RunStatus"]
    assert schema.enums[0].variants == ("Queued", "Running", "Succeeded")
    assert [nt.name for nt in schema.newtypes] == ["ProjectId"]
    struct = schema.structs[0]
    assert struct.name == "RunSummary"
    assert {field.name for field in struct.fields} == {
        "id", "project_id", "status", "created_at_ms", "finished_at_ms", "outputs",
    }


def test_python_generator(schema) -> None:
    out = python.render(schema)
    assert "class RunStatus(str, Enum):" in out
    assert '    QUEUED = "Queued"' in out
    assert "@dataclass(frozen=True)" in out
    assert "finished_at_ms: int | None" in out
    assert "outputs: list[str]" in out
    assert 'ProjectId = NewType("ProjectId", str)' in out


def test_typescript_generator(schema) -> None:
    out = typescript.render(schema)
    assert 'export type RunStatus = "Queued" | "Running" | "Succeeded";' in out
    assert "export interface RunSummary {" in out
    assert "finished_at_ms?: number | null;" in out
    assert "outputs: Array<string>;" in out


def test_protobuf_generator(schema) -> None:
    out = protobuf.render(schema)
    assert 'syntax = "proto3";' in out
    assert "enum RunStatus {" in out
    assert "RUN_STATUS_UNSPECIFIED = 0;" in out
    assert "repeated string outputs" in out
    assert "optional uint64 finished_at_ms" in out


def test_rust_prelude(schema) -> None:
    out = render(schema)
    assert "pub use lawsynth_api_types::{" in out
    assert "RunStatus," in out


def test_generation_is_deterministic(schema) -> None:
    assert python.render(schema) == python.render(scan_source(SAMPLE))


@pytest.mark.skipif(not CRATE.is_dir(), reason="api-types crate not present")
def test_real_crate_scans() -> None:
    from rust import scan_crate

    schema = scan_crate(CRATE)
    names = schema.type_names
    assert {"ProjectId", "RunStatus", "SimulationRequest"} <= names
    # Every generator must run cleanly over the real surface.
    for generator in (python.render, typescript.render, protobuf.render, render):
        assert generator(schema).strip()
