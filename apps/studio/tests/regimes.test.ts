import { addRegimeInterval, regimeWorkspace } from "../src/regimes.js";
import { equal, rejects, world } from "./support.js";

export async function regimesTests(): Promise<void> {
  const workspace = regimeWorkspace(world); equal(workspace.uncoveredLawIds.join(","), "predator_decay"); equal(workspace.timeline?.lanes.length, 1);
  const expanded = addRegimeInterval(world.regimes!, { regime: "baseline", start: 10, end: 12, confidence: 0.7 });
  equal(expanded.intervals?.length, 2);
  await rejects(() => Promise.resolve(addRegimeInterval(world.regimes!, { regime: "baseline", start: 9, end: 11 })), /overlaps/);
}
