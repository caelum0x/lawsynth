import type { WorldDefinition } from "@lawsynth/world-schema";
import { playgroundEmbedUrl, parseEmbedOptions } from "../src/embed.js";
import { createShareUrl, parseShareUrl } from "../src/share.js";

/** Build an embeddable, read-only link from a world that has already passed schema validation. */
export function embedSharedWorld(base: string | URL, world: WorldDefinition): string {
  const shareUrl = createShareUrl(base, { version: 1, world });
  const shared = new URLSearchParams(new URL(shareUrl).hash.slice(1)).get("world");
  if (shared === null) throw new Error("share encoder did not produce a world payload");
  const embedded = playgroundEmbedUrl(base, { shared, readOnly: true, theme: "midnight" });
  const options = parseEmbedOptions(embedded);
  if (options.shared === undefined || options.readOnly !== true) throw new Error("embed options were not preserved");
  return embedded;
}

/** Recover the schema-bearing share payload before handing it to a playground controller. */
export function sharedWorldFromEmbed(url: string | URL): WorldDefinition | undefined {
  const shared = parseEmbedOptions(url).shared;
  if (shared === undefined) return undefined;
  return parseShareUrl(`https://playground.invalid/#world=${shared}`)?.world;
}
