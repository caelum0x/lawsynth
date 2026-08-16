import type { ViewerPanel } from "./layout.js";

export type ToolbarAction = "copy-link" | "download-json" | "download-svg" | "reset-view" | "toggle-theme";

export interface ToolbarItem {
  readonly action: ToolbarAction;
  readonly label: string;
  readonly title: string;
  readonly disabled: boolean;
}

export interface ToolbarContext {
  readonly panel: ViewerPanel;
  readonly canCopyLink?: boolean;
  readonly canDownload?: boolean;
  readonly canExportSvg?: boolean;
}

export function toolbarItems(context: ToolbarContext): readonly ToolbarItem[] {
  const items: ToolbarItem[] = [
    { action: "copy-link", label: "Copy link", title: "Copy a link to this view", disabled: context.canCopyLink === false },
    { action: "download-json", label: "Export JSON", title: "Download the inspected data", disabled: context.canDownload === false },
  ];
  if (context.panel === "graph" || context.panel === "trajectory" || context.panel === "regimes") {
    items.push({ action: "download-svg", label: "Export SVG", title: "Download the current visual", disabled: context.canExportSvg === false });
    items.push({ action: "reset-view", label: "Reset view", title: "Reset zoom and selection", disabled: false });
  }
  items.push({ action: "toggle-theme", label: "Theme", title: "Toggle viewer color theme", disabled: false });
  return Object.freeze(items.map((item) => Object.freeze(item)));
}

export class ToolbarController extends EventTarget {
  #busy = new Set<ToolbarAction>();

  isBusy(action: ToolbarAction): boolean { return this.#busy.has(action); }

  async run(action: ToolbarAction, operation: () => void | Promise<void>): Promise<void> {
    if (this.#busy.has(action)) return;
    this.#busy.add(action);
    this.dispatchEvent(new CustomEvent("busychange", { detail: { action, busy: true } }));
    try {
      await operation();
      this.dispatchEvent(new CustomEvent("action", { detail: { action } }));
    } finally {
      this.#busy.delete(action);
      this.dispatchEvent(new CustomEvent("busychange", { detail: { action, busy: false } }));
    }
  }
}
