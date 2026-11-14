# Lab Cloud AWS

Game-like desktop app to learn AWS in Spanish, with every lab running locally against
[AWS Emulator](https://github.com/aws/aws-emulator). See [PLAN.md](PLAN.md).

## Requirements

- Docker (running)
- AWS CLI v2
- Rust (stable)

## Run

```sh
cargo run
```

The app checks for Docker, the AWS CLI and the emulator first, and offers a button to
install or start whatever is missing. The emulator can also be managed by hand:
`scripts/emulator.sh start` builds the image once and starts it on 127.0.0.1:9297, and
`scripts/emulator.sh stop` stops the emulator (its resources live in memory and are lost).
To use the AWS CLI against the emulator in your own shell: `eval "$(scripts/emulator.sh env)"`.

## Course content

The foundations island groups its four existing stops into two numbered classes:

1. **Buckets y objetos:** explanation and worked example → games → versioning lab → review.
2. **Claves y prefijos:** explanation and worked example → games → two-file archive lab → review.

Open a class from the map before practicing. During a game or lab, **Ver explicación**
lets you consult the teaching material and return without restarting your attempt.
Complete a class's games to unlock its lab. Existing node progress and mastery stars
are preserved. The archive lab checks complete object keys and resets its bucket on replay.
IAM/STS, DynamoDB, SQS and SNS are planned next; Phase 4 is not complete.

## Adding content

Course content is Spanish TOML under `content/`, no code changes needed:

- `content/course.toml` — the map: units (islands) and the ids of their stops, in order.
- `content/lessons/<id>.toml` — one stop. `kind` is `leccion`, `lab` or `desafio`.
  - Lessons/challenges list `[[exercises]]` with `type`: `choice` (`options`, `answer`
    index), `command` (`answers`), `order` (`steps` in the right order) or `match`
    (`pairs` in the right order). Each has a `prompt` and a `hint`.
  - A lesson's optional `[teaching]` table defines a guided class: `number`, `title`,
    `lab` (the next mapped lab id), `explanation`, `visual`, `example` and `review`.
    Class numbers follow map order starting at 1; all teaching strings must be nonempty.
  - Labs list `[[objectives]]` with `text`, `hint` and a `check`: `bucket_exists`,
    `bucket_versioning` (`status`) or `object_exists` (`key`).

`cargo test` validates every file (ids, answers in range, labs with objectives, …).

Headless snapshots of the screens: `cargo run -- --snap out.png --scene <name>` with `intro`, `setup`, `profiles`, `map`, `lesson`, `lesson-wrong`, `lesson-order`, `lesson-match`, `lesson-done`, `class` or `lab`.

Progress lives in the OS app-data folder (macOS: `~/Library/Application Support/LabCloud/`).
The lab terminal runs only `aws` commands, without a shell, inside `labs/<id>/` there.

## Test

```sh
cargo test                       # unit tests
cargo test -- --include-ignored  # also the tests that need the emulator running
```
