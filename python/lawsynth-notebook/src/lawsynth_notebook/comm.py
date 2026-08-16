"""In-process message transport useful to notebook frontends and tests.

This is intentionally not a Jupyter comm implementation: consumers bridge it
to their own runtime rather than this package assuming a server is available.
"""

from __future__ import annotations

from collections.abc import Callable, Mapping
from typing import Any

from .errors import ArtifactValidationError


class LocalComm:
    def __init__(self) -> None:
        self._subscribers: list[Callable[[dict[str, Any]], None]] = []
        self.messages: list[dict[str, Any]] = []

    def subscribe(self, callback: Callable[[dict[str, Any]], None]) -> Callable[[], None]:
        self._subscribers.append(callback)
        def unsubscribe() -> None:
            if callback in self._subscribers:
                self._subscribers.remove(callback)
        return unsubscribe

    def send(self, message: Mapping[str, Any]) -> None:
        if not isinstance(message, Mapping) or not all(isinstance(key, str) for key in message):
            raise ArtifactValidationError("comm messages must be string-keyed objects")
        copy = dict(message)
        self.messages.append(copy)
        for callback in tuple(self._subscribers):
            callback(dict(copy))
