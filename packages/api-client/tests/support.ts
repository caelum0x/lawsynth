/** A deterministic Fetch peer for testing request construction without a network listener. */
export function createFetchEndpoint(handler) {
  const requests = [];
  return {
    baseUrl: "https://lawsynth.test",
    requests,
    fetch: async (input, init) => {
      const request = new Request(input, init);
      const item = { method: request.method, url: new URL(request.url).pathname + new URL(request.url).search, headers: Object.fromEntries(request.headers), body: await request.text() };
      requests.push(item);
      return handler(item);
    },
  };
}
export function json(status, value, headers = {}) { return new Response(JSON.stringify(value), { status, headers: { "content-type": "application/json", ...headers } }); }
