import { intersectsRect } from "./collision.js";
import type { Rect } from "./layout.js";
export type LabelPosition = "top" | "right" | "bottom" | "left" | "center";
export interface LabelPlacement { readonly position: LabelPosition; readonly rect: Rect; readonly overlapCount: number; }
export function placeLabel(anchor: Rect, size: { readonly width: number; readonly height: number }, occupied: readonly Rect[] = [], gap = 6): LabelPlacement {
  if (size.width < 0 || size.height < 0 || gap < 0) throw new RangeError("label dimensions and gap must be non-negative");
  const candidates: readonly [LabelPosition, Rect][] = [["top",{x:anchor.x+(anchor.width-size.width)/2,y:anchor.y-size.height-gap,...size}],["right",{x:anchor.x+anchor.width+gap,y:anchor.y+(anchor.height-size.height)/2,...size}],["bottom",{x:anchor.x+(anchor.width-size.width)/2,y:anchor.y+anchor.height+gap,...size}],["left",{x:anchor.x-size.width-gap,y:anchor.y+(anchor.height-size.height)/2,...size}],["center",{x:anchor.x+(anchor.width-size.width)/2,y:anchor.y+(anchor.height-size.height)/2,...size}]];
  return candidates.map(([position,rect]) => ({ position, rect, overlapCount: occupied.filter((item)=>intersectsRect(rect,item)).length })).sort((a,b)=>a.overlapCount-b.overlapCount || candidates.findIndex(([p])=>p===a.position)-candidates.findIndex(([p])=>p===b.position))[0]!;
}
