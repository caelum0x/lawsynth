import { phasePortrait, normalizeTrajectory } from "../src/index.js";

const oscillator = normalizeTrajectory({ variables: ["x", "v"], times: [0, 1, 2, 3], values: [[1, 0], [0, -1], [-1, 0], [0, 1]] });
export const oscillatorPhasePortrait = phasePortrait(oscillator, "x", "v");
