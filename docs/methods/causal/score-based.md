# Graph scoring boundary

The implemented causal graph type is a validated DAG with deterministic topological ordering, parent/child lookup, and cycle rejection. LawSynth does not implement a score-based graph search, BIC structure learner, or intervention-aware optimizer in this crate.

Constructing a graph and computing a score elsewhere must not be represented as discovered causality without a separately stated identification argument.
