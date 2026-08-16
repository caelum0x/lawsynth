"""Errors raised before an artifact is rendered."""


class NotebookError(Exception):
    """Base class for notebook-support failures."""


class ArtifactValidationError(NotebookError, ValueError):
    """The supplied decoded artifact does not meet a view's contract."""


class UnsupportedCapabilityError(NotebookError):
    """An operation needs a live service or a codec this package does not ship."""
