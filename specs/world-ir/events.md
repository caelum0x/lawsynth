# Event markers

`Event { id, time, direction }` is a runtime marker. Construction returns no event for a non-finite time. `EventDirection` is `Any`, `Rising`, or `Falling`.

`crosses_zero(previous, current, direction)` requires two finite values. Rising accepts `previous < 0 && current >= 0`; falling accepts `previous > 0 && current <= 0`; any is their union. A zero at the previous endpoint does not retrigger, avoiding duplicate interval marks. The function detects bracket crossings only; it does not localize roots or alter laws. Events are not encoded in a 0.1 `.lsworld` bundle.
