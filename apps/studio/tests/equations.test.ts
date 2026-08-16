import { compareCandidates, equationCatalog, replaceLaw } from "../src/equations.js";
import { deepEqual, equal, rejects, world } from "./support.js";

export async function equationsTests(): Promise<void> {
  const comparisons = compareCandidates([{ id: "b", run_id: "run", score: 0.4 }, { id: "a", run_id: "run", score: 0.9 }], "b");
  deepEqual(comparisons.map((entry) => entry.rank), [1, 2]); equal(comparisons[1]?.scoreDelta, 0.5); equal(comparisons[1]?.selected, true);
  equal(equationCatalog(world).disabled, 1);
  const replacement = { ...world.laws[0]!, enabled: false };
  equal(replaceLaw(world, replacement).laws[0]?.enabled, false);
  await rejects(() => Promise.resolve(compareCandidates([{ id: "a", run_id: "run", score: 1 }, { id: "a", run_id: "run", score: 0 }])), /unique/);
}
