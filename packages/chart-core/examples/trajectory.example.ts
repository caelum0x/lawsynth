import { createChartModel, normalizeTrajectory, seriesFromAllTrajectoryComponents } from "../src/index.js";

const trajectory = normalizeTrajectory({
  variables: ["position", "velocity"],
  times: [0, 0.5, 1, 1.5],
  values: [[0, 1], [0.5, 1], [1, 0.8], [1.3, 0.4]],
  metadata: { integrator: "rk4" },
});

export const trajectoryChart = createChartModel({ title: "Trajectory", series: seriesFromAllTrajectoryComponents(trajectory), xLabel: "time (s)", yLabel: "state" });
