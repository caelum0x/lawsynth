import { equal, throws } from "./assert.js";
import { createChartModel } from "../src/chart.js";

const chart = createChartModel({ title: "Position", series: [{ id: "x", label: "x", points: [{ x: 0, y: 4 }, { x: 1, y: 4 }] }] });
equal(chart.xAxis.domain.min < 0, true); equal(chart.yAxis.domain.max > 4, true);
throws(() => createChartModel({ title: "", series: [] }));
throws(() => createChartModel({ title: "x", series: [{ id: "a", label: "a", points: [] }, { id: "a", label: "b", points: [] }] }));
