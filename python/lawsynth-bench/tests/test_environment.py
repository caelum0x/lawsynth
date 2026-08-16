from lawsynth_bench.environment import compatible, Environment
def test_environment_compatibility():
    value = Environment.capture().to_dict()
    assert compatible(value, value)
