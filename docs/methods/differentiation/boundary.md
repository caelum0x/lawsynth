# Boundary behavior

Finite differences use one-sided slopes at the two ends. Savitzky–Golay shifts a full local window to remain inside the observed range. Natural cubic splines impose zero endpoint curvature, while spectral differentiation treats the sequence as periodic.

These assumptions can dominate short records. There is no boundary extrapolation API, padding policy, or automatic warning based on the distance to a boundary.
