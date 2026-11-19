# synapse

Personal cross-device "brain" bot — one hub that watches all my devices and talks to me through Feishu.

## Architecture

- **Rust watcher client** — runs on every device (laptop / PC / 24h mini PC); watches filesystem & system events and reports them to the hub over a small device protocol.
- **Hub ("brain") on the always-on mini PC** — aggregates device state, runs the decision logic, and dispatches actions back to devices.
- **Feishu loop** — Feishu bot is the human interface: the hub pushes alerts there and takes commands back, closing the sense → decide → act → notify loop.

Day 1 milestone: Rust file-watching module on the mini PC, events visible in Feishu.
