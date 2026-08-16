export interface Redirect { readonly from: string; readonly to: string; readonly permanent: boolean; }

function assertPath(value: string): void {
  if (!value.startsWith("/") || value.includes("..") || /[\r\n\0]/u.test(value)) {
    throw new RangeError(`unsafe redirect path: ${value}`);
  }
}

export function validateRedirects(values: readonly Redirect[]): readonly Redirect[] {
  const from = new Set<string>();
  const targets = new Map(values.map((value) => [value.from, value.to]));
  for (const value of values) {
    assertPath(value.from);
    assertPath(value.to);
    if (value.from === value.to || from.has(value.from)) throw new RangeError(`invalid or duplicate redirect: ${value.from}`);
    from.add(value.from);

    let current = value.to;
    const seen = new Set([value.from]);
    while (targets.has(current)) {
      if (seen.has(current)) throw new RangeError(`redirect cycle involving ${current}`);
      seen.add(current);
      current = targets.get(current)!;
    }
  }
  return Object.freeze(values.map((value) => Object.freeze(value)));
}

export function resolveRedirect(values: readonly Redirect[], input: string): Redirect | undefined {
  return validateRedirects(values).find((value) => value.from === input);
}
