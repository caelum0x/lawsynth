# `@lawsynth/api-client`

The TypeScript client owns HTTP request construction, response decoding, error
normalization, cursor pagination, idempotency headers, and incremental SSE
parsing. It uses the platform Fetch API and has no runtime dependency beyond
`@lawsynth/world-schema` for World import typing.

```ts
import { BearerTokenAuth, LawSynthClient } from "@lawsynth/api-client";

const client = new LawSynthClient({
  baseUrl: "https://service.example",
  auth: new BearerTokenAuth(() => process.env.LAWSYNTH_TOKEN ?? ""),
});
const project = await client.projects.create({ name: "orbital-model" }, "create-project-0001");
```

The transport retries only replay-safe requests and writes with an explicit
idempotency key. SSE is exposed only by `client.events(runId)`. Endpoint
availability and authentication semantics are determined by the deployed
service; this package does not manufacture a server implementation.

Run `npm test` inside this directory to build the client and exercise it
against local HTTP servers.
