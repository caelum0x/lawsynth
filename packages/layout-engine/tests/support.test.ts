import { animationFrame, applyConstraints, createViewport, LayoutCache, screenToWorld, zoomAt } from "../src/index.js";
import { equal, ok } from "./assert.js";

export async function runSupportTests(): Promise<void> {
  const viewport = createViewport(800, 600, 2, 10, 20);
  const zoomed = zoomAt(viewport, 1.5, { x: 400, y: 300 });
  const invariant = screenToWorld(zoomed, { x: 400, y: 300 });
  equal(invariant.x, 210); equal(invariant.y, 170);
  const constrained = applyConstraints([{ id: "a", width: 10, height: 10, x: 4, y: 6 }, { id: "b", width: 10, height: 10, x: 0, y: 0 }], [{ kind: "pin", id: "a", x: 0, y: 0 }, { kind: "minimumGap", first: "a", second: "b", axis: "x", gap: 5 }]);
  equal(constrained[1]!.x, 15);
  const cache = new LayoutCache<string, number>(1); cache.set("a", 1); equal(cache.get("a"), 1); equal(cache.stats.hits, 1);
  const frame = animationFrame(0, 10, 500, 0, 1000, (from, to, progress) => from + (to - from) * progress); ok(frame.value > 4 && frame.value < 6);
}
