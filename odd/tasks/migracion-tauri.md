# Migración a Tauri

Objective: replace the Rune UI with Tauri v2 (Rust backend reusing the `nubi` lib) + React/TypeScript/Vite, to get full design freedom and responsiveness.
Branch: `feat/tauri`. Delivery: ask-on-risk (forecast ~3–4k authored lines; chain strategy asked before the first PR). Runners: `cargo test`, `npm --prefix ui test` (Vitest).

## Specs

- S1 — Move the UI to Tauri because Rune limits the design: "la app creo que tiene limitaciones aun en diseño, como que rune aun no tiene todo lo qur quiero, seria mejor tauri?" / "todo lo del diseño, responsividad y todo". Frontend: React + TypeScript + Vite (proposed, accepted with "arranca").
- S2 — Keep the engine: `course`, `grading`, `lesson`, `progress`, `lab`, `checker`, `emulator`, `setup` stay in Rust and are exposed as Tauri commands; behavior (profiles, progress files, lab allow-list, emulator checks) unchanged.
- S3 — Feature parity with the Rune app: intro, guided setup (with the emulator warning), profiles, Ajustes (edit/delete profile, reduced motion), CUENTA menu, Inicio dashboard, Clase, Práctica (choice, command, order, match), Laboratorio with terminal, results and stars. Spanish copy.
- S4 — Responsive at any window size (pending items from `ui-dashboard.md` T14 / `fase-4-fundamentos.md` T3: long breadcrumb truncated, layouts work small and large, live resize).
- S5 — Rune removed once parity holds; README, PLAN and release workflow updated (Tauri bundler).

## Tasks

- [x] T1 · S1, S2 · inline · scaffold: workspace, rune optional, `src-tauri` crate, `ui/` Vite React TS, first command (course) rendered — tests written with the code (no observed RED): 2 Vitest + Rust JSON-shape test; live `tauri dev` window lists both classes
- [ ] T2 · S2 · inline · command surface: profiles, progress, course, setup probe/fix, lab start/run/finish + typed TS client
- [ ] T3 · S3, S4 · inline · theme + app shell (sidebar, top bar, transitions, reduced motion), responsive
- [ ] T4 · S3 · inline · intro, setup, profiles, Ajustes, CUENTA menu
- [ ] T5 · S3 · inline · Inicio + Clase
- [ ] T6 · S3 · inline · Práctica + results
- [ ] T7 · S3 · inline · Laboratorio + terminal
- [ ] T8 · S5 · inline · remove Rune, docs, release workflow

## Log

- L1 (2026-11-16, user): "la app creo que tiene limitaciones aun en diseño, como que rune aun no tiene todo lo qur quiero, seria mejor tauri?" → "todo lo del diseño, responsividad y todo" → "arranca".
- 2026-11-16: probe panic fix committed on main first (47c56d8). Route: all tasks inline (one sequential UI migration, no parallel writer reason).
- T1: root is a workspace (`src-tauri` member); `rune` is optional behind default feature `rune-ui`, so `nubi-desktop` builds without Rune (`cargo tree` shows none). Course types gained `Serialize` (additive). Run the app with `ui/node_modules/.bin/tauri dev` from the repo root (the CLI looks for tauri.conf.json below the cwd). Old Rune app still builds; 82 Rust tests pass.
