import { componentNode, type ComponentNode } from "./tokens.js";

export interface DialogProps {
  readonly id: string;
  readonly title: string;
  readonly content: readonly ComponentNode[];
  readonly open: boolean;
  readonly modal?: boolean;
  readonly initialFocusId?: string;
  readonly returnFocusId?: string;
  readonly dismissAction?: string;
}

export function createDialog(props: DialogProps): ComponentNode {
  if (!props.id.trim() || !props.title.trim()) throw new RangeError("dialog id and title are required");
  if (props.open && props.dismissAction === undefined) throw new RangeError("an open dialog requires a dismissal command");
  const titleId = `${props.id}-title`;
  return componentNode("dialog", { id: props.id, "aria-labelledby": titleId, ...(props.open ? { open: true } : {}), ...(props.modal ?? true ? { "aria-modal": true } : {}) }, {
    role: "dialog",
    focus: { trap: props.modal ?? true, ...(props.initialFocusId === undefined ? {} : { initialId: props.initialFocusId }), ...(props.returnFocusId === undefined ? {} : { returnId: props.returnFocusId }) },
    events: props.dismissAction === undefined ? [] : [{ type: "dismiss", action: props.dismissAction }],
    children: [componentNode("h2", { id: titleId }, { text: props.title }), ...props.content],
  });
}
