# Studio boundary

There is no supported LawSynth Studio runtime in this repository today. The
implemented TypeScript packages provide dependency-light contracts that a
future UI can use: world-source validation, chart data transformation, UI
state reduction, design-system contracts, and an HTTP client.

They do not decode `.lsworld` bundles, execute simulation, establish service
connections, persist Worlds, or provide a browser application. In particular,
directories named `apps/studio`, `apps/playground`, and `packages/world-viewer`
are planning locations, not an installation target.

For the executable local experience, use the Rust CLI or built Python native
package. For TypeScript boundary validation, see the contribution workflow in
the TypeScript section of [development](../contributing/development.md).
