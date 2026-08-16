# Quadratic profile summary

`profile_quadratic` fits a least-squares parabola to at least three finite `(parameter, objective)` points. It requires positive fitted curvature, reports the vertex and minimum, and gives a normal-approximation interval using an inverse-normal approximation at the requested confidence.

The interval is a local quadratic approximation, not a constrained re-optimization profile over a model. Multimodal, asymmetric, or flat objectives can make it misleading or cause validation failure.
