"""scikit-learn-compatible estimators for LawSynth.

Three adapters wrap LawSynth's deterministic governing-equation discovery in the
scikit-learn estimator contract so the sklearn ecosystem can adopt LawSynth with
near-zero switching cost:

- :class:`LawSynthDynamics`  — the flagship: discover governing dynamics from a
  multivariate time-series, then ``predict`` / ``simulate`` / ``score``.
- :class:`LawSynthRegressor` — a strict ``RegressorMixin`` for the static-fit
  framing, ready for ``Pipeline`` / ``GridSearchCV``.
- :class:`LawSynthTransformer` — a ``TransformerMixin`` emitting the polynomial /
  trigonometric feature library, with an optional correlation prune.

The estimators inherit real scikit-learn mixins when sklearn is installed and
degrade to a standalone implementation of the same contract otherwise. Everything
is deterministic and offline.
"""

from ._compat import HAS_SKLEARN, NotFittedError
from ._parsimony import ParsimonyCandidate, ParsimonyResult, auto_parsimony
from .dynamics import LawSynthDynamics
from .regressor import LawSynthRegressor
from .transformer import LawSynthTransformer

__version__ = "0.1.0"

__all__ = [
    "LawSynthDynamics",
    "LawSynthRegressor",
    "LawSynthTransformer",
    "auto_parsimony",
    "ParsimonyResult",
    "ParsimonyCandidate",
    "NotFittedError",
    "HAS_SKLEARN",
    "__version__",
]
