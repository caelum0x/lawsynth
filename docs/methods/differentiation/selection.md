# Selecting a derivative method

Use finite differences for clean data or as the baseline. Use local quadratic fitting or TV regularization when reducing local noise matters; use natural splines when smooth global interpolation is appropriate; use the spectral method only for genuinely uniform, periodic samples.

Compare methods on held-out downstream residuals and inspect endpoint behavior. LawSynth does not choose the method automatically, estimate a noise model, or declare a derivative scientifically valid from a fit score alone.
