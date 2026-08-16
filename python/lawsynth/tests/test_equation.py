from lawsynth.equation import Equation
from lawsynth.errors import ValidationError


def test_equation_requires_target_and_expression():
    assert Equation("x", "x + 1").target == "x"
    try:
        Equation("x", " ")
    except ValidationError:
        pass
    else:
        raise AssertionError("empty expression accepted")
