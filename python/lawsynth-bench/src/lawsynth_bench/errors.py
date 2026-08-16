"""Errors raised while validating recorded benchmark artifacts."""

class BenchmarkError(ValueError):
    """Base class for invalid benchmark inputs."""

class SchemaError(BenchmarkError):
    """A benchmark document has an invalid schema."""

class ComparisonError(BenchmarkError):
    """Two benchmark sets cannot be compared fairly."""

class RegressionError(BenchmarkError):
    """A configured performance regression was detected."""
