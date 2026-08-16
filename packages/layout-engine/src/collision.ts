import type { Point, Rect } from "./layout.js";
export function intersectsRect(a: Rect, b: Rect, padding = 0): boolean { if (padding < 0) throw new RangeError("padding must be non-negative"); return a.x - padding < b.x + b.width + padding && a.x + a.width + padding > b.x - padding && a.y - padding < b.y + b.height + padding && a.y + a.height + padding > b.y - padding; }
export function containsPoint(rect: Rect, point: Point): boolean { return point.x >= rect.x && point.x <= rect.x + rect.width && point.y >= rect.y && point.y <= rect.y + rect.height; }
export function separation(a: Rect, b: Rect, padding = 0): Point | undefined {
  if (!intersectsRect(a, b, padding)) return undefined; const ax = a.x + a.width / 2, ay = a.y + a.height / 2, bx = b.x + b.width / 2, by = b.y + b.height / 2; const dx = bx - ax, dy = by - ay; const overlapX = (a.width + b.width) / 2 + 2 * padding - Math.abs(dx), overlapY = (a.height + b.height) / 2 + 2 * padding - Math.abs(dy); return overlapX < overlapY ? { x: (dx >= 0 ? overlapX : -overlapX), y: 0 } : { x: 0, y: (dy >= 0 ? overlapY : -overlapY) };
}
export function resolveCollisions<T extends Rect>(items: readonly T[], padding = 0, iterations = 4): T[] {
  if (!Number.isInteger(iterations) || iterations < 0) throw new RangeError("iterations must be a non-negative integer"); type Mutable = { -readonly [K in keyof T]: T[K] }; const output = items.map((item) => ({ ...item })) as Mutable[];
  for (let round = 0; round < iterations; round++) for (let i = 0; i < output.length; i++) for (let j = i + 1; j < output.length; j++) { const move = separation(output[i]!, output[j]!, padding); if (move) { output[j]!.x += move.x; output[j]!.y += move.y; } }
  return output as T[];
}
