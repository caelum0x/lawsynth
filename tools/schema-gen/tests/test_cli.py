from __future__ import annotations

import json
import sys
from pathlib import Path

SRC = Path(__file__).resolve().parents[1] / "src"
if str(SRC) not in sys.path:
    sys.path.insert(0, str(SRC))

import jsonschema as js  # noqa: E402
import main  # noqa: E402
import python as py  # noqa: E402
import schema  # noqa: E402
import typescript as ts  # noqa: E402


def test_registry_has_expected_contracts() -> None:
    names = set(schema.contracts())
    assert {"Variable", "Parameter", "Law", "WorldBundle", "ObservationDataset"} <= names


def test_json_schema_is_valid_draft_object() -> None:
    doc = js.to_json_schema(schema.get_contract("WorldBundle"))
    assert doc["type"] == "object"
    assert doc["additionalProperties"] is False
    assert doc["required"] == ["spec_version", "kind", "variables", "parameters", "laws"]
    # Array of objects becomes a $ref to the element contract.
    assert doc["properties"]["variables"]["items"] == {"$ref": "./Variable.schema.json"}
    assert doc["properties"]["kind"]["enum"] == ["continuous", "discrete"]


def test_optional_field_is_not_required() -> None:
    doc = js.to_json_schema(schema.get_contract("Variable"))
    assert "unit" not in doc["required"]
    assert doc["properties"]["id"]["pattern"].startswith("^")


def test_typescript_interface_shape() -> None:
    source = ts.to_typescript(schema.get_contract("Variable"))
    assert "export interface Variable {" in source
    assert "unit?: string;" in source
    assert '"State" | "Control"' in source


def test_python_dataclass_orders_optionals_last() -> None:
    source = py.to_python(schema.get_contract("Variable"))
    assert "@dataclass(frozen=True)" in source
    assert source.index("id: str") < source.index("unit: str | None = None")


def test_render_json_is_deterministic() -> None:
    first = main.render("json", None)
    second = main.render("json", None)
    assert first == second
    parsed = json.loads(first)
    assert set(parsed) == set(schema.contracts())


def test_cli_list_and_write(tmp_path: Path, capsys) -> None:
    assert main.main(["list"]) == 0
    listed = capsys.readouterr().out.splitlines()
    assert "WorldBundle" in listed

    assert main.main(["json", "--out", str(tmp_path)]) == 0
    schema_file = tmp_path / "WorldBundle.schema.json"
    assert schema_file.is_file()
    doc = json.loads(schema_file.read_text(encoding="utf-8"))
    assert doc["title"] == "World IR bundle payload"


def test_cli_unknown_contract_reports_error() -> None:
    assert main.main(["ts", "--contract", "Nope"]) == 2
