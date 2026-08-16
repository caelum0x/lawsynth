from .errors import ValidationError
from .repositories import Repository


class WorldRepository(Repository):
    def __init__(self) -> None:
        super().__init__("world")

    def create(self, organization_id: str, values: dict[str, object]) -> dict[str, object]:
        equations = values.get("equations")
        if not isinstance(equations, list) or not equations or not all(isinstance(x, str) and x.strip() for x in equations):
            raise ValidationError("world requires non-empty equation strings")
        return super().create(organization_id, values)
