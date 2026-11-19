# synapse

Personal cross-device "brain" bot — one hub that watches all my devices and talks to me through Feishu.

## Architecture

- **Rust watcher client** — runs on every device (laptop / PC / 24h mini PC); watches filesystem & system events and reports them to the hub over a small device protocol.
- **Hub ("brain") on the always-on mini PC** — aggregates device state, runs the decision logic, and dispatches actions back to devices.
- **Devin Bot as the hands** — when the hub sees an error worth fixing (or I assign a task in Feishu), it opens a Devin cloud session via API; Devin reproduces the issue, writes the fix, and opens a PR in this repo. It also doubles as the automated reviewer on PRs.
- **Feishu loop** — Feishu bot is the human interface: commands go in, notifications and Devin's PR results come back — closing the sense → decide → act → notify loop.

`Feishu 一句话派活 → hub 判定 → Devin session 修 bug / 提 PR → 结果回推飞书确认`

Day 1 milestone: Rust file-watching module on the mini PC, events visible in Feishu.
