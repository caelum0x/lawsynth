export interface ApiParameter {
  readonly name: string; readonly location: string; readonly required: boolean;
  readonly description?: string; readonly schemaType?: string;
}
export interface ApiOperation {
  readonly id: string; readonly method: string; readonly path: string; readonly summary: string;
  readonly description?: string; readonly tags: readonly string[];
  readonly parameters: readonly ApiParameter[]; readonly responseCodes: readonly string[];
}

function record(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function parametersFor(value: unknown): readonly ApiParameter[] {
  if (!Array.isArray(value)) return [];
  return Object.freeze(value.flatMap((candidate): ApiParameter[] => {
    if (!record(candidate) || typeof candidate.name !== "string" || typeof candidate.in !== "string") return [];
    const schema = record(candidate.schema) ? candidate.schema : {};
    return [{
      name: candidate.name,
      location: candidate.in,
      required: candidate.required === true,
      ...(typeof candidate.description === "string" ? { description: candidate.description } : {}),
      ...(typeof schema.type === "string" ? { schemaType: schema.type } : {}),
    }];
  }));
}

export function extractApiOperations(document: unknown): readonly ApiOperation[] {
  if (!record(document) || !record(document.paths)) throw new TypeError("OpenAPI document must contain paths");
  const operations: ApiOperation[] = [];
  for (const [path, item] of Object.entries(document.paths)) {
    if (!record(item)) continue;
    for (const method of ["get", "post", "put", "patch", "delete"] as const) {
      const operation = item[method];
      if (!record(operation)) continue;
      const generatedId = path.replace(/[^a-z0-9]+/giu, "-").replace(/^-+|-+$/gu, "");
      const id = typeof operation.operationId === "string" ? operation.operationId : `${method}-${generatedId}`;
      operations.push(Object.freeze({
        id,
        method: method.toUpperCase(),
        path,
        summary: typeof operation.summary === "string" ? operation.summary : id,
        ...(typeof operation.description === "string" ? { description: operation.description } : {}),
        tags: Object.freeze(Array.isArray(operation.tags) ? operation.tags.filter((tag): tag is string => typeof tag === "string") : []),
        parameters: parametersFor(operation.parameters),
        responseCodes: Object.freeze(record(operation.responses) ? Object.keys(operation.responses).sort() : []),
      }));
    }
  }
  return Object.freeze(operations.sort((left, right) => left.path.localeCompare(right.path) || left.method.localeCompare(right.method)));
}

export function groupApiOperations(operations: readonly ApiOperation[]): ReadonlyMap<string, readonly ApiOperation[]> {
  const groups = new Map<string, ApiOperation[]>();
  for (const operation of operations) {
    for (const tag of operation.tags.length > 0 ? operation.tags : ["Other"]) {
      groups.set(tag, [...(groups.get(tag) ?? []), operation]);
    }
  }
  return groups;
}
