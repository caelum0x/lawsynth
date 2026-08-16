# Ensembles

Build an ensemble explicitly from selected discovery candidates or scenario runs. Evaluate all members on the same data split and time grid, then retain member weights and aggregation rules outside the current crate.

LawSynth does not train weighted ensembles, average symbolic expressions, or select ensemble weights from validation data. An unweighted collection of candidates is not a calibrated predictive distribution.
