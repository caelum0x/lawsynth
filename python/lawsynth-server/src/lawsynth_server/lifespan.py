from __future__ import annotations

from contextlib import AbstractContextManager

from .dependencies import Services, build_services
from .settings import Settings


class ServiceLifespan(AbstractContextManager[Services]):
    def __init__(self, settings: Settings) -> None:
        self.settings, self.services = settings, None

    def __enter__(self) -> Services:
        self.services = build_services(self.settings)
        return self.services

    def __exit__(self, *_: object) -> None:
        if self.services:
            self.services.database.close()
            self.services = None
