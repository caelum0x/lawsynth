import { gridLayout } from "../src/grid.js"; import { equal } from "./assert.js";
export function runGridTests(): void { const result=gridLayout([{id:"a",width:10,height:10},{id:"b",width:10,height:10},{id:"c",width:10,height:10}],{columns:2,gap:5}); equal(result.nodes[2]!.y,15); equal(result.width,25); }
