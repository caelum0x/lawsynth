import { componentNode, type ComponentNode } from "./tokens.js";

export interface SelectOption { readonly value: string; readonly label: string; readonly disabled?: boolean; }
export interface SelectProps {
  readonly id: string;
  readonly label: string;
  readonly options: readonly SelectOption[];
  readonly value?: string;
  readonly required?: boolean;
  readonly disabled?: boolean;
  readonly changeAction?: string;
}

export function createSelect(props: SelectProps): ComponentNode {
  if (!props.id.trim() || !props.label.trim() || props.options.length === 0) throw new RangeError("select requires id, label, and at least one option");
  const values = new Set<string>();
  for (const option of props.options) {
    if (!option.value.trim() || !option.label.trim() || values.has(option.value)) throw new RangeError("select options need unique non-empty values and labels");
    values.add(option.value);
  }
  if (props.value !== undefined && !values.has(props.value)) throw new RangeError("selected value is not an option");
  return componentNode("div", {}, { children: [
    componentNode("label", { for: props.id }, { text: props.label }),
    componentNode("select", { id: props.id, ...(props.value === undefined ? {} : { value: props.value }), ...(props.required ? { required: true, "aria-required": true } : {}), ...(props.disabled ? { disabled: true } : {}) }, {
      events: props.changeAction === undefined || props.disabled ? [] : [{ type: "change", action: props.changeAction }],
      children: props.options.map((option) => componentNode("option", { value: option.value, ...(option.disabled ? { disabled: true } : {}), ...(option.value === props.value ? { selected: true } : {}) }, { text: option.label })),
    }),
  ] });
}
