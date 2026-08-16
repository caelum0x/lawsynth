const workspaceEntries = {
  "@lawsynth/api-client": "../../../packages/api-client/dist/index.js",
  "@lawsynth/state-store": "../../../packages/state-store/dist/index.js",
  "@lawsynth/world-schema": "../../../packages/world-schema/dist/src/index.js",
  "@lawsynth/world-viewer": "../../../packages/world-viewer/dist/src/index.js",
  "@lawsynth/chart-core": "../../../packages/chart-core/dist/src/index.js",
  "@lawsynth/layout-engine": "../../../packages/layout-engine/dist/src/index.js",
};

/** Resolve workspace package specifiers while running the compiled Studio output without a package-manager symlink farm. */
export function resolve(specifier, context, nextResolve) {
  const entry = workspaceEntries[specifier];
  if (entry !== undefined) return { url: new URL(entry, import.meta.url).href, shortCircuit: true };
  return nextResolve(specifier, context);
}
