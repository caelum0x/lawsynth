import { componentNode, type ComponentNode } from "./tokens.js";

export interface IconDefinition { readonly name: IconName; readonly viewBox: "0 0 24 24"; readonly paths: readonly string[]; }
export type IconName = "check" | "close" | "info" | "warning" | "chevronDown";

const icons: Readonly<Record<IconName, IconDefinition>> = Object.freeze({
  check: Object.freeze({ name: "check", viewBox: "0 0 24 24", paths: Object.freeze(["M5 12.5 9.2 16.7 19 6.9"]) }),
  close: Object.freeze({ name: "close", viewBox: "0 0 24 24", paths: Object.freeze(["M6 6 18 18", "M18 6 6 18"]) }),
  info: Object.freeze({ name: "info", viewBox: "0 0 24 24", paths: Object.freeze(["M12 11v6", "M12 7h.01", "M4 12a8 8 0 1 0 16 0 8 8 0 0 0-16 0"]) }),
  warning: Object.freeze({ name: "warning", viewBox: "0 0 24 24", paths: Object.freeze(["M12 3 2.5 20h19L12 3", "M12 9v4", "M12 17h.01"]) }),
  chevronDown: Object.freeze({ name: "chevronDown", viewBox: "0 0 24 24", paths: Object.freeze(["m6 9 6 6 6-6"]) }),
});

export function getIcon(name: IconName): IconDefinition { return icons[name]; }

export function createIcon(name: IconName, options: { readonly label?: string; readonly decorative?: boolean } = {}): ComponentNode {
  if (options.decorative && options.label !== undefined) throw new RangeError("decorative icons cannot have an accessible label");
  if (!options.decorative && (options.label === undefined || !options.label.trim())) throw new RangeError("meaningful icons require an accessible label");
  const icon = getIcon(name);
  return componentNode("svg", { viewBox: icon.viewBox, ...(options.decorative ? { "aria-hidden": true } : { role: "img", "aria-label": options.label as string }), "data-icon": name }, {
    children: icon.paths.map((path) => componentNode("path", { d: path }, {})),
  });
}
