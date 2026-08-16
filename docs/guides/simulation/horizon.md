# Horizon and step size

Continuous simulation requires `--start T --end T --step DT`; all three values must be finite and the configuration must be valid. Select units consistent with the time axis used to discover the model. A step that is adequate for a slowly varying state may be inadequate for a fast or stiff one.

Run a convergence check: simulate the same scenario at the intended step and a smaller step, then compare the quantities that drive your decision. Treat meaningful disagreement as a numerical warning, not an invitation to select the prettier curve.

Extremely long horizons can magnify structural model error even when numerical integration succeeds. The CLI does not automatically detect stiffness, select solvers, or certify long-run stability.
