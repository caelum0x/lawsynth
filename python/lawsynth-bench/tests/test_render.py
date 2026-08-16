from lawsynth_bench.render import markdown
from lawsynth_bench.report import build
def test_markdown_has_table(rows):
    assert "| Problem |" in markdown(build(rows))
