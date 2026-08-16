import { trajectoryChartSet } from "../src/charts.js";
import { deepEqual, equal, test, throws } from "./testkit.js";

await test("trajectory charts retain requested variables and measured duration", () => {
  const charts = trajectoryChartSet({
    variables: ["x", "y"], times: [0, 0.5, 1], values: [[1, 4], [0.8, 3], [0.6, 2]],
  }, { visibleVariables: ["x"], title: "Decay" });
  equal(charts.sampleCount, 3);
  equal(charts.duration, 1);
  equal(charts.combined.title, "Decay");
  deepEqual(charts.individual.map((chart) => chart.title), ["x"]);
  throws(() => trajectoryChartSet({ variables: ["x"], times: [0], values: [[1]] }, { visibleVariables: ["missing"] }), /unknown trajectory variable/);
});
