from lawsynth_server.lifespan import ServiceLifespan


def test_lifespan_opens_and_closes(settings):
    lifespan = ServiceLifespan(settings)
    with lifespan as services:
        assert services.database.connection.execute("SELECT 1").fetchone() == (1,)
    assert lifespan.services is None
