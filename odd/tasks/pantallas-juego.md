# Game screens — intro, profiles, sky map, exercises
Objective: the rest of the approved design as real screens: animated intro, local profile picker, horizontal map in the clouds, and the one-exercise-per-screen lesson flow, wired to the engine.
Branch: `feat/pantallas` · Delivery: ask-on-risk · Runner: `cargo test` (+ `-- --include-ignored` with the emulator running)

## Specs

- S1 (PLAN.md): "**Animated intro.** A short launch animation (Nubi rises, logo, \"turning on your local cloud\" checks for Docker and the emulator), then the profile screen."
- S2 (PLAN.md): "**Local profiles, no accounts.** \"Who is learning today?\" picker with name + avatar, no password. Everything is saved on the student's machine."
- S3 (PLAN.md): "**Horizontal map in the clouds.** The home screen is a horizontally scrolling sky map: units are cloud islands, nodes are cloud platforms joined by a flight path. It must not look like Duolingo's vertical path."
- S4 (PLAN.md): "Bite-sized exercises (one question per screen), a mascot (Nubi), celebration screens." / "No lives, no streaks, no XP. Unlimited attempts, never a penalty for mistakes." / "Mistakes are listed back to the student as learning, and any node can be replayed to perfect it."
- S5 (approved design, Main board): selecting a stop shows its detail with "Empezar", "Perfeccionar" or "Repasar"; locked stops say "Se desbloquea al completar la parada anterior."
- Assumption: exercise options are shuffled deterministically (rotated), no randomness dependency.
- Assumption: profile backup import/export stays engine-only this round (needs a file dialog); the button comes later.
- Non-goal: new course content (Phase 4).

## Tasks

- [x] T1 · S4 · inline · lesson session logic + stop status for the map — a51c4c4 (RED 5 failing → GREEN 35 lib passing)
- [x] T2 · S4 · inline · exercise screen (choice, command, order, match) + lesson result, progress saved — f35e110 (UI tests written with the views, no separate RED)
- [x] T3 · S3, S5 · inline · horizontal sky map with cloud islands, Nubi, stop detail, navigation to lessons/labs — f35e110 (UI tests written with the views, no separate RED)
- [x] T4 · S1, S2 · inline · animated intro + profile picker/creation; setup screen only when something is missing — f35e110 (UI tests written with the views, no separate RED)

Route evidence: one app sharing State, Job channel and view module; no disjoint parallel units → inline.

## Log

- L1 (2026-11-14, user): "contrinua con lo siguiente" (after: "Lo siguiente sería el inicio animado, los perfiles, el mapa horizontal en las nubes y la pantalla de ejercicios.")
- T1 done: src/lesson.rs (LessonSession check/retry/next, mistakes once per exercise, shuffled rotation), grading::stop_status.
- T2–T4 done together in f35e110: they share State, navigation and the Job channel in main.rs. src/view/{mod,intro,profiles,map,lesson,lab,setup}.rs; SVG art (Nubi, clouds) via ImageCache; 10 headless UI tests + 10 snapshot scenes reviewed (intro, profiles, map, lesson, wrong, order, match, done, lab). Fixes after review: flight path under stops; match pair tints distinct from selection. Not verified: live window flow (worker) — needs cargo run.
