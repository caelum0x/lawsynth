import { deepEqual, equal, throws } from "./assert.js";
import { nearestPoint, normalizeSeries, seriesFromTrajectory } from "../src/series.js";
import { normalizeTrajectory } from "../src/trajectory.js";

const trajectory = normalizeTrajectory({ variables: ["x"], times: [0, 1, 2], values: [[1], [3], [2]] });
const series = seriesFromTrajectory(trajectory, "x", { unit: "m" });
deepEqual(series.points, [{ x: 0, y: 1 }, { x: 1, y: 3 }, { x: 2, y: 2 }]);
deepEqual(nearestPoint(series, 1.6), { x: 2, y: 2 });
equal(series.unit, "m");
throws(() => normalizeSeries({ id: "x", label: "x", points: [{ x: 2, y: 0 }, { x: 1, y: 0 }] }));
