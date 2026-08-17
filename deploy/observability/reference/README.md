# LawSynth observability reference

A ready-to-adapt observability stack for a self-hosted LawSynth deployment:
metrics (Prometheus), logs (Loki via Promtail), traces (Tempo via the OTel
Collector), dashboards, and SLO-driven alerts. Everything here targets the
internal observability signals and SLOs from the production architecture
(section 23) and enforces the telemetry privacy rule from section 22.

> **Privacy first.** Dataset names, column values, equations, and user prompts
> must never appear in telemetry. Both the metrics pipeline (label discipline)
> and the trace/log pipelines (redaction processors in `otel-collector.yaml`
> and `logging.yaml`) drop such fields by construction.

## Contents

| File | Component | Purpose |
|---|---|---|
| `prometheus.yaml` | Prometheus | Scrape config for all services + exporters; loads `alerts.yaml` |
| `alerts.yaml` | Prometheus | Recording + alerting rules mapped to the SLOs |
| `otel-collector.yaml` | OTel Collector | OTLP in → Tempo (traces) + Prometheus (metrics) + Loki (logs), with redaction |
| `logging.yaml` | Promtail | Ship container JSON logs to Loki with trace correlation and redaction |
| `grafana-datasources.yaml` | Grafana | Provision Prometheus + Loki + Tempo with cross-signal linking |
| `api-dashboard.json` | Grafana | API/gateway: availability, latency, rate limiting, cancel-ack |
| `worker-dashboard.json` | Grafana | Runs, queue delay, leases, resources, artifact integrity |
| `science-dashboard.json` | Grafana | Discovery stages, candidates, eval/sim latency, warnings |
| `runbook.md` | — | Response procedures linked from each alert |

## Signals and conventions

- **Metrics namespace:** `lawsynth_*`. HTTP services expose `/metrics` on their
  service port; the scheduler and workers expose it on `:9102`.
- **Exporters expected:** `postgres-exporter:9187`, `nats-exporter:7777`,
  `node-exporter:9100`, `blackbox-exporter:9115`, and MinIO's built-in
  `/minio/v2/metrics/cluster`. Add/remove jobs in `prometheus.yaml` to match.
- **Correlation:** the event envelope carries `trace_id`, `run_id`, and
  `organization_id`; datasources link logs↔traces↔metrics on these.
- **Templating:** dashboards use a `deployment` variable so one Grafana can
  serve multiple environments.

## Wiring it up

1. **Prometheus** — mount `prometheus.yaml` at `/etc/prometheus/prometheus.yml`
   and `alerts.yaml` beside it. Set `LAWSYNTH_DEPLOYMENT` / `LAWSYNTH_DOMAIN`.
2. **OTel Collector** — mount `otel-collector.yaml`; point services'
   `OTEL_EXPORTER_OTLP_ENDPOINT` at `http://otel-collector:4318`.
3. **Promtail** — mount `logging.yaml` and the Docker socket read-only.
4. **Grafana** — place `grafana-datasources.yaml` under
   `/etc/grafana/provisioning/datasources/` and import the three dashboards
   (or provision them from a dashboards provider pointing at this directory).
5. **Alertmanager** — route the alerts; severities are `critical` (page),
   `warning` (ticket), `info` (dashboard-only).

## SLO coverage

| SLO (section 23) | Signal | Alert |
|---|---|---|
| API availability 99.9% | probe + 5xx ratio | `LawSynthApiProbeDown`, `LawSynthApiHighErrorRatio*` |
| Accepted run persistence 99.99% | persist failures | `LawSynthAcceptedRunPersistenceLoss` |
| Artifact checksum integrity 100% | checksum failures | `LawSynthArtifactChecksumFailure` |
| Event ordering monotonic | ordering violations | `LawSynthEventOrderingViolation` |
| Cancellation ack p95 < 2s | cancel-ack histogram | `LawSynthCancelAckSlow` |
| Metadata backup RPO 15m | backup timestamp | `LawSynthBackupRpoBreached` |

## Boundaries

These are reference configurations, not a turnkey stack: they assume the
exporters and the Prometheus/Loki/Tempo/Grafana components are deployed
separately (Compose, Helm, or managed). Tune thresholds and retention to your
traffic and burn-rate policy before relying on the alerts for paging.
