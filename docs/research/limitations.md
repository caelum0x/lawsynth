# Limitations

Current discovery consumes finite aligned numeric data and targets explicit continuous-time laws. It does not infer missing values, latent states, measurement-error models, causal identification, automatic regime segmentation inside discovery, or calibrated posterior model probabilities. Preprocessing and scientific assumptions remain caller responsibilities.

The native Parquet reader intentionally supports only a constrained uncompressed, flat, PLAIN numeric subset. It rejects compression, dictionary encoding, nested columns, and level encoding rather than silently decoding them incorrectly. Convert other inputs through a validated upstream reader.

Simulation is numerical. RK4, discrete stepping, hybrid event splitting, and seeded Euler--Maruyama are implemented, but a computed trajectory is not a stability proof, uncertainty certificate, or guarantee of physical feasibility.
