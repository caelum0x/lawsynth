from .errors import ValidationError
from .native import world_spec
from .repositories import Repository


class WorldRepository(Repository):
    def __init__(self) -> None:
        super().__init__("world")

    def create(self, organization_id: str, values: dict[str, object]) -> dict[str, object]:
        equations = values.get("equations")
        if isinstance(equations, dict):
            normalized = world_spec(values)
            return super().create(organization_id, {**values, **normalized})
        if isinstance(equations, list) and equations and all(isinstance(x, str) and x.strip() for x in equations):
            return super().create(organization_id, values)
        raise ValidationError("world requires executable equations or non-empty equation strings")
