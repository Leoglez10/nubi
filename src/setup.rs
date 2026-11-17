//! Guided setup: what the student's machine is missing before playing, and the one-click
//! action that fixes each item on their OS.

use std::process::{Command, Stdio};

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

/// Probes the tools (not the emulator itself, which needs an HTTPS call).
pub fn probe_tools() -> (bool, bool, bool) {
    let ok = |program: &str, args: &[&str]| {
        Command::new(program).args(args).stdout(Stdio::null()).stderr(Stdio::null()).status().is_ok_and(|s| s.success())
    };
    let docker = ok("docker", &["--version"]);
    let running = docker && ok("docker", &["info"]);
    (docker, running, ok("aws", &["--version"]))
}

pub fn has_brew() -> bool {
    Command::new("brew").arg("--version").stdout(Stdio::null()).stderr(Stdio::null()).status().is_ok_and(|s| s.success())
}

#[cfg(test)]
mod tests {
    use super::*;

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
