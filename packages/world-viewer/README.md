# `@lawsynth/world-viewer`

`world-viewer` is the read-only browser inspection surface for a LawSynth
`WorldDefinition`. It renders equations, dependency structure, parameters,
simulation trajectories, declared regimes, uncertainty records, and provenance
without evaluating model code or silently inventing missing evidence.

The package is framework-neutral. `WorldViewer` can mount into an element owned
by any application, while `defineWorldViewerElement()` provides a declarative
custom element. All model strings are assigned through DOM `textContent`; the
viewer does not render model-provided HTML.

```ts
import { createViewerBundle, createWorldViewer } from "@lawsynth/world-viewer";

const viewer = createWorldViewer({
  bundle: createViewerBundle(world, trajectory),
  theme: "paper",
});

viewer.mount(document.querySelector("#world")!);
```

## Bundle boundary

The browser envelope is JSON with format `lawsynth-viewer`, version `1`. It may
contain a rich TypeScript World and an aligned trajectory. It is deliberately
not the canonical `.lsworld` binary archive. Native archives are validated and
decoded by the Rust bundle implementation or a trusted service, then delivered
to the viewer as the bounded JSON envelope.

Remote embedding applies a request timeout and a 32 MiB default response limit:

```ts
import { defineWorldViewerElement } from "@lawsynth/world-viewer";

defineWorldViewerElement();
// <lawsynth-world-viewer src="/worlds/sir/viewer.json" panel="equations" />
```

Use `loadViewerBundle()` directly to provide a smaller `maxBytes` limit, an
abort signal, credentials policy, or application-specific `fetch` adapter.

## Lifecycle and export

Call `destroy()` when an imperative viewer is permanently removed. Custom
elements abort in-flight loads when disconnected. JSON, trajectory CSV, and
sanitized SVG exports are provided as data-first functions; browser download is
an explicit helper so server-side callers are not forced to depend on the DOM.

The viewer explains the record it receives. It does not simulate a World,
estimate uncertainty, infer physical meanings for regimes, verify remote trust,
or decode native ZIP archives.
