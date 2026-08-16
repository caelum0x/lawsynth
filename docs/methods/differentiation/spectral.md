# Spectral derivative

`spectral_derivative` computes a direct O(n²) discrete Fourier transform, multiplies each frequency component by its signed angular frequency, and reconstructs the derivative. It is intended for modest, uniformly sampled periodic signals.

It checks uniform spacing to a relative tolerance and rejects irregular, non-finite, or undersampled data. This is not an FFT implementation, has no taper/windowing or de-aliasing, and periodic boundary behavior can be inappropriate for transient or non-periodic data.
