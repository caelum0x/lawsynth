from lawsynth_server.telemetry import Telemetry


def test_telemetry_is_opt_in():
    disabled, enabled = Telemetry(), Telemetry(True)
    disabled.record("request", 200); enabled.record("request", 200)
    assert disabled.snapshot() == {}
    assert enabled.snapshot() == {"request:200": 1}
