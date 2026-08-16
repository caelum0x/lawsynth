from lawsynth_server.dependencies import build_services


def test_services_are_composed(settings):
    services = build_services(settings)
    assert services.projects.kind == "project"
    services.database.close()
