# Studio

Studio is the visual surface for the LawSynth loop, driven by the shared TypeScript
packages in `apps/studio` and `packages/*`. It reads the same `.lsworld` bundles and
World IR as the CLI and SDK. The nine navigable screens follow the product's
`observe → discover → understand → use → share` order:

| # | Screen | Does |
| --- | --- | --- |
| 1 | **Data Lens** | Profile the input dataset and its quality |
| 2 | **Discovery Canvas** | Configure a run and inspect candidate laws |
| 3 | **Equation Explorer** | Read discovered laws and their terms |
| 4 | **Structure Map** | Variable coupling graph from law dependencies |
| 5 | **Regime Timeline** | Regime segments over time |
| 6 | **Uncertainty Lens** | Confidence bands on a trajectory |
| 7 | **World Lab** | Simulate, forecast, and intervene |
| 8 | **Scenario Board** | Compare what-if scenarios against a baseline |
| 9 | **Export** | Equations, LaTeX, Python, and the raw World IR |

Screens are exported from `apps/studio/src/screens` and enumerated in the
`SCREEN_REGISTRY`. A navigation controller moves between them and shares the loaded
world, its trajectories, and scenario state.

For the terminal or notebook experience of the same loop, use the
[CLI](cli.md) or the [Python `Study` API](python.md). For the hosted, multi-user
surface, see the [services](../self-hosting/README.md) documentation.
