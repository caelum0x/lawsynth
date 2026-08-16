import { LayoutCache, layoutDag } from "../src/index.js";

const graph = { nodes: [{ id: "cause", width: 120, height: 40 }, { id: "effect", width: 120, height: 40 }], edges: [{ source: "cause", target: "effect" }] };
const cache = new LayoutCache<string, ReturnType<typeof layoutDag>>(4);
export const cachedLayout = cache.getOrCompute("cause-effect", () => layoutDag(graph));
export const cacheStats = cache.stats;
