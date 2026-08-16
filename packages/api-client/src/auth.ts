export type AuthHeaders = Readonly<Record<string, string>>;

export interface AuthProvider {
  headers(signal?: AbortSignal): Promise<AuthHeaders> | AuthHeaders;
  invalidate?(): Promise<void> | void;
}

export class BearerTokenAuth implements AuthProvider {
  #token: string | (() => Promise<string> | string);

  constructor(token: string | (() => Promise<string> | string)) {
    this.#token = token;
  }

  async headers(): Promise<AuthHeaders> {
    const token = typeof this.#token === "function" ? await this.#token() : this.#token;
    if (!token || /[\r\n]/u.test(token)) throw new TypeError("Bearer token is empty or contains a newline");
    return { Authorization: `Bearer ${token}` };
  }

  setToken(token: string | (() => Promise<string> | string)): void {
    this.#token = token;
  }
}

export class ApiKeyAuth implements AuthProvider {
  constructor(
    private readonly key: string,
    private readonly header = "X-API-Key",
  ) {
    if (!key || /[\r\n]/u.test(key) || !/^[A-Za-z0-9-]+$/u.test(header)) {
      throw new TypeError("Invalid API key or header name");
    }
  }

  headers(): AuthHeaders {
    return { [this.header]: this.key };
  }
}
