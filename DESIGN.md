# Trigger policy & Feishu review flow

Watcher events are *signals*, not triggers. A raw file save never reaches
Devin — the hub's job is to turn thousands of events a day into a handful of
session requests.

## Trigger sources

| Source | What reaches the hub |
|--------|----------------------|
| Local  | watcher sees changes → debounced `cargo test`/`cargo build` → non-zero exit code + stderr tail |
| CI     | GitHub `check_run` failure webhook → failed job + log URL |
| Manual | Feishu command → always opens a session, no gate |

## The gate — an auto-trigger must pass all four

1. **Debounce** — events coalesce per path tree over a 30s window; one burst = one signal.
2. **Classify** — only a non-zero build/test exit code, or a panic signature in a
   watched log (`panicked at`, `thread '…' panicked`, `ERROR` + backtrace), counts
   as a failure. Plain saves are noise and die here.
3. **Dedup + cooldown** — `fingerprint = hash(repo, error_kind, top_frame)`. The
   same fingerprint inside the cooldown window (default 6h) attaches to the open
   session instead of spawning a new one. Hard cap on auto-sessions/day.
4. **Repro check** — auto-trigger only if the failure reproduces on the rerun;
   flaky-once failures go to Feishu as a *suggestion card*, never a session.

## Feishu card → state machine

Devin opens a PR; the hub posts an interactive card with
`✅ 合并 / 🔁 重试+附言 / ❌ 驳回` callbacks to the hub webhook.

```
WATCH → DEBOUNCE → CLASSIFY → DEDUP → SESSION → PR_OPEN → REVIEW
                                            ├─ ✅ 合并        → DONE（记录到触发统计）
                                            ├─ 🔁 重试+附言   → SESSION（attempt++，反馈追加到同分支）
                                            └─ ❌ 驳回        → REJECTED：PR 带原因关闭，
                                                                该 fingerprint 永久拉黑，
                                                                同类失败只进人工队列
```

- Retry carries the human's note back into the session on the same branch;
  max 2 auto-retries, then it escalates to a human instead of burning credits.
- Reject is terminal for that fingerprint — a human-vetoed auto-trigger never
  silently re-fires. Re-enabling requires an explicit Feishu command.

> Status: the watcher exists (`watcher/`); the hub and this gate are the next
> milestone — the watcher already speaks the event format the gate consumes.
