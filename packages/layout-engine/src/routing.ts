import { intersectsRect } from "./collision.js";
import type { Point, Rect } from "./layout.js";
export interface Route { readonly points: readonly Point[]; readonly length: number; }
function segmentRect(a: Point, b: Point, rect: Rect): boolean { const left = Math.min(a.x,b.x), right=Math.max(a.x,b.x), top=Math.min(a.y,b.y), bottom=Math.max(a.y,b.y); return intersectsRect({x:left,y:top,width:right-left || 0.001,height:bottom-top || 0.001}, rect); }
export function orthogonalRoute(source: Point, target: Point, obstacles: readonly Rect[] = [], clearance = 12): Route {
  if (clearance < 0) throw new RangeError("clearance must be non-negative"); const directBend = { x: target.x, y: source.y }; const alternateBend = { x: source.x, y: target.y };
  const clear = (points: readonly Point[]) => points.slice(1).every((point, index) => !obstacles.some((rect) => segmentRect(points[index]!, point, { x: rect.x-clearance, y:rect.y-clearance, width:rect.width+2*clearance, height:rect.height+2*clearance })));
  let points: Point[] = clear([source,directBend,target]) ? [source,directBend,target] : clear([source,alternateBend,target]) ? [source,alternateBend,target] : [source,{x:source.x,y:source.y-clearance},{x:target.x,y:source.y-clearance},target];
  points = points.filter((point, index) => index === 0 || point.x !== points[index-1]!.x || point.y !== points[index-1]!.y); return { points, length: points.slice(1).reduce((sum, point, index) => sum + Math.abs(point.x-points[index]!.x) + Math.abs(point.y-points[index]!.y), 0) };
}
export function routeBounds(route: Route): Rect { const xs=route.points.map((p)=>p.x), ys=route.points.map((p)=>p.y); return {x:Math.min(...xs),y:Math.min(...ys),width:Math.max(...xs)-Math.min(...xs),height:Math.max(...ys)-Math.min(...ys)}; }
