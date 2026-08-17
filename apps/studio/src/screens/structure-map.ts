import { categoricalColor, normalizeGraph, type Graph } from "@lawsynth/chart-core";
import { forceLayout, type LayoutGraph, type PositionedNode } from "@lawsynth/layout-engine";
import type { WorldDefinition } from "@lawsynth/world-schema";
import { equationView, expressionSymbols } from "@lawsynth/world-viewer";
import type { ControlField, GraphEdgeView, GraphNodeView, GraphView, Metric, ScreenModel, ScreenSection, TableRow } from "./types.js";

export interface StructureMapInput {
  readonly world: WorldDefinition;
  readonly selectedVariableId?: string;
  readonly width?: number;
  readonly height?: number;
}

interface Coupling {
  readonly source: string;
  readonly target: string;
}

const NODE_WIDTH = 128;
const NODE_HEIGHT = 52;

/** Directed couplings: `source → target` when `source` appears in the law for `target`. */
function couplings(world: WorldDefinition, variableIds: ReadonlySet<string>): readonly Coupling[] {
  const seen = new Set<string>();
  const edges: Coupling[] = [];
  for (const law of world.laws) {
    if (!("target" in law) || !variableIds.has(law.target)) continue;
    for (const source of expressionSymbols(law.expression)) {
      if (source === law.target || !variableIds.has(source)) continue;
      const key = `${source}->${law.target}`;
      if (seen.has(key)) continue;
      seen.add(key);
      edges.push({ source, target: law.target });
    }
  }
  return edges;
}

/** Clips the centre-to-centre segment to the target rectangle so the arrow lands on its edge. */
function boundaryPoint(from: PositionedNode, to: PositionedNode): { x: number; y: number; angle: number } {
  const fx = from.x + from.width / 2;
  const fy = from.y + from.height / 2;
  const tx = to.x + to.width / 2;
  const ty = to.y + to.height / 2;
  const dx = fx - tx;
  const dy = fy - ty;
  const halfW = to.width / 2;
  const halfH = to.height / 2;
  const scaleX = dx === 0 ? Infinity : halfW / Math.abs(dx);
  const scaleY = dy === 0 ? Infinity : halfH / Math.abs(dy);
  const scale = Math.min(scaleX, scaleY);
  return { x: tx + dx * scale, y: ty + dy * scale, angle: Math.atan2(ty - fy, tx - fx) };
}

/**
 * Visualizes the discovered world's coupling structure: variables are nodes and
 * law dependencies (which variables appear in each derivative) are directed
 * edges. Layout comes from `layout-engine`'s deterministic force solver (which
 * handles the cyclic dependency graphs that are common in dynamical systems),
 * and the graph is validated through `chart-core`'s graph primitive. Selecting a
 * node highlights its incident couplings and lists the laws that touch it.
 */
export function structureMapModel(input: StructureMapInput): ScreenModel {
  const { world } = input;
  const variableIds = new Set(world.variables.map((variable) => variable.id));
  const edges = couplings(world, variableIds);

  const graph: Graph = normalizeGraph({
    nodes: world.variables.map((variable) => ({ id: variable.id, label: variable.name ?? variable.id })),
    edges: edges.map((edge) => ({ source: edge.source, target: edge.target })),
  });

  const layoutGraph: LayoutGraph = {
    nodes: graph.nodes.map((node) => ({ id: node.id, width: NODE_WIDTH, height: NODE_HEIGHT })),
    edges: graph.edges.map((edge) => ({ source: edge.source, target: edge.target })),
  };
  const layout = forceLayout(layoutGraph, { seed: 7, padding: 40, springLength: 150, iterations: 320 });
  const positioned = new Map(layout.nodes.map((node) => [node.id, node]));

  const selectedId = input.selectedVariableId !== undefined && variableIds.has(input.selectedVariableId)
    ? input.selectedVariableId
    : undefined;
  const neighbors = new Set<string>();
  if (selectedId !== undefined) {
    for (const edge of edges) {
      if (edge.source === selectedId) neighbors.add(edge.target);
      if (edge.target === selectedId) neighbors.add(edge.source);
    }
  }

  const nodeViews: readonly GraphNodeView[] = layout.nodes.map((node): GraphNodeView => {
    const variable = world.variables.find((entry) => entry.id === node.id);
    return {
      id: node.id,
      label: variable?.name ?? node.id,
      x: node.x,
      y: node.y,
      width: node.width,
      height: node.height,
      color: categoricalColor(variable?.role ?? node.id),
      selected: node.id === selectedId,
      highlighted: neighbors.has(node.id),
      ...(variable?.unit === undefined ? {} : { sublabel: variable.unit }),
    };
  });

  const edgeViews: readonly GraphEdgeView[] = edges.flatMap((edge, index): GraphEdgeView[] => {
    const source = positioned.get(edge.source);
    const target = positioned.get(edge.target);
    if (source === undefined || target === undefined) return [];
    const tail = boundaryPoint(target, source);
    const head = boundaryPoint(source, target);
    const highlighted = selectedId !== undefined && (edge.source === selectedId || edge.target === selectedId);
    return [{
      id: `edge-${index}-${edge.source}-${edge.target}`,
      source: edge.source,
      target: edge.target,
      path: `M${tail.x.toFixed(2)},${tail.y.toFixed(2)} L${head.x.toFixed(2)},${head.y.toFixed(2)}`,
      headX: head.x,
      headY: head.y,
      angle: head.angle,
      highlighted,
    }];
  });

  const view: GraphView = { nodes: nodeViews, edges: edgeViews, width: layout.width, height: layout.height };

  const inDegree = (id: string): number => edges.filter((edge) => edge.target === id).length;
  const outDegree = (id: string): number => edges.filter((edge) => edge.source === id).length;
  const maxEdges = variableIds.size * Math.max(1, variableIds.size - 1);
  const metrics: readonly Metric[] = [
    { label: "Variables", value: String(variableIds.size) },
    { label: "Couplings", value: String(edges.length) },
    { label: "Density", value: maxEdges === 0 ? "—" : `${Math.round((edges.length / maxEdges) * 100)}%` },
    selectedId === undefined
      ? { label: "Selected", value: "—" }
      : { label: `${selectedId} degree`, value: `${inDegree(selectedId)} in / ${outDegree(selectedId)} out` },
  ];

  const variableOptions = [{ value: "", label: "No focus" }, ...world.variables.map((variable) => ({ value: variable.id, label: variable.name ?? variable.id }))];
  const controls: readonly ControlField[] = [
    { id: "structure:variable", label: "Focus variable", kind: "select", value: selectedId ?? "", options: variableOptions, help: "Highlight a variable's couplings and focus it in Equation Explorer." },
  ];

  const timeSymbol = world.time.symbol ?? "t";
  const relatedLaws = selectedId === undefined
    ? world.laws
    : world.laws.filter((law) => ("target" in law && law.target === selectedId) || expressionSymbols(law.expression).includes(selectedId));
  const lawRows: readonly TableRow[] = relatedLaws.map((law) => {
    const eq = equationView(law, timeSymbol);
    const relation = selectedId === undefined
      ? "law"
      : "target" in law && law.target === selectedId
        ? "defines"
        : "influences";
    return { id: eq.id, cells: [eq.target ?? eq.id, relation, eq.text], emphasis: relation === "defines" };
  });

  const sections: readonly ScreenSection[] = [
    { kind: "metrics", id: "structure-metrics", title: "Coupling", metrics },
    { kind: "controls", id: "structure-controls", title: "Focus", fields: controls },
    { kind: "graph", id: "structure-graph", title: "Dependency graph", graph: view },
    {
      kind: "table",
      id: "structure-laws",
      title: selectedId === undefined ? "All laws" : `Laws touching ${selectedId}`,
      columns: [
        { key: "target", label: "Target" },
        { key: "relation", label: "Relation" },
        { key: "equation", label: "Equation" },
      ],
      rows: lawRows,
      empty: "No laws reference this variable.",
    },
  ];

  return { id: "structure-map", title: "Structure Map", subtitle: "Variable coupling graph from law dependencies", sections };
}
