import { deepEqual, equal, throws } from "./assert.js";
import { createScale, extent, linearTicks, padDomain } from "../src/scales.js";

deepEqual(extent([-2, 5, 1]), { min: -2, max: 5 });
deepEqual(padDomain({ min: 7, max: 7 }), { min: 6, max: 8 });
equal(createScale({ min: 0, max: 10 }, { min: 0, max: 100 })(2.5), 25);
deepEqual(linearTicks({ min: 0, max: 10 }, 3), [0, 5, 10]);
throws(() => extent([NaN])); throws(() => createScale({ min: 0, max: 0 }, { min: 0, max: 1 }));
