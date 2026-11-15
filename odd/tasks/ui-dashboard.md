# Professional dashboard UI in Rune
Objective: port the approved dashboard prototype (https://claude.ai/artifact/XTfPiRMTVhToPKn2jMmUeT) to the Rune app, keeping Nubi.
Branch: `feat/ui-pulido` · Delivery: ask-on-risk, local work-unit commits; no push · Runner: cargo test (incl. emulator), Clippy -D warnings, headless snapshots

## Specs

- S1 (user): "y su lo hacemos mas estili no juego? mas estilo app dashboard profesional?" — light gray ground, white cards with 1px borders, one blue action color, Inter + JetBrains Mono, no gradients.
- S2 (user): "me gusta mas, pero deja la mascota de la nube" — Nubi stays: greeting callout on Inicio, tip card in the class, reacting face in the practice footer.
- S3 (user): "si, pásalo a rune" — the Rune app gets the prototype's shell (sidebar: Inicio, Clase actual, Práctica, Laboratorio, environment status, profile; top bar: breadcrumb, stars) and its four screens: Inicio, Clase, Práctica, Laboratorio.
- S4 (carried from ui-clases-guiadas): flat ✕/back control, more class visuals (example + review), keyboard reaches every control, reduced motion respected.
- S5 (user): "rediseñame esto" (profiles screen screenshot).
- S6 (user): "y el boton de movimiento ponlo en unapartado en ajustes".
- S7 (user): "que cuando le piques a cuenta despliegue varias opciones sobre el. como cerrar sesion, cambiar perfil."
- S8 (user): "en ajustes que exista lo de eliminar y editar." — edit name/color; delete asks for confirmation and removes the profile folder.
- S9 (user): "si" to saving the motion preference with the profile — optional `reduced_motion` in profile.json (old files read as false).
- S10 (user): "la transicion entre pestañas quisiera que fuera lo mas rapido posible. mejora eso"
- S11 (user): "la 3 combinada con la 5" — subtle slide (~8 px, ~120 ms) applied only to the content area; sidebar and top bar stay fixed.
- S12 (user): "has responsividad. si esta corto que se respeten tamaños o que se activen barras de desplazamiento" / "aqui pon a nubi" (sidebar logo).
- Assumptions: Splash, Setup and Profiles keep their flow and only take the new palette. Inicio replaces the cloud map; its numbers come from real progress (no invented stats). Existing lesson/lab logic, grading, progress format and mastery rules are unchanged. The search box is left out until there is something to search (YAGNI).

## Tasks

- [x] T0 · S4 · inline · settle the open feat/ui-pulido work: flat close, class example/review visuals, keyboard test — RED class-2 scene locked → GREEN; 65 non-emulator tests + Clippy clean
- [x] T1 · S1 · inline · theme: palette, flat buttons with hover fill, focus ring, fonts — Rune .hover(tint) replaces per-key hover state; Inter + Plex Mono are already Rune defaults (no bundling); button tests rewritten for tint-not-grow; 64 tests + Clippy clean
- [x] T2 · S3 · inline · app shell: sidebar + top bar around Inicio/Clase/Práctica/Laboratorio — sidebar resumes attempts (no restart), lab entry gated on practice; motion toggle moved into the sidebar; tests written after code (no observed RED); keyboard test now checks focus via Tx::focused
- [x] T3 · S2, S3 · inline · Inicio dashboard: greeting + Nubi, progress cards, continue card, module table — replaces the cloud map; all numbers from saved progress; snapshot inspected
- [x] T4 · S2, S3, S4 · inline · Clase: article + steps aside + Nubi tip — tip shows the class's visual line; RED y=930 (sidebar pushed off-screen by long page) → GREEN via grow+scroll; snapshot inspected
- [x] T5 · S2, S3 · inline · Práctica: card layout, Nubi mood in the footer — three Nubi faces (neutral/happy/worried) rasterized once; result says 'Volver al inicio'; snapshots of question, wrong answer and result inspected
- [x] T6 · S3 · inline · Laboratorio inside the shell — header in content, flat objectives, darker terminal; ✕ control removed (sidebar is the way back, test updated); snapshot inspected
- [x] T7 · S1 · inline · short fade transition, Splash/Setup/Profiles palette, docs, snapshots — fade through the background keeps the tested cover/reveal timing; unused cloud art removed; 'Lab Cloud' wordmark; README updated; intro/setup/profiles snapshots inspected

- [x] T8 · S5, S6 · inline · profiles redesign + Ajustes screen holding the reduced-motion switch — floating toggle removed; tests: settings reachable from profiles and sidebar, switch toggles, Volver returns; snapshots inspected
- [x] T9 · S8, S9 · inline · Store: update_profile, delete_profile, reduced_motion field (Risk item 1: deletes stored data) — RED compile → GREEN; scratch-dir tests: rename keeps id+progress, empty/missing rejected, delete removes only that folder, `../` id rejected, legacy profile.json loads
- [x] T10 · S7 · inline · CUENTA menu: Ajustes, Cambiar de perfil, Cerrar sesión — header and profile row both open it; sign-out clears memory and tells the worker (Job::SignOut)
- [x] T11 · S8, S9 · inline · Ajustes: edit profile, delete with confirmation, motion switch saved to profile — cancel sends nothing, confirm sends DeleteProfile; motion loads on select
- [x] T12 · S10 · inline · fastest screen change — COVER 0 / REVEAL 0.08: swap on next frame + 80 ms fade-in; const-asserted ≤ 0.1 s

- [x] T13 · S11 · inline · slide-in content only — content rises 8 pt while fading in over 120 ms; sidebar/top bar never move; non-shell screens slide whole; reduced motion: none

- [ ] T14 · S12 · inline · responsiveness (PENDING — user: "lo de la responsividad dejala como que aun falta"); partial work in 2030917: shared page_scroll with edge fade, sidebar nav scrolls with pinned footer, narrower lab column, Nubi as logo — RED terminal 334 pt at 960×680 → GREEN; system scroll-bar preference honored (Rune follows macOS, no app override)

## Log

- L1 (user): "si, pásalo a rune" (after approving the dashboard prototype with Nubi: "me gusta mas, pero deja la mascota de la nube").
- 2026-11-14: T0 e9e932c, T1 af75952. T2+T3 one commit: the old map test pinned the cloud map that Inicio replaces, so the shell and Inicio ship together to keep every commit green. Mono font stays IBM Plex Mono (Rune default) instead of JetBrains Mono.
- 2026-11-14 close: T4 6fe519c, T5 143f6cd, T6 93ecb85, T7 52e0562. Full suite incl. emulator 74 passing (35 lib + 30 UI + 3 + 3 + 1 + 2), Clippy -D warnings clean. Branch feat/ui-pulido is ~7 commits ahead of main (well over 400 authored lines); not pushed. Pending: live window run (cargo run) not done; reduced-motion fade only covered by existing timing tests.
- L-next (user): "rediseñame esto. y el boton de movimiento ponlo en unapartado en ajustes" — T8 added.
- T9–T12 checks: 76 tests + Clippy clean (emulator suite not rerun: no lab/emulator code changed). Independent verifier for the delete path (Risk item 1) not run in this session.
- 2026-11-14 (user): "mit. has los comandos, y lo de la responsividad dejala como que aun falta" — MIT LICENSE added; About/topics and release v0.1.0 applied; T14 reopened as pending. Open responsiveness gaps to revisit: no forced scroll bars (Rune follows macOS), long top-bar breadcrumb not truncated, layouts only checked at 960×680 and 1180×760, no live-window resize check.
