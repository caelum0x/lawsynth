# Affine constant calibration

`calibrate_affine` evaluates one expression against supplied environments and fits `target ≈ scale × expression + offset` through `lawsynth-opt`. It returns the simplified calibrated expression together with the affine fit.

Only two affine constants are fitted. Expression evaluation errors, length mismatch, and invalid optimization input are returned; nonlinear constants, bounds, priors, and joint multi-equation calibration are not supported.
