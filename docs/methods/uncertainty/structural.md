# Structural uncertainty

`StructuralUncertainty` accepts explicitly named, non-negative standard-deviation contributions labeled `Structural` and combines them by root-sum-of-squares. `structural_score` converts finite competing scores into an Akaike-style normalized ambiguity `1 - Σw²`.

Neither calculation discovers alternatives or proves their independence. Root-sum-of-squares assumes the contributions may be combined as independent variances; the score ambiguity is not a posterior probability over causal structures.
