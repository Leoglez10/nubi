//! Lab Cloud: learn AWS in Spanish against a local AWS Emulator.
//!
//! The Rune UI holds the state. One worker thread does everything slow — probing tools,
//! starting the emulator, running `aws`, checking objectives, saving progress — and hands
//! results back with `AppHandle::update`.

mod view;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant};

use labcloud::course::{self, Content, Node, NodeKind};
use labcloud::grading::{self, Level};
use labcloud::lab::{self, Session};
use labcloud::progress::{NodeResult, Progress, Store};
use labcloud::setup::{self, Action, Need, Os, Report};
use labcloud::{checker, emulator};
use rune::{App, AppHandle, AppSpec, TextEdit};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Screen {
    Setup,
    Lab,
}

pub enum Job {
    Probe,
    Fix(Need),
    StartLab,
    Run(Vec<String>),
    /// The lab was completed with this many errors: record and save it.
    Finish(u32),
}

pub struct LabState {
    pub title: String,
    /// (objective text, hint) in order.
    pub objectives: Vec<(String, String)>,
    pub session: Session,
    pub input: TextEdit,
    /// A command (or the lab's setup) is in flight; typing waits for it.
    pub running: bool,
    pub best: Option<NodeResult>,
}

impl LabState {
    fn new(node: &Node, best: Option<NodeResult>) -> LabState {
        LabState {
            title: node.title.clone(),
            objectives: node.objectives.iter().map(|o| (o.text.clone(), o.hint.clone())).collect(),
            session: Session::new(node.objectives.len()),
            input: TextEdit::new(""),
            running: true,
            best,
        }
    }
}

pub struct State {
    jobs: Option<Sender<Job>>,
    pub screen: Screen,
    /// `None` while probing.
    pub setup: Option<Report>,
    pub has_brew: bool,
    pub busy: Option<String>,
    pub setup_note: Option<String>,
    pub lab: LabState,
    pub profile_name: String,
    pub level: Level,
    pub total_stars: u32,
}

impl Default for State {
    fn default() -> Self {
        State {
            jobs: None,
            screen: Screen::Setup,
            setup: None,
            has_brew: false,
            busy: None,
            setup_note: None,
            lab: LabState {
                title: String::new(),
                objectives: Vec::new(),
                session: Session::new(0),
                input: TextEdit::new(""),
                running: false,
                best: None,
            },
            profile_name: String::new(),
            level: grading::level(0),
            total_stars: 0,
        }
    }
}

impl State {
    pub fn send(&self, job: Job) {
        if let Some(jobs) = &self.jobs {
            let _ = jobs.send(job);
        }
    }

    /// Replays the lab from scratch: a fresh session, fresh resources, fresh sandbox.
    pub fn restart_lab(&mut self) {
        self.lab.session = Session::new(self.lab.objectives.len());
        self.lab.input = TextEdit::new("");
        self.lab.running = true;
        self.send(Job::StartLab);
    }
}

struct Worker {
    ui: AppHandle<State>,
    rt: tokio::runtime::Runtime,
    content: Arc<Content>,
    store: Store,
    root: PathBuf,
    profile: String,
    /// `None` when the saved progress could not be read: then nothing is saved over it.
    progress: Option<Progress>,
    lab_id: String,
    dir: PathBuf,
}

impl Worker {
    fn node(&self) -> &Node {
        &self.content.nodes[&self.lab_id]
    }

    fn busy(&self, msg: &str) {
        let msg = msg.to_string();
        self.ui.update(move |s, _| {
            s.busy = Some(msg);
            s.setup_note = None;
        });
    }

    fn note(&self, msg: String) {
        self.ui.update(move |s, _| {
            s.busy = None;
            s.setup_note = Some(msg);
        });
    }

    fn handle(&mut self, job: Job) {
        match job {
            Job::Probe => self.probe(),
            Job::Fix(need) => self.fix(need),
            Job::StartLab => {
                let node = self.node();
                match lab::sandbox(&self.root, &node.id, &node.files) {
                    Ok(dir) => self.dir = dir,
                    Err(e) => self.ui.update(move |s, _| s.lab.session.finish_run(false, "", &format!("No se pudo preparar la carpeta del lab: {e}"))),
                }
                let objectives = &self.content.nodes[&self.lab_id].objectives;
                let done = self.rt.block_on(async {
                    let s3 = checker::client().await;
                    checker::reset(&s3, objectives).await;
                    checker::evaluate(&s3, objectives).await
                });
                self.ui.update(move |s, _| {
                    s.lab.session.update(&done);
                    s.lab.running = false;
                });
            }
            Job::Run(args) => {
                let (ok, out, err) = lab::run_aws(&args, &self.dir);
                let objectives = &self.content.nodes[&self.lab_id].objectives;
                let done = self.rt.block_on(async { checker::evaluate(&checker::client().await, objectives).await });
                self.ui.update(move |s, _| {
                    let was_complete = s.lab.session.complete();
                    s.lab.session.finish_run(ok, &out, &err);
                    s.lab.session.update(&done);
                    s.lab.running = false;
                    if !was_complete && s.lab.session.complete() {
                        s.send(Job::Finish(s.lab.session.errors));
                    }
                });
            }
            Job::Finish(errors) => {
                let Some(progress) = self.progress.as_mut() else { return };
                let best = progress.record(&self.lab_id, errors).clone();
                let total = progress.total_stars();
                if let Err(e) = self.store.save_progress(&self.profile, progress) {
                    eprintln!("labcloud: could not save progress: {e}");
                }
                self.ui.update(move |s, _| {
                    s.lab.best = Some(best);
                    s.total_stars = total;
                    s.level = grading::level(total);
                });
            }
        }
    }

    fn probe(&self) {
        let (docker, docker_running, aws_cli) = setup::probe_tools();
        let emulator = docker_running && self.rt.block_on(emulator::healthy());
        let report = Report { docker, docker_running, aws_cli, emulator };
        let brew = setup::has_brew();
        self.ui.update(move |s, _| {
            s.setup = Some(report);
            s.has_brew = brew;
            s.busy = None;
            if report.ready() && s.screen == Screen::Setup {
                s.screen = Screen::Lab;
                s.send(Job::StartLab);
            }
        });
    }

    fn fix(&self, need: Need) {
        match setup::action(need, Os::current(), setup::has_brew()) {
            Action::Run { program, args } => {
                self.busy(if need == Need::DockerRunning { "Abriendo Docker…" } else { "Instalando… esto puede tardar unos minutos." });
                match Command::new(program).args(&args).output() {
                    Ok(out) if out.status.success() => {
                        if need == Need::DockerRunning {
                            wait_until(Duration::from_secs(90), || setup::probe_tools().1);
                        }
                        self.probe();
                    }
                    Ok(out) => self.note(format!("No se pudo completar: {}", last_line(&out.stderr))),
                    Err(e) => self.note(format!("No se pudo ejecutar {program}: {e}")),
                }
            }
            Action::OpenUrl(url) => {
                open_url(url);
                self.note("Abrimos la página oficial de descarga. Cuando termines de instalar, pulsa «Comprobar de nuevo».".into());
            }
            Action::StartEmulator => {
                self.busy("Encendiendo tu nube local… la primera vez descarga el emulador (unos minutos).");
                match emulator::start() {
                    Ok(()) => {
                        let rt = &self.rt;
                        if wait_until(Duration::from_secs(90), || rt.block_on(emulator::healthy())) {
                            self.probe();
                        } else {
                            self.note(format!("El emulador no respondió a tiempo. Revisa: docker logs {}", emulator::CONTAINER));
                        }
                    }
                    Err(e) => self.note(format!("No se pudo iniciar el emulador: {e}")),
                }
            }
            Action::Manual(_) => {}
        }
    }
}

fn wait_until(limit: Duration, mut ok: impl FnMut() -> bool) -> bool {
    let start = Instant::now();
    while start.elapsed() < limit {
        if ok() {
            return true;
        }
        std::thread::sleep(Duration::from_secs(1));
    }
    false
}

fn last_line(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("error desconocido").to_string()
}

fn open_url(url: &str) {
    let _ = match Os::current() {
        Os::Mac => Command::new("open").arg(url).spawn(),
        Os::Windows => Command::new("cmd").args(["/C", "start", "", url]).spawn(),
        Os::Linux => Command::new("xdg-open").arg(url).spawn(),
    };
}

/// The first lab on the map. Until the map screen exists, the app opens it directly.
fn first_lab(content: &Content) -> &Node {
    content.order().into_iter().map(|id| &content.nodes[id]).find(|n| n.kind == NodeKind::Lab).expect("the course has a lab")
}

fn main() {
    emulator::point_at_emulator();

    // ponytail: content is read from the source tree; bundle it with the app in the installers phase.
    let content = match course::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("content")) {
        Ok(c) => Arc::new(c),
        Err(e) => {
            eprintln!("labcloud: invalid course content: {e}");
            std::process::exit(1);
        }
    };
    let root = Store::default_root();
    let store = Store::new(&root);
    // Until the profile screen exists: the first local profile, or a new "Estudiante".
    let profile = match store.profiles().ok().and_then(|p| p.into_iter().next()) {
        Some(p) => p,
        None => store.create_profile("Estudiante", "#BFE0FB").expect("create the local profile"),
    };
    let progress = match store.load_progress(&profile.id) {
        Ok(p) => Some(p),
        Err(e) => {
            eprintln!("labcloud: could not read saved progress, it will not be overwritten: {e}");
            None
        }
    };
    let node = first_lab(&content);
    let total = progress.as_ref().map_or(0, Progress::total_stars);

    let (tx, rx) = mpsc::channel();
    tx.send(Job::Probe).expect("queue the first probe");
    let state = State {
        jobs: Some(tx),
        lab: LabState::new(node, progress.as_ref().and_then(|p| p.nodes.get(&node.id).cloned())),
        profile_name: profile.name.clone(),
        level: grading::level(total),
        total_stars: total,
        ..State::default()
    };
    let lab_id = node.id.clone();

    let spec = AppSpec { scenes: SCENES, ..AppSpec::new("Lab Cloud", (1120.0, 740.0), State::default, view::view) };
    let app = App::new(spec).init(move || state);
    let ui = app.handle();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("tokio runtime");
        let mut worker = Worker { ui, rt, content, store, root, profile: profile.id, progress, lab_id, dir: PathBuf::new() };
        run(&mut worker, rx);
    });
    app.run();
}

/// Headless snapshots: `cargo run -- --snap out.png --scene lab`.
const SCENES: &[(&str, fn(&mut State))] = &[
    ("setup", |s| {
        s.setup = Some(Report { docker: true, docker_running: true, aws_cli: true, emulator: false });
    }),
    ("lab", |s| {
        s.screen = Screen::Lab;
        s.lab.running = false;
        let hints: Vec<String> = s.lab.objectives.iter().map(|(_, h)| h.clone()).collect();
        s.lab.session.submit("pista", &hints);
        s.lab.session.submit("aws s3api create-bucket --bucket mi-primer-bucket", &hints);
        s.lab.session.finish_run(true, "{\n    \"Location\": \"/mi-primer-bucket\"\n}", "");
        s.lab.session.update(&[true, false, false]);
    }),
    ("result", |s| {
        s.screen = Screen::Lab;
        s.lab.running = false;
        s.lab.session.submit("aws s3api put-bucket-versioning --bucket mi-primer-bucket --versioning-configuration Status=Enabled", &[]);
        s.lab.session.finish_run(false, "", "aws: [ERROR]: An error occurred (NoSuchBucket) when calling the PutBucketVersioning operation: The specified bucket does not exist");
        s.lab.session.update(&[true, true, true]);
    }),
];

fn run(worker: &mut Worker, jobs: Receiver<Job>) {
    for job in jobs {
        worker.handle(job);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rune::{Env, Runtime};

    fn content() -> Content {
        course::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("content")).unwrap()
    }

    fn render(state: State) -> Runtime<State> {
        let mut rt = Runtime::new(state, view::view, (1120.0, 740.0), Env { system_dark: false, scale: 1.0 });
        rt.frame(0.0);
        rt
    }

    #[test]
    fn the_setup_screen_offers_a_fix_for_each_missing_tool() {
        let report = Report { docker: false, docker_running: false, aws_cli: false, emulator: false };
        let rt = render(State { setup: Some(report), has_brew: true, ..State::default() });
        assert!(rt.find("fix-Docker").is_some());
        assert!(rt.find("fix-AWS CLI").is_some());
        assert!(rt.find("fix-AWS Emulator").is_none(), "the emulator waits for Docker");
        assert!(rt.find("recheck").is_some());
    }

    #[test]
    fn the_setup_screen_says_it_is_checking_first() {
        let rt = render(State::default());
        assert!(rt.find("probing").is_some());
    }

    #[test]
    fn the_lab_screen_shows_objectives_terminal_and_result() {
        let c = content();
        let node = first_lab(&c);
        let mut lab = LabState::new(node, None);
        lab.running = false;
        let mut rt = render(State { screen: Screen::Lab, lab, ..State::default() });
        assert!(rt.find("input").is_some());
        assert!(rt.find("goal-2").is_some());
        assert!(rt.find("chip-siguiente paso").is_some());
        assert!(rt.find("result").is_none());

        rt.update(|s, _| s.lab.session.update(&[true, true, true]));
        rt.frame(1.0 / 60.0);
        assert!(rt.find("result").is_some());
        assert!(rt.find("again").is_some(), "a perfect run offers a replay");
    }

    #[test]
    fn a_lab_with_errors_offers_to_perfect_it() {
        let c = content();
        let mut lab = LabState::new(first_lab(&c), None);
        lab.running = false;
        lab.session.submit("aws s3 ls", &[]);
        lab.session.finish_run(false, "", "An error occurred");
        lab.session.update(&[true, true, true]);
        let rt = render(State { screen: Screen::Lab, lab, ..State::default() });
        assert!(rt.find("perfect").is_some());
    }
}
