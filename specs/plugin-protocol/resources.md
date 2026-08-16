# Resources

ResourceLimits contains positive max_cpu_millis, max_memory_bytes,
max_output_bytes, and max_requests. Defaults are 30 seconds, 256 MiB, 16 MiB,
and 1,000 requests. Validation caps memory at 8 GiB and requires output not
exceed memory.

The API validates declared limits and compares a requested limit set against a
host set. It does not meter CPU, terminate execution, count requests, or enforce
memory; those are mandatory host responsibilities. Simulation plugin response
validation separately checks estimated output size against max_output_bytes.
