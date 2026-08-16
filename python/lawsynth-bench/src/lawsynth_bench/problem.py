"""Definitions of benchmark problems, independent of any solver."""
from dataclasses import dataclass, field
from typing import Mapping
from .errors import SchemaError

@dataclass(frozen=True, slots=True)
class Problem:
    name: str
    category: str
    description: str = ""
    tags: tuple[str, ...] = ()
    metadata: Mapping[str, str] = field(default_factory=dict)

    def __post_init__(self) -> None:
        if not self.name.strip() or not self.category.strip():
            raise SchemaError("problem name and category are required")

    @classmethod
    def from_dict(cls, value: Mapping[str, object]) -> "Problem":
        try:
            return cls(str(value["name"]), str(value["category"]), str(value.get("description", "")),
                       tuple(sorted(map(str, value.get("tags", ())))),
                       {str(k): str(v) for k, v in dict(value.get("metadata", {})).items()})
        except (KeyError, TypeError) as exc:
            raise SchemaError("invalid problem document") from exc

    def to_dict(self) -> dict[str, object]:
        return {"name": self.name, "category": self.category, "description": self.description,
                "tags": list(self.tags), "metadata": dict(sorted(self.metadata.items()))}
