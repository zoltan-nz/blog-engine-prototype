# WebSocket Agent Protocol — Design Spec

**Date:** 2026-03-22
**Status:** Draft

---

## Overview

Spec 0003 Decision 2 documents the evolution path from the current HTTP REST interface
(`astro-supervisor`) to a persistent WebSocket channel. This spec defines the protocol,
channel topology, and deployment adaptation. No implementation is scoped — this is the
target design for deployment model B (cloud CMS ↔ remote agent).

Current state: agent is an HTTP server, backend calls it. Future state: agent initiates an
outbound WebSocket connection to the CMS backend; the handler logic is unchanged, only the
dispatch layer changes.

---

## Transport Decision

| Transport       | Verdict    | Reason                                                                                 |
|-----------------|------------|----------------------------------------------------------------------------------------|
| **WebSocket**   | ✓ Use      | Bidirectional, browser-native, no extra infrastructure                                 |
| gRPC-Web        | ✗ Skip     | Client-side streaming not supported in browsers; bidi requires separate channel        |
| NATS            | ✗ Skip     | Requires a running NATS server; adds infrastructure overhead for a two-party system    |
| SSE + HTTP POST | ~ Fallback | SSE for event streaming, POST for commands — fallback when WebSocket unavailable       |
| MQTT            | ✗ Skip     | Designed for IoT device-command patterns; browser support requires MQTT-over-WebSocket |

---

## Shared Protocol Crate

A standalone `admin-protocol` crate holds all shared types. The CMS backend, agent, CLI tool,
and Tauri desktop app depend on it. A new command variant is a compile error in every consumer
until handled.

### Wire Types

```rust
// Adjacently tagged — produces {"type": "BuildSite", "payload": {...}}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum Command {
    CreateSite { name: String, domain: String },
    BuildSite { site_id: Uuid, force: bool },
    StartPreview { site_id: Uuid, port: Option<u16> },
    StopPreview { site_id: Uuid },
    GetStatus { site_id: Uuid },
    Ping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum Event {
    SiteCreated { site_id: Uuid, name: String },
    BuildStarted { build_id: Uuid, site_id: Uuid },
    BuildProgress { build_id: Uuid, phase: String, percent: f32 },
    BuildLog { build_id: Uuid, stream: LogStream, data: String },
    BuildCompleted { build_id: Uuid, duration_ms: u64 },
    BuildFailed { build_id: Uuid, error: String, retryable: bool },
    PreviewReady { site_id: Uuid, url: String, port: u16 },
    PreviewStopped { site_id: Uuid },
    Pong,
    Error { code: ErrorCode, message: String, command_id: Option<Uuid> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope<T> {
    pub id: Uuid,
    pub correlation_id: Option<Uuid>,  // links response to request
    pub idempotency_key: Option<Uuid>, // client-generated; same across retries
    pub sequence: u64,                 // monotonic; used for replay-from detection
    pub timestamp: DateTime<Utc>,
    pub payload: T,
}
```

TypeScript discriminated union on the frontend mirrors the Rust enum exactly — no manual
mapping needed, the wire format IS the discriminant.

---

## Tokio Channel Topology

Four channel types, each chosen for a specific communication pattern:

| Channel          | Pattern      | Role                                                        |
|------------------|--------------|-------------------------------------------------------------|
| `mpsc` (bounded) | N → 1        | Commands from HTTP/WS/CLI → single command processor        |
| `oneshot`        | 1 → 1        | Immediate ack/response per command ("send the sender")      |
| `broadcast`      | 1 → N        | Build events/logs fan-out to all subscribed browser clients |
| `watch`          | latest value | Current agent connection status                             |

The "send the sender" pattern is the key wiring: any transport creates a `oneshot`, bundles
it with the command into a `CommandMessage`, sends it over the shared `mpsc`. The processor
handles it and responds through the `oneshot`. All command processing serialises through a
single task — no concurrency bugs, unlimited concurrent transport connections.

```rust
struct CommandMessage {
    envelope: Envelope<Command>,
    response_tx: oneshot::Sender<Result<Event, CommandError>>,
}
```

Backpressure: bounded `mpsc` blocks senders when full. `broadcast` drops messages for slow
receivers and returns `RecvError::Lagged(n)` — the client requests a replay from the ring
buffer.

---

## Axum Transport Layer

Three routes share identical `Envelope<Command>` / `Envelope<Event>` types:

```
POST /api/commands   — HTTP; CLI tools, API integrations, one-shot commands
GET  /api/ws         — WebSocket; browser UI, Tauri desktop
GET  /api/events     — SSE; fallback log streaming when WebSocket unavailable
```

All three funnel through the same `mpsc` sender. Adding a new transport is a routing change
only — no business logic duplication.

---

## Build Log Streaming

```
Agent process (PTY)
  │ tokio::process stdout/stderr lines
  ▼
Envelope<Event::BuildLog>  ──wss──▶  CMS Backend
                                          │ broadcast::Sender
                                     ┌───┴──────────┐
                                     ▼              ▼
                                 Browser A      Browser B
                                 xterm.js       xterm.js
```

**Agent:** captures stdout/stderr line-by-line via `tokio::process`; wraps each line in
`Event::BuildLog { stream: LogStream::Stdout | Stderr, data }`.

**CMS backend:** fans out via `broadcast` channel; maintains a `VecDeque` ring buffer
(capacity ~1 000 events) for reconnection replay. Client sends
`{"type": "ReplayFrom", "sequence": N}` on reconnect; backend replays buffered messages
before resuming live stream.

**Frontend:** xterm.js with WebGL addon for GPU-accelerated rendering; 10 000-line scrollback.
Processes raw ANSI escape sequences (colours, progress bars) without server-side stripping.

---

## Event Log (Git)

Build events are appended to `.cms/events/builds.jsonl` in each site repo — one JSON object
per line. State is reconstructed by replaying the log; no database required. Git commits act
as the audit trail.

```rust
// Append
writeln!(file, "{}", serde_json::to_string(&envelope)?)?;

// Reconstruct — fold over JSONL, match on Event variants
let history: Vec<BuildRecord> = BufReader::new(File::open(log_path) ? )
.lines()
.filter_map( | l| serde_json::from_str::<Envelope<Event> > ( & l.ok() ? ).ok())
.fold(HashMap::new(), | mut acc, env| { /* update acc per variant */ acc })
.into_values()
.collect();
```

---

## Deployment Adaptation

Protocol code is identical across all models. Only the connection URL and auth differ:

| Model                    | Agent connects to                        | Auth                    |
|--------------------------|------------------------------------------|-------------------------|
| Docker Compose (current) | `ws://cms:8080/api/agent/ws`             | none (internal network) |
| Cloud CMS + remote agent | `wss://cms.example.com/api/agent/ws`     | bearer token            |
| Tauri desktop            | `ws://127.0.0.1:8080/api/agent/ws`       | none (localhost)        |
| CLI tool                 | `POST /api/commands` + SSE `/api/events` | bearer token            |

Agent reconnection loop: exponential backoff, 1 s initial → 60 s cap.

```rust
async fn agent_main_loop(config: AgentConfig) {
    let mut delay = Duration::from_secs(1);
    loop {
        match connect_and_run(&config).await {
            Ok(_) => { delay = Duration::from_secs(1); }
            Err(e) => tracing::error!(error = %e, delay_secs = delay.as_secs(), "reconnecting"),
        }
        tokio::time::sleep(delay).await;
        delay = (delay * 2).min(Duration::from_secs(60));
    }
}
```

---

## Migration from HTTP REST

Ordered steps from current Milestone 1 HTTP interface:

1. Extract `Command` / `Event` / `Envelope` types into `admin-protocol` crate
2. Add `GET /api/agent/ws` endpoint to CMS backend — accepts agent registration, routes
   inbound events to `broadcast` sender
3. Add WebSocket client loop to `astro-supervisor` — connects outbound, receives
   `Envelope<Command>`, dispatches to existing handlers, sends `Envelope<Event>` back
4. Add `POST /api/commands` and `GET /api/events` (SSE) to CMS backend for CLI/browser
5. Update SvelteKit frontend — replace fetch calls with WebSocket client; add xterm.js
   log terminal
6. Switch Docker Compose `astro-server` to agent-initiated model; remove
   `ASTRO_MANAGEMENT_URL` from `backend-rust` env
7. Remove direct HTTP agent calls from `backend-rust` handlers

Handler logic (steps 1–4 of spec 0002 handler implementation) is unchanged throughout —
only the dispatch layer changes, per spec 0003 Decision 2.
