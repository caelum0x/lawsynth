from .errors import ValidationError
from .repositories import Repository


class DatasetRepository(Repository):
    def __init__(self) -> None:
        super().__init__("dataset")

    def create(self, organization_id: str, values: dict[str, object]) -> dict[str, object]:
        schema = values.get("schema")
        if not isinstance(schema, list) or not schema or not all(isinstance(column, str) and column for column in schema):
            raise ValidationError("dataset schema must be a non-empty list of column names")
        if len(set(schema)) != len(schema):
            raise ValidationError("dataset schema column names must be unique")
        return super().create(organization_id, values)
