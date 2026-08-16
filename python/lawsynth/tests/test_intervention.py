from lawsynth.intervention import Intervention


def test_intervention_records_input_or_parameter_kind():
    assert Intervention(1.0, "u", 2.0, "input").kind == "input"
