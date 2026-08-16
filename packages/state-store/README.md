# @lawsynth/state-store

`@lawsynth/state-store` is a dependency-free, synchronous TypeScript state core for LawSynth Studio. It holds navigation context, graph selection, panel layout, and user preferences. State changes are explicit commands reduced into immutable snapshots, so the same event sequence always produces the same state.

```ts
import { StateStore, activeWorldId } from "@lawsynth/state-store";

const store = new StateStore();
store.dispatch({
  kind: "workspace.update",
  patch: { projectId: "project:climate", worldId: "world:energy-balance" },
});
console.log(activeWorldId(store.state));
```

## Boundaries

The package has no browser, database, WebSocket, or HTTP dependency. `PersistenceAdapter` is an explicit three-method boundary so an application can attach IndexedDB, a filesystem, or its own server-backed mechanism. `mergeEventLogs` deterministically deduplicates event records received from a caller-managed transport; it deliberately does not establish connections, retry operations, or choose authoritative server state.

Persistence stores only Studio UI state. Worlds, datasets, runs, and artifacts remain owned by their service APIs and are referenced only by validated identifiers.

## Event model

Commands convert to events, then `StateStore` reduces them atomically. An event envelope has an `eventId` distinct from payload IDs such as the selected variable ID. Event ordering uses `(at, eventId)`, making merged histories stable. `expectedRevision` supplies compare-and-swap behavior for callers that must reject stale writes.

`undo.ts` intentionally requires callers to provide a domain-specific inverse event. It does not attempt to synthesize an inverse for API-backed world changes.

## Verification

Run `tsc -p tsconfig.json --noEmit` to type-check and `tsx --test tests/*.test.ts` to execute the dependency-free Node test suite. The fixtures are valid persisted/UI documents; they are not generated placeholder data.
