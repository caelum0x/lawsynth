import { activeRegimeAt, transitionProbability } from "../src/regime.js";
import { equal } from "./test-support.js";
export function runRegimeTests(): void { const model = { regimes: [{ id: "base" }, { id: "shock" }], intervals: [{ regime: "base", start: 0, end: 2 }], transitions: [{ from: "base", to: "shock", probability: 0.2 }] }; equal(activeRegimeAt(model, 1), "base"); equal(activeRegimeAt(model, 2), undefined); equal(transitionProbability(model, "base", "shock"), 0.2); }
