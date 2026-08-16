import { createWorldViewModel, exportTrajectoryCsv, exportViewerJson, type ViewerWorld } from "../src/index.js";
const world: ViewerWorld = { formatVersion: "0.1.0", id: "exported", time: { kind: "continuous" }, variables: [{ id: "x", role: "state" }], laws: [{ id: "dx", kind: "continuous", target: "x", expression: { kind: "constant", value: 1 } }] };
const model = createWorldViewModel(world, { variables: ["x"], times: [0, 0.5, 1], values: [[0], [0.5], [1]] });
console.log(exportViewerJson(model)); console.log(exportTrajectoryCsv(model));
