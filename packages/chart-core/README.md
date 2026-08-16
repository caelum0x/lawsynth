# @lawsynth/chart-core

`chart-core` is a dependency-free TypeScript package for turning LawSynth numerical output into validated, renderer-neutral chart data. It deliberately does not include a canvas, SVG, DOM, browser event system, or browser rendering adapter.

It provides:

- validated trajectories and named data series;
- deterministic Largest-Triangle-Three-Buckets decimation for large traces;
- linear and logarithmic coordinate transforms, padded domains, and readable ticks;
- chart models, annotations, brushes, pan/zoom domains, legends, tooltips, phase portraits, heatmap grids, and dependency graphs;
- CSV/JSON data export.

## Usage

```ts
import { createChartModel, normalizeTrajectory, seriesFromAllTrajectoryComponents } from "@lawsynth/chart-core";

const trajectory = normalizeTrajectory({
  variables: ["x"], times: [0, 1, 2], values: [[0], [1], [0.4]],
});
const model = createChartModel({
  title: "State", series: seriesFromAllTrajectoryComponents(trajectory),
  xLabel: "time (s)", yLabel: "x",
});
```

`ChartModel` is intentionally data only. A React, D3, Canvas, WebGL, or server renderer may consume it, but rendering is outside this package. Inputs containing non-finite values, duplicate variables/series, invalid grid dimensions, or unordered series are rejected at the boundary rather than silently repaired.

Run `npm test` from this package to compile and execute the invariant tests.
