import { componentNode, type ComponentNode } from "./tokens.js";

export interface PanelProps { readonly id?: string; readonly title?: string; readonly children: readonly ComponentNode[]; readonly tone?: "default" | "muted" | "danger"; }

export function createPanel(props: PanelProps): ComponentNode {
  if (props.title !== undefined && !props.title.trim()) throw new RangeError("panel title cannot be blank");
  const titleId = props.id === undefined || props.title === undefined ? undefined : `${props.id}-title`;
  return componentNode("section", { ...(props.id === undefined ? {} : { id: props.id }), "data-tone": props.tone ?? "default", ...(titleId === undefined ? {} : { "aria-labelledby": titleId }) }, {
    children: [
      ...(props.title === undefined ? [] : [componentNode("h2", { ...(titleId === undefined ? {} : { id: titleId }) }, { text: props.title })]),
      ...props.children,
    ],
  });
}
