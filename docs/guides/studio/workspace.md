# Workspace state

Use `@lawsynth/state-store` to hold local UI context such as project/world selection, panels, and preferences. Commands reduce into immutable snapshots, and event ordering is deterministic by `(at, eventId)`.

```ts
import { StateStore, activeWorldId } from "@lawsynth/state-store";

const store = new StateStore();
store.dispatch({ kind: "workspace.update", patch: { projectId: "project:lab", worldId: "world:model-1" } });
console.log(activeWorldId(store.state));
```

This store does not create projects, persist a bundle, authenticate a user, or open a network connection. Provide a `PersistenceAdapter` only after defining durability, encryption, migration, and conflict semantics for the host application. Keep scientific artifacts in the system that owns them; local UI state should reference validated IDs rather than duplicate model data.
