export interface AnalyticsEvent {
  readonly name: "page_view" | "search" | "copy_code" | "outbound_link";
  readonly path: string;
  readonly properties?: Readonly<Record<string, string | number | boolean>>;
}

export interface AnalyticsSink { send(event: AnalyticsEvent): void | Promise<void>; }

const SENSITIVE_PROPERTY = /email|token|name|query|address|authorization/iu;

export class PrivacyAnalytics {
  #enabled: boolean;
  #queue: AnalyticsEvent[] = [];

  constructor(readonly sink: AnalyticsSink, enabled = false, readonly maximumQueue = 100) {
    if (!Number.isSafeInteger(maximumQueue) || maximumQueue < 1 || maximumQueue > 10_000) {
      throw new RangeError("analytics maximumQueue must be in 1..10000");
    }
    this.#enabled = enabled;
  }

  setEnabled(enabled: boolean): void {
    this.#enabled = enabled;
    if (!enabled) this.#queue = [];
  }

  track(event: AnalyticsEvent): void {
    if (!this.#enabled) return;
    if (!event.path.startsWith("/") || /[?#]/u.test(event.path)) {
      throw new RangeError("analytics paths must exclude query strings and fragments");
    }
    const properties = event.properties === undefined ? undefined : Object.freeze(
      Object.fromEntries(Object.entries(event.properties).filter(([key]) => !SENSITIVE_PROPERTY.test(key))),
    );
    this.#queue.push(Object.freeze({ ...event, ...(properties === undefined ? {} : { properties }) }));
    if (this.#queue.length >= this.maximumQueue) void this.flush();
  }

  async flush(): Promise<void> {
    const pending = this.#queue.splice(0);
    for (let index = 0; index < pending.length; index += 1) {
      const event = pending[index]!;
      try { await this.sink.send(event); }
      catch (error) {
        this.#queue.unshift(...pending.slice(index));
        throw error;
      }
    }
  }
}
