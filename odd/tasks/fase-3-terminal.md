# Phase 3 — Guided setup, integrated terminal v1 and the lab screen
Objective: first real screens in the new design: a setup screen that detects and fixes what is missing, and a lab screen with the in-app terminal, live objectives and mastery stars saved to the local profile.
Branch: `feat/fase-3` (from `feat/fase-2`) · Delivery: ask-on-risk · Runner: `cargo test` (+ `-- --include-ignored` with the emulator running)

## Specs

- S1 (user, L2): "Que tenga un botón rápido in the application de initiar esto, instalar esto. No lo tienes installado, instala esto, como previo top" → PLAN.md: "If Docker, the AWS CLI or the emulator image is missing, or the emulator is not running, the app shows a setup screen first: what is missing, why it is needed, and one-click buttons (\"Instalar Docker\", \"Instalar AWS CLI\", \"Iniciar emulador\"; via Homebrew/winget/package manager or the official download page)."
- S2 (PLAN.md, Integrated terminal v1): "An input line plus a scrollback of output, drawn with Rune. Each command is run as a child process (`aws ...`) with `AWS_ENDPOINT_URL` and the emulator CA bundle set. Only `aws` commands are allowed (plus a few built-ins like `clear`, `help`, `hint`), so it is not a general shell. Command history, colored output, autocomplete for the lab's commands."
- S3 (PLAN.md, Phase 3): "Live objective ticking from checker results."
- S4 (PLAN.md, product decisions): "Unlimited attempts, never a penalty for mistakes." / "Mistakes are listed back to the student as learning, and any node can be replayed to perfect it." / "Emulator resources are not progress: each lab resets the resources it needs on start."
- S5 (PLAN.md): "Look: cloud palette. Light theme, white and blue with an orange accent for rewards." All UI copy in Spanish.
- Assumption: "autocomplete" in v1 = suggestion chips for the next objective's command (as in the approved design), not Tab completion.
- Assumption: a lab error = an `aws` command that exits non-zero. Built-ins and rejected input are not errors.
- Assumption: until the profile and map screens exist (next phase), the app uses the first local profile (creating "Estudiante" if none) and opens the first lab.
- Safety: commands run without a shell, in a per-lab sandbox folder (so `aws s3 cp` reads/writes only there), and `--endpoint-url` / `--profile` are rejected so commands cannot leave the emulator.

## Tasks

- [x] T1 · S1 · inline · setup: detect Docker / daemon / AWS CLI / emulator, platform actions, start emulator from the app — b25ceab (RED 3 failing → GREEN; + ignored emulator start test passing)
- [x] T2 · S2, S4 · inline · lab session logic: command parsing + allow-list, built-ins, history, errors, hints, completion, lab sandbox and resource reset — fa0a570 (RED 7 lab + 1 reset failing → GREEN; file-name validation written with its test, no separate RED)
- [x] T3 · S1, S2, S3, S5 · inline · UI: light theme, setup screen, lab screen with terminal, live objectives, result + Perfeccionar, progress saved — d9d64c5 (4 headless UI tests; snapshots of setup, lab and result reviewed)

Route evidence: one app, units share the worker/job channel and state; no disjoint parallel units → inline.

## Log

- L1 (2026-11-14, user): "sigue" (after the proposal: "Fase 3: la terminal integrada y las primeras pantallas reales con el diseño, empezando por la pantalla de preparación")
- T1 done: src/setup.rs (needs, per-OS actions), src/emulator.rs (start/build from embedded Dockerfile, run_args 127.0.0.1-only, healthy, PATH gets /opt/homebrew/bin and /usr/local/bin for Finder launches), tests/emulator.rs.
- T2 done: src/lab.rs (parse allow-list + forbidden --endpoint-url/--profile, Session: built-ins, history, errors+mistakes, objective ticks, sandbox), course LabFile starter files, checker::reset (deletes all versions + bucket). 30 lib + 3 checker + 1 emulator + 2 bin passing.
- T3 done: src/view.rs (palette, setup + lab + result panel), src/main.rs (State, Worker jobs Probe/Fix/StartLab/Run/Finish, scenes setup|lab|result). Close: cargo test → 30 lib + 4 bin; with --include-ignored + 3 checker + 1 emulator passing; clippy clean. Not verified: the live window flow end to end (Rune headless does not run the worker); needs a manual cargo run.
