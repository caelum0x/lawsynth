import { componentNode, type ComponentNode } from "./tokens.js";

export interface ProgressProps { readonly label: string; readonly value?: number; readonly min?: number; readonly max?: number; }

export function createProgress(props: ProgressProps): ComponentNode {
  if (!props.label.trim()) throw new RangeError("progress label is required");
  const min = props.min ?? 0;
  const max = props.max ?? 100;
  if (!Number.isFinite(min) || !Number.isFinite(max) || max <= min) throw new RangeError("progress range must have max greater than min");
  if (props.value !== undefined && (!Number.isFinite(props.value) || props.value < min || props.value > max)) throw new RangeError("progress value must be within range");
  return componentNode("progress", { "aria-label": props.label, ...(props.value === undefined ? {} : { value: props.value, min, max }) }, { role: "progressbar" });
}
