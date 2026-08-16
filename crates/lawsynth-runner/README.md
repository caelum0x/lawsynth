# lawsynth-runner

`lawsynth-runner` is a synchronous, cooperative execution substrate. It admits
work against CPU/memory/disk capacity, exposes cancellation, records checksummed
monotonic checkpoints, and offers heartbeat freshness tracking. It does not
claim process isolation, scheduling, persistence, or distributed coordination;
deployments must provide those policies around `WorkProcess`.

Run its checks with `cargo test -p lawsynth-runner`.
