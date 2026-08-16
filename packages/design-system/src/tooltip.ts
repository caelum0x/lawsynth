import { componentNode, type ComponentNode } from "./tokens.js";

export interface TooltipProps { readonly id: string; readonly triggerId: string; readonly text: string; readonly visible: boolean; }

export function createTooltip(props: TooltipProps): ComponentNode {
  if (!props.id.trim() || !props.triggerId.trim() || !props.text.trim()) throw new RangeError("tooltip id, trigger, and text are required");
  return componentNode("div", { id: props.id, "data-for": props.triggerId, ...(props.visible ? {} : { hidden: true }) }, { role: "tooltip", text: props.text });
}
