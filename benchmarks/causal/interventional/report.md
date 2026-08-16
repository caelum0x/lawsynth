# causal/interventional report

## Method

The generator creates 128 deterministic observations using a seed derived from the benchmark identity. The resulting data retain oracle fields solely to verify generation; they are never used as a LawSynth answer.

## Result

The benchmark passes when LawSynth's public boundary refuses **causal identification and effect estimation** with a structured, explicit capability message. A rejection is the correct result until that engine is implemented and wired through its public API.
