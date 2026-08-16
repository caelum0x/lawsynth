export type ViewerPanel = "overview" | "equations" | "graph" | "trajectory" | "parameters" | "uncertainty" | "regimes" | "provenance";
export type ViewerDensity = "comfortable" | "compact";

export interface ViewerLayout {
  readonly navigationWidth: number;
  readonly evidenceWidth: number;
  readonly contentMinWidth: number;
  readonly gap: number;
  readonly density: ViewerDensity;
  readonly collapsedNavigation: boolean;
  readonly collapsedEvidence: boolean;
}

export interface ResponsiveLayoutOptions {
  readonly density?: ViewerDensity;
  readonly navigationWidth?: number;
  readonly evidenceWidth?: number;
}

function positive(value: number, name: string): void {
  if (!Number.isFinite(value) || value <= 0) throw new RangeError(`${name} must be positive`);
}

/** Chooses a stable three-column, two-column, or single-column inspection layout. */
export function responsiveViewerLayout(width: number, options: ResponsiveLayoutOptions = {}): ViewerLayout {
  positive(width, "viewer width");
  const navigationWidth = options.navigationWidth ?? 224;
  const evidenceWidth = options.evidenceWidth ?? 288;
  positive(navigationWidth, "navigation width");
  positive(evidenceWidth, "evidence width");
  const compact = width < 920;
  const narrow = width < 680;
  return Object.freeze({
    navigationWidth,
    evidenceWidth,
    contentMinWidth: narrow ? 280 : 480,
    gap: options.density === "compact" ? 12 : 18,
    density: options.density ?? "comfortable",
    collapsedNavigation: narrow,
    collapsedEvidence: compact,
  });
}

export const VIEWER_PANELS: readonly { readonly id: ViewerPanel; readonly label: string }[] = Object.freeze([
  { id: "overview", label: "Overview" },
  { id: "equations", label: "Equations" },
  { id: "graph", label: "Dependencies" },
  { id: "trajectory", label: "Trajectory" },
  { id: "parameters", label: "Parameters" },
  { id: "uncertainty", label: "Uncertainty" },
  { id: "regimes", label: "Regimes" },
  { id: "provenance", label: "Provenance" },
]);

export function availablePanels(capabilities: {
  readonly trajectory: boolean;
  readonly uncertainty: boolean;
  readonly regimes: boolean;
  readonly provenance: boolean;
}): readonly { readonly id: ViewerPanel; readonly label: string }[] {
  return VIEWER_PANELS.filter(({ id }) =>
    (id !== "trajectory" || capabilities.trajectory) &&
    (id !== "uncertainty" || capabilities.uncertainty) &&
    (id !== "regimes" || capabilities.regimes) &&
    (id !== "provenance" || capabilities.provenance));
}
