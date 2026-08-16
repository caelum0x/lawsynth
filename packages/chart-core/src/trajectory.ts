/** A sampled state vector at a simulation time. */
export interface TrajectorySample {
  readonly time: number;
  readonly values: readonly number[];
}

/** Immutable chart-facing representation of a numerical trajectory. */
export interface Trajectory {
  readonly variables: readonly string[];
  readonly samples: readonly TrajectorySample[];
  readonly metadata?: Readonly<Record<string, string | number | boolean>>;
}

export interface TrajectoryInput {
  readonly variables: readonly string[];
  readonly times: readonly number[];
  readonly values: readonly (readonly number[])[];
  readonly metadata?: Readonly<Record<string, string | number | boolean>>;
}

function finite(value: number, label: string): void {
  if (!Number.isFinite(value)) throw new RangeError(`${label} must be finite`);
}

/**
 * Validates and copies a trajectory. Times must be monotonic; equal timestamps
 * are retained because discontinuities can legitimately be sampled at one time.
 */
export function normalizeTrajectory(input: TrajectoryInput): Trajectory {
  if (input.variables.length === 0) throw new RangeError("a trajectory needs at least one variable");
  if (input.times.length !== input.values.length) throw new RangeError("times and values must have equal length");
  const seen = new Set<string>();
  const variables = input.variables.map((name, i) => {
    const clean = name.trim();
    if (!clean) throw new TypeError(`variable ${i} is empty`);
    if (seen.has(clean)) throw new RangeError(`duplicate variable: ${clean}`);
    seen.add(clean);
    return clean;
  });
  let previous = Number.NEGATIVE_INFINITY;
  const samples = input.times.map((time, row) => {
    finite(time, `time ${row}`);
    if (time < previous) throw new RangeError("trajectory times must be monotonic");
    previous = time;
    const values = input.values[row];
    if (values === undefined || values.length !== variables.length) {
      throw new RangeError(`row ${row} must contain ${variables.length} values`);
    }
    return { time, values: values.map((value, column) => {
      finite(value, `value at row ${row}, column ${column}`);
      return value;
    }) };
  });
  return { variables, samples, ...(input.metadata === undefined ? {} : { metadata: { ...input.metadata } }) };
}

/** Selects and validates a single named component without allocating all series. */
export function trajectoryComponent(trajectory: Trajectory, variable: string): readonly [number, number][] {
  const index = trajectory.variables.indexOf(variable);
  if (index < 0) throw new RangeError(`unknown variable: ${variable}`);
  return trajectory.samples.map((sample) => [sample.time, sample.values[index]!] as [number, number]);
}

/** Returns a structural copy suitable for transfer across a worker boundary. */
export function cloneTrajectory(trajectory: Trajectory): Trajectory {
  return normalizeTrajectory({
    variables: trajectory.variables,
    times: trajectory.samples.map((sample) => sample.time),
    values: trajectory.samples.map((sample) => sample.values),
    ...(trajectory.metadata === undefined ? {} : { metadata: trajectory.metadata }),
  });
}
