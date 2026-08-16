# Simulations

Run submission validates simulation horizon, step, and maximum step count
before a run is persisted. The service records a run request; executing native
simulation and dispatching remote workers are distinct responsibilities and
must be supplied by a worker adapter.
