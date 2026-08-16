import { componentNode, type ComponentNode } from "./tokens.js";

export type ButtonVariant = "primary" | "secondary" | "danger" | "ghost";
export type ButtonType = "button" | "submit" | "reset";

export interface ButtonProps {
  readonly id: string;
  readonly label: string;
  readonly variant?: ButtonVariant;
  readonly type?: ButtonType;
  readonly disabled?: boolean;
  readonly busy?: boolean;
  readonly action?: string;
}

/** Creates a semantic control contract. A renderer binds the action to application code. */
export function createButton(props: ButtonProps): ComponentNode {
  if (!props.id.trim() || !props.label.trim()) throw new RangeError("button id and label are required");
  if (props.busy && props.disabled) throw new RangeError("a busy button remains discoverable and cannot also be disabled");
  const disabled = props.disabled ?? false;
  const busy = props.busy ?? false;
  if (props.action !== undefined && !props.action.trim()) throw new RangeError("button action cannot be blank");
  return componentNode("button", {
    id: props.id,
    type: props.type ?? "button",
    "data-variant": props.variant ?? "primary",
    "aria-disabled": disabled,
    ...(busy ? { "aria-busy": true } : {}),
    ...(disabled ? { disabled: true } : {}),
  }, { text: props.label, events: props.action === undefined || disabled ? [] : [{ type: "activate", action: props.action }] });
}
