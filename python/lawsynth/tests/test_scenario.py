from lawsynth.intervention import Intervention
from lawsynth.scenario import Scenario


def test_scenario_sorts_scheduled_interventions():
    scenario = Scenario({"x": 1.0}, interventions=(Intervention(2, "p", 2), Intervention(1, "p", 1)))
    assert [item.time for item in scenario.interventions] == [1, 2]
