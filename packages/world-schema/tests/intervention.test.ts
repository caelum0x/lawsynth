import { interventionIsActive, type Intervention } from "../src/intervention.js";
import { equal } from "./test-support.js";
export function runInterventionTests(): void { const intervention: Intervention = { id: "set-x", kind: "set", target: "x", value: 1, time: 2, end: 4 }; equal(interventionIsActive(intervention, 2), true); equal(interventionIsActive(intervention, 4), false); }
