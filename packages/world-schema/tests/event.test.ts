import { crossesZero } from "../src/event.js";
import { equal } from "./test-support.js";
export function runEventTests(): void { equal(crossesZero(-1, 0, "rising"), true); equal(crossesZero(-1, 1, "falling"), false); equal(crossesZero(Number.NaN, 1), false); }
