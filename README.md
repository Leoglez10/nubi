# Lab Cloud AWS

Game-like desktop app to learn AWS in Spanish, with every lab running locally against
[AWS Emulator](https://github.com/aws/aws-emulator). See [PLAN.md](PLAN.md).

## Requirements

- Docker (running)
- AWS CLI v2
- Rust (stable)

## Run

```sh
scripts/emulator.sh start     # builds the emulator image once, starts it on 127.0.0.1:9297
cargo run                     # opens the app
```

`scripts/emulator.sh stop` stops the emulator (its resources live in memory and are lost).
To use the AWS CLI against the emulator in your own shell: `eval "$(scripts/emulator.sh env)"`.

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

## Test

```sh
cargo test                       # unit tests
cargo test -- --include-ignored  # also the tests that need the emulator running
```
