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

The foundations island currently has four stops: S3 buckets, a versioning lab,
object keys and prefixes, and an archive lab with two starter files. The archive
lab checks the complete object keys and resets its bucket when replayed.
IAM/STS, DynamoDB, SQS and SNS are planned next; Phase 4 is not complete.

## Adding content

Course content is Spanish TOML under `content/`, no code changes needed:

- `content/course.toml` — the map: units (islands) and the ids of their stops, in order.
- `content/lessons/<id>.toml` — one stop. `kind` is `leccion`, `lab` or `desafio`.
  - Lessons/challenges list `[[exercises]]` with `type`: `choice` (`options`, `answer`
    index), `command` (`answers`), `order` (`steps` in the right order) or `match`
    (`pairs` in the right order). Each has a `prompt` and a `hint`.
  - Labs list `[[objectives]]` with `text`, `hint` and a `check`: `bucket_exists`,
    `bucket_versioning` (`status`) or `object_exists` (`key`).

`cargo test` validates every file (ids, answers in range, labs with objectives, …).

Headless snapshots of the screens: `cargo run -- --snap out.png --scene <name>` with `intro`, `setup`, `profiles`, `map`, `lesson`, `lesson-wrong`, `lesson-order`, `lesson-match`, `lesson-done` or `lab`.

Progress lives in the OS app-data folder (macOS: `~/Library/Application Support/LabCloud/`).
The lab terminal runs only `aws` commands, without a shell, inside `labs/<id>/` there.

## Test

```sh
cargo test                       # unit tests
cargo test -- --include-ignored  # also the tests that need the emulator running
```
