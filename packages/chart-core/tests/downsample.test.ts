import { deepEqual, equal, throws } from "./assert.js";
import { largestTriangleThreeBuckets } from "../src/downsample.js";

const points = Array.from({ length: 100 }, (_, x) => ({ x, y: x === 50 ? 100 : Math.sin(x / 10) }));
const sampled = largestTriangleThreeBuckets(points, 12);
equal(sampled.length, 12);
deepEqual(sampled[0], points[0]); deepEqual(sampled.at(-1), points.at(-1));
equal(sampled.some((point) => point.x === 50), true, "extreme must survive LTTB");
throws(() => largestTriangleThreeBuckets(points, 1));
