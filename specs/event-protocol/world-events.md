# World events

The World IR has a runtime marker `lawsynth_world::Event { id, time,
direction }`. Construction rejects non-finite times. `crosses_zero` detects a
bracketed finite-value crossing: rising is `previous < 0 && current >= 0`,
falling is `previous > 0 && current <= 0`, and `Any` is their union. A zero at
the prior endpoint does not retrigger.

These are numerical simulation helpers, not emitted protocol events. They are
not stored in format-0.1 `.lsworld` bundles, do not modify a world law, and do
not provide root localization, event scheduling, subscriptions, or audit
history.
