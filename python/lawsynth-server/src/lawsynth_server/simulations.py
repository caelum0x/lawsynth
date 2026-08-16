from .errors import ValidationError


def validate_simulation_spec(spec: dict[str, object]) -> dict[str, object]:
    horizon, step = spec.get("horizon"), spec.get("step")
    if not isinstance(horizon, (int, float)) or not isinstance(step, (int, float)) or horizon <= 0 or step <= 0 or step > horizon:
        raise ValidationError("simulation requires positive horizon and step <= horizon")
    steps = round(float(horizon) / float(step))
    if steps > 1_000_000:
        raise ValidationError("simulation exceeds maximum step count")
    return {"horizon": float(horizon), "step": float(step), "method": spec.get("method", "rk4")}
