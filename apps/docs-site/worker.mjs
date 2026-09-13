const CANONICAL_HOST = "lawsynth.dev";
const WWW_HOST = `www.${CANONICAL_HOST}`;

/**
 * Return the canonical URL for the one duplicate host served by this Worker.
 * Paths and query strings remain untouched; URL fragments never reach a server.
 */
export function canonicalRedirectUrl(requestUrl) {
  const url = new URL(requestUrl);
  if (url.hostname !== WWW_HOST) return null;

  url.protocol = "https:";
  url.hostname = CANONICAL_HOST;
  url.port = "";
  return url.toString();
}

export default {
  async fetch(request, env) {
    const canonicalUrl = canonicalRedirectUrl(request.url);
    if (canonicalUrl) return Response.redirect(canonicalUrl, 301);
    return env.ASSETS.fetch(request);
  },
};
