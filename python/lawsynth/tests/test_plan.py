from lawsynth.plan import DiscoveryPlan, STAGES


def test_plan_preserves_the_native_stage_contract():
    plan = DiscoveryPlan(("x",))
    assert plan.stages == STAGES
