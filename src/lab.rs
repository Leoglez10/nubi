//! One lab attempt: what the student typed, what the terminal shows, which objectives are
//! done and how many errors they made. Pure logic; the UI and its worker drive it.

use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::course::LabFile;

#[derive(Debug, Clone, PartialEq)]
pub enum Input {
    Empty,
    Help,
    Hint,
    Clear,
    /// Arguments after `aws`.
    Aws(Vec<String>),
    /// Not allowed here; the message says why.
    Rejected(String),
}

/// Flags that would point a command away from the local emulator.
const FORBIDDEN: [&str; 2] = ["--endpoint-url", "--profile"];

// ponytail: whitespace split, no quotes; add a shell-words parser when a lab needs quoted args.
pub fn parse(line: &str) -> Input {
    let mut words = line.split_whitespace();
    match words.next() {
        None => Input::Empty,
        Some("ayuda" | "help") => Input::Help,
        Some("pista" | "hint") => Input::Hint,
        Some("limpiar" | "clear") => Input::Clear,
        Some("aws") => {
            let args: Vec<String> = words.map(String::from).collect();
            let leaves = args.iter().find(|a| FORBIDDEN.iter().any(|f| a == f || a.starts_with(&format!("{f}="))));
            match leaves {
                Some(flag) => Input::Rejected(format!("{} no está permitido: aquí todo corre en tu emulador local.", flag.split('=').next().unwrap_or(flag))),
                None => Input::Aws(args),
            }
        }
        Some(_) => Input::Rejected("Solo se permiten comandos aws aquí. Escribe \"ayuda\".".into()),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Cmd,
    Out,
    Err,
    Ok,
    Info,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub kind: Kind,
    pub text: String,
}

/// A command that failed and what AWS answered, shown back as learning.
#[derive(Debug, Clone, PartialEq)]
pub struct Mistake {
    pub command: String,
    pub error: String,
}

pub struct Session {
    pub lines: Vec<Line>,
    pub done: Vec<bool>,
    pub errors: u32,
    pub mistakes: Vec<Mistake>,
    history: Vec<String>,
    cursor: Option<usize>,
    pending: Option<String>,
}

impl Session {
    pub fn new(objectives: usize) -> Session {
        Session {
            lines: vec![
                info("Nubi · terminal conectada a tu AWS Emulator local"),
                info("Escribe \"ayuda\" para ver qué puedes hacer, o \"pista\" si te atoras."),
            ],
            done: vec![false; objectives],
            errors: 0,
            mistakes: Vec::new(),
            history: Vec::new(),
            cursor: None,
            pending: None,
        }
    }

    /// Handles one typed line. Returns the `aws` arguments to run, if any; the caller runs
    /// them and reports back with [`Session::finish_run`].
    pub fn submit(&mut self, line: &str, hints: &[String]) -> Option<Vec<String>> {
        let line = line.split_whitespace().collect::<Vec<_>>().join(" ");
        let input = parse(&line);
        if input == Input::Empty {
            return None;
        }
        self.cursor = None;
        if input == Input::Clear {
            self.lines.clear();
            return None;
        }
        self.history.push(line.clone());
        self.lines.push(Line { kind: Kind::Cmd, text: line.clone() });
        match input {
            Input::Help => {
                self.lines.push(info("Escribe comandos aws reales: se ejecutan contra tu emulador local."));
                self.lines.push(info("Extras: pista · limpiar · ↑/↓ para repetir comandos."));
            }
            Input::Hint => {
                let hint = self.next_objective().and_then(|i| hints.get(i));
                self.lines.push(info(&hint.map_or("¡Ya completaste todos los objetivos!".to_string(), |h| format!("Pista: {h}"))));
            }
            Input::Rejected(msg) => self.lines.push(Line { kind: Kind::Err, text: msg }),
            Input::Aws(args) => {
                self.pending = Some(line);
                return Some(args);
            }
            Input::Empty | Input::Clear => {}
        }
        None
    }

    pub fn finish_run(&mut self, ok: bool, stdout: &str, stderr: &str) {
        let command = self.pending.take().unwrap_or_default();
        for (text, kind) in [(stdout, Kind::Out), (stderr, Kind::Err)] {
            self.lines.extend(text.lines().filter(|l| !l.trim().is_empty()).map(|l| Line { kind, text: l.to_string() }));
        }
        if !ok {
            self.errors += 1;
            let error = stderr.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("").trim();
            let error = error.strip_prefix("aws: [ERROR]: ").unwrap_or(error).to_string();
            self.mistakes.push(Mistake { command, error });
        }
    }

    /// Applies fresh checker results; newly met objectives get a line in the terminal.
    pub fn update(&mut self, done: &[bool]) {
        for (i, (&now, was)) in done.iter().zip(self.done.iter_mut()).enumerate() {
            if now && !*was {
                self.lines.push(Line { kind: Kind::Ok, text: format!("✓ Objetivo {} completado", i + 1) });
            }
            *was = now;
        }
    }

    pub fn complete(&self) -> bool {
        !self.done.is_empty() && self.done.iter().all(|&d| d)
    }

    pub fn next_objective(&self) -> Option<usize> {
        self.done.iter().position(|&d| !d)
    }

    /// Up arrow: an older command, or `None` when there is none.
    pub fn history_prev(&mut self) -> Option<&str> {
        let i = match self.cursor {
            None => self.history.len().checked_sub(1)?,
            Some(i) => i.saturating_sub(1),
        };
        self.cursor = Some(i);
        self.history.get(i).map(String::as_str)
    }

    /// Down arrow: a newer command, or `None` back at the empty prompt.
    pub fn history_next(&mut self) -> Option<&str> {
        let i = self.cursor? + 1;
        if i >= self.history.len() {
            self.cursor = None;
            return None;
        }
        self.cursor = Some(i);
        self.history.get(i).map(String::as_str)
    }
}

fn info(text: &str) -> Line {
    Line { kind: Kind::Info, text: text.to_string() }
}

/// A fresh folder for one lab's terminal, holding only the lab's starter files.
/// `lab_id` and file names are validated by the content loader.
pub fn sandbox(root: &Path, lab_id: &str, files: &[LabFile]) -> io::Result<PathBuf> {
    let dir = root.join("labs").join(lab_id);
    match std::fs::remove_dir_all(&dir) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => return Err(e),
        _ => {}
    }
    std::fs::create_dir_all(&dir)?;
    for f in files {
        std::fs::write(dir.join(&f.name), &f.content)?;
    }
    Ok(dir)
}

/// Runs `aws <args>` in `cwd` without a shell. Returns (succeeded, stdout, stderr).
pub fn run_aws(args: &[String], cwd: &Path) -> (bool, String, String) {
    match Command::new("aws").args(args).current_dir(cwd).output() {
        Err(e) => (false, String::new(), format!("No se pudo ejecutar aws: {e}")),
        Ok(out) => (
            out.status.success(),
            String::from_utf8_lossy(&out.stdout).into_owned(),
            String::from_utf8_lossy(&out.stderr).into_owned(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hints() -> Vec<String> {
        vec!["crea el bucket".into(), "activa el versionado".into()]
    }

    #[test]
    fn only_aws_commands_and_built_ins_are_accepted() {
        assert_eq!(parse("  aws   s3 ls "), Input::Aws(vec!["s3".into(), "ls".into()]));
        assert_eq!(parse(""), Input::Empty);
        assert_eq!(parse("ayuda"), Input::Help);
        assert_eq!(parse("help"), Input::Help);
        assert_eq!(parse("pista"), Input::Hint);
        assert_eq!(parse("limpiar"), Input::Clear);
        assert_eq!(parse("clear"), Input::Clear);
        assert!(matches!(parse("rm -rf /"), Input::Rejected(_)));
        assert!(matches!(parse("awsx s3 ls"), Input::Rejected(_)));
    }

    #[test]
    fn commands_cannot_leave_the_emulator() {
        assert!(matches!(parse("aws s3 ls --endpoint-url https://s3.amazonaws.com"), Input::Rejected(_)));
        assert!(matches!(parse("aws s3 ls --endpoint-url=https://s3.amazonaws.com"), Input::Rejected(_)));
        assert!(matches!(parse("aws s3 ls --profile prod"), Input::Rejected(_)));
    }

    #[test]
    fn only_failed_aws_commands_count_as_errors_and_become_lessons() {
        let mut s = Session::new(2);
        assert_eq!(s.submit("rm -rf /", &hints()), None);
        assert_eq!(s.submit("ayuda", &hints()), None);
        assert_eq!(s.submit("pista", &hints()), None);
        assert_eq!(s.errors, 0, "rejected input and built-ins are free");
        assert!(s.lines.iter().any(|l| l.kind == Kind::Info && l.text.contains("crea el bucket")), "hint for the next objective");

        let args = s.submit("aws s3api head-bucket --bucket nope", &hints()).unwrap();
        assert_eq!(args, vec!["s3api", "head-bucket", "--bucket", "nope"]);
        s.finish_run(false, "", "\nAn error occurred (404) when calling the HeadBucket operation: Not Found\n");
        assert_eq!(s.errors, 1);
        assert_eq!(s.mistakes, vec![Mistake {
            command: "aws s3api head-bucket --bucket nope".into(),
            error: "An error occurred (404) when calling the HeadBucket operation: Not Found".into(),
        }]);
        assert_eq!(s.lines.last().unwrap().kind, Kind::Err);

        s.submit("aws s3 ls", &hints()).unwrap();
        s.finish_run(true, "2026-11-14 fotos\n", "");
        assert_eq!(s.errors, 1, "a successful command adds no error");
        assert_eq!(s.lines.last().unwrap(), &Line { kind: Kind::Out, text: "2026-11-14 fotos".into() });
    }

    #[test]
    fn clear_empties_the_terminal_but_not_the_score() {
        let mut s = Session::new(1);
        s.submit("aws nope", &hints());
        s.finish_run(false, "", "boom");
        s.submit("limpiar", &hints());
        assert!(s.lines.is_empty());
        assert_eq!(s.errors, 1);
    }

    #[test]
    fn objectives_tick_once_and_complete_the_lab() {
        let mut s = Session::new(2);
        s.update(&[true, false]);
        s.update(&[true, false]);
        let ticks = s.lines.iter().filter(|l| l.kind == Kind::Ok).count();
        assert_eq!(ticks, 1, "an objective is announced once");
        assert_eq!(s.next_objective(), Some(1));
        assert!(!s.complete());
        s.update(&[true, true]);
        assert!(s.complete());
        assert_eq!(s.next_objective(), None);
    }

    #[test]
    fn arrows_walk_the_command_history() {
        let mut s = Session::new(1);
        assert_eq!(s.history_prev(), None);
        s.submit("aws s3 ls", &hints());
        s.submit("aws sts get-caller-identity", &hints());
        assert_eq!(s.history_prev(), Some("aws sts get-caller-identity"));
        assert_eq!(s.history_prev(), Some("aws s3 ls"));
        assert_eq!(s.history_prev(), Some("aws s3 ls"), "stays at the oldest");
        assert_eq!(s.history_next(), Some("aws sts get-caller-identity"));
        assert_eq!(s.history_next(), None, "back to an empty prompt");
    }

    #[test]
    fn each_attempt_gets_a_clean_sandbox_with_the_starter_files() {
        let root = std::env::temp_dir().join(format!("labcloud-sandbox-{}", std::process::id()));
        let files = vec![LabFile { name: "hola.txt".into(), content: "Hola\n".into() }];
        let dir = sandbox(&root, "lab-a", &files).unwrap();
        std::fs::write(dir.join("descarga.txt"), "x").unwrap();
        let again = sandbox(&root, "lab-a", &files).unwrap();
        assert_eq!(again, dir);
        assert_eq!(std::fs::read_to_string(dir.join("hola.txt")).unwrap(), "Hola\n");
        assert!(!dir.join("descarga.txt").exists(), "leftovers from the last attempt are gone");
        let _ = std::fs::remove_dir_all(&root);
    }
}
