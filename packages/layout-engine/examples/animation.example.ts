import { animationFrame, interpolatePoint } from "../src/index.js";

export const halfway = animationFrame({ x: 0, y: 0 }, { x: 120, y: 40 }, 500, 0, 1000, (from, to, progress) => interpolatePoint(from, to, progress));
