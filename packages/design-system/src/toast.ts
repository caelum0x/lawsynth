import { componentNode, type ComponentNode } from "./tokens.js";

export type ToastTone = "info" | "success" | "warning" | "error";
export interface ToastProps { readonly id: string; readonly message: string; readonly tone?: ToastTone; readonly dismissAction?: string; readonly timeoutMs?: number; }

export function createToast(props: ToastProps): ComponentNode {
  if (!props.id.trim() || !props.message.trim()) throw new RangeError("toast id and message are required");
  if (props.timeoutMs !== undefined && (!Number.isInteger(props.timeoutMs) || props.timeoutMs < 5_000)) throw new RangeError("auto-dismiss toasts must remain visible for at least five seconds");
  const tone = props.tone ?? "info";
  return componentNode("div", { id: props.id, "data-tone": tone, ...(props.timeoutMs === undefined ? {} : { "data-timeout-ms": props.timeoutMs }) }, {
    role: tone === "error" ? "alert" : "status",
    text: props.message,
    events: props.dismissAction === undefined ? [] : [{ type: "dismiss", action: props.dismissAction }],
  });
}
