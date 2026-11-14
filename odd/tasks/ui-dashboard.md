# Professional dashboard UI in Rune
Objective: port the approved dashboard prototype (https://claude.ai/artifact/XTfPiRMTVhToPKn2jMmUeT) to the Rune app, keeping Nubi.
Branch: `feat/ui-pulido` · Delivery: ask-on-risk, local work-unit commits; no push · Runner: cargo test (incl. emulator), Clippy -D warnings, headless snapshots

## Specs

- S1 (user): "y su lo hacemos mas estili no juego? mas estilo app dashboard profesional?" — light gray ground, white cards with 1px borders, one blue action color, Inter + JetBrains Mono, no gradients.
- S2 (user): "me gusta mas, pero deja la mascota de la nube" — Nubi stays: greeting callout on Inicio, tip card in the class, reacting face in the practice footer.
- S3 (user): "si, pásalo a rune" — the Rune app gets the prototype's shell (sidebar: Inicio, Clase actual, Práctica, Laboratorio, environment status, profile; top bar: breadcrumb, stars) and its four screens: Inicio, Clase, Práctica, Laboratorio.
- S4 (carried from ui-clases-guiadas): flat ✕/back control, more class visuals (example + review), keyboard reaches every control, reduced motion respected.
- Assumptions: Splash, Setup and Profiles keep their flow and only take the new palette. Inicio replaces the cloud map; its numbers come from real progress (no invented stats). Existing lesson/lab logic, grading, progress format and mastery rules are unchanged. The search box is left out until there is something to search (YAGNI).

## Tasks

- [x] T0 · S4 · inline · settle the open feat/ui-pulido work: flat close, class example/review visuals, keyboard test — RED class-2 scene locked → GREEN; 65 non-emulator tests + Clippy clean
- [ ] T1 · S1 · inline · theme: palette, flat buttons with hover fill, focus ring, fonts
- [ ] T2 · S3 · inline · app shell: sidebar + top bar around Inicio/Clase/Práctica/Laboratorio
- [ ] T3 · S2, S3 · inline · Inicio dashboard: greeting + Nubi, progress cards, continue card, module table
- [ ] T4 · S2, S3, S4 · inline · Clase: article + steps aside + Nubi tip
- [ ] T5 · S2, S3 · inline · Práctica: card layout, Nubi mood in the footer
- [ ] T6 · S3 · inline · Laboratorio inside the shell
- [ ] T7 · S1 · inline · short fade transition, Splash/Setup/Profiles palette, docs, snapshots

## Log

- L1 (user): "si, pásalo a rune" (after approving the dashboard prototype with Nubi: "me gusta mas, pero deja la mascota de la nube").
