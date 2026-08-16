"""Privacy-preserving counters: no payload or dataset content is retained."""

from __future__ import annotations

from collections import Counter


class Telemetry:
    def __init__(self, enabled: bool = False) -> None:
        self.enabled, self._counts = enabled, Counter()

    def record(self, name: str, status: int) -> None:
        if self.enabled:
            self._counts[(name, str(status))] += 1

    def snapshot(self) -> dict[str, int]:
        return {f"{name}:{status}": count for (name, status), count in self._counts.items()}
