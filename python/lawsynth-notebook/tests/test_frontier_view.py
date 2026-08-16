from lawsynth_notebook.frontier_view import normalize_frontier


def test_frontier_sorts_by_score_then_complexity():
    result = normalize_frontier([{"id":"b","score":2,"complexity":1}, {"id":"a","score":1,"complexity":4}])
    assert [item["id"] for item in result] == ["a", "b"]
