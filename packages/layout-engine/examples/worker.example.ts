import { LayoutCoordinator, layoutDag } from "../src/index.js";

const graph = { nodes: [{ id: "input", width: 100, height: 36 }, { id: "output", width: 100, height: 36 }], edges: [{ source: "input", target: "output" }] };
const coordinator = new LayoutCoordinator<typeof graph, ReturnType<typeof layoutDag>>();
/** A host may call this from a real Worker message handler; the coordinator itself is runtime-neutral. */
export const requestedLayout = coordinator.run(graph, (input, signal) => { if (signal.cancelled()) throw new Error("cancelled before layout"); return layoutDag(input); });
