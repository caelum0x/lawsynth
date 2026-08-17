import type { ChartModel } from "@lawsynth/chart-core";
import type { PlotGeometry } from "@lawsynth/world-viewer";
import type { ScreenId } from "./ids.js";

export type { ScreenId } from "./ids.js";
export { SCREEN_IDS, isScreenId } from "./ids.js";

/**
 * A screen is a pure view-model producer. It never touches the DOM; it emits a
 * `ScreenModel` render description that a renderer (or an SSR / worker context)
 * turns into pixels. The structure mirrors how `chart-core` and `world-viewer`
 * already return structured render data instead of raw nodes.
 */
export interface ScreenModel {
  readonly id: ScreenId;
  readonly title: string;
  readonly subtitle?: string;
  readonly sections: readonly ScreenSection[];
}

export interface Metric {
  readonly label: string;
  readonly value: string;
  readonly hint?: string;
  readonly tone?: NoticeTone;
}

export type NoticeTone = "info" | "success" | "warning" | "error";

export interface Notice {
  readonly tone: NoticeTone;
  readonly message: string;
}

export interface TableColumn {
  readonly key: string;
  readonly label: string;
  readonly align?: "start" | "end";
}

export interface TableRow {
  readonly id: string;
  readonly cells: readonly string[];
  readonly selected?: boolean;
  readonly emphasis?: boolean;
}

export type ControlKind = "number" | "text" | "select" | "toggle" | "range";

export interface ControlOption {
  readonly value: string;
  readonly label: string;
}

export interface ControlField {
  readonly id: string;
  readonly label: string;
  readonly kind: ControlKind;
  readonly value: string | number | boolean;
  readonly options?: readonly ControlOption[];
  readonly min?: number;
  readonly max?: number;
  readonly step?: number;
  readonly unit?: string;
  readonly help?: string;
  readonly disabled?: boolean;
}

export interface ActionButton {
  readonly id: string;
  readonly label: string;
  readonly tone?: NoticeTone;
  readonly disabled?: boolean;
}

/** A single readable term inside a rendered equation (Equation Explorer). */
export interface EquationTerm {
  readonly id: string;
  readonly sign: "+" | "-";
  readonly text: string;
  readonly symbols: readonly string[];
}

export interface EquationBlock {
  readonly id: string;
  readonly heading: string;
  readonly text: string;
  readonly enabled: boolean;
  readonly selected: boolean;
  readonly terms: readonly EquationTerm[];
}

/** Pixel-space timeline (Regime Timeline). */
export interface TimelineSegment {
  readonly id: string;
  readonly label: string;
  readonly regime: string;
  readonly start: number;
  readonly end: number;
  readonly x: number;
  readonly y: number;
  readonly width: number;
  readonly height: number;
  readonly color: string;
  readonly confidence?: number;
  readonly selected: boolean;
}

export interface TimelineBoundary {
  readonly id: string;
  readonly time: number;
  readonly x: number;
  readonly label: string;
}

export interface TimelineAxisTick {
  readonly value: number;
  readonly x: number;
  readonly label: string;
}

export interface TimelineView {
  readonly segments: readonly TimelineSegment[];
  readonly boundaries: readonly TimelineBoundary[];
  readonly ticks: readonly TimelineAxisTick[];
  readonly width: number;
  readonly height: number;
  readonly start: number;
  readonly end: number;
}

/** A shaded band overlaid on a line trajectory (Uncertainty Lens). */
export interface BandOverlay {
  readonly id: string;
  readonly label: string;
  readonly variable: string;
  readonly confidence: number;
  readonly polygon: string;
  readonly medianPath?: string;
  readonly color: string;
}

export type ScreenSection =
  | { readonly kind: "notices"; readonly id: string; readonly notices: readonly Notice[] }
  | { readonly kind: "metrics"; readonly id: string; readonly title?: string; readonly metrics: readonly Metric[] }
  | { readonly kind: "controls"; readonly id: string; readonly title?: string; readonly fields: readonly ControlField[] }
  | { readonly kind: "actions"; readonly id: string; readonly title?: string; readonly buttons: readonly ActionButton[] }
  | {
      readonly kind: "table";
      readonly id: string;
      readonly title?: string;
      readonly columns: readonly TableColumn[];
      readonly rows: readonly TableRow[];
      readonly empty?: string;
    }
  | {
      readonly kind: "chart";
      readonly id: string;
      readonly title?: string;
      readonly chart: ChartModel;
      readonly geometry: PlotGeometry;
      readonly bands?: readonly BandOverlay[];
    }
  | { readonly kind: "timeline"; readonly id: string; readonly title?: string; readonly timeline: TimelineView }
  | { readonly kind: "equations"; readonly id: string; readonly title?: string; readonly equations: readonly EquationBlock[] };

/** Interaction surface a renderer wires back into the screen controller. */
export interface ScreenActions {
  readonly onSelect: (sectionId: string, rowId: string) => void;
  readonly onControl: (fieldId: string, rawValue: string) => void;
  readonly onAction: (actionId: string) => void;
}
