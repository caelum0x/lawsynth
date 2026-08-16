import { componentNode, type ComponentNode } from "./tokens.js";

export interface PopoverProps {
  readonly id: string;
  readonly triggerId: string;
  readonly content: readonly ComponentNode[];
  readonly open: boolean;
  readonly dismissAction?: string;
}

export function createPopover(props: PopoverProps): ComponentNode {
  if (!props.id.trim() || !props.triggerId.trim() || props.content.length === 0) throw new RangeError("popover requires id, trigger, and content");
  if (props.open && props.dismissAction === undefined) throw new RangeError("an open popover requires a dismissal command");
  return componentNode("div", { id: props.id, "aria-labelledby": props.triggerId, ...(props.open ? {} : { hidden: true }) }, {
    role: "dialog",
    events: props.dismissAction === undefined ? [] : [{ type: "dismiss", action: props.dismissAction }],
    children: props.content,
  });
}
