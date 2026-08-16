export type DocsTheme = "paper" | "midnight";

export function docsThemeScript(storageKey = "lawsynth:docs:theme"): string {
  if (!/^[A-Za-z0-9:._-]+$/u.test(storageKey)) throw new RangeError("theme storage key is invalid");
  return `(()=>{try{const v=localStorage.getItem(${JSON.stringify(storageKey)});const d=v==='midnight'||(v!=='paper'&&matchMedia('(prefers-color-scheme:dark)').matches);document.documentElement.dataset.theme=d?'midnight':'paper'}catch{}})()`;
}

export const DOCS_STYLES = `:root{
  --doc-bg:#f3f0e8;--doc-surface:#fffdf7;--doc-ink:#18201d;--doc-muted:#59635e;--doc-accent:#b54b2a;--doc-line:#c8c6ba;color-scheme:light
}[data-theme=midnight]{
  --doc-bg:#101714;--doc-surface:#17201c;--doc-ink:#eef3ef;--doc-muted:#a9b7b0;--doc-accent:#ef7d55;--doc-line:#3c4a43;color-scheme:dark
}body{margin:0;background:var(--doc-bg);color:var(--doc-ink);font:16px/1.65 Inter,system-ui,sans-serif}
.skip-link{position:absolute;left:-9999px}.skip-link:focus{left:12px;top:12px;background:var(--doc-surface);padding:10px;z-index:10}
.docs-shell{display:grid;grid-template-columns:260px minmax(0,1fr);gap:32px;max-width:1280px;margin:auto;padding:24px}
main{max-width:76ch}h1,h2,h3{font-family:Georgia,serif;line-height:1.15}code,pre{font-family:ui-monospace,SFMono-Regular,monospace}
pre{overflow:auto;padding:16px;background:var(--doc-surface);border:1px solid var(--doc-line)}a{color:var(--doc-accent);text-underline-offset:3px}
:focus-visible{outline:3px solid var(--doc-accent);outline-offset:3px}@media(max-width:760px){.docs-shell{display:block}}@media(prefers-reduced-motion:reduce){*{scroll-behavior:auto!important}}`;
