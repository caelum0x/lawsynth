from lawsynth.errors import ValidationError
from lawsynth.units import Unit


def test_native_unit_vocabulary_accepts_composites_and_rejects_unknowns():
    assert str(Unit("km/min")) == "km/min"
    try:
        Unit("furlong")
    except ValidationError:
        pass
    else:
        raise AssertionError("unknown native unit accepted")
