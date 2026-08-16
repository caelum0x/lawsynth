"""Immutable client-side configuration for discovery and simulation."""

from dataclasses import dataclass

from .errors import ValidationError


@dataclass(frozen=True, slots=True)
class DiscoveryConfig:
    """Validated options forwarded to :func:`lawsynth.discover`."""

    polynomial_degree: int = 2
    threshold: float = 0.05
    solver: str = "stlsq"
    derivative_method: str = "finite"
    include_trigonometric: bool = False
    include_rational: bool = False
    smoothing_radius: int | None = None
    symbolic_depth: int | None = None

    def __post_init__(self) -> None:
        if self.polynomial_degree < 0:
            raise ValidationError("polynomial_degree must be non-negative")
        if self.threshold < 0:
            raise ValidationError("threshold must be non-negative")
        if self.solver not in {"stlsq", "sr3"}:
            raise ValidationError("solver must be 'stlsq' or 'sr3'")
        if self.derivative_method not in {"finite", "savgol", "spline", "spectral", "tvreg"}:
            raise ValidationError("unsupported derivative_method")
        if self.smoothing_radius is not None and self.smoothing_radius < 1:
            raise ValidationError("smoothing_radius must be positive")
        if self.symbolic_depth is not None and self.symbolic_depth < 0:
            raise ValidationError("symbolic_depth must be non-negative")
