# LawSynth single-node systemd deployment

Runs the full LawSynth service layer on one bare host under systemd -- the
"single-node server" deployment mode from the architecture (a lab or small
company: API + gateway + scheduler + worker + artifact, backed by Postgres, a
local object tree, and NATS). This is the step between local Studio and a
Kubernetes deployment; do not reach for Kubernetes until jobs exceed one
machine.

## Topology

```
  internet ──▶ gateway (0.0.0.0:8081) ──▶ api (127.0.0.1:8080)
                                             │
                 ┌───────────────────────────┼───────────────┐
                 ▼                            ▼               ▼
           scheduler (127.0.0.1:8083)   artifact (127.0.0.1:8082)
                 ▲                            ▲
                 └──────── worker ────────────┘
                    (leases jobs, pushes artifacts)

  backing services (prerequisites): postgresql :5432, nats :4222
```

Only the gateway binds a public interface; every other service binds loopback.

## Units

| Unit | Binds | Notes |
|---|---|---|
| `lawsynth-artifact.service` | `127.0.0.1:8082` | Content-addressed object store; owns `objects/` and `cache/` |
| `lawsynth-scheduler.service` | `127.0.0.1:8083` | Singleton; needs Postgres + NATS |
| `lawsynth-api.service` | `127.0.0.1:8080` | Runs migrations via `ExecStartPre`; needs Postgres |
| `lawsynth-gateway.service` | `0.0.0.0:8081` | Public request-admission boundary |
| `lawsynth-worker.service` | `127.0.0.1:8084` | Discovery execution; generous CPU/memory quotas |
| `lawsynth.target` | — | Starts/stops/enables the whole stack |

## Layout

| Path | Owner | Contents |
|---|---|---|
| `/opt/lawsynth/bin` | root | Rust binaries: `lawsynth-{artifact,scheduler,worker}` |
| `/opt/lawsynth/venv` | root | Python env with `lawsynth_api`, `lawsynth_gateway` |
| `/var/lib/lawsynth` | lawsynth | `objects/`, `cache/`, `work/` state (StateDirectory) |
| `/etc/lawsynth/environment` | root:lawsynth 0640 | Secrets + config (EnvironmentFile) |

## Install

Prerequisites: PostgreSQL and NATS running locally, plus the built binaries and
Python venv staged into `/opt/lawsynth`.

```sh
sudo ./install.sh
sudo $EDITOR /etc/lawsynth/environment   # fill in real secrets
sudo systemctl start lawsynth.target
systemctl status 'lawsynth-*'
```

## Operate

```sh
systemctl restart lawsynth.target        # restart the whole stack (PartOf=)
journalctl -u lawsynth-worker -f         # follow one service's logs
systemctl stop lawsynth-worker           # stop a single service
```

## Uninstall

```sh
sudo ./uninstall.sh            # stop + remove units, keep data and secrets
sudo ./uninstall.sh --purge    # also delete /var/lib, /etc, /opt, and the user
```

## Hardening

Every unit runs as the unprivileged `lawsynth` user with an empty capability
bounding set, `NoNewPrivileges`, `ProtectSystem=strict`, `PrivateTmp`, kernel
and cgroup protections, a `@system-service` syscall allow-list, and address
families restricted to IPv4/IPv6/UNIX. `MemoryDenyWriteExecute` is enabled where
possible but intentionally omitted on the API and worker, which load the native
discovery engine and may allocate executable pages. `/etc/lawsynth/environment`
holds secrets and is mode `0640 root:lawsynth`.
