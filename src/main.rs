//! Lab Cloud — proof of concept (Phase 1).
//!
//! One Rune window that talks to the local AWS Emulator two ways:
//! - the AWS SDK for Rust lists S3 buckets (this is how the future checker reads state);
//! - an in-app terminal runs real `aws` CLI commands and shows their output.
//!
//! Start the emulator first: `scripts/emulator.sh start`.

use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc::{self, Receiver, Sender};

use rune::tok::{self, space, type_};
use rune::{App, AppHandle, AppSpec, Cx, El, ScrollSpec, TextEdit, Tx, col, el, row, text, text_input};

const PORT: u16 = 9297;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Kind {
    Cmd,
    Out,
    Err,
}

struct Line {
    kind: Kind,
    text: String,
}

enum Job {
    Refresh,
    Run(Vec<String>),
}

enum Emulator {
    Checking,
    Up(Vec<String>),
    Down(String),
}

struct State {
    jobs: Option<Sender<Job>>,
    input: TextEdit,
    lines: Vec<Line>,
    emulator: Emulator,
}

impl Default for State {
    fn default() -> Self {
        State {
            jobs: None,
            input: TextEdit::new(""),
            lines: vec![
                Line { kind: Kind::Out, text: "Lab Cloud · terminal conectada a tu AWS Emulator local".into() },
                Line { kind: Kind::Out, text: "Prueba: aws s3 ls · aws s3api create-bucket --bucket mi-bucket · limpiar".into() },
            ],
            emulator: Emulator::Checking,
        }
    }
}

impl State {
    fn send(&self, job: Job) {
        if let Some(jobs) = &self.jobs {
            let _ = jobs.send(job);
        }
    }
}

/// Splits a terminal line into `aws` arguments. Only `aws` commands may run: the terminal is
/// not a general shell.
// ponytail: whitespace split, no quotes; add a shell-words parser when a lab needs quoted args.
fn parse_command(line: &str) -> Result<Vec<String>, String> {
    let mut words = line.split_whitespace();
    match words.next() {
        Some("aws") => Ok(words.map(String::from).collect()),
        Some(_) => Err("Solo se permiten comandos aws aquí.".into()),
        None => Err(String::new()),
    }
}

fn submit(s: &mut State, _: &mut Tx) {
    let line = s.input.text.trim().to_string();
    s.input.clear();
    if line.is_empty() {
        return;
    }
    if line == "limpiar" || line == "clear" {
        s.lines.clear();
        return;
    }
    s.lines.push(Line { kind: Kind::Cmd, text: line.clone() });
    match parse_command(&line) {
        Ok(args) => s.send(Job::Run(args)),
        Err(msg) => s.lines.push(Line { kind: Kind::Err, text: msg }),
    }
}

fn view(s: &State, _: &mut Cx<State>) -> El {
    let t = tok::theme(true);
    let accent = tok::accent(0, true);

    let status = match &s.emulator {
        Emulator::Checking => "Emulador: comprobando…".to_string(),
        Emulator::Up(buckets) if buckets.is_empty() => "Emulador activo · sin buckets".to_string(),
        Emulator::Up(buckets) => format!("Emulador activo · buckets (SDK): {}", buckets.join(", ")),
        Emulator::Down(err) => format!("Emulador no disponible: {err}"),
    };
    let header = row().key("header").w_full().gap(space::S3).center_y()
        .child(text("labcloud").font(type_::TITLE).color(accent))
        .child(text(status).key("status").font(type_::BODY).color(t.text).grow(1.0))
        .child(
            el().key("refresh").role("button").label("Actualizar").focusable(true)
                .px(space::S3).h(28.0).radius(tok::radius::MD).fill(t.card).border(1.0, t.card_border).center_y()
                .on_press(|s: &mut State| s.send(Job::Refresh))
                .child(text("Actualizar").font(type_::LABEL).color(t.text)),
        );

    let mut log = col().key("log").w_full().gap(2.0);
    for (i, line) in s.lines.iter().enumerate() {
        let (prefix, color) = match line.kind {
            Kind::Cmd => ("$ ", accent),
            Kind::Out => ("", t.text),
            Kind::Err => ("", tok::accent(2, true)),
        };
        log = log.child(
            text(format!("{prefix}{}", line.text)).key(format!("line-{i}"))
                .font(type_::BODY.mono()).color(color).wrap().selectable(true),
        );
    }

    let input = row().key("prompt").w_full().gap(space::S2).center_y()
        .child(text("$").font(type_::BODY.mono()).color(accent))
        .child(
            text_input(&s.input, type_::BODY.mono()).key("input").label("Comando").single_line().grow(1.0)
                .color(t.text).placeholder("escribe un comando aws y presiona Enter", t.text_secondary)
                .on_edit(|s: &mut State| &mut s.input)
                .on_submit(submit),
        );

    let terminal = col().key("terminal").w_full().grow(1.0).padding(space::S3).gap(space::S2)
        .radius(tok::radius::LG).fill(t.card).border(1.0, t.card_border)
        .child(el().key("scroll").w_full().grow(1.0).scroll(ScrollSpec { stick_bottom: true }).child(log))
        .child(input);

    col().w_full().h_full().padding(space::S6).gap(space::S3).fill(t.bg)
        .child(header)
        .child(terminal)
}

fn emulator_dir() -> PathBuf {
    let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
    home.join(".aws/emulator").join(PORT.to_string())
}

/// Points this process — and every `aws` child it spawns — at the emulator.
/// Same variables as `scripts/emulator.sh env`.
fn point_at_emulator() {
    let dir = emulator_dir();
    let ca = dir.join("ca.pem");
    // SAFETY: called first thing in `main`, before any other thread exists.
    unsafe {
        for var in ["AWS_ACCESS_KEY_ID", "AWS_SECRET_ACCESS_KEY", "AWS_SESSION_TOKEN", "AWS_PROFILE"] {
            std::env::remove_var(var);
        }
        std::env::set_var("AWS_ENDPOINT_URL", format!("https://127.0.0.1:{PORT}"));
        std::env::set_var("AWS_SHARED_CREDENTIALS_FILE", dir.join("emulator_credentials"));
        std::env::set_var("AWS_CONFIG_FILE", dir.join("config"));
        std::env::set_var("AWS_CA_BUNDLE", &ca);
        std::env::set_var("SSL_CERT_FILE", &ca);
        std::env::set_var("AWS_EC2_METADATA_DISABLED", "true");
        std::env::set_var("AWS_DISABLE_HOST_PREFIX_INJECTION", "true");
        std::env::set_var("AWS_REGION", "us-east-1");
        std::env::set_var("AWS_DEFAULT_REGION", "us-east-1");
        std::env::set_var("AWS_PAGER", "");
    }
}

/// Lists buckets through the SDK. A fresh client each time picks up the new credentials the
/// emulator writes on every restart.
async fn list_buckets() -> Result<Vec<String>, String> {
    let config = aws_config::load_from_env().await;
    let out = aws_sdk_s3::Client::new(&config).list_buckets().send().await.map_err(|e| {
        let msg = format!("{e:?}");
        msg.lines().next().unwrap_or("error").chars().take(160).collect::<String>()
    })?;
    Ok(out.buckets().iter().filter_map(|b| b.name().map(String::from)).collect())
}

fn run_aws(args: &[String]) -> Vec<Line> {
    match Command::new("aws").args(args).output() {
        Err(e) => vec![Line { kind: Kind::Err, text: format!("No se pudo ejecutar aws: {e}") }],
        Ok(out) => {
            let mut lines = Vec::new();
            for (bytes, kind) in [(&out.stdout, Kind::Out), (&out.stderr, Kind::Err)] {
                for l in String::from_utf8_lossy(bytes).lines() {
                    lines.push(Line { kind, text: l.to_string() });
                }
            }
            lines
        }
    }
}

fn worker(jobs: Receiver<Job>, ui: AppHandle<State>) {
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("tokio runtime");
    for job in jobs {
        if let Job::Run(args) = job {
            let lines = run_aws(&args);
            ui.update(move |s, _| s.lines.extend(lines));
        }
        // Every command can change emulator state, so refresh after each one.
        let emulator = match rt.block_on(list_buckets()) {
            Ok(buckets) => Emulator::Up(buckets),
            Err(e) => Emulator::Down(e),
        };
        ui.update(move |s, _| s.emulator = emulator);
    }
}

fn main() {
    point_at_emulator();
    let (tx, rx) = mpsc::channel();
    tx.send(Job::Refresh).expect("queue first refresh");
    let app = App::new(AppSpec::new("Lab Cloud", (900.0, 600.0), State::default, view))
        .init(move || State { jobs: Some(tx), ..State::default() });
    let ui = app.handle();
    std::thread::spawn(move || worker(rx, ui));
    app.run();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_aws_commands_reach_the_terminal() {
        assert_eq!(parse_command("aws s3 ls").unwrap(), vec!["s3", "ls"]);
        assert_eq!(parse_command("  aws   sts get-caller-identity ").unwrap(), vec!["sts", "get-caller-identity"]);
        assert!(parse_command("rm -rf /").is_err());
        assert!(parse_command("awsx s3 ls").is_err());
        assert!(parse_command("").is_err());
    }

    /// Needs the emulator running: `scripts/emulator.sh start && cargo test -- --ignored`.
    #[test]
    #[ignore]
    fn the_terminal_and_the_sdk_both_reach_the_emulator() {
        point_at_emulator();
        let who = run_aws(&["sts".into(), "get-caller-identity".into()]);
        assert!(who.iter().any(|l| l.kind == Kind::Out && l.text.contains("000000000000")), "emulator account");

        let bad = run_aws(&["s3api".into(), "head-bucket".into(), "--bucket".into(), "no-such-bucket-labcloud".into()]);
        assert!(bad.iter().any(|l| l.kind == Kind::Err), "errors land in the terminal as errors");

        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        assert!(rt.block_on(list_buckets()).is_ok(), "SDK trusts the emulator certificate");
    }
}
