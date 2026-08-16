import type { Page } from "./generated.js";

export interface PageRequest {
  after?: string;
  limit?: number;
  signal?: AbortSignal;
}

export type PageLoader<T> = (request: PageRequest) => Promise<Page<T>>;

export async function* paginate<T>(loader: PageLoader<T>, request: PageRequest = {}): AsyncGenerator<T, void, void> {
  const seen = new Set<string>();
  let after = request.after;
  do {
    request.signal?.throwIfAborted();
    const page = await loader({ ...request, ...(after === undefined ? {} : { after }) });
    for (const item of page.items) yield item;
    const next = page.next ?? undefined;
    if (next !== undefined && seen.has(next)) throw new Error("Pagination cursor cycle detected");
    if (next !== undefined) seen.add(next);
    after = next;
  } while (after !== undefined);
}

export function pageQuery(request: PageRequest): Readonly<Record<string, string | number | undefined>> {
  const limit = request.limit ?? 50;
  if (!Number.isInteger(limit) || limit < 1 || limit > 250) throw new RangeError("Page limit must be in 1..=250");
  if (request.after !== undefined && !/^[A-Za-z0-9_-]{1,512}$/u.test(request.after)) {
    throw new TypeError("Invalid pagination cursor");
  }
  // The service calls its opaque continuation token `cursor`; the SDK uses
  // `after` to make the iterator API read naturally.
  return { cursor: request.after, limit };
}

export function parsePage<T>(value: unknown): Page<T> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) throw new TypeError("Page response must be an object");
  const candidate = value as { items?: unknown; next_cursor?: unknown };
  if (!Array.isArray(candidate.items)) throw new TypeError("Page response must contain items");
  if (candidate.next_cursor !== undefined && candidate.next_cursor !== null && typeof candidate.next_cursor !== "string") {
    throw new TypeError("Page response next_cursor must be a string or null");
  }
  return { items: candidate.items as T[], next: candidate.next_cursor ?? null };
}
