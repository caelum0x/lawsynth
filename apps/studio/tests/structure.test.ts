import { filterStructure, updateDependencyStatus } from "../src/structure.js";
import { equal, rejects, world } from "./support.js";

export async function structureTests(): Promise<void> {
  const filtered = filterStructure(world, { statuses: ["candidate"], minimumStrength: 0.5, includeUndirected: false });
  equal(filtered.visibleEdgeIds.has("prey_to_predator"), true); equal(filtered.hiddenEdges, 1);
  const updated = updateDependencyStatus(world.dependencies!, "prey_to_predator", "required");
  equal(updated.edges[0]?.status, "required");
  await rejects(() => Promise.resolve(updateDependencyStatus(world.dependencies!, "missing", "required")), /unknown/);
}
