import { handleViewerWorkerRequest, type ViewerWorld } from "../src/index.js";
const world: ViewerWorld = { formatVersion: "0.1.0", id: "worker_world", time: { kind: "discrete", step: 1 }, variables: [{ id: "x", role: "state" }], laws: [{ id: "update", kind: "discrete", target: "x", expression: { kind: "symbol", id: "x" } }] };
const response = handleViewerWorkerRequest({ id: "request-1", type: "build", world, trajectory: { variables: ["x"], times: [0, 1], values: [[1], [2]] } });
console.log(JSON.stringify(response));
