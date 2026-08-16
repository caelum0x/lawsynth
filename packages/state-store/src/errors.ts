export class StateStoreError extends Error {
  constructor(message: string, options?: { cause?: unknown }) {
    super(message, options);
    this.name = "StateStoreError";
  }
}

export class InvariantError extends StateStoreError {
  constructor(message: string) {
    super(message);
    this.name = "InvariantError";
  }
}

export class RevisionConflictError extends StateStoreError {
  readonly expected: number;
  readonly actual: number;

  constructor(expected: number, actual: number) {
    super(`Revision conflict: expected ${expected}, received ${actual}`);
    this.name = "RevisionConflictError";
    this.expected = expected;
    this.actual = actual;
  }
}

export class UnknownEventError extends StateStoreError {
  constructor(type: string) {
    super(`Unknown state event: ${type}`);
    this.name = "UnknownEventError";
  }
}

export class PersistenceError extends StateStoreError {
  constructor(message: string, options?: { cause?: unknown }) {
    super(message, options);
    this.name = "PersistenceError";
  }
}
