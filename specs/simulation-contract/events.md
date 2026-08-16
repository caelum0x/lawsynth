# Events

lawsynth-world supplies an Event marker and crosses_zero predicate for finite
scalar values with Any, Rising, and Falling direction. A zero at the left
endpoint does not retrigger in the next interval.

The current continuous simulator does not accept event functions, locate roots,
emit event records, or apply resets. Therefore an Event is a World IR utility,
not simulation output. Integrations requiring root localization or event-driven
state transitions MUST implement that policy outside this release.
