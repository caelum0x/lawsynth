from lawsynth_bench.problem import Problem
def test_problem_roundtrip():
    problem = Problem.from_dict({"name": "linear", "category": "ode", "tags": ["a"]})
    assert Problem.from_dict(problem.to_dict()) == problem
