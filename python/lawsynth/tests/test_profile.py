"""Unit tests for the pure-stdlib data profiling product."""

import lawsynth
from lawsynth import Dataset, DataProfile, profile
from lawsynth.errors import ValidationError


def _dataset() -> Dataset:
    time = [round(0.5 * i, 3) for i in range(12)]
    x = [float(i * i) for i in range(12)]
    bias = [3.0 for _ in time]  # constant / degenerate
    return Dataset.from_columns(time, {"x": x, "bias": bias})


def test_profiles_dataset_columns_and_time():
    result = profile(_dataset(), name="t")
    assert isinstance(result, DataProfile)
    assert result.rows == 12
    assert {c.name for c in result.columns} == {"x", "bias"}
    x = result.column("x")
    assert x.count == 12 and x.missing == 0
    assert x.minimum == 0.0 and x.maximum == 121.0
    assert not x.is_constant
    assert result.column("bias").is_constant
    assert result.time.monotonic and result.time.regular


def test_constant_column_and_short_series_emit_warnings():
    result = profile(Dataset.from_columns([0, 1, 2], {"c": [5.0, 5.0, 5.0]}), name="c")
    joined = "\n".join(result.warnings)
    assert "constant" in joined
    assert "rows" in joined  # short-series warning


def test_csv_reports_missing_and_irregular_sampling():
    rows = [
        "time,x,y",
        "0,1.0,2.0",
        "1,2.0,",  # missing y
        "2,3.0,4.0",
        "3.5,4.0,5.0",  # irregular step
    ]
    result = profile("\n".join(rows), time="time", state=["x", "y"])
    assert result.rows == 4
    assert result.column("y").missing == 1
    assert result.column("y").count == 3
    assert result.time.monotonic
    assert not result.time.regular
    joined = "\n".join(result.warnings)
    assert "missing" in joined and "irregular" in joined


def test_to_dict_and_to_text_are_serialisable():
    result = profile(_dataset(), name="t")
    payload = result.to_dict()
    assert payload["rows"] == 12
    assert payload["time"]["monotonic"] is True
    assert isinstance(payload["columns"], list)
    text = result.to_text()
    assert "Data profile" in text and "bias" in text


def test_repr_html_uses_brand_palette():
    html = profile(_dataset(), name="t")._repr_html_()
    for token in ("#18201d", "#f3f0e8".replace("f3f0e8", "fffdf7"), "#b54b2a", "Georgia", "ui-monospace"):
        assert token in html


def test_study_profile_matches_dataset_profile():
    dataset = _dataset()
    study = lawsynth.Study.from_dataset(dataset, state=["x"], name="s")
    from_study = study.profile()
    assert from_study.rows == profile(dataset).rows
    assert from_study.name == "s"


def test_unknown_columns_raise():
    try:
        profile(_dataset(), state=["nope"])
    except ValidationError:
        pass
    else:
        raise AssertionError("unknown column was accepted")
