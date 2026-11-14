# Lab Cloud AWS — Plan

Game-like desktop app to learn AWS, in the spirit of Duolingo: short lessons, a visual
learning path, XP, streaks, lives and rewards. Every hands-on lab runs 100% locally
against [aws-emulator](https://github.com/aws/aws-emulator), inside a terminal built into
the app. The UI is a native app built with [Rune](https://github.com/anomalyco/rune).

## Product decisions

- **Language: Spanish first.** All UI copy and course content are written in Spanish.
  A language switcher (e.g. English) is a later decision, not part of v1. Keep copy in
  data files rather than hard-coded strings so a second language can be added later.
- **Game-first, not docs-first.** Bite-sized exercises (one question per screen),
  a winding learning path, a mascot, XP, streak, lives, chests and celebration screens.
  Theory is shown through animations and interactive diagrams, not long text.
- **Integrated terminal.** The student practices in a terminal that lives inside the app,
  next to the mission objectives, which tick off live as commands succeed.
- **Look: cloud palette.** Light theme, white and blue with an orange accent for rewards.

## Core idea

```
┌──────────────────────── Rune app ────────────────────────┐
│ Path · lessons · games · animations · rewards            │
│                                                          │
│ Integrated terminal ──spawns──▶ aws CLI ──┐              │
│ Checker ──aws-sdk-rust────────────────────┼──▶ aws-emulator (Docker :9297)
└──────────────────────────────────────────────────────────┘
```

- The terminal runs real `aws` commands with the environment pointed at the emulator,
  and shows their real output.
- The checker never runs the lab for the student; it reads emulator state through the
  SDK (e.g. "bucket X exists with versioning enabled") and ticks objectives / unlocks
  the next node.

### Integrated terminal: approach

1. **v1 — command runner (own widget).** An input line plus a scrollback of output, drawn
   with Rune. Each command is run as a child process (`aws ...`) with `AWS_ENDPOINT_URL`
   and the emulator CA bundle set. Only `aws` commands are allowed (plus a few built-ins
   like `clear`, `help`, `hint`), so it is not a general shell. Command history, colored
   output, autocomplete for the lab's commands.
2. **v2 — full terminal (optional).** A real PTY (e.g. `portable-pty` + a VT parser) if
   labs need a real shell (files, pipes, `jq`). Rune has no terminal widget, so this is
   custom rendering work; only do it if v1 falls short.

## Key constraints (verified 2026-11-14)

- aws-emulator serves HTTPS on `127.0.0.1:9297` with a self-signed certificate, no GUI.
- On macOS/Windows it runs only in Docker; source builds are Linux-only (~100 GB `target/`).
- Embedded mode is Rust + Linux only, so the desktop app talks to the Docker server instead.
- Fidelity varies per service (thousands of open gaps; Lambda, EC2 and S3 have the most).
  Each lab must be validated against the emulator before it ships.
- Telemetry is on by default; the app starts the emulator with `--telemetry off`.
- Rune is pre-release: pin a `dev` rev; its API may change. It has no terminal widget.
- The student needs Docker and AWS CLI v2 installed; the app checks both on first run.

## Phases

### Phase 0 — Design (current)
- Visual mockup in Spanish, cloud palette, game-like:
  learning path, exercise screen, lab with integrated terminal, mini-game, reward screen.

### Phase 1 — Proof of concept
- Start aws-emulator in Docker.
- Minimal Rune app from `examples/hello`.
- Background thread calls `ListBuckets` via `aws-sdk-s3` and shows the result.
- Terminal spike: input line → spawn `aws s3 ls` → show output in the app.
- Validates: endpoint, self-signed cert trust, child process + background thread → UI update.

### Phase 2 — Game engine and lesson format
- Lessons as data files (TOML), in Spanish: exercises (multiple choice, fill the command,
  order the steps, match concepts), lab missions with checker rules.
- Progress saved locally: XP, streak, lives, completed nodes.
- Content can be added without touching UI code.

### Phase 3 — Integrated terminal v1
- Command runner widget, allow-list, history, colored output, hints.
- Live objective ticking from checker results.

### Phase 4 — Module 1: foundations (highest-fidelity services)
- IAM/STS, S3, DynamoDB, SQS, SNS.

### Phase 5 — Module 2: serverless
- Lambda, API Gateway, EventBridge.
- First verify how faithfully the emulator executes Lambda code.

### Phase 6 — More game
- Mini-games (architecture builder, command speed-run), chests, leagues/achievements,
  "what happens inside" animations.

### Later
- Language switcher (English).
- Web version via Rune's WebGPU target. Blockers: browser trust of the self-signed cert,
  CORS, and the terminal (no child processes in a browser).

## Fallback

If Rune blocks progress, the same architecture works as a web/desktop app
(TypeScript + AWS SDK for JavaScript, xterm.js for the terminal) with a small local proxy.
