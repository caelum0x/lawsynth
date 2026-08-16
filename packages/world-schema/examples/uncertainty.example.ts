import { intervalContains, type UncertaintyModel } from "../src/uncertainty.js";

export const uncertaintyExample: UncertaintyModel = {
  method: "bootstrap",
  seed: 9,
  entries: [{ level: "parameter", parameter: "rate", interval: { lower: 0.2, upper: 0.3, confidence: 0.95 } }],
};

export const nominalRateIsInside = intervalContains({ lower: 0.2, upper: 0.3, confidence: 0.95 }, 0.25);
