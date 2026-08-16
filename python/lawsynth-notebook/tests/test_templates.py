from lawsynth_notebook.templates import panel, table


def test_templates_escape_values():
    assert "&lt;x&gt;" in table(["x"], [["<x>"]])
    assert "lawsynth-notebook" in panel("t", "body")
