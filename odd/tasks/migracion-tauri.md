# Migración a Tauri

Objective: replace the Rune UI with Tauri v2 (Rust backend reusing the `nubi` lib) + React/TypeScript/Vite, to get full design freedom and responsiveness.
Branch: `feat/tauri`. Delivery: single-pr (user chose "3", 2026-11-16). Runners: `cargo test`, `npm --prefix ui test` (Vitest).

## Specs

- S1 — Move the UI to Tauri because Rune limits the design: "la app creo que tiene limitaciones aun en diseño, como que rune aun no tiene todo lo qur quiero, seria mejor tauri?" / "todo lo del diseño, responsividad y todo". Frontend: React + TypeScript + Vite (proposed, accepted with "arranca").
- S2 — Keep the engine: `course`, `grading`, `lesson`, `progress`, `lab`, `checker`, `emulator`, `setup` stay in Rust and are exposed as Tauri commands; behavior (profiles, progress files, lab allow-list, emulator checks) unchanged.
- S3 — Feature parity with the Rune app: intro, guided setup (with the emulator warning), profiles, Ajustes (edit/delete profile, reduced motion), CUENTA menu, Inicio dashboard, Clase, Práctica (choice, command, order, match), Laboratorio with terminal, results and stars. Spanish copy.
- S4 — Responsive at any window size (pending items from `ui-dashboard.md` T14 / `fase-4-fundamentos.md` T3: long breadcrumb truncated, layouts work small and large, live resize).
- S5 — Rune removed once parity holds; README, PLAN and release workflow updated (Tauri bundler).

## Tasks

- [x] T1 · S1, S2 · inline · scaffold: workspace, rune optional, `src-tauri` crate, `ui/` Vite React TS, first command (course) rendered — tests written with the code (no observed RED): 2 Vitest + Rust JSON-shape test; live `tauri dev` window lists both classes
- [x] T2 · S2 · inline · command surface: profiles, progress, course, setup probe/fix, lab start/run/finish + typed TS client — 18 commands over `engine::Engine`; 6 engine tests (1 vs live emulator), tests written after code (wrapping already-tested engine; no observed RED); workspace 88 tests + Clippy clean
- [x] T3 · S3, S4 · inline · theme + app shell (sidebar, top bar, transitions, reduced motion), responsive — 9 Vitest (nav + shell), tests written with code (no observed RED); checked in browser preview at 1024 and 760 px (icon rail)
- [x] T4 · S3 · delegated (context backstop) · intro, setup, profiles, Ajustes, CUENTA menu — 13 Vitest in screens/entry.test.tsx, 22 total; tsc + build clean; parent checked intro and setup (emulator warning) in browser preview; profiles/Ajustes not checked by eye
- [x] T5 · S3, S4 · delegated (context backstop) · Inicio + Clase — 6 Vitest (learn.test.tsx), 28 total, tests passed on first run (no observed RED); parent checked both in browser preview at 1024 px
- [ ] T6 · S3 · inline · Práctica + results
- [ ] T7 · S3 · inline · Laboratorio + terminal
- [ ] T8 · S5 · inline · remove Rune, docs, release workflow

## Log

- L1 (2026-11-16, user): "la app creo que tiene limitaciones aun en diseño, como que rune aun no tiene todo lo qur quiero, seria mejor tauri?" → "todo lo del diseño, responsividad y todo" → "arranca".
- 2026-11-16: probe panic fix committed on main first (47c56d8). Route: all tasks inline (one sequential UI migration, no parallel writer reason).
- T1: root is a workspace (`src-tauri` member); `rune` is optional behind default feature `rune-ui`, so `nubi-desktop` builds without Rune (`cargo tree` shows none). Course types gained `Serialize` (additive). Run the app with `ui/node_modules/.bin/tauri dev` from the repo root (the CLI looks for tauri.conf.json below the cwd). Old Rune app still builds; 82 Rust tests pass.
- T2: lesson and lab sessions stay in Rust state (Mutex in Engine); a finished practice/lab is recorded and saved by the backend (`saved` in the view), not by the UI. Serde derives added to setup/lesson/grading/lab types (additive; tags `type`/`value`). Setup fix logic copied from the Rune worker into engine.rs; the Rune copy goes away in T8.
- 2026-11-16 (user): "3" → single PR at the end. Native review of T1+T2 stopped with `lens_context_budget_exceeded` (40 files / 6907 lines, mostly lockfiles); no authority created. A single PR will exceed it too: review per work-unit commit when the gate asks.
- T3: tokens in ui/src/theme.css (Rune palette); shell = sidebar (icon rail < 900px, labels kept as accessible names) + top bar (crumb ellipsis, course name yields first, level name hidden < 560px container). Screen changes use View Transitions on the content only, skipped when the profile or OS asks for reduced motion; focus moves to #page-title. Dev preview: `npm run dev` in a browser answers commands with sample data (dev-preview.ts, not in the bundle). Other screens are placeholders until T4–T7.
- T4 (writer): setup rows built from the report, ready requires every report field (keeps the emulator warning). Delete uses native <dialog> showModal with autofocus on Cancelar. Open: reduced-motion switch disabled when nobody is signed in (store has no session-only field); new copy "Escribe tu nombre." for the empty-name error.
- T5 (writer): lock state from me.stops with a fallback mirroring grading::stop_status (stopStatus in Home.tsx); module table stacks under 760px content width; Clase two columns from 900px. Not ported yet: class_return/class_review modes (reached from T6/T7). Invented: side-panel "Ejemplo" shows done once the practice is completed.
- 2026-11-16: native review consent for T1–T4 (62 files) declined by the user ("Omitir esta vez").
