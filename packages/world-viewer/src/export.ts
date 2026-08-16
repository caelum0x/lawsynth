import { chartToCsv, type ChartModel } from "@lawsynth/chart-core";
import type { WorldDefinition } from "@lawsynth/world-schema";
import { encodeViewerBundle, type ViewerBundle } from "./bundle.js";

export type ExportFormat = "json" | "csv" | "svg";

export interface ExportDocument {
  readonly filename: string;
  readonly mediaType: string;
  readonly content: string;
}

function safeFilename(value: string): string {
  const normalized = value.trim().replace(/[^A-Za-z0-9._-]+/gu, "-").replace(/^-+|-+$/gu, "");
  return (normalized || "lawsynth-world").slice(0, 120);
}

export function exportWorldJson(world: WorldDefinition, pretty = true): ExportDocument {
  return Object.freeze({ filename: `${safeFilename(world.id)}.world.json`, mediaType: "application/json", content: JSON.stringify(world, null, pretty ? 2 : undefined) });
}

export function exportViewerBundle(bundle: ViewerBundle, pretty = true): ExportDocument {
  return Object.freeze({ filename: `${safeFilename(bundle.world.id)}.viewer.json`, mediaType: "application/vnd.lawsynth.viewer+json", content: encodeViewerBundle(bundle, pretty) });
}

export function exportTrajectoryCsv(chart: ChartModel, worldId = "lawsynth-world"): ExportDocument {
  return Object.freeze({ filename: `${safeFilename(worldId)}.trajectory.csv`, mediaType: "text/csv;charset=utf-8", content: chartToCsv(chart) });
}

function escapeXml(value: string): string {
  return value.replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;").replaceAll('"', "&quot;").replaceAll("'", "&apos;");
}

/** Serializes an owned SVG element; scripts and event attributes are rejected. */
export function exportSvg(element: SVGSVGElement, filename = "lawsynth-view.svg"): ExportDocument {
  const clone = element.cloneNode(true) as SVGSVGElement;
  clone.querySelectorAll("script,foreignObject").forEach((node) => node.remove());
  for (const descendant of clone.querySelectorAll("*")) {
    for (const attribute of [...descendant.attributes]) if (/^on/iu.test(attribute.name)) descendant.removeAttribute(attribute.name);
  }
  if (!clone.hasAttribute("xmlns")) clone.setAttribute("xmlns", "http://www.w3.org/2000/svg");
  const serialized = new XMLSerializer().serializeToString(clone);
  if (!serialized.startsWith("<svg") && !serialized.includes("<svg")) throw new Error("selected element did not serialize as SVG");
  return Object.freeze({ filename: safeFilename(filename.replace(/\.svg$/iu, "")) + ".svg", mediaType: "image/svg+xml;charset=utf-8", content: serialized });
}

export function svgDocument(
  title: string,
  width: number,
  height: number,
  body: string,
): ExportDocument {
  if (![width, height].every(Number.isFinite) || width <= 0 || height <= 0) throw new RangeError("SVG dimensions must be positive");
  if (/<\/?(?:script|foreignObject)\b|\son\w+\s*=/iu.test(body)) throw new RangeError("unsafe SVG body");
  const content = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${width} ${height}" role="img"><title>${escapeXml(title)}</title>${body}</svg>`;
  return Object.freeze({ filename: `${safeFilename(title)}.svg`, mediaType: "image/svg+xml;charset=utf-8", content });
}

/** Browser-only download helper. It always revokes the temporary object URL. */
export function downloadExport(document: ExportDocument, ownerDocument: Document = globalThis.document): void {
  if (ownerDocument === undefined) throw new Error("downloads require a browser document");
  const blob = new Blob([document.content], { type: document.mediaType });
  const url = URL.createObjectURL(blob);
  const anchor = ownerDocument.createElement("a");
  anchor.href = url;
  anchor.download = safeFilename(document.filename);
  anchor.hidden = true;
  ownerDocument.body.append(anchor);
  try { anchor.click(); }
  finally { anchor.remove(); URL.revokeObjectURL(url); }
}

export async function copyText(text: string, ownerDocument: Document = globalThis.document): Promise<void> {
  if (globalThis.navigator?.clipboard !== undefined && globalThis.isSecureContext) {
    await globalThis.navigator.clipboard.writeText(text);
    return;
  }
  if (ownerDocument === undefined) throw new Error("clipboard fallback requires a browser document");
  const textarea = ownerDocument.createElement("textarea");
  textarea.value = text;
  textarea.readOnly = true;
  textarea.style.position = "fixed";
  textarea.style.opacity = "0";
  ownerDocument.body.append(textarea);
  textarea.select();
  try {
    if (!ownerDocument.execCommand("copy")) throw new Error("browser rejected clipboard copy");
  } finally { textarea.remove(); }
}
