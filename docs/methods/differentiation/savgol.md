# Savitzky–Golay derivative

`savgol_series` fits a quadratic polynomial in local time offsets by solving a 3×3 normal system at every sample; the linear coefficient is the derivative estimate. The requested window must be odd, at least three samples, and no longer than the signal. Endpoint windows shift inward while remaining the requested width.

The implementation supports arbitrary supplied times but rejects singular local fits. It is a local least-squares smoother, not a general Savitzky–Golay convolution table, and it provides neither robust loss functions nor automatic window selection.
