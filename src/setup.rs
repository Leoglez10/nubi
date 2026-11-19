//! Guided setup: what the student's machine is missing before playing, and the one-click
//! action that fixes each item on their OS.

use std::io::{self, BufRead, BufReader, Read, Write};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Default, Serialize)]
pub struct Report {
    pub docker: bool,
    pub docker_running: bool,
    pub aws_cli: bool,
    pub emulator: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Need {
    Docker,
    DockerRunning,
    AwsCli,
    Emulator,
}

impl Report {
    /// Missing items, in the order to fix them.
    pub fn needs(&self) -> Vec<Need> {
        let mut needs = Vec::new();
        if !self.docker {
            needs.push(Need::Docker);
        } else if !self.docker_running {
            needs.push(Need::DockerRunning);
        }
        if !self.aws_cli {
            needs.push(Need::AwsCli);
        }
        if !self.emulator {
            needs.push(Need::Emulator);
        }
        needs
    }

    pub fn ready(&self) -> bool {
        self.needs().is_empty()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Os {
    Mac,
    Windows,
    Linux,
}

impl Os {
    pub fn current() -> Os {
        if cfg!(target_os = "macos") {
            Os::Mac
        } else if cfg!(windows) {
            Os::Windows
        } else {
            Os::Linux
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum Action {
    /// Run a program (an installer through the OS package manager, or an app launcher).
    Run { program: &'static str, args: Vec<&'static str> },
    /// Open the official download/instructions page.
    OpenUrl(&'static str),
    /// Build the image if needed and start the emulator container.
    StartEmulator,
    /// Nothing the app can do for the student: show the instruction text instead.
    Manual(&'static str),
}

/// The one-click fix for `need` on `os`.
pub fn action(need: Need, os: Os, has_brew: bool) -> Action {
    let run = |program, args: &[&'static str]| Action::Run { program, args: args.to_vec() };
    match (need, os) {
        (Need::Docker, Os::Mac) if has_brew => run("brew", &["install", "--cask", "docker"]),
        (Need::Docker, Os::Mac) => Action::OpenUrl("https://www.docker.com/products/docker-desktop/"),
        (Need::Docker, Os::Windows) => run("winget", &["install", "-e", "--id", "Docker.DockerDesktop"]),
        (Need::Docker, Os::Linux) => Action::OpenUrl("https://docs.docker.com/engine/install/"),
        (Need::DockerRunning, Os::Mac) => run("open", &["-a", "Docker"]),
        (Need::DockerRunning, Os::Windows) => Action::Manual("Abre Docker Desktop desde el menú Inicio."),
        (Need::DockerRunning, Os::Linux) => Action::Manual("Inicia Docker con: sudo systemctl start docker"),
        (Need::AwsCli, Os::Mac | Os::Linux) if has_brew => run("brew", &["install", "awscli"]),
        (Need::AwsCli, Os::Windows) => run("winget", &["install", "-e", "--id", "Amazon.AWSCLI"]),
        (Need::AwsCli, _) => Action::OpenUrl("https://docs.aws.amazon.com/cli/latest/userguide/getting-started-install.html"),
        (Need::Emulator, _) => Action::StartEmulator,
    }
}

#[derive(Serialize)]
pub struct Progress {
    pub kind: &'static str,
    pub text: String,
}

pub type Feed<'a> = &'a (dyn Fn(Progress) + Sync);

pub fn stage(feed: Feed<'_>, text: impl Into<String>) {
    feed(Progress { kind: "stage", text: text.into() });
}

pub struct CommandResult {
    pub status: ExitStatus,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
}

impl CommandResult {
    pub fn check(self) -> Result<(), String> {
        if self.status.success() { Ok(()) }
        else { Err(self.stderr.or(self.stdout).unwrap_or_else(|| self.status.to_string())) }
    }
}

/// Drain both pipes concurrently, keeping only the last nonblank line from each.
pub fn stream(command: &mut Command, input: Option<&[u8]>, feed: Feed<'_>) -> io::Result<CommandResult> {
    let mut child = command
        .stdin(if input.is_some() { Stdio::piped() } else { Stdio::null() })
        .stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
    let mut stdin = child.stdin.take();
    let stdout = child.stdout.take().expect("piped stdout");
    let stderr = child.stderr.take().expect("piped stderr");
    // Keep callbacks serialized, even as both pipes are draining.
    let feed = Mutex::new(feed);
    std::thread::scope(|scope| {
        let out = scope.spawn(|| drain(stdout, "stdout", &feed));
        let err = scope.spawn(|| drain(stderr, "stderr", &feed));
        let written = input.map_or(Ok(()), |bytes| stdin.as_mut().expect("piped stdin").write_all(bytes));
        drop(stdin);
        let status = child.wait();
        let stdout = out.join().expect("stdout reader")?;
        let stderr = err.join().expect("stderr reader")?;
        let status = status?;
        if status.success() { written?; }
        Ok(CommandResult { status, stdout, stderr })
    })
}

fn drain(pipe: impl Read, kind: &'static str, feed: &Mutex<Feed<'_>>) -> io::Result<Option<String>> {
    let mut reader = BufReader::new(pipe);
    let mut line = Vec::new();
    let mut last = None;
    loop {
        // Treat CR (CLI progress redraws) as a line break and cap records before IPC.
        let bytes = reader.fill_buf()?;
        let end = bytes.iter().position(|b| matches!(b, b'\r' | b'\n'));
        let len = end.unwrap_or(bytes.len());
        line.extend_from_slice(&bytes[..len.min(4096 - line.len())]);
        let eof = bytes.is_empty();
        reader.consume(len + usize::from(end.is_some()));
        if eof || end.is_some() {
            let text = String::from_utf8_lossy(&line).into_owned();
            if !text.trim().is_empty() {
                last = Some(text.clone());
                feed.lock().expect("setup feed lock")(Progress { kind, text });
            }
            line.clear();
        }
        if eof { return Ok(last); }
    }
}

fn probe(program: &str, args: &[&str], feed: Feed<'_>) -> bool {
    stage(feed, format!("$ {program} {}", args.join(" ")));
    match stream(Command::new(program).args(args), None, feed) {
        Ok(out) => out.status.success(),
        Err(e) => {
            stage(feed, format!("No se pudo ejecutar {program}: {e}"));
            false
        }
    }
}

/// Probes the tools (not the emulator itself, which needs an HTTPS call).
pub fn probe_tools(feed: Feed<'_>) -> (bool, bool, bool) {
    let docker = probe("docker", &["--version"], feed);
    let running = docker && probe("docker", &["info", "--format", "{{.ServerVersion}}"], feed);
    (docker, running, probe("aws", &["--version"], feed))
}

pub fn has_brew(feed: Feed<'_>) -> bool {
    !matches!(Os::current(), Os::Windows) && probe("brew", &["--version"], feed)
}

pub fn docker_running() -> bool {
    Command::new("docker").arg("info").stdout(Stdio::null()).stderr(Stdio::null()).status().is_ok_and(|s| s.success())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn streams_both_pipes_and_keeps_the_child_failure() {
        let events = Mutex::new(Vec::new());
        let feed = |event: Progress| events.lock().unwrap().push(event);
        let result = stream(
            Command::new("sh").args(["-c", "cat; printf 'uno\\rdos' >&2; exit 7"]),
            Some(b"desde stdin\n"),
            &feed,
        ).unwrap();
        assert_eq!(result.status.code(), Some(7));
        assert_eq!(result.check(), Err("dos".into()));
        let events = events.into_inner().unwrap();
        let lines = |kind| events.iter().filter(|e| e.kind == kind).map(|e| e.text.as_str()).collect::<Vec<_>>();
        assert_eq!(lines("stdout"), ["desde stdin"]);
        assert_eq!(lines("stderr"), ["uno", "dos"]);
    }

    #[test]
    fn needs_list_what_is_missing_in_fixing_order() {
        let all = Report { docker: true, docker_running: true, aws_cli: true, emulator: true };
        assert!(all.ready());
        assert_eq!(Report::default().needs(), vec![Need::Docker, Need::AwsCli, Need::Emulator], "no daemon step without Docker");
        let stopped = Report { docker: true, aws_cli: true, ..Report::default() };
        assert_eq!(stopped.needs(), vec![Need::DockerRunning, Need::Emulator]);
        assert_eq!(Report { emulator: false, ..all }.needs(), vec![Need::Emulator]);
    }

    #[test]
    fn each_need_has_a_fix_for_each_os() {
        assert_eq!(action(Need::Docker, Os::Mac, true), Action::Run { program: "brew", args: vec!["install", "--cask", "docker"] });
        assert_eq!(action(Need::Docker, Os::Mac, false), Action::OpenUrl("https://www.docker.com/products/docker-desktop/"));
        assert_eq!(action(Need::Docker, Os::Windows, false), Action::Run { program: "winget", args: vec!["install", "-e", "--id", "Docker.DockerDesktop"] });
        assert_eq!(action(Need::Docker, Os::Linux, false), Action::OpenUrl("https://docs.docker.com/engine/install/"));
        assert_eq!(action(Need::DockerRunning, Os::Mac, false), Action::Run { program: "open", args: vec!["-a", "Docker"] });
        assert!(matches!(action(Need::DockerRunning, Os::Linux, false), Action::Manual(_)));
        assert_eq!(action(Need::AwsCli, Os::Mac, true), Action::Run { program: "brew", args: vec!["install", "awscli"] });
        assert_eq!(action(Need::AwsCli, Os::Windows, false), Action::Run { program: "winget", args: vec!["install", "-e", "--id", "Amazon.AWSCLI"] });
        assert_eq!(action(Need::AwsCli, Os::Linux, false), Action::OpenUrl("https://docs.aws.amazon.com/cli/latest/userguide/getting-started-install.html"));
        assert_eq!(action(Need::Emulator, Os::Linux, false), Action::StartEmulator);
    }
}
