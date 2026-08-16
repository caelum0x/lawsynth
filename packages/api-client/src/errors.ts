import type { ProblemDetails } from "./generated.js";

export class ApiError extends Error {
  readonly status: number;
  readonly code: string | undefined;
  readonly requestId: string | undefined;
  readonly problem: ProblemDetails | undefined;
  readonly retryAfterMs: number | undefined;

  constructor(message: string, options: {
    status: number;
    code?: string | undefined;
    requestId?: string | undefined;
    problem?: ProblemDetails | undefined;
    retryAfterMs?: number | undefined;
    cause?: unknown;
  }) {
    super(message, { cause: options.cause });
    this.name = "ApiError";
    this.status = options.status;
    this.code = options.code;
    this.requestId = options.requestId;
    this.problem = options.problem;
    this.retryAfterMs = options.retryAfterMs;
  }

  get isRetryable(): boolean {
    return this.status === 408 || this.status === 425 || this.status === 429 || this.status >= 500;
  }
}

export class NetworkError extends Error {
  constructor(message: string, options?: { cause?: unknown }) {
    super(message, options);
    this.name = "NetworkError";
  }
}

/** The server returned a successful response whose declared JSON was invalid. */
export class ResponseFormatError extends Error {
  constructor(message: string, options?: { cause?: unknown }) {
    super(message, options);
    this.name = "ResponseFormatError";
  }
}

export function retryAfterMilliseconds(value: string | null, now = Date.now()): number | undefined {
  if (!value) return undefined;
  const seconds = Number(value);
  if (Number.isFinite(seconds) && seconds >= 0) return Math.min(seconds * 1_000, 86_400_000);
  const date = Date.parse(value);
  return Number.isFinite(date) ? Math.max(0, Math.min(date - now, 86_400_000)) : undefined;
}

export function apiErrorFromResponse(
  status: number,
  body: unknown,
  headers: Headers,
): ApiError {
  const requestId = headers.get("x-request-id") ?? readRequestId(body);
  const retryAfterMs = retryAfterMilliseconds(headers.get("retry-after"));
  const legacy = isRecord(body) && isRecord(body.error) ? body.error : undefined;
  const problem = isProblemDetails(body) ? body : undefined;
  const code = stringField(legacy, "code") ?? problem?.code;
  const message = stringField(legacy, "message") ?? problem?.detail ?? problem?.title ?? `API request failed with status ${status}`;
  return new ApiError(message, {
    status,
    ...(code === undefined ? {} : { code }),
    ...(requestId === undefined ? {} : { requestId }),
    ...(problem === undefined ? {} : { problem }),
    ...(retryAfterMs === undefined ? {} : { retryAfterMs }),
  });
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function stringField(value: Record<string, unknown> | undefined, field: string): string | undefined {
  const candidate = value?.[field];
  return typeof candidate === "string" ? candidate : undefined;
}

function readRequestId(body: unknown): string | undefined {
  if (!isRecord(body)) return undefined;
  const direct = stringField(body, "request_id");
  if (direct !== undefined) return direct;
  return isRecord(body.error) ? stringField(body.error, "request_id") : undefined;
}

function isProblemDetails(value: unknown): value is ProblemDetails {
  return isRecord(value) && typeof value.title === "string" && typeof value.status === "number";
}
