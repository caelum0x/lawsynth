export type CodeLanguage = "bash" | "json" | "python" | "rust" | "typescript" | "text";
export interface CodeBlock { readonly language: CodeLanguage; readonly source: string; readonly caption?: string; readonly highlightedHtml: string; }

export function escapeHtml(value: string): string {
  return value.replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;").replaceAll('"', "&quot;").replaceAll("'", "&#39;");
}

const LANGUAGE_ALIASES: Readonly<Record<string, CodeLanguage>> = { sh: "bash", shell: "bash", console: "bash", js: "typescript", javascript: "typescript", ts: "typescript", py: "python", rs: "rust", plaintext: "text", txt: "text" };
export function normalizeLanguage(value: string): CodeLanguage {
  const language = (LANGUAGE_ALIASES[value.toLowerCase()] ?? value.toLowerCase()) as CodeLanguage;
  return (["bash", "json", "python", "rust", "typescript", "text"] as const).includes(language) ? language : "text";
}

/** Small deterministic highlighter; source is escaped before spans are introduced. */
export function highlightCode(source: string, language: CodeLanguage): string {
  let html = escapeHtml(source);
  if (language === "json") html = html.replace(/(&quot;(?:\\.|[^&])*?&quot;)(\s*:)?/gu, (_match, string: string, colon: string | undefined) => colon ? `<span class="tok-key">${string}</span>${colon}` : `<span class="tok-string">${string}</span>`);
  else if (language === "python") html = html.replace(/\b(def|class|return|import|from|as|if|else|for|in|True|False|None)\b/gu, '<span class="tok-keyword">$1</span>');
  else if (language === "rust") html = html.replace(/\b(fn|let|mut|pub|impl|struct|enum|match|use|mod|Result|Option|Self)\b/gu, '<span class="tok-keyword">$1</span>');
  else if (language === "typescript") html = html.replace(/\b(const|let|function|class|interface|type|import|export|from|return|async|await|readonly)\b/gu, '<span class="tok-keyword">$1</span>');
  else if (language === "bash") html = html.replace(/(^|\n)(\$ )/gu, '$1<span class="tok-prompt">$2</span>');
  return html;
}

export function codeBlock(source: string, language = "text", caption?: string): CodeBlock {
  const normalized = normalizeLanguage(language);
  return Object.freeze({ language: normalized, source, ...(caption === undefined ? {} : { caption }), highlightedHtml: highlightCode(source, normalized) });
}
