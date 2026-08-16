export type PlaygroundTheme = "paper" | "midnight";

export interface PlaygroundPalette {
  readonly canvas: string;
  readonly surface: string;
  readonly ink: string;
  readonly muted: string;
  readonly accent: string;
  readonly line: string;
  readonly danger: string;
}

export const PLAYGROUND_COLORS: Readonly<Record<PlaygroundTheme, PlaygroundPalette>> = Object.freeze({
  paper: Object.freeze({ canvas: "#eee9dd", surface: "#fffdf7", ink: "#17201c", muted: "#5d665f", accent: "#b54b2a", line: "#c8c4b8", danger: "#a3382b" }),
  midnight: Object.freeze({ canvas: "#101713", surface: "#19221d", ink: "#eef4ef", muted: "#a7b5ad", accent: "#ef7d55", line: "#3a4941", danger: "#ff8b7b" }),
});

export function playgroundThemeProperties(theme: PlaygroundTheme): Readonly<Record<string, string>> {
  return Object.freeze(Object.fromEntries(
    Object.entries(PLAYGROUND_COLORS[theme]).map(([key, value]) => [`--lsp-${key}`, value]),
  ));
}

export function playgroundThemeCss(theme: PlaygroundTheme): string {
  return Object.entries(playgroundThemeProperties(theme)).map(([key, value]) => `${key}:${value}`).join(";");
}

export function applyPlaygroundTheme(element: HTMLElement, theme: PlaygroundTheme): void {
  for (const [property, value] of Object.entries(playgroundThemeProperties(theme))) {
    element.style.setProperty(property, value);
  }
  element.dataset.theme = theme;
}

export function resolvePlaygroundTheme(value: string | null, prefersDark = false): PlaygroundTheme {
  if (value === "paper" || value === "midnight") return value;
  return prefersDark ? "midnight" : "paper";
}
