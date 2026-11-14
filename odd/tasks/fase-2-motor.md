# Phase 2 — Game engine and lesson format
Objective: lesson/lab content as Spanish TOML data, grading by mastery (stars), local profiles and progress on disk, lab checker.
Branch: `feat/fase-2` · Delivery: ask-on-risk · Runner: `cargo test` (+ `-- --include-ignored` with the emulator running)

## Specs

- S1 (PLAN.md, Phase 2): "Lessons as data files (TOML), in Spanish: exercises (multiple choice, fill the command, order the steps, match concepts), lab missions with checker rules."
- S2 (PLAN.md, Phase 2): "Local profiles and progress saved on disk (see \"Local persistence\"): best stars and errors per node, unlocked nodes, level."
- S3 (PLAN.md, Product decisions): "No lives, no streaks, no XP. Unlimited attempts, never a penalty for mistakes." and "Every lesson, lab and challenge records its best result as stars: 1 = completed, 2 = completed with few errors, 3 = perfect (zero errors). ... any node can be replayed to perfect it. The student's **level** comes from total stars."
- S4 (PLAN.md, Local persistence): "One folder per profile in the OS app-data directory", "`profile.json` (name, avatar) and `progress.json` ... Writes go to a temp file and are then renamed over the old one", "Export / import a profile as one backup file".
- S5 (PLAN.md, Phase 2): "Content can be added without touching UI code."
- Assumption: "few errors" = 1–2 errors (2 stars); 3+ errors = 1 star. Level thresholds 0/5/15/30/50/75 stars (matches the design: level 2 at 10/15 ★ to level 3).
- Assumption: lab checks in this phase cover S3 only (the first lab); other services are added with their labs.
- Non-goal: UI screens (Phase 3+ consume this engine).

## Tasks

- [x] T1 · S1, S5 · inline · course format + loader + validation + first Spanish content — 1125db0 (RED 5 failing → GREEN 5 passing)
- [x] T2 · S3 · inline · grading: answer checks, stars, level, unlock — 6c6097b (RED 7 failing → GREEN 7 passing)
- [x] T3 · S2, S4 · inline · profiles + progress store, atomic writes, export/import — cde2cf9 (RED 7 failing → GREEN 7 passing)
- [ ] T4 · S1 · inline · lab checker (S3 checks via SDK) + emulator integration test — pending

Route evidence: one coherent engine (~600 lines) with shared types; no parallel units with disjoint surfaces, so no writer trigger → inline.

## Log

- L1 (2026-11-14, user): "arranca con la fase 2"
- L2 (2026-11-14, user, mid-T1): "al completar las fases hacer una página web de esto que estamos haciendo con un repo que te voy a pasar para subirlo a las GitHub pages y al finalizar usar el give GitHub Actions para hacer instaladores de ya que sea .dmg .exe y .paralinux ... dejarlo explícito en la página ... avisos, actualizaciones automáticas y agregar instalación con Breu ... opcional al final, de integración de conexión MSP con la con el juego" → recorded in PLAN.md as Phases 7–8 + optional MCP; out of scope for this feature.
- L3 (2026-11-14, user, mid-T1): "Que tenga un botón rápido in the application de initiar esto, instalar esto. No lo tienes installado, instala esto ... tenerlo en cuenta cuando hagamos las pantallas." → recorded in PLAN.md product decisions (guided setup screen); UI phase, out of scope here.
- T1 done: content/course.toml, content/lessons/{s3-buckets,lab-primer-bucket}.toml, src/course.rs.
- T2 done: src/grading.rs; level names 1–6: Despegando, Explorador, Constructor de nubes, Arquitecto, Arquitecto senior, Maestro de la nube.
- T3 done: src/progress.rs; ids are slugs [a-z0-9-] (path-safe), corrupt progress.json errors instead of resetting, import derives a fresh id.
- L4 (2026-11-14, user, mid-T3): "Este es el repo con el que quiero que te bases o la skill en la que instalemos y hagas el diseño de la página web" (https://github.com/Leonxlnx/taste-skill) → recorded in PLAN.md Phase 7; out of scope here.
