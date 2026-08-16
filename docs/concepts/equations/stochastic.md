# Stochastic equations

The simulation crate provides `euler_maruyama` for caller-supplied scalar drift and diffusion closures with a seeded generator. It returns a reproducible SDE trajectory for that narrow numerical experiment.

World laws and bundle files remain deterministic scalar expressions. Discovery does not estimate diffusion terms, likelihoods, latent noise processes, or stochastic differential equations from a dataset.

Keep a stochastic model’s random seed, timestep, drift, and diffusion definition with any reported trajectory. Do not present a deterministic discovered World as a fitted stochastic law.
