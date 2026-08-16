import { applyPlaygroundTheme, playgroundThemeCss, resolvePlaygroundTheme, type PlaygroundTheme } from "../src/theme.js";

/** Apply a user choice or media-query fallback to a mounted playground shell. */
export function configurePlaygroundTheme(root: HTMLElement, requested: string | null, prefersDark: boolean): PlaygroundTheme {
  const theme = resolvePlaygroundTheme(requested, prefersDark);
  applyPlaygroundTheme(root, theme);
  return theme;
}

/** Server-side shells can emit the identical custom-property set before hydration. */
export function playgroundThemeStyle(theme: PlaygroundTheme): string {
  return playgroundThemeCss(theme);
}
