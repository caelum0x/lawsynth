import type { AuthProvider } from "./auth.js";
import { ApiError, apiErrorFromResponse, NetworkError } from "./errors.js";

export type HttpMethod = "GET" | "POST" | "PUT" | "PATCH" | "DELETE" | "HEAD";
export type QueryValue = string | number | boolean | null | undefined;
export type ResponseKind = "json" | "text" | "blob" | "arrayBuffer" | "response" | "void";

export interface TransportRequest {
  method?: HttpMethod | undefined;
  path: string;
  query?: Readonly<Record<string, QueryValue | readonly QueryValue[]>> | undefined;
  headers?: Readonly<Record<string, string>> | undefined;
  body?: unknown;
  signal?: AbortSignal | undefined;
  timeoutMs?: number | undefined;
  response?: ResponseKind | undefined;
  idempotencyKey?: string | undefined;
  retry?: boolean | undefined;
}

export interface Transport {
  request<T>(request: TransportRequest): Promise<T>;
}

export interface FetchTransportOptions {
  baseUrl: string | URL;
  auth?: AuthProvider | undefined;
  fetch?: typeof globalThis.fetch | undefined;
  defaultTimeoutMs?: number | undefined;
  maxRetries?: number | undefined;
  userAgent?: string | undefined;
}

export function pathSegment(value: string): string {
  if (!value || value.length > 512 || /[\0\r\n]/u.test(value)) throw new TypeError("Invalid path identifier");
  return encodeURIComponent(value);
}

export class FetchTransport implements Transport {
  readonly #baseUrl: URL;
  readonly #auth: AuthProvider | undefined;
  readonly #fetch: typeof globalThis.fetch;
  readonly #timeoutMs: number;
  readonly #maxRetries: number;
  readonly #userAgent: string | undefined;

  constructor(options: FetchTransportOptions) {
    this.#baseUrl = normalizeBaseUrl(options.baseUrl);
    this.#auth = options.auth;
    this.#fetch = options.fetch ?? globalThis.fetch;
    if (typeof this.#fetch !== "function") throw new Error("No Fetch implementation is available");
    this.#timeoutMs = positiveInteger(options.defaultTimeoutMs ?? 30_000, "defaultTimeoutMs");
    this.#maxRetries = nonNegativeInteger(options.maxRetries ?? 2, "maxRetries");
    this.#userAgent = options.userAgent;
  }

  async request<T>(request: TransportRequest): Promise<T> {
    const method = request.method ?? "GET";
    const replaySafe = method === "GET" || method === "HEAD" || method === "PUT" || method === "DELETE" || request.idempotencyKey !== undefined;
    const attempts = request.retry === false || !replaySafe ? 1 : this.#maxRetries + 1;
    let lastError: unknown;

    for (let attempt = 0; attempt < attempts; attempt += 1) {
      request.signal?.throwIfAborted();
      try {
        const response = await this.#send(request, method);
        if (response.ok) return await decode<T>(response, request.response ?? "json");
        const error = await toApiError(response);
        if (response.status === 401 && attempt === 0) await this.#auth?.invalidate?.();
        if (!error.isRetryable || attempt + 1 >= attempts) throw error;
        lastError = error;
        await wait(error.retryAfterMs ?? retryDelay(attempt), request.signal);
      } catch (error) {
        if (request.signal?.aborted) throw request.signal.reason;
        if (error instanceof ApiError) throw error;
        lastError = error;
        if (attempt + 1 >= attempts) throw new NetworkError(isAbortError(error) ? "Request timed out" : "Network request failed", { cause: error });
        await wait(retryDelay(attempt), request.signal);
      }
    }
    throw new NetworkError("Network request failed", { cause: lastError });
  }

  async #send(request: TransportRequest, method: HttpMethod): Promise<Response> {
    const timeoutMs = positiveInteger(request.timeoutMs ?? this.#timeoutMs, "timeoutMs");
    const timerController = new AbortController();
    const timer = setTimeout(() => timerController.abort(new DOMException("Request timed out", "TimeoutError")), timeoutMs);
    const signal = mergeSignals(request.signal, timerController.signal);
    try {
      const auth = await this.#auth?.headers(signal) ?? {};
      const headers = new Headers({ Accept: "application/json", ...auth, ...request.headers });
      if (request.idempotencyKey !== undefined) {
        if (!/^[A-Za-z0-9._:-]{1,255}$/u.test(request.idempotencyKey)) throw new TypeError("Invalid idempotency key");
        headers.set("Idempotency-Key", request.idempotencyKey);
      }
      if (this.#userAgent && typeof window === "undefined") headers.set("User-Agent", this.#userAgent);
      const body = encodeBody(request.body, headers);
      return await this.#fetch(buildUrl(this.#baseUrl, request.path, request.query), {
        method,
        headers,
        ...(body === undefined ? {} : { body }),
        signal,
        redirect: "follow",
        credentials: "same-origin",
      });
    } finally {
      clearTimeout(timer);
    }
  }
}

function encodeBody(value: unknown, headers: Headers): BodyInit | undefined {
  if (value === undefined) return undefined;
  if (typeof value === "string" || value instanceof Blob || value instanceof FormData || value instanceof URLSearchParams || value instanceof ArrayBuffer || ArrayBuffer.isView(value)) return value as BodyInit;
  headers.set("Content-Type", "application/json");
  return JSON.stringify(value);
}

async function decode<T>(response: Response, kind: ResponseKind): Promise<T> {
  if (kind === "response") return response as T;
  if (kind === "void" || response.status === 204 || response.status === 205) return undefined as T;
  if (kind === "text") return await response.text() as T;
  if (kind === "blob") return await response.blob() as T;
  if (kind === "arrayBuffer") return await response.arrayBuffer() as T;
  const text = await response.text();
  if (!text) return undefined as T;
  try { return JSON.parse(text) as T; }
  catch (cause) {
    const requestId = response.headers.get("x-request-id") ?? undefined;
    throw new ApiError("Server returned invalid JSON", { status: response.status, ...(requestId ? { requestId } : {}), cause });
  }
}

async function toApiError(response: Response): Promise<ApiError> {
  let body: unknown = undefined;
  try {
    const text = await response.text();
    if (text && (response.headers.get("content-type") ?? "").includes("json")) body = JSON.parse(text) as unknown;
    else if (text) body = { title: text.slice(0, 4_096), status: response.status };
  } catch { /* The HTTP status remains authoritative even for malformed error bodies. */ }
  return apiErrorFromResponse(response.status, body, response.headers);
}

function buildUrl(base: URL, path: string, query?: TransportRequest["query"]): URL {
  if (!path.startsWith("/") || path.startsWith("//")) throw new TypeError("API paths must start with one slash");
  const url = new URL(path.slice(1), base);
  for (const [key, raw] of Object.entries(query ?? {})) {
    for (const value of Array.isArray(raw) ? raw : [raw]) if (value !== undefined && value !== null) url.searchParams.append(key, String(value));
  }
  return url;
}

function normalizeBaseUrl(value: string | URL): URL {
  const url = new URL(value);
  if (url.protocol !== "http:" && url.protocol !== "https:") throw new TypeError("baseUrl must use HTTP or HTTPS");
  url.pathname = `${url.pathname.replace(/\/+$/u, "")}/`;
  url.search = ""; url.hash = "";
  return url;
}

function mergeSignals(...values: readonly (AbortSignal | undefined)[]): AbortSignal {
  const signals = values.filter((value): value is AbortSignal => value !== undefined);
  if (typeof AbortSignal.any === "function") return AbortSignal.any(signals);
  const controller = new AbortController();
  for (const signal of signals) {
    if (signal.aborted) { controller.abort(signal.reason); break; }
    signal.addEventListener("abort", () => controller.abort(signal.reason), { once: true });
  }
  return controller.signal;
}

function wait(milliseconds: number, signal?: AbortSignal): Promise<void> {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(resolve, milliseconds);
    signal?.addEventListener("abort", () => { clearTimeout(timer); reject(signal.reason); }, { once: true });
  });
}

function retryDelay(attempt: number): number { const ceiling = Math.min(250 * 2 ** attempt, 5_000); return ceiling / 2 + Math.random() * ceiling / 2; }
function positiveInteger(value: number, name: string): number { if (!Number.isSafeInteger(value) || value <= 0) throw new RangeError(`${name} must be positive`); return value; }
function nonNegativeInteger(value: number, name: string): number { if (!Number.isSafeInteger(value) || value < 0) throw new RangeError(`${name} must be non-negative`); return value; }
function isAbortError(value: unknown): boolean { return typeof value === "object" && value !== null && "name" in value && ["AbortError", "TimeoutError"].includes(String((value as { name: unknown }).name)); }
