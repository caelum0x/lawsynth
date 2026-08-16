import { componentNode, type ComponentNode } from "./tokens.js";

export type InputKind = "text" | "email" | "number" | "search" | "password" | "url";
export interface InputProps {
  readonly id: string;
  readonly label: string;
  readonly kind?: InputKind;
  readonly value?: string;
  readonly placeholder?: string;
  readonly description?: string;
  readonly error?: string;
  readonly required?: boolean;
  readonly disabled?: boolean;
  readonly changeAction?: string;
}

export function createInput(props: InputProps): ComponentNode {
  if (!props.id.trim() || !props.label.trim()) throw new RangeError("input id and visible label are required");
  if (props.error !== undefined && !props.error.trim()) throw new RangeError("input error cannot be blank");
  const descriptionId = props.description === undefined ? undefined : `${props.id}-description`;
  const errorId = props.error === undefined ? undefined : `${props.id}-error`;
  const describedBy = [descriptionId, errorId].filter((value): value is string => value !== undefined).join(" ");
  const children: ComponentNode[] = [componentNode("label", { for: props.id }, { text: props.label })];
  children.push(componentNode("input", {
    id: props.id,
    type: props.kind ?? "text",
    ...(props.value === undefined ? {} : { value: props.value }),
    ...(props.placeholder === undefined ? {} : { placeholder: props.placeholder }),
    ...(props.required ? { required: true, "aria-required": true } : {}),
    ...(props.disabled ? { disabled: true } : {}),
    ...(props.error === undefined ? {} : { "aria-invalid": true }),
    ...(describedBy ? { "aria-describedby": describedBy } : {}),
  }, { events: props.changeAction === undefined || props.disabled ? [] : [{ type: "change", action: props.changeAction }] }));
  if (props.description !== undefined) children.push(componentNode("p", { id: descriptionId as string }, { text: props.description }));
  if (props.error !== undefined) children.push(componentNode("p", { id: errorId as string, role: "alert" }, { text: props.error }));
  return componentNode("div", {}, { children });
}
