import { renderSeo, softwareStructuredData, type SeoMetadata } from "../src/seo.js";

/** Render document head metadata from page data already validated by the site compiler. */
export function renderDocumentationSeo(metadata: SeoMetadata): string {
  return renderSeo(metadata, "LawSynth");
}

/** JSON-LD remains a string so a static-site host can place it in a script tag safely. */
export function lawSynthStructuredData(version: string): string {
  return softwareStructuredData("LawSynth", version, "https://lawsynth.dev");
}
