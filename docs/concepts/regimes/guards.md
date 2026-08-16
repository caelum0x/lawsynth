# Guards

Regime configuration validates non-empty data, positive minimum segment length, finite non-negative penalty, and allowable segment boundaries. `Segmentation::new` validates contiguity and complete coverage before it stores a result.

These guards stop malformed numerical inputs from producing a plausible segmentation. They do not select a penalty, test whether a change exists, or assess the practical importance of a detected boundary.
