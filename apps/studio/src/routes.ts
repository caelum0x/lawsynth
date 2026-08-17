import { isScreenId, type ScreenId } from "./screens/ids.js";

export type StudioRouteName = "home" | "project" | "dataset" | "discovery" | "world" | "simulation" | "screen" | "settings";

export type StudioRoute =
  | { readonly name: "home" }
  | { readonly name: "settings" }
  | { readonly name: "project"; readonly projectId: string }
  | { readonly name: "dataset"; readonly projectId: string; readonly datasetId: string }
  | { readonly name: "discovery"; readonly projectId: string; readonly runId?: string }
  | { readonly name: "world"; readonly projectId: string; readonly worldId: string; readonly panel?: string }
  | { readonly name: "simulation"; readonly projectId: string; readonly simulationId: string }
  | { readonly name: "screen"; readonly projectId: string; readonly screen: ScreenId };

const safeId = /^[A-Za-z0-9][A-Za-z0-9._:-]{0,255}$/u;
function id(value: string | undefined, label: string): string {
  if (value === undefined || !safeId.test(value)) throw new RangeError(`${label} is invalid`);
  return value;
}
function segment(value: string): string { return encodeURIComponent(id(value, "route identifier")); }

export function routePath(route: StudioRoute): string {
  switch (route.name) {
    case "home": return "/";
    case "settings": return "/settings";
    case "project": return `/projects/${segment(route.projectId)}`;
    case "dataset": return `/projects/${segment(route.projectId)}/datasets/${segment(route.datasetId)}`;
    case "discovery": return `/projects/${segment(route.projectId)}/discovery${route.runId === undefined ? "" : `/${segment(route.runId)}`}`;
    case "world": return `/projects/${segment(route.projectId)}/worlds/${segment(route.worldId)}${route.panel === undefined ? "" : `?panel=${encodeURIComponent(route.panel)}`}`;
    case "simulation": return `/projects/${segment(route.projectId)}/simulations/${segment(route.simulationId)}`;
    case "screen": return `/projects/${segment(route.projectId)}/screens/${encodeURIComponent(route.screen)}`;
  }
}

export function parseRoute(input: string | URL): StudioRoute {
  const url = input instanceof URL ? input : new URL(input, "https://studio.invalid");
  const parts = url.pathname.split("/").filter(Boolean).map((part) => decodeURIComponent(part));
  if (parts.length === 0) return { name: "home" };
  if (parts.length === 1 && parts[0] === "settings") return { name: "settings" };
  if (parts[0] !== "projects") throw new RangeError(`unknown Studio route: ${url.pathname}`);
  const projectId = id(parts[1], "project id");
  if (parts.length === 2) return { name: "project", projectId };
  if (parts[2] === "datasets" && parts.length === 4) return { name: "dataset", projectId, datasetId: id(parts[3], "dataset id") };
  if (parts[2] === "discovery" && (parts.length === 3 || parts.length === 4)) return { name: "discovery", projectId, ...(parts[3] === undefined ? {} : { runId: id(parts[3], "run id") }) };
  if (parts[2] === "worlds" && parts.length === 4) {
    const panel = url.searchParams.get("panel")?.trim();
    return { name: "world", projectId, worldId: id(parts[3], "world id"), ...(panel ? { panel } : {}) };
  }
  if (parts[2] === "simulations" && parts.length === 4) return { name: "simulation", projectId, simulationId: id(parts[3], "simulation id") };
  if (parts[2] === "screens" && parts.length === 4) {
    const screen = parts[3];
    if (!isScreenId(screen)) throw new RangeError(`unknown Studio screen: ${String(screen)}`);
    return { name: "screen", projectId, screen };
  }
  throw new RangeError(`unknown Studio route: ${url.pathname}`);
}

export class StudioRouter extends EventTarget {
  #route: StudioRoute;
  constructor(initial: StudioRoute = { name: "home" }) { super(); this.#route = initial; }
  get current(): StudioRoute { return this.#route; }
  navigate(route: StudioRoute, replace = false): void {
    const prior = this.#route;
    this.#route = route;
    this.dispatchEvent(new CustomEvent("navigate", { detail: { prior, route, path: routePath(route), replace } }));
  }
}
