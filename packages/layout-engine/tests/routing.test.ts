import { orthogonalRoute } from "../src/routing.js"; import { equal, ok } from "./assert.js";
export function runRoutingTests(): void { const route=orthogonalRoute({x:0,y:0},{x:10,y:20}); equal(route.points.length,3); equal(route.length,30); ok(route.points.every((point,index)=>index===0 || point.x===route.points[index-1]!.x || point.y===route.points[index-1]!.y)); }
