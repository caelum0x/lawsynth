# Time grid

SimulationConfig::new(start, end, step) accepts only finite values with end >
start and step > 0. Continuous output includes start and proceeds to end; the
last integration segment may be shorter than step. A scheduled change strictly
inside a segment splits it, so the change timestamp is emitted.

DiscreteSimulationConfig::new(start, steps) requires a finite start; its grid is
integer-indexed and contains the initial sample plus one sample per transition.
No calendar duration or non-uniform discrete spacing is encoded.

Consumers MUST treat trajectory times as authoritative rather than reconstructing
them from a requested step, because intervention boundaries add samples.
