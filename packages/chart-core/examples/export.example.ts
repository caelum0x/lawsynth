import { chartToCsv, createChartModel } from "../src/index.js";

const chart = createChartModel({ title: "Measured state", series: [{ id: "x", label: "x", points: [{ x: 0, y: 2 }, { x: 1, y: 3 }] }] });
export const measuredStateCsv = chartToCsv(chart);
