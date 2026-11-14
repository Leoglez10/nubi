# Tactile UI and guided classes
Objective: polish motion and teach concepts before games and labs, using the existing cloud identity.
Branch: `feat/ui-clases-guiadas` · Delivery: feature-branch-chain, local work-unit commits (~700–1000 forecast lines); no push · Runner: cargo test, Clippy, headless snapshots

## Specs

- S1 (user): "los botones hazlos animados que al hacer hover se suman como botones reales"
- S2 (user): "animaciones de cambios de pantallas smooth, o transiciones smooth, o como si pasaras entre nubes como en los simpsons."
- S3 (user): "algo que me mueve un poco el piso es que inicias la practica sin saber bien las cosas, estaria bien que furera como clase 1, y dentro de la clase haya como estos juegos o practicas"
- S4 (approved proposal): "Clases guiadas: «Clase 1: buckets y objetos» → explicación visual breve → ejemplo resuelto → juegos → laboratorio → repaso." / "con la explicación antes de las prácticas y la posibilidad de volver a consultarla"
- S5 (approved proposal): "Botones con profundidad: se hunden suavemente al pasar el cursor y al pulsarlos." / "Transiciones entre nubes: las nubes cubren la pantalla y revelan la siguiente, con movimiento reducido opcional."
- S6 (user): "si, ve anotando todo eso en el plan.md" / "y hazlo"
- Assumptions: adapt the two current S3 lesson/lab pairs into two numbered classes; no new AWS services. Preserve existing profile/progress format and mastery rules. Keep keyboard focus/activation and access to reduced motion; explanation revisits must not reset an active attempt.

## Tasks

- [ ] T1 · S1, S2, S5 · motion unit · tactile buttons, cloud transitions, reduced motion, focused tests — in progress (API mapping)
- [ ] T2 · S3, S4 · class-flow unit · numbered class entry, explanation/example before games, associated lab and review, revisit teaching content — pending mapping
- [ ] T3 · S1–S6 · parent · integration, docs, visual proof, functional checks and applicable independent verification — pending

## Log

- L1 (user): "antes de iniciar lo soguiente enfoquemosnos en arreglar la ui., los botones hazlos animados que al hacer hover se suman como botones reales, animaciones de cambios de pantallas smooth, o transiciones smooth, o como si pasaras entre nubes como en los simpsons. algo que me mueve un poco el piso es que inicias la practica sin saber bien las cosas, estaria bien que furera como clase 1, y dentro de la clase haya como estos juegos o practicas"
- L2 (user): "si, ve anotando todo eso en el plan.md"
- L3 (user): "y hazlo"
- Priority: pause Phase 4 service expansion. Existing opening S3 slice remains intact; origin/main stays at 3c40623.
- PLAN.md updated with the approved motion and class sequence before source edits. Explorer mapping Rune APIs and integration surfaces. No changes to credentials, persisted data formats or AWS dependencies authorized here.
- Delivery choice: local feature-branch work-unit chain; do not create a single oversized delivery or publish this work. Forecast 700–1000 authored lines; preserve cohesive code/tests/docs, do not compress to meet a line target.
