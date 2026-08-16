import { equal, throws } from "./assert.js";
import { makeAxisTicks, normalizeAnnotation } from "../src/axis.js";

equal(makeAxisTicks({ domain: { min: 0, max: 10 }, label: "time", tickCount: 3 }).length, 3);
equal(normalizeAnnotation({ id: "threshold", kind: "line", y: 2, label: "critical" }).y, 2);
throws(() => normalizeAnnotation({ id: "broken", kind: "point", label: "x", x: 1 }));
