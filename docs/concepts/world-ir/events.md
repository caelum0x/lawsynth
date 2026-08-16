# Events

`Event` records an identifier, finite time, and crossing direction. `crosses_zero` detects rising, falling, or either-direction sign crossings from two scalar values. `split_at_events` can divide a requested time range at supplied event times.

Event utilities support inspection and interval management. The RK4 simulator does not search for roots inside a step, localize a crossing, or apply a reset map after detection.

For discontinuities that affect dynamics, schedule an input or parameter value at a known time. Root-triggered hybrid state changes remain outside the executable path.
