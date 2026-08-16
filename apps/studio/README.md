# LawSynth Studio

Studio is the browser orchestration layer for project workspaces, dataset
inspection, law discovery, simulation, model structure, uncertainty, regimes,
and provenance. It composes the typed API client and state store; numerical
execution remains on the LawSynth service/runtime boundary.

The application controller is framework-neutral and can mount its accessible
shell into an owned DOM element. Providers are explicit so authentication,
persistence, logging, notification delivery, clocks, and identifiers remain
replaceable at the application boundary.

```ts
const app = createStudioApp({ providers: scope, route: parseRoute(location.href) });
app.mount(document.querySelector("#app")!);
await app.start();
```

Call `stop()` during permanent teardown. It flushes pending settings,
unsubscribes state listeners, detaches keyboard shortcuts, cancels the active
workspace, and disposes providers in reverse registration order.
