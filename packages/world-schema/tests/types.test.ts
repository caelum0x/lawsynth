import { isIdentifier, parseSemanticVersion } from "../src/types.js";
import { equal, ok } from "./test-support.js";
export function runTypesTests(): void { ok(isIdentifier("supply-demand_2")); equal(isIdentifier("dotted.name"), false); equal(parseSemanticVersion("1.2.3")?.minor, 2); equal(parseSemanticVersion("01.2.3"), undefined); }
