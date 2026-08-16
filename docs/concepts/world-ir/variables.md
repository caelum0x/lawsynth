# Variables

Each variable has a validated `Identifier`, a `VariableRole`, and an optional `Unit`. The available roles are `State`, `Control`, and `Observed`. A world constructor requires laws only for `State`; a simulation request can supply values only for non-state controls.

Use `Observed` for columns retained in a world’s expression namespace without treating them as integrated state. Discovery builds state roles from the configured state columns and marks remaining dataset columns as controls.

Roles document execution rules. They do not encode measurement noise, latent variables, distributions, or causal status.
