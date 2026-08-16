from math import isfinite

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
        has_observations = "time" in values or "columns" in values
        if has_observations:
            time, columns = values.get("time"), values.get("columns")
            if not isinstance(time, list) or len(time) < 2 or not isinstance(columns, dict):
                raise ValidationError("dataset observations require time and columns")
            if set(columns) != set(schema):
                raise ValidationError("dataset schema must exactly match observation columns")
            previous: float | None = None
            for item in time:
                if isinstance(item, bool) or not isinstance(item, (int, float)) or not isfinite(float(item)):
                    raise ValidationError("dataset time must contain finite numbers")
                current = float(item)
                if previous is not None and current <= previous:
                    raise ValidationError("dataset time must be strictly increasing")
                previous = current
            for name, series in columns.items():
                if not isinstance(series, list) or len(series) != len(time):
                    raise ValidationError(f"dataset column {name!r} must align with time")
                if any(isinstance(value, bool) or not isinstance(value, (int, float)) or not isfinite(float(value)) for value in series):
                    raise ValidationError(f"dataset column {name!r} must contain finite numbers")
        return super().create(organization_id, values)
