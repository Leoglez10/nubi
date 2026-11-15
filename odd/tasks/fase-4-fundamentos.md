# Phase 4 — foundations, opening S3 slice
Objective: begin Phase 4 with a Spanish S3 lesson and a replayable local lab using existing checks.
Branch: `feat/fase-4-fundamentos` · Delivery: single-pr (local opening slice only, forecast ~250 lines) · Runner: `cargo test`, emulator integration, Clippy

## Specs

- S1 (user): "sibe a main e inicia la fase 4"
- S2 (PLAN.md): "### Phase 4 — Module 1: foundations (highest-fidelity services)" / "- IAM/STS, S3, DynamoDB, SQS, SNS."
- S3 (PLAN.md): "Each lab must be validated against the emulator before it ships."
- Assumption: "inicia" authorizes a bounded first slice, not delivery of all Phase 4 services now. Start with S3 object keys/prefixes and an upload lab, retaining the original two stops and adding two more. IAM/STS, DynamoDB, SQS and SNS remain subsequent slices, not completion claims.
- Non-goals: profile backup buttons, UI changes, new dependencies, cloud access or publishing Phase 4.

## Tasks

- [x] T1 · S1, S2 · inline · S3 object keys/prefixes lesson and map continuation, original stops preserved — RED missing stops → GREEN, 46 tests passing; a4978a9
- [x] T2 · S2, S3 · inline · two-object lab, executable hints and isolated emulator checks including replay/reset — GREEN, two full lab attempts; a4978a9
- [ ] T3 · pending from UI · inline · responsiveness: truncate the long breadcrumb, check more window sizes and a live resize; scroll bars follow macOS (document, cannot force)
- [ ] T4 · pending from UI · delegated verifier · independent review of the profile delete path (Risk item 1: removes stored data)
- [ ] T5 · pending from UI · inline · live `cargo run` pass: hover, tab slide-in, keyboard focus, reduced motion
- [ ] T7 · pending from UI · inline · app logo: Nubi-based icon for the window and Dock (source art reusable for Phase 8 installers)
- [ ] T6 · S2 · next service slice (suggested: DynamoDB), after T3–T5 and T7

## Log

- L1 (user): "sibe a main e inicia la fase 4"
- Published existing screens to origin/main at 3c40623 after cargo test (45 passing, 4 integration tests ignored), clean Clippy and diff check. Native assess returned medium, reviewDue false / under_budget. Manual live-window flow remains unverified.
- Scope: begin Phase 4 in one small S3 work unit (~250 authored lines), leaving service expansion for later work units. No writer trigger: one tightly coupled lesson/lab behavior, parent inline. Risk: none; data-only content and tests, no shared engine changes or dependencies.
- T1: observed RED (expected four map stops, found two; exit 101), then GREEN. All four exercise formats covered; original lesson/lab preserved. Teaching source: https://docs.aws.amazon.com/AmazonS3/latest/userguide/object-keys.html and official AWS CLI cp examples through Context7.
- T2: integration test exercises shipped hints via lab::Session and lab::run_aws, isolated process-specific bucket and project target sandbox, verifies objective-by-objective state, stored file contents, zero errors, cleanup and two complete attempts. Existing checks implement the rule already; no separate failing engine test was meaningful.
- Closure checks: scripts/emulator.sh start && cargo test -- --include-ignored --test-threads=1 && cargo clippy --all-targets -- -D warnings: exit 0, 51 tests passed (35 logic, 10 UI, 3 existing checker, 1 emulator, 2 foundations), none ignored; git diff --check clean. Live window flow remains unverified; no UI code changed.
- Work unit a4978a9: 188 authored changed lines. Native assess against main: medium, reviewDue false (under_budget), no independent verifier required. Initial assessment had a mistyped base ref and created no authority; corrected read-only assessment used main successfully.
- Next slice: DynamoDB lesson/lab and service-specific checks, then other Phase 4 services. This opening slice is complete; Phase 4 is not. Phase 4 stays local; origin/main ends at 3c40623.
- (user, 2026-11-14): "en la fase 4 pon lo de responsividad y lo que quedo pendiente antes de emoezarla" — T3–T5 added before the next service slice (T6).
- (user, 2026-11-14): "uno que falta es ponerle logo a la app" — T7 added before the next service slice.
