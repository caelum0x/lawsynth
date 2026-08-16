# Studio integration boundary

The repository contains TypeScript packages for deterministic UI state and renderer-neutral chart data, but it does not currently ship an executable Studio application, authenticated workspace service, or browser deployment. Treat the Studio directories as integration contracts, not a hosted product.

`@lawsynth/state-store` provides synchronous immutable UI state, commands, event ordering, and an explicit persistence adapter boundary. `@lawsynth/chart-core` validates trajectories and produces chart models, ticks, domains, and exports without rendering a DOM, canvas, or SVG. A host application must provide rendering, storage, authentication, transport, and its own API integration.

Use these guides to build an honest client integration. Do not document a screen, server endpoint, collaboration flow, or deployment command that this repository does not implement.
