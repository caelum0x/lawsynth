from .errors import ValidationError
from .repositories import Repository


class RunRepository(Repository):
    VALID_STATUSES = frozenset({"queued", "running", "succeeded", "failed", "cancelled"})
    def __init__(self) -> None:
        super().__init__("run")

    def create(self, organization_id: str, values: dict[str, object]) -> dict[str, object]:
        if values.get("status", "queued") not in self.VALID_STATUSES:
            raise ValidationError("invalid run status")
        return super().create(organization_id, {"status": "queued", **values})
