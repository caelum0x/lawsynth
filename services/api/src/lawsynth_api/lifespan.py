"""Lifecycle ownership for one HTTP process."""

from __future__ import annotations

from lawsynth_server.app import Application
from lawsynth_server.settings import Settings as ServerSettings


class ApiLifespan:
    """Own the local domain resources and close them exactly once."""

    def __init__(self, settings: ServerSettings, application: Application | None = None) -> None:
        self._application = application or Application(settings)
        self._closed = False

    @property
    def application(self) -> Application:
        if self._closed:
            raise RuntimeError("the LawSynth API application is closed")
        return self._application

    def close(self) -> None:
        if not self._closed:
            self._application.services.database.close()
            self._closed = True

    def __enter__(self) -> "ApiLifespan":
        return self

    def __exit__(self, *_: object) -> None:
        self.close()
