"""A collision-safe in-memory registry of benchmark problem definitions."""
from dataclasses import dataclass, field
from collections.abc import Iterable
from .problem import Problem
from .errors import SchemaError

@dataclass(slots=True)
class Registry:
    _items: dict[str, Problem] = field(default_factory=dict)
    def register(self, problem: Problem) -> None:
        if problem.name in self._items: raise SchemaError(f"duplicate problem: {problem.name}")
        self._items[problem.name] = problem
    def get(self, name: str) -> Problem: return self._items[name]
    def all(self) -> tuple[Problem, ...]: return tuple(self._items[k] for k in sorted(self._items))
    @classmethod
    def from_problems(cls, problems: Iterable[Problem]) -> "Registry":
        registry = cls()
        for problem in problems: registry.register(problem)
        return registry
