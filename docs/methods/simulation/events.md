# Event times

Continuous integration recognizes scheduled changes by splitting at their known timestamps. A change becomes active at its timestamp, and pre-change values are used to complete the interval leading to that time.

`split_at_events` only validates, sorts, deduplicates, and returns nonzero intervals from supplied event times. The runtime does not locate roots of event functions or apply arbitrary event actions.
