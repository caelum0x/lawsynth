# `@lawsynth/layout-engine`

Dependency-free, deterministic layout primitives for the LawSynth studio.  The package operates only on plain data: it does **not** create DOM elements, measure a browser font, draw on a canvas, or provide a worker runtime.

It validates graph references and dimensions before layout. `layoutDag` performs layered placement for a directed acyclic graph; `forceLayout` is a seeded, repeatable spring layout for general graphs. Grid and timeline algorithms cover non-graph views. Collision, orthogonal routing, label placement, constraints, animation, cache, and viewport functions are independent helpers that can be used by any renderer.

```ts
import { layoutDag, orthogonalRoute } from "@lawsynth/layout-engine";

const graph = { nodes: [{ id: "x", width: 120, height: 44 }, { id: "y", width: 120, height: 44 }], edges: [{ source: "x", target: "y" }] };
const layout = layoutDag(graph, { direction: "TB", rankGap: 64 });
const route = orthogonalRoute({ x: 60, y: 44 }, { x: 60, y: 108 });
```

For real typography, pass measurements from the host renderer into node dimensions. `measureText` is intentionally a stable estimate useful before those measurements are available, not a replacement for browser or native text shaping.
