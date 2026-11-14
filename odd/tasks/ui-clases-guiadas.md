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

- [x] T1 · S1, S2, S5 · motion unit · tactile buttons, cloud transitions, reduced motion, focused tests — done; 67 tests incl. emulator pass, Clippy clean; e62bb7f
- [x] T2 · S3, S4 · class-flow unit · numbered class entry, explanation/example before games, associated lab and review, revisit teaching content — done; e62bb7f
- [x] T3 · S1–S6 · parent · integration, docs, visual proof, functional checks — e62bb7f; live-window proof pending

## Log

- L1 (user): "antes de iniciar lo soguiente enfoquemosnos en arreglar la ui., los botones hazlos animados que al hacer hover se suman como botones reales, animaciones de cambios de pantallas smooth, o transiciones smooth, o como si pasaras entre nubes como en los simpsons. algo que me mueve un poco el piso es que inicias la practica sin saber bien las cosas, estaria bien que furera como clase 1, y dentro de la clase haya como estos juegos o practicas"
- L2 (user): "si, ve anotando todo eso en el plan.md"
- L3 (user): "y hazlo"
- L4 (user): "ve haciendo commits no?"
- L5 (user): "y giardando en engram tamben"
- L6 (user): "si, te autorizo" — explicit parent continuation with at most two corrections, keyboard/integration/snapshots and verified commits.
- Parent correction 1: reproduced keyboard test failure (exit 101, intro_started false), changed activation from Enter to Key::Text(" "). Job-3 passed focused keyboard, full suite including emulator (65 tests), and Clippy.
- Parent continuation round 2 authorized by user ("continua"): reproduce map-entry test failure. Root cause was test event delivery: on_press needs a frame after pointer_up; S3's motion test already went through frames. Also its second assertion was invalid because an untouched session reports finished() for zero exercises. Screen assertion now passes; one correction in this round.
- Parent correction 2: visual inspection caught an unrequested replacement of the horizontal cloud map with cards; restored cloud class stops, dotted route, Nubi, island labels and selected-class detail. Added regression test (RED missing map-scroll). Added hover-rebuild regression test (RED scale reset); retained each keyed Live binding in Motion and passed State to shared button helpers. Finished the associated mechanical retry-button call-site adaptation after one compilation error. Job-4 now has 23 UI tests passing, one failing: view::map::tests::classes_keep_the_horizontal_cloud_map_and_mascot, src/view/map.rs:101, expected Screen::Class after pointer down/up, actual Map. Full remaining suites and Clippy were short-circuited; no further corrections or retries authorized in this cycle.
- Job-5 generated final map/class/lesson/lab captures successfully. Parent inspected target/ui-final-map.png: clouds, Nubi, route and class detail restored. No claim of successful live navigation. Both workers and all background commands are settled.
- Verified independent teaching-data unit committed f515753 (88 lines: course loader optional metadata, two S3 teaching tables, classes tests); evidence from job-3 and worker checks unchanged. Native assess HEAD^..HEAD medium, reviewDue false/under_budget. UI integration remains uncommitted and incomplete; no push.
- Continuation round 2 (user: "continua" + asked to record remaining work in this plan and Engram):
  - Correction 1: map-entry test fixed (frame after pointer release); invalid second assertion replaced with "a class opens teaching, not a practice".
  - Correction 2: Clippy rejected my test's Default field reassignment; switched to struct-update initialization.
  - Suite job-6 reached the end with every test binary passing; Clippy was the only failure. Fresh suite + Clippy job-7 and snapshots job-8 are running.
- Final checks: job-7 full suite incl. emulator: 35 lib + 24 UI + 3 checker + 2 foundations + 1 emulator + 2 classes = 67 passing, 0 ignored; Clippy --all-targets -D warnings clean; job-8 regenerated map/class/lesson/lab captures, all inspected. UI integration committed e62bb7f (688 authored lines). Native assess f515753..e62bb7f: medium, reviewDue false (under_budget), no independent verifier required. Slice overage accepted: motion and class flow share main.rs/view/mod.rs, so splitting would ship an unbuildable intermediate.
- Round 3 (user reported visual defects): "los botones chocan con las orillas…", "me gustaria que las clases tivieran mas imagenes o visualizadores", "mejorar el diseño al jace rhover en los botones", "las nubes estan como mal el diseño de los numeros, no se centran", "las transiciones no son tan smooth, como que se traban al continuar", padding "de un poco de 9px".
  - Root cause of the sticky transitions: Rune interpolates layout changes by default, so the curtain width lagged the clock every frame. cloud-cover now uses instant_changes(); cover/reveal became 0.22s/0.26s with smoothstep easing.
  - App chrome: 9pt global inset plus a 26pt safe band and a rounded clipped frame; the reduced-motion control is now a small floating pill instead of a 64pt bottom bar.
  - Hover now lifts (1.03) and press sinks; cloud numbers re-centred on the cloud body.
  - Classes gained optional `diagram` (bucket_objects / key_prefixes) drawn in the view, with content values and a data test.
  - Map: island plate moved below the cloud labels, canvas grew to fit.
  - Closure: 68 tests passing incl. emulator, Clippy clean, map/class/lesson captures re-inspected. Committed 731f642 (178 lines); native assess medium, reviewDue false (under_budget).
  - Still pending from the user's list: more class imagery than one diagram each (only the first idea is drawn), live-window proof of the hover/transition feel, and the ✕ close button style.
- Pending / handoff for a later session:
  - Cosmetic: the 44px ✕ close button inherits the raised shadow and reads as a clipped crescent in the lesson and lab headers; give it a flat style.
  - Unverified: live `cargo run` flow and real-cloud transition feel; cross-screen keyboard focus; live OS reduced-motion changes; hover over a long real pointer session.
  - No push, PR or merge is authorized for this UI work; origin/main remains 3c40623 and Phase 4 content stays local.
- Checkpoint 45151a9: PLAN.md and this feature specification committed; structural diff check clean, native assessment passive / reviewDue false. Exploration is complete; implementation now runs as the two bounded units below. Keep local work-unit commits and refresh the full Engram mirror at task transitions.
- Continuation results: class worker mwgyb4kv-6-r86l settled partial only because full cargo test fixture cleanup is prohibited in the child; 20 UI, 3 class/foundations, 4 lesson, 8 grading tests passed plus Clippy and target/ui-class.png inspected by parent. No behavioral RED observed (new State API tests first failed to compile). Parent full suite still required after motion settles.
- Motion continuation mwgyaws8-5-dc3x settled partial with all commands/jobs stopped: cargo test --bin labcloud: exit 101, 21 passed, 1 failed. Exact failure: view::motion_tests::keyboard_skips_disabled_and_activates_the_button, src/view/mod.rs:229, assertion failed: rt.state.intro_started. Worker reports pinned Rune ordinary buttons activate on Space but test sends Enter; two corrections exhausted. Clippy and snapshots target/ui-motion.png / target/ui-motion-lesson.png passed. Controller, hover, press, curtain coverage (960×680, 1180×760), toggle and simulated reduced motion checks passed. No behavioral RED for hover/curtain/toggle; live OS changes, real transition presentation, cross-screen focus and hover continuity across unrelated rebuilds unverified. Both children settled, no active/uncertain child. Do not correct automatically: needs human-controlled bounded parent continuation to correct keyboard expectation, run full suite/Clippy, inspect snapshots, normalize and assess candidate, then commit verified units. No feature completion or delivery claimed.
- First worker results: T1 partial (controller/dispatch only, 13 UI tests and Clippy passing; tactile buttons/curtain/toggle missing). Parent relayed verified public Rune APIs; one bounded completion/correction batch remains. T2 made no edits and requested target build/snapshot permission; human explicitly selected allow-target-artifacts. Both workers are confirmed completed before continuations. Native assess over partial changes was unassessable due untracked scope; no review authority created, assessment deferred to normalized candidate.
- Worker launches: T1 mwgxz4td-3-6lb7 owns src/motion.rs and shared view motion/button kit; T2 mwgxzhvh-4-2rnh owns class state/content/map/lesson/lab views. Disjoint allowed edits; no commits/push by children. Both run strict TDD and focused/full checks with at most two corrections per failing check; parent owns seam checks and commits. Source interface scaffold compiled with 10 existing UI tests passing.
- Interface preparation: parent adds inert Class dispatch, Motion/default and reduced_motion state, poll seam. This scaffold has no meaningful behavioral RED; writers must observe RED for their actual rules before implementing. Two independent units, disjoint surfaces: motion controller/shared view kit vs class state/content/views. Parent owns final integration/docs.
- Priority: pause Phase 4 service expansion. Existing opening S3 slice remains intact; origin/main stays at 3c40623.
- PLAN.md updated with the approved motion and class sequence before source edits. Explorer mapping Rune APIs and integration surfaces. No changes to credentials, persisted data formats or AWS dependencies authorized here.
- Delivery choice: local feature-branch work-unit chain; do not create a single oversized delivery or publish this work. Forecast 700–1000 authored lines; preserve cohesive code/tests/docs, do not compress to meet a line target.
