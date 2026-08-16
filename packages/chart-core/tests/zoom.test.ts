import { deepEqual } from "./assert.js";
import { panDomain, zoomDomain } from "../src/zoom.js";

deepEqual(zoomDomain({ min: 0, max: 10 }, 2, 5), { min: 2.5, max: 7.5 });
deepEqual(panDomain({ min: 2, max: 4 }, -1), { min: 1, max: 3 });
