import { deepEqual } from "./assert.js";
import { tooltipAtX } from "../src/tooltip.js";

deepEqual(tooltipAtX([{ id: "x", label: "X", unit: "m", points: [{ x: 0, y: 2 }, { x: 2, y: 4 }] }], 1.7), [{ seriesId: "x", label: "X", x: 2, y: 4, unit: "m" }]);
