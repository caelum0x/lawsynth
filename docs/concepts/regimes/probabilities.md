# Change probabilities

`bocpd` processes a scalar series in order and returns `BocpdPoint` values with a change probability at each observation. Configure finite positive observation variance, finite prior mean, positive prior variance, and hazard in `(0, 1]`.

The calculation uses the package’s Gaussian scalar model and its supplied hyperparameters. Its probabilities depend on that model and do not serve as calibrated posterior guarantees under arbitrary noise or nonstationarity.
