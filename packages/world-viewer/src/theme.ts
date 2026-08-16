export type ViewerThemeName = "paper" | "midnight";

export interface ViewerTheme {
  readonly name: ViewerThemeName;
  readonly colors: {
    readonly canvas: string;
    readonly surface: string;
    readonly raised: string;
    readonly text: string;
    readonly muted: string;
    readonly border: string;
    readonly accent: string;
    readonly accentText: string;
    readonly positive: string;
    readonly warning: string;
    readonly grid: string;
  };
}

export const paperTheme: ViewerTheme = Object.freeze({
  name: "paper",
  colors: Object.freeze({ canvas: "#f3f0e8", surface: "#fffdf7", raised: "#ffffff", text: "#18201d", muted: "#59635e", border: "#c8c6ba", accent: "#b54b2a", accentText: "#ffffff", positive: "#176b52", warning: "#9a5b16", grid: "#e5e1d7" }),
});

export const midnightTheme: ViewerTheme = Object.freeze({
  name: "midnight",
  colors: Object.freeze({ canvas: "#101714", surface: "#17201c", raised: "#1d2823", text: "#eef3ef", muted: "#a9b7b0", border: "#3c4a43", accent: "#ef7d55", accentText: "#17110e", positive: "#67c9a8", warning: "#e6b566", grid: "#25332d" }),
});

const HEX_COLOR = /^#[0-9a-f]{6}$/iu;

export function validateViewerTheme(theme: ViewerTheme): ViewerTheme {
  if (theme.name !== "paper" && theme.name !== "midnight") throw new RangeError(`unsupported viewer theme name: ${String(theme.name)}`);
  for (const [name, color] of Object.entries(theme.colors)) {
    if (!HEX_COLOR.test(color)) throw new RangeError(`theme color ${name} must be a six-digit hexadecimal color`);
  }
  return theme;
}

export function resolveViewerTheme(theme: ViewerThemeName | ViewerTheme | undefined): ViewerTheme {
  if (theme === undefined || theme === "paper") return paperTheme;
  if (theme === "midnight") return midnightTheme;
  return validateViewerTheme(theme);
}

export function themeProperties(theme: ViewerTheme): Readonly<Record<string, string>> {
  return Object.freeze(Object.fromEntries(Object.entries(theme.colors).map(([key, value]) => [`--lsv-${key}`, value])));
}

/** Scoped stylesheet. The ruled evidence rail is the viewer's audit-ledger signature. */
export function viewerStyles(theme: ViewerTheme): string {
  validateViewerTheme(theme);
  const variables = Object.entries(themeProperties(theme)).map(([name, value]) => `${name}:${value}`).join(";");
  return `:host,.lsv-root{${variables};color:var(--lsv-text);font-family:Inter,ui-sans-serif,system-ui,sans-serif;color-scheme:${theme.name === "midnight" ? "dark" : "light"}}
.lsv-root{box-sizing:border-box;display:grid;grid-template-rows:auto 1fr;min-height:360px;background:var(--lsv-canvas);border:1px solid var(--lsv-border);container-type:inline-size}
.lsv-root *{box-sizing:border-box}.lsv-header{display:flex;align-items:center;gap:12px;padding:14px 18px;border-bottom:1px solid var(--lsv-border);background:var(--lsv-surface)}
.lsv-kicker,.lsv-label{font:600 11px/1.2 ui-monospace,SFMono-Regular,monospace;letter-spacing:.09em;text-transform:uppercase;color:var(--lsv-muted)}
.lsv-title{font:650 18px/1.25 Inter,ui-sans-serif,system-ui,sans-serif;margin:2px 0}.lsv-kind{margin-left:auto;padding:4px 8px;border:1px solid var(--lsv-border);font:600 11px/1 ui-monospace,monospace}
.lsv-shell{display:grid;grid-template-columns:210px minmax(0,1fr) 280px;min-height:0}.lsv-nav{padding:14px;border-right:1px solid var(--lsv-border);background:var(--lsv-surface)}
.lsv-nav button,.lsv-toolbar button{width:100%;min-height:44px;border:0;background:transparent;color:inherit;text-align:left;padding:8px 10px;font:600 13px/1.2 inherit;cursor:pointer}
.lsv-nav button[aria-current=page]{background:var(--lsv-text);color:var(--lsv-surface)}.lsv-nav button:focus-visible,.lsv-toolbar button:focus-visible{outline:3px solid var(--lsv-accent);outline-offset:2px}
.lsv-main{min-width:0;padding:22px;overflow:auto}.lsv-evidence{padding:18px;border-left:1px solid var(--lsv-border);background-color:var(--lsv-surface);background-image:repeating-linear-gradient(to bottom,transparent 0,transparent 27px,var(--lsv-grid) 28px);overflow:auto}
.lsv-panel{max-width:960px}.lsv-panel h2{font:650 24px/1.15 Georgia,serif;margin:0 0 18px}.lsv-grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(190px,1fr));gap:12px}
.lsv-card{background:var(--lsv-raised);border:1px solid var(--lsv-border);padding:14px}.lsv-value{font:500 18px/1.4 ui-monospace,SFMono-Regular,monospace}.lsv-muted{color:var(--lsv-muted)}
.lsv-equation{padding:14px 0;border-bottom:1px solid var(--lsv-border)}.lsv-equation code{display:block;margin-top:6px;white-space:pre-wrap;font:500 15px/1.55 ui-monospace,SFMono-Regular,monospace;color:var(--lsv-text)}
.lsv-table{width:100%;border-collapse:collapse}.lsv-table th,.lsv-table td{text-align:left;padding:9px 8px;border-bottom:1px solid var(--lsv-border)}.lsv-table th{font-size:11px;text-transform:uppercase;letter-spacing:.06em;color:var(--lsv-muted)}
.lsv-svg{width:100%;min-height:280px;background:var(--lsv-raised);border:1px solid var(--lsv-border)}.lsv-edge{stroke:var(--lsv-muted);stroke-width:1.5;fill:none}.lsv-node{fill:var(--lsv-surface);stroke:var(--lsv-border)}
.lsv-toolbar{display:flex;gap:4px;margin-left:auto}.lsv-toolbar button{width:auto}.lsv-status{padding-left:9px;border-left:3px solid var(--lsv-accent);margin-bottom:18px}
@container (max-width:900px){.lsv-shell{grid-template-columns:180px minmax(0,1fr)}.lsv-evidence{display:none}}
@container (max-width:620px){.lsv-shell{display:block}.lsv-nav{display:flex;overflow:auto;border-right:0;border-bottom:1px solid var(--lsv-border)}.lsv-nav button{white-space:nowrap;width:auto}.lsv-main{padding:16px}.lsv-header{align-items:flex-start;flex-wrap:wrap}}
@media (prefers-reduced-motion:reduce){.lsv-root *{scroll-behavior:auto!important;transition:none!important}}`;
}

export function applyTheme(element: HTMLElement, theme: ViewerTheme): void {
  for (const [property, value] of Object.entries(themeProperties(theme))) element.style.setProperty(property, value);
  element.dataset.theme = theme.name;
}
