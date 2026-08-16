import { intersectsRect, resolveCollisions } from "../src/collision.js"; import { equal, ok } from "./assert.js";
export function runCollisionTests(): void { equal(intersectsRect({x:0,y:0,width:10,height:10},{x:9,y:0,width:10,height:10}),true); const items=resolveCollisions([{x:0,y:0,width:10,height:10},{x:5,y:0,width:10,height:10}],0,2); ok(!intersectsRect(items[0]!,items[1]!)); }
