# Initialization

`initialize_population` draws the bounded deterministic enumeration into a `Population`; it does not sample random trees. Candidate order derives from canonical expression fingerprints and deterministic terminal ordering.

Population diversity is therefore bounded by grammar and configuration, not a stochastic initialization distribution.
