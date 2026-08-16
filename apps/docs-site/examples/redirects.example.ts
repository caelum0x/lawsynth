import { resolveRedirect, validateRedirects, type Redirect } from "../src/redirects.js";

export const legacyDocumentationRedirects: readonly Redirect[] = validateRedirects([
  { from: "/docs/quickstart", to: "/guide/quickstart", permanent: true },
  { from: "/docs/api", to: "/reference/api", permanent: true },
]);

/** Resolve only known legacy paths; callers should leave unknown paths untouched. */
export function resolveLegacyDocumentationPath(path: string): Redirect | undefined {
  return resolveRedirect(legacyDocumentationRedirects, path);
}
