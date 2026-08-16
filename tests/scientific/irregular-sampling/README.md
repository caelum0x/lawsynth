# Irregular sampling case

The observations follow exponential growth at deliberately nonuniform timestamps.
The runner passes the CSV directly to the native discovery implementation and
requires its resulting world to simulate with finite values. It is a time-axis
validity test, not a claim about arbitrary sparse or gapped sampling.
