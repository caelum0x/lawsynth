import { DOCS_STYLES, docsThemeScript } from "../src/theme.js";

/** Static hosts may inline these deterministic assets without a runtime framework. */
export function documentationThemeBootstrap(): string {
  return docsThemeScript();
}

export function documentationThemeStyles(): string {
  return DOCS_STYLES;
}
