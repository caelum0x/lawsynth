import pytest
from lawsynth_bench.errors import SchemaError
from lawsynth_bench.problem import Problem
from lawsynth_bench.registry import Registry
def test_registry_rejects_duplicate_names():
    registry = Registry(); registry.register(Problem("linear", "ode"))
    with pytest.raises(SchemaError): registry.register(Problem("linear", "ode"))
