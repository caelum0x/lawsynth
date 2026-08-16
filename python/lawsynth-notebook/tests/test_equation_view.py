from lawsynth_notebook.equation_view import equation_rows, render_equations


def test_equations_are_never_executed():
    rows = equation_rows({"x": "__import__('os').system('false')"})
    assert rows[0][0] == "x" and "system" in render_equations(dict(rows)).html
