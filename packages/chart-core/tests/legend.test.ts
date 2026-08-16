import { deepEqual, equal } from "./assert.js";
import { buildLegend, toggleLegendSeries } from "../src/legend.js";

const series = [{ id: "x", label: "Position", points: [] }, { id: "v", label: "Velocity", points: [] }];
equal(buildLegend(series, new Set(["v"]))[1]!.visible, false);
deepEqual([...toggleLegendSeries(new Set(["x"]), "x")], []);
