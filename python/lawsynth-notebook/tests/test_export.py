from lawsynth_notebook.display import render_json
from lawsynth_notebook.export import export_html, export_json, reproducible_notebook_cell


def test_exports_are_static_and_reproducible(tmp_path):
    view = render_json("x", {"a": 1})
    assert export_html(view, tmp_path / "x.html").exists()
    assert export_json(view, tmp_path / "x.json").read_text() == '{"a":1}\n'
    assert reproducible_notebook_cell(view)["metadata"]["lawsynth"]["reproducible"]
