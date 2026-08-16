# Symbolic search bounds

`--symbolic-depth N` enables the implemented bounded symbolic refinement configuration. Keep `N` small enough that candidates can be inspected and independently simulated. Symbolic complexity grows rapidly; depth is a resource and interpretability control, not a guarantee of a closed-form law.

Use it after a sparse baseline has established that the dataset, time scale, and validation procedure are sound. Compare the refined result against the baseline on untouched data and reject algebraically elaborate expressions that do not provide a predefined gain.

The interface does not accept arbitrary executable expressions or load user-defined symbolic plugins. This prevents a model-search option from becoming an unreviewed code-execution channel.
