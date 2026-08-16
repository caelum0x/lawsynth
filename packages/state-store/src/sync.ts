import { compareEvents, type StateEvent } from "./events.js";

/**
 * Deterministic merge primitive for a transport chosen by the application.
 * This package deliberately does not open sockets, retry requests, or infer
 * server authority; callers exchange these events through their own protocol.
 */
export function mergeEventLogs(local: readonly StateEvent[], remote: readonly StateEvent[]): readonly StateEvent[] {
  const byId = new Map<string, StateEvent>();
  for (const event of [...local, ...remote]) {
    const prior = byId.get(event.eventId);
    if (prior !== undefined && JSON.stringify(prior) !== JSON.stringify(event)) throw new Error(`Event id collision with different payload: ${event.eventId}`);
    byId.set(event.eventId, event);
  }
  return Object.freeze([...byId.values()].sort(compareEvents));
}

export function eventsAfter(events: readonly StateEvent[], cursor: string | undefined): readonly StateEvent[] {
  if (cursor === undefined) return events;
  const index = events.findIndex((event) => event.eventId === cursor);
  if (index < 0) throw new RangeError(`Unknown event cursor: ${cursor}`);
  return events.slice(index + 1);
}
