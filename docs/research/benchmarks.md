# Benchmarks

Benchmarks should measure a stated operation over a stated dataset size, not merely report an elapsed time. The workspace includes Criterion-style Rust benches for core operations and discovery-oriented examples. Run the relevant package benchmark with `cargo bench -p <crate>` on an otherwise idle machine and record CPU model, operating system, Rust version, build profile, input generation, and repetitions.

For recovery experiments, report coefficient or derivative error at fixed probe states, rollout error on a withheld interval, model complexity, and wall time separately. The discovery crate's Lorenz and Lotka--Volterra checks use deterministic synthetic data and documented sample counts; they are regression checks, not evidence of performance on arbitrary experimental data.

Do not compare methods using different feature families, derivative estimates, stopping rules, or validation intervals without saying so. Those choices often dominate the apparent ranking.
