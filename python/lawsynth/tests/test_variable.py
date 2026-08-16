from lawsynth.errors import ValidationError
from lawsynth.variable import Variable


def test_variable_requires_a_valid_name_and_role():
    assert Variable("x", "state").name == "x"
    try:
        Variable("not valid", "state")
    except ValidationError:
        pass
    else:
        raise AssertionError("invalid variable accepted")
