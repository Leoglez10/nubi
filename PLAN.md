# Lab Cloud AWS — Plan

Interactive, game-like desktop course to learn AWS. Every hands-on lab runs 100% locally
against [aws-emulator](https://github.com/aws/aws-emulator); the UI is a native app built
with [Rune](https://github.com/anomalyco/rune).

## Core idea

```
[Rune app] ──aws-sdk-rust──▶ [aws-emulator in Docker :9297] ◀── aws CLI ── [student terminal]
   │
   ├─ Learning path / lessons / games / animations
   └─ Checker: queries emulator state to verify the student completed the lab
```

The student practices with real commands in their own terminal. The app never runs the
lab for them; it only checks the resulting state (e.g. "bucket X exists with versioning
enabled") and unlocks the next step.

## Key constraints (verified 2026-11-14)

- aws-emulator serves HTTPS on `127.0.0.1:9297` with a self-signed certificate, no GUI.
- On macOS/Windows it runs only in Docker; source builds are Linux-only (~100 GB `target/`).
- Embedded mode is Rust + Linux only, so the desktop app talks to the Docker server instead.
- Fidelity varies per service (thousands of open gaps; Lambda, EC2 and S3 have the most).
  Each lab must be validated against the emulator before it ships.
- Telemetry is on by default; the app's setup guide starts it with `--telemetry off`.
- Rune is pre-release: pin a `dev` rev; its API may change.

## Phases

### Phase 0 — Design (current)
- Visual mockup of the main screens to agree on look and flow:
  learning path, lesson, lab + checker, mini-game, emulator status.

### Phase 1 — Proof of concept
- Start aws-emulator in Docker.
- Minimal Rune app from `examples/hello`.
- Background thread calls `ListBuckets` via `aws-sdk-s3` and shows the result.
- Validates: endpoint, self-signed cert trust, background thread → UI update.

### Phase 2 — Lesson engine
- Lessons as data files (TOML): theory, activity, checker rule.
- Progress saved locally (XP, completed lessons).
- Content can be added without touching UI code.

### Phase 3 — Module 1: foundations (highest-fidelity services)
- IAM/STS, S3, DynamoDB, SQS, SNS.

### Phase 4 — Module 2: serverless
- Lambda, API Gateway, EventBridge.
- First verify how faithfully the emulator executes Lambda code.

### Phase 5 — Gamification
- XP, branching path, concept mini-games (drag blocks to build an architecture),
  "what happens inside" animations.

### Phase 6 — Optional web version
- Rune WebGPU target. Blockers to solve: browser trust of the self-signed cert and CORS.

## Fallback

If Rune blocks progress, the same architecture works as a web app (TypeScript + AWS SDK
for JavaScript) with a small local proxy.
