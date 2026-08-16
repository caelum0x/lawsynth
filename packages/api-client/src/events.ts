import type { ApiEvent, RunId } from "./generated.js";
import { pathSegment, type Transport } from "./transport.js";
export interface EventStreamOptions { after?: number; signal?: AbortSignal; }
/** Parses the server's text/event-stream endpoint incrementally. */
export async function* streamRunEvents(transport: Transport, runId: RunId, options: EventStreamOptions = {}): AsyncGenerator<ApiEvent, void, void> {
  const response = await transport.request<Response>({ path: `/v1/runs/${pathSegment(runId)}/events`, query: { after: options.after }, headers: { Accept: "text/event-stream" }, response: "response", signal: options.signal, timeoutMs: 24 * 60 * 60 * 1_000, retry: false });
  if (!response.body) throw new Error("Event stream response has no body");
  const reader = response.body.pipeThrough(new TextDecoderStream()).getReader(); let buffer = "";
  try {
    while (true) {
      options.signal?.throwIfAborted(); const chunk = await reader.read(); if (chunk.done) break; buffer += chunk.value;
      let boundary = eventBoundary(buffer);
      while (boundary !== undefined) { const raw = buffer.slice(0, boundary.index); buffer = buffer.slice(boundary.index + boundary.length); const event = parseServerEvent(raw); if (event !== undefined) yield event; boundary = eventBoundary(buffer); }
    }
  } finally { await reader.cancel().catch(() => undefined); reader.releaseLock(); }
}
function parseServerEvent(raw: string): ApiEvent | undefined {
  const data = raw.split(/\r?\n/u).filter((line) => line.startsWith("data:")).map((line) => line.slice(5).replace(/^ /u, "")).join("\n");
  if (!data) return undefined; const event = JSON.parse(data) as ApiEvent;
  if (typeof event.event_id !== "string" || typeof event.topic !== "string" || typeof event.occurred_at !== "string") throw new TypeError("Malformed API event"); return event;
}
function eventBoundary(value: string): { index: number; length: number } | undefined { const unix = value.indexOf("\n\n"); const windows = value.indexOf("\r\n\r\n"); if (unix < 0 && windows < 0) return undefined; return windows >= 0 && (unix < 0 || windows < unix) ? { index: windows, length: 4 } : { index: unix, length: 2 }; }
