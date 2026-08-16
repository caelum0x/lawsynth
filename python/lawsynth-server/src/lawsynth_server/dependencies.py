from __future__ import annotations

from dataclasses import dataclass

from .artifacts import Artifact
from .auth import TokenAuthenticator
from .database import Database
from .events import EventJournal
from .idempotency import IdempotencyStore
from .projects import ProjectRepository
from .datasets import DatasetRepository
from .runs import RunRepository
from .settings import Settings
from .storage import FileObjectStore
from .telemetry import Telemetry
from .worlds import WorldRepository


@dataclass(slots=True)
class Services:
    settings: Settings
    database: Database
    auth: TokenAuthenticator
    projects: ProjectRepository
    datasets: DatasetRepository
    worlds: WorldRepository
    runs: RunRepository
    events: EventJournal
    idempotency: IdempotencyStore
    storage: FileObjectStore
    telemetry: Telemetry


def build_services(settings: Settings) -> Services:
    return Services(settings, Database(settings.database_url), TokenAuthenticator(settings.tokens), ProjectRepository(), DatasetRepository(), WorldRepository(), RunRepository(), EventJournal(), IdempotencyStore(), FileObjectStore(settings.object_root, max_bytes=settings.max_upload_bytes), Telemetry(settings.telemetry_enabled))
