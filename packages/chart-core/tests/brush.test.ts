import { deepEqual, equal } from "./assert.js";
import { normalizeBrush, pointInBrush } from "../src/brush.js";

deepEqual(normalizeBrush({ x: { min: 5, max: 1 } }), { x: { min: 1, max: 5 } });
equal(pointInBrush({ x: 2, y: 3 }, { x: { min: 1, max: 3 }, y: { min: 2, max: 4 } }), true);
equal(pointInBrush({ x: 4, y: 3 }, { x: { min: 1, max: 3 } }), false);
