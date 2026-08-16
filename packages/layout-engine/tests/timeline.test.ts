import { timelineLayout } from "../src/timeline.js"; import { equal } from "./assert.js";
export function runTimelineTests(): void { const items=timelineLayout([{id:"b",start:2,lane:"one"},{id:"a",start:1,end:3,lane:"two"}],{pixelsPerUnit:10}); equal(items[0]!.id,"a"); equal(items[0]!.width,20); equal(items[1]!.laneIndex,0); }
