import { componentNode, type ComponentNode } from "./tokens.js";

export type BadgeTone = "neutral" | "info" | "success" | "warning" | "danger";
export interface BadgeProps { readonly label: string; readonly tone?: BadgeTone; readonly announce?: boolean; }

export function createBadge(props: BadgeProps): ComponentNode {
  if (!props.label.trim()) throw new RangeError("badge label cannot be blank");
  return componentNode("span", { "data-tone": props.tone ?? "neutral", ...(props.announce ? { "aria-live": "polite" } : {}) }, { text: props.label });
}
