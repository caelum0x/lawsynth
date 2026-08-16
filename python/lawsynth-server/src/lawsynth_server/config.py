"""Environment configuration adapter kept separate from runtime settings."""

from __future__ import annotations

import os
from pathlib import Path

from .settings import Settings


def settings_from_environment(prefix: str = "LAWSYNTH_") -> Settings:
    """Load non-secret local configuration from an explicit environment prefix."""
    page_size = int(os.environ.get(f"{prefix}MAX_PAGE_SIZE", "100"))
    max_upload = int(os.environ.get(f"{prefix}MAX_UPLOAD_BYTES", str(64 * 1024 * 1024)))
    root = Path(os.environ.get(f"{prefix}OBJECT_ROOT", ".lawsynth-objects"))
    return Settings(
        database_url=os.environ.get(f"{prefix}DATABASE_URL", ":memory:"),
        object_root=root,
        max_page_size=page_size,
        max_upload_bytes=max_upload,
        telemetry_enabled=os.environ.get(f"{prefix}TELEMETRY", "false").lower() == "true",
    )
