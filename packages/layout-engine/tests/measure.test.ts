import { measureText, paddedSize } from "../src/measure.js"; import { equal, ok } from "./assert.js";
export function runMeasureTests(): void { const measured=measureText("one two three",{fontSize:10,maxWidth:25}); ok(measured.lines.length>1); equal(paddedSize({width:3,height:4},2).width,7); }
