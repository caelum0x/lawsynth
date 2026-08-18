import { escapeHtml } from "./code.js";
export interface SeoMetadata {
  readonly title: string; readonly description: string; readonly canonicalUrl: string;
  readonly imageUrl?: string; readonly type?: "website" | "article"; readonly noIndex?: boolean;
  readonly publishedAt?: string; readonly modifiedAt?: string;
}
export function renderSeo(metadata: SeoMetadata, siteName = "LawSynth"): string {
  const canonical = new URL(metadata.canonicalUrl);
  if (!["http:", "https:"].includes(canonical.protocol)) throw new RangeError("canonical URL must use HTTP(S)");
  if (!metadata.title.trim() || metadata.description.trim().length < 20) throw new RangeError("SEO title and a descriptive summary are required");
  const title = metadata.title.includes(siteName) ? metadata.title : `${metadata.title} · ${siteName}`;
  const tags = [
    `<title>${escapeHtml(title)}</title>`,
    `<meta name="description" content="${escapeHtml(metadata.description)}">`,
    `<link rel="canonical" href="${escapeHtml(canonical.toString())}">`,
    `<meta property="og:site_name" content="${escapeHtml(siteName)}">`,
    `<meta property="og:title" content="${escapeHtml(title)}">`,
    `<meta property="og:description" content="${escapeHtml(metadata.description)}">`,
    `<meta property="og:url" content="${escapeHtml(canonical.toString())}">`,
    `<meta property="og:type" content="${metadata.type ?? "website"}">`,
    `<meta name="twitter:card" content="${metadata.imageUrl ? "summary_large_image" : "summary"}">`,
    `<meta name="twitter:title" content="${escapeHtml(title)}">`,
    `<meta name="twitter:description" content="${escapeHtml(metadata.description)}">`,
  ];
  if (metadata.imageUrl) {
    const image = escapeHtml(new URL(metadata.imageUrl, canonical).toString());
    tags.push(`<meta property="og:image" content="${image}">`);
    tags.push(`<meta name="twitter:image" content="${image}">`);
  }
  if (metadata.noIndex) tags.push('<meta name="robots" content="noindex,nofollow">');
  if (metadata.publishedAt) tags.push(`<meta property="article:published_time" content="${escapeHtml(metadata.publishedAt)}">`);
  if (metadata.modifiedAt) tags.push(`<meta property="article:modified_time" content="${escapeHtml(metadata.modifiedAt)}">`);
  return tags.join("\n");
}
export function softwareStructuredData(name: string, version: string, url: string): string {
  return JSON.stringify({ "@context": "https://schema.org", "@type": "SoftwareApplication", name, softwareVersion: version, url, applicationCategory: "DeveloperApplication", operatingSystem: "Linux, macOS, Windows" }).replaceAll("<", "\\u003c");
}
