import { lawTarget, type Law } from "../src/law.js";
import { equal } from "./test-support.js";
export function runLawTests(): void { const law: Law = { id: "dx", kind: "continuous", target: "x", expression: { kind: "constant", value: 0 } }; equal(lawTarget(law), "x"); equal(lawTarget({ id: "constraint", kind: "constraint", expression: { kind: "constant", value: true } }), undefined); }
