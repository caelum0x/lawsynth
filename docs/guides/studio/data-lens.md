# Data lens integration

Use `@lawsynth/chart-core` to validate a trajectory and derive renderer-neutral series before passing data to a visualization layer. It accepts finite, ordered numerical data and rejects malformed axes rather than silently sorting measurements.

```ts
import { normalizeTrajectory, seriesFromAllTrajectoryComponents } from "@lawsynth/chart-core";

const trajectory = normalizeTrajectory({ variables: ["x"], times: [0, 1], values: [[1], [0.8]] });
const series = seriesFromAllTrajectoryComponents(trajectory);
```

The package is not a data warehouse, CSV importer, or browser chart. The host must fetch or import observations, preserve units and provenance, choose any resampling policy, and render the resulting model with its own accessibility support.
