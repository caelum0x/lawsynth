from lawsynth_notebook.display import render_json
from lawsynth_notebook.widget import NotebookWidget


def test_widget_delegates_ipython_protocol():
    assert "text/html" in NotebookWidget(render_json("x", {"a": 1}))._repr_mimebundle_()
