from lawsynth_notebook.display import render_json


def test_html_escapes_artifact_data():
    assert "&lt;script&gt;" in render_json("x", {"x": "<script>"}).html
