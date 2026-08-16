# Variables

Dataset variables are scalar `NumericColumn`s named by validated LawSynth
identifiers. A discovery configuration names one or more of these columns as
state variables; all remaining columns are retained and become control
variables in the emitted world.

Every state name must exist in the input dataset. State selection is explicit:
the engine does not infer states, parameters, categorical variables, or latent
variables from column names. A column's optional unit is metadata carried by
the dataset and fingerprint; the core discovery executor does not parse or
convert that string while fitting laws.
