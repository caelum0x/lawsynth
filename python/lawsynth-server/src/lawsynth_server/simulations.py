from .errors import ValidationError


def validate_simulation_spec(spec: object) -> dict[str, object]:
    if not isinstance(spec, dict):
        raise ValidationError("simulation must be an object")
    horizon, step = spec.get("horizon"), spec.get("step")
    if isinstance(horizon, bool) or isinstance(step, bool) or not isinstance(horizon, (int, float)) or not isinstance(step, (int, float)) or horizon <= 0 or step <= 0 or step > horizon:
        raise ValidationError("simulation requires positive horizon and step <= horizon")
    start = spec.get("start", 0.0)
    if isinstance(start, bool) or not isinstance(start, (int, float)):
        raise ValidationError("simulation start must be numeric")
    steps = round(float(horizon) / float(step))
    if steps > 1_000_000:
        raise ValidationError("simulation exceeds maximum step count")
    method = spec.get("method", "rk4")
    if method != "rk4":
        raise ValidationError("the native runtime currently supports only the rk4 method")
    result = {"horizon": float(horizon), "step": float(step), "start": float(start), "method": method}
    for field in ("initial", "parameters", "inputs"):
        if field in spec:
            result[field] = spec[field]
    return result
