# Lab Cloud AWS — Plan

Game-like desktop app to learn AWS: short lessons, a horizontal learning map in the
clouds, levels and mastery ("perfection") instead of punishment. Every hands-on lab runs 100% locally
against [aws-emulator](https://github.com/aws/aws-emulator), inside a terminal built into
the app. The UI is a native app built with [Rune](https://github.com/anomalyco/rune).

## Product decisions

> **2026-11-14 update:** the UI moved from a game look (cloud map, chunky buttons, cloud
> curtain) to a professional dashboard: sidebar navigation, Inicio with progress cards,
> flat controls and a short content-only slide between tabs. Nubi stays. The "horizontal
> map" and "cloud transitions" items below are superseded; see `odd/tasks/ui-dashboard.md`.

- **Language: Spanish first.** All UI copy and course content are written in Spanish.
  A language switcher (e.g. English) is a later decision, not part of v1. Keep copy in
  data files rather than hard-coded strings so a second language can be added later.
- **Game-first, not docs-first.** Bite-sized exercises (one question per screen),
  a mascot (Nubi), celebration screens. Theory is shown through animations and
  interactive diagrams, not long text.
- **Guided classes before practice.** Each class follows: a short visual explanation,
  a worked example, games/exercises, a hands-on lab, then review. Use numbered classes
  (e.g. "Clase 1: buckets y objetos"); students can revisit the explanation while
  practicing instead of starting with an unexplained question.
- **Tactile animated buttons.** Buttons smoothly depress on hover and press, with
  depth/shadow feedback, visible keyboard focus and equivalent keyboard activation.
- **Cloud transitions.** Smooth screen changes: moving clouds cover the old screen
  and reveal the next, inspired by flying through clouds. Offer reduced motion.
- **No lives, no streaks, no XP.** Unlimited attempts, never a penalty for mistakes.
- **Mastery instead of points.** Every lesson, lab and challenge records its best result
  as stars: 1 = completed, 2 = completed with few errors, 3 = perfect (zero errors).
  Mistakes are listed back to the student as learning, and any node can be replayed to
  perfect it. The student's **level** comes from total stars.
- **Horizontal map in the clouds.** The home screen is a horizontally scrolling sky map:
  units are cloud islands, nodes are cloud platforms joined by a flight path. It must not
  look like Duolingo's vertical path.
- **Animated intro.** A short launch animation (Nubi rises, logo, "turning on your local
  cloud" checks for Docker and the emulator), then the profile screen.
- **Guided setup before playing.** If Docker, the AWS CLI or the emulator image is missing,
  or the emulator is not running, the app shows a setup screen first: what is missing, why
  it is needed, and one-click buttons ("Instalar Docker", "Instalar AWS CLI",
  "Iniciar emulador"; via Homebrew/winget/package manager or the official download page).
  The same requirements and steps are documented on the website.
- **Local profiles, no accounts.** "Who is learning today?" picker with name + avatar,
  no password. Everything is saved on the student's machine.
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

### Local persistence: approach

- One folder per profile in the OS app-data directory
  (macOS: `~/Library/Application Support/LabCloud/profiles/<id>/`).
- `profile.json` (name, avatar) and `progress.json` (best stars, errors per node,
  unlocked nodes), written with `serde_json`. Writes go to a temp file and are then
  renamed over the old one, so a crash never corrupts progress.
- Export / import a profile as one backup file to move it to another machine.
- Optional local PIN per profile later; not needed for v1 (it is the student's own machine).
- Emulator resources are not progress: each lab resets the resources it needs on start.

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
- Visual mockup in Spanish, cloud palette, game-like: intro animation, local profile
  picker, horizontal sky map, exercise screen, lab with integrated terminal, mini-game,
  result screen with mastery stars.

### Phase 1 — Proof of concept (done 2026-11-14, branch `feat/poc`)
Findings:
- `scripts/emulator.sh` builds the image from the official Dockerfile (aarch64 on Apple
  silicon) and starts it with `--telemetry off`, bound to 127.0.0.1 only.
- The emulator requires SigV4 and writes new credentials, `config` and `ca.pem` to
  `~/.aws/emulator/9297/` on every start; the app points itself at those files.
- The Rust SDK trusts the emulator through `SSL_CERT_FILE=ca.pem`; the CLI through `AWS_CA_BUNDLE`.
- Apps that depend on Rune must repeat Rune's `[patch.crates-io]` table at the same rev.
- Rune's headless `--script` runs on simulated time, so it does not wait for a real `aws`
  process; the emulator round trip is covered by an `#[ignore]` integration test instead.

Scope:
- Start aws-emulator in Docker.
- Minimal Rune app from `examples/hello`.
- Background thread calls `ListBuckets` via `aws-sdk-s3` and shows the result.
- Terminal spike: input line → spawn `aws s3 ls` → show output in the app.
- Validates: endpoint, self-signed cert trust, child process + background thread → UI update.

### Phase 2 — Game engine and lesson format (done 2026-11-14, branch `feat/fase-2`)
Engine in `src/{course,grading,progress,checker,emulator}.rs`; tracking in
`odd/tasks/fase-2-motor.md`. Lab checks cover S3 so far; more check types come with their labs.

Scope:
- Lessons as data files (TOML), in Spanish: exercises (multiple choice, fill the command,
  order the steps, match concepts), lab missions with checker rules.
- Local profiles and progress saved on disk (see "Local persistence"): best stars and
  errors per node, unlocked nodes, level.
- Content can be added without touching UI code.

### Phase 3 — Integrated terminal v1 (done 2026-11-14, branch `feat/fase-3`)
Shipped with the guided setup screen and the lab screen in the new design; tracking in
`odd/tasks/fase-3-terminal.md`. Until the map and profile screens exist, the app opens the
first lab with the first local profile.

Scope:
- Command runner widget, allow-list, history, colored output, hints.
- Live objective ticking from checker results.

### UI and guided classes — current priority, before more Phase 4 services
- Add tactile hover/press animations to buttons and smooth cloud screen transitions.
- Offer reduced motion without removing navigation or feedback.
- Reorganize the current content as numbered classes: visual explanation → worked
  example → games/practice → lab → review. Keep explanations available for reference.
- Validate motion and navigation visually, plus deterministic tests for class flow.
- Status: done 2026-11-14 as the professional dashboard (`odd/tasks/ui-dashboard.md`),
  except responsiveness, which moves to the start of Phase 4 below.

### Phase 4 — Module 1: foundations (highest-fidelity services)
- **Before new services, close what the UI work left open:**
  - Responsiveness: scroll bars follow the macOS preference (the app cannot force them);
    the top-bar breadcrumb is not truncated when long; layouts were only checked at
    960×680 and 1180×760; no check by resizing a live window.
  - An independent review of the profile delete path (it removes stored data).
  - A live `cargo run` pass over hover, the tab slide-in, keyboard focus and reduced motion.
  - ~~An app logo~~ done: Nubi on a blue tile (`assets/icon.svg`), set as the Dock and
    window icon; reuse it for the installers (`.icns`, `.ico`) in Phase 8.
- IAM/STS, S3, DynamoDB, SQS, SNS. Suggested next service: DynamoDB.
- Started on `feat/fase-4-fundamentos`: S3 object keys/prefixes lesson and a
  two-file archive lab. Tracking: `odd/tasks/fase-4-fundamentos.md`.
- Remaining services need their own checker types and emulator validation; this
  opening slice does not complete Phase 4.

### Phase 5 — Module 2: serverless
- Lambda, API Gateway, EventBridge.
- First verify how faithfully the emulator executes Lambda code.

### Phase 6 — More game
- Mini-games (architecture builder, command puzzles), achievements for perfect units,
  "what happens inside" animations.

### Phase 7 — Website (GitHub Pages)
- Landing page for the project, published with GitHub Pages.
- Design it with the agent skill from https://github.com/Leonxlnx/taste-skill (MIT; art
  direction, reference images and a browser check). Install the skill at the start of
  this phase, after reviewing its contents.
- Explicit download section per OS (.dmg, .exe, Linux packages), Homebrew install
  command, notices (requirements: Docker + AWS CLI, unsigned-build warnings until code
  signing exists, telemetry note for the emulator) and release notes.

### Phase 8 — Installers and updates (GitHub Actions)
- Release workflow on tag: build and attach macOS `.dmg`, Windows `.exe` installer and
  Linux packages (AppImage / .deb / .rpm) to a GitHub Release.
- Homebrew: a tap with a cask (macOS) / formula (Linux) updated by the release workflow.
- Automatic updates: the app checks the latest GitHub Release and offers to update.
- Code signing / notarization (Apple, Windows) to avoid OS warnings; needs certificates.

### Optional (future) — MCP integration
- An MCP server that exposes the game (progress, current mission, hints, checker results)
  so an AI assistant can act as a tutor. Decide later whether it adds real value.

### Later
- Language switcher (English).
- Web version via Rune's WebGPU target. Blockers: browser trust of the self-signed cert,
  CORS, and the terminal (no child processes in a browser).

## Fallback

If Rune blocks progress, the same architecture works as a web/desktop app
(TypeScript + AWS SDK for JavaScript, xterm.js for the terminal) with a small local proxy.
