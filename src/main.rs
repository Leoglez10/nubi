//! Lab Cloud: learn AWS in Spanish against a local AWS Emulator.
//!
//! The Rune UI holds the state. One worker thread does everything slow — probing tools,
//! starting the emulator, running `aws`, checking objectives, loading and saving profiles —
//! and hands results back with `AppHandle::update`.

mod view;
mod motion;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant};

use labcloud::course::{self, Content, Course, Node, NodeKind};
use labcloud::lab::{self, Session};
use labcloud::lesson::LessonSession;
use labcloud::progress::{Profile, Progress, Store};
use labcloud::setup::{self, Action, Need, Os, Report};
use labcloud::{checker, emulator};
use rune::{App, AppHandle, AppSpec, Image, ImageCache, TextEdit, Tx, Waker};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Screen {
    Splash,
    Setup,
    Profiles,
    Map,
    Class,
    Lesson,
    Lab,
}

pub enum Job {
    Probe,
    Fix(Need),
    LoadProfiles,
    CreateProfile { name: String, color: String },
    SelectProfile(String),
    StartLab(String),
    Run(Vec<String>),
    /// A stop was completed with this many errors: record and save it.
    Finish { id: String, errors: u32 },
}

/// The vector art, rasterized once at startup.
pub struct Art {
    _cache: ImageCache,
    pub nubi: Image,
    pub cloud: Image,
    pub cloud_current: Image,
    pub cloud_locked: Image,
    pub cloud_soft: Image,
    pub icon_home: Image,
    pub icon_book: Image,
    pub icon_check: Image,
    pub icon_term: Image,
}

/// A 24px stroke icon in the sidebar's ink.
fn icon_svg(path: &str) -> String {
    format!(r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#344054" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"><path d="{path}"/></svg>"##)
}

const NUBI: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 120 96"><path d="M30 80h62a22 22 0 0 0 2-44 30 30 0 0 0-57-6A24 24 0 0 0 30 80z" fill="#FFFFFF" stroke="#1F6FD1" stroke-width="4" stroke-linejoin="round"/><circle cx="50" cy="54" r="5" fill="#0F2A44"/><circle cx="74" cy="54" r="5" fill="#0F2A44"/><circle cx="52" cy="52" r="1.6" fill="#FFFFFF"/><circle cx="76" cy="52" r="1.6" fill="#FFFFFF"/><path d="M54 64q8 7 16 0" fill="none" stroke="#0F2A44" stroke-width="3" stroke-linecap="round"/><circle cx="42" cy="64" r="4" fill="#FFB8A8"/><circle cx="82" cy="64" r="4" fill="#FFB8A8"/></svg>"##;

fn cloud_svg(fill: &str, stroke: &str) -> String {
    format!(r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 120 96"><path d="M30 80h62a22 22 0 0 0 2-44 30 30 0 0 0-57-6A24 24 0 0 0 30 80z" fill="{fill}" stroke="{stroke}" stroke-width="4" stroke-linejoin="round"/></svg>"##)
}

fn load_art(s: &mut State, waker: Waker) {
    let cache = ImageCache::new(waker);
    let svg = |text: String, size: f32| cache.svg(text.into_bytes(), size, 2.0);
    s.art = Some(Art {
        nubi: svg(NUBI.to_string(), 200.0),
        cloud: svg(cloud_svg("#FFFFFF", "#1F6FD1"), 120.0),
        cloud_current: svg(cloud_svg("#FFFFFF", "#FF9900"), 120.0),
        cloud_locked: svg(cloud_svg("#EEF3F8", "#C3D0DE"), 120.0),
        cloud_soft: svg(cloud_svg("#FFFFFF", "#FFFFFF"), 300.0),
        icon_home: svg(icon_svg("M3 10.5L12 3l9 7.5V20a1 1 0 0 1-1 1h-5v-6H9v6H4a1 1 0 0 1-1-1z"), 18.0),
        icon_book: svg(icon_svg("M4 5a2 2 0 0 1 2-2h13v16H6a2 2 0 0 0-2 2zM4 5v16M8 7h7"), 18.0),
        icon_check: svg(icon_svg("M9 11l3 3 8-8M20 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11"), 18.0),
        icon_term: svg(icon_svg("M4 17l6-5-6-5M12 19h8"), 18.0),
        _cache: cache,
    });
}

/// The exercise being answered and the student's in-progress answer.
#[derive(Default)]
pub struct LessonView {
    pub id: String,
    pub session: LessonSession,
    pub choice: Option<usize>,
    pub command: TextEdit,
    /// Display positions picked so far, in order (Order exercises).
    pub picked: Vec<usize>,
    /// Selected left item waiting for its right partner (Match exercises).
    pub left: Option<usize>,
    /// `(left, right)` pairs in content indices (Match exercises).
    pub pairs: Vec<(usize, usize)>,
    pub saved: bool,
}

impl LessonView {
    pub fn new(node: &Node) -> LessonView {
        LessonView { id: node.id.clone(), session: LessonSession::new(node.exercises.len()), ..LessonView::default() }
    }

    /// Clears the answer for the next exercise or a retry.
    pub fn clear_answer(&mut self) {
        self.choice = None;
        self.command = TextEdit::new("");
        self.picked.clear();
        self.left = None;
        self.pairs.clear();
    }
}

pub struct LabState {
    pub id: String,
    pub title: String,
    /// (objective text, hint) in order.
    pub objectives: Vec<(String, String)>,
    pub session: Session,
    pub input: TextEdit,
    /// A command (or the lab's setup) is in flight; typing waits for it.
    pub running: bool,
    pub saved: bool,
}

impl LabState {
    pub fn new(node: &Node) -> LabState {
        LabState {
            id: node.id.clone(),
            title: node.title.clone(),
            objectives: node.objectives.iter().map(|o| (o.text.clone(), o.hint.clone())).collect(),
            session: Session::new(node.objectives.len()),
            input: TextEdit::new(""),
            running: true,
            saved: false,
        }
    }
}

pub struct ProfileCard {
    pub profile: Profile,
    pub stars: u32,
}

pub struct State {
    jobs: Option<Sender<Job>>,
    pub content: Arc<Content>,
    pub art: Option<Art>,
    pub screen: Screen,
    pub motion: motion::Motion,
    pub reduced_motion: bool,
    /// Set on the first poll, so the intro's pieces enter with motion (nothing animates on
    /// the first frame).
    pub intro_started: bool,
    /// `None` while probing.
    pub setup: Option<Report>,
    pub has_brew: bool,
    pub busy: Option<String>,
    pub setup_note: Option<String>,
    /// `None` while loading.
    pub profiles: Option<Vec<ProfileCard>>,
    pub creating: bool,
    pub new_name: TextEdit,
    pub new_color: usize,
    pub profile: Option<Profile>,
    pub progress: Progress,
    /// Selected stop on the map, by position in the course order.
    pub selected: usize,
    pub class_id: Option<String>,
    pub class_return: Option<Screen>,
    pub class_review: bool,
    pub lesson: LessonView,
    pub lab: LabState,
}

impl Default for State {
    fn default() -> Self {
        let empty = Content { course: Course { units: Vec::new() }, nodes: BTreeMap::new() };
        State {
            jobs: None,
            content: Arc::new(empty),
            art: None,
            screen: Screen::Splash,
            motion: motion::Motion::default(),
            reduced_motion: false,
            intro_started: false,
            setup: None,
            has_brew: false,
            busy: None,
            setup_note: None,
            profiles: None,
            creating: false,
            new_name: TextEdit::new(""),
            new_color: 0,
            profile: None,
            progress: Progress::default(),
            selected: 0,
            class_id: None,
            class_return: None,
            class_review: false,
            lesson: LessonView::default(),
            lab: LabState { id: String::new(), title: String::new(), objectives: Vec::new(), session: Session::new(0), input: TextEdit::new(""), running: false, saved: false },
        }
    }
}

pub const AVATAR_COLORS: [(&str, &str); 5] = [
    ("#BFE0FB", "Azul cielo"),
    ("#FFE2B8", "Atardecer"),
    ("#CFF2DF", "Menta"),
    ("#E7DBFF", "Lavanda"),
    ("#FFFFFF", "Blanco"),
];

impl State {
    pub fn send(&self, job: Job) {
        if let Some(jobs) = &self.jobs {
            let _ = jobs.send(job);
        }
    }

    pub fn order(&self) -> Vec<&str> {
        self.content.order()
    }

    /// The first stop that is not done yet (or the last stop).
    pub fn current_index(&self) -> usize {
        let order = self.order();
        order.iter().position(|id| !self.progress.completed(id)).unwrap_or(order.len().saturating_sub(1))
    }

    /// Opens a stop from the map: a lesson screen or a lab (with fresh resources).
    pub fn open_stop(&mut self, id: &str) {
        let Some(node) = self.content.nodes.get(id) else { return };
        match node.kind {
            NodeKind::Lab => {
                self.lab = LabState::new(node);
                self.screen = Screen::Lab;
                self.send(Job::StartLab(node.id.clone()));
            }
            NodeKind::Lesson | NodeKind::Challenge => {
                self.lesson = LessonView::new(node);
                self.screen = Screen::Lesson;
            }
        }
    }

    /// Map entry always presents the teaching before launching a practice.
    pub fn open_class(&mut self, id: &str) {
        let found = self.content.classes().into_iter().find(|(node, t)| node.id == id || t.lab == id)
            .map(|(node, _)| node.id.clone());
        let Some(id) = found else { return };
        if !labcloud::grading::is_unlocked(&self.order(), &id, |prev| self.progress.completed(prev)) { return; }
        self.class_id = Some(id);
        self.class_return = None;
        self.class_review = false;
        self.screen = Screen::Class;
    }

    pub fn class_lab_ready(&self) -> bool {
        self.class_id.as_ref().is_some_and(|id| self.progress.completed(id)
            || (self.lesson.id == *id && self.lesson.session.finished()))
    }

    pub fn start_class_lesson(&mut self) {
        if self.class_return.is_some() { return; }
        if let Some(id) = self.class_id.clone() { self.open_stop(&id); }
    }

    pub fn start_class_lab(&mut self) {
        if self.class_return.is_some() || !self.class_lab_ready() { return; }
        let lab = self.class_id.as_ref().and_then(|id| self.content.nodes.get(id))
            .and_then(|node| node.teaching.as_ref()).map(|t| t.lab.clone());
        if let Some(id) = lab { self.open_stop(&id); }
    }

    pub fn revisit_class(&mut self) {
        let id = match self.screen { Screen::Lesson => &self.lesson.id, Screen::Lab => &self.lab.id, _ => return };
        let found = self.content.classes().into_iter().find(|(node, t)| node.id == *id || t.lab == *id)
            .map(|(node, _)| node.id.clone());
        if let Some(id) = found {
            self.class_id = Some(id);
            self.class_return = Some(self.screen);
            self.class_review = false;
            self.screen = Screen::Class;
        }
    }

    pub fn return_to_attempt(&mut self) {
        if let Some(screen) = self.class_return.take() { self.screen = screen; }
    }

    pub fn lesson_to_lab(&mut self) {
        if self.screen != Screen::Lesson || !self.lesson.session.finished() { return; }
        self.revisit_class();
        if self.screen == Screen::Class {
            self.class_return = None;
            self.start_class_lab();
        }
    }

    pub fn show_class_review(&mut self) {
        self.revisit_class();
        if self.screen == Screen::Class { self.class_review = true; }
    }

    /// The class to work on next: the first whose lab is not done (or the last one).
    pub fn current_class(&self) -> Option<(String, String)> {
        let classes = self.content.classes();
        classes.iter().find(|(_, t)| !self.progress.completed(&t.lab)).or(classes.last())
            .map(|(node, t)| (node.id.clone(), t.lab.clone()))
    }

    /// Number and title of the class the current screen belongs to.
    pub fn shown_class(&self) -> Option<(usize, String)> {
        let id = match self.screen {
            Screen::Class => self.class_id.as_deref()?,
            Screen::Lesson => &self.lesson.id,
            Screen::Lab => &self.lab.id,
            _ => return None,
        };
        self.content.classes().into_iter().find(|(node, t)| node.id == id || t.lab == id)
            .map(|(_, t)| (t.number, t.title.clone()))
    }

    /// Sidebar navigation: resumes an attempt in progress instead of restarting it.
    pub fn go_to(&mut self, target: Screen) {
        if target == self.screen { return; }
        if target == Screen::Map { return self.to_map(); }
        let Some((lesson, lab)) = self.current_class() else { return };
        match target {
            Screen::Lesson if self.lesson.id == lesson && !self.lesson.session.finished() => self.screen = Screen::Lesson,
            Screen::Lab if self.lab.id == lab && !self.lab.session.complete() => self.screen = Screen::Lab,
            _ => {
                self.open_class(&lesson);
                match target {
                    Screen::Lesson => self.start_class_lesson(),
                    Screen::Lab => self.start_class_lab(),
                    _ => {}
                }
            }
        }
    }

    pub fn to_map(&mut self) {
        self.selected = self.current_index();
        self.screen = Screen::Map;
    }

    /// Replays the lab from scratch: a fresh session, fresh resources, fresh sandbox.
    pub fn restart_lab(&mut self) {
        let id = self.lab.id.clone();
        self.open_stop(&id);
    }

    /// Leaves the intro: to setup when something is missing, else to the profiles.
    pub fn enter(&mut self) {
        self.screen = match &self.setup {
            Some(r) if r.ready() => Screen::Profiles,
            _ => Screen::Setup,
        };
    }
}

fn poll(s: &mut State, now: f64, _: &mut Tx) -> bool {
    let changed = motion::tick(s, now);
    if s.intro_started {
        return changed;
    }
    s.intro_started = true;
    true
}

struct Worker {
    ui: AppHandle<State>,
    rt: tokio::runtime::Runtime,
    content: Arc<Content>,
    store: Store,
    root: PathBuf,
    profile: Option<String>,
    /// `None` when the saved progress could not be read: then nothing is saved over it.
    progress: Option<Progress>,
    /// The lab being played and its terminal folder.
    lab_id: String,
    dir: PathBuf,
}

impl Worker {
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
            Job::LoadProfiles => self.load_profiles(),
            Job::CreateProfile { name, color } => match self.store.create_profile(&name, &color) {
                Ok(p) => self.select(p.id),
                Err(e) => self.ui.update(move |s, _| s.setup_note = Some(format!("No se pudo crear el perfil: {e}"))),
            },
            Job::SelectProfile(id) => self.select(id),
            Job::StartLab(id) => self.start_lab(id),
            Job::Run(args) => {
                let (ok, out, err) = lab::run_aws(&args, &self.dir);
                let done = self.check();
                self.ui.update(move |s, _| {
                    s.lab.session.finish_run(ok, &out, &err);
                    s.lab.session.update(&done);
                    s.lab.running = false;
                    if s.lab.session.complete() && !s.lab.saved {
                        s.lab.saved = true;
                        s.send(Job::Finish { id: s.lab.id.clone(), errors: s.lab.session.errors });
                    }
                });
            }
            Job::Finish { id, errors } => {
                let (Some(profile), Some(progress)) = (self.profile.as_ref(), self.progress.as_mut()) else { return };
                progress.record(&id, errors);
                if let Err(e) = self.store.save_progress(profile, progress) {
                    eprintln!("labcloud: could not save progress: {e}");
                }
                let progress = progress.clone();
                self.ui.update(move |s, _| s.progress = progress);
            }
        }
    }

    fn check(&self) -> Vec<bool> {
        let Some(node) = self.content.nodes.get(&self.lab_id) else { return Vec::new() };
        self.rt.block_on(async { checker::evaluate(&checker::client().await, &node.objectives).await })
    }

    fn start_lab(&mut self, id: String) {
        let Some(node) = self.content.nodes.get(&id) else { return };
        match lab::sandbox(&self.root, &node.id, &node.files) {
            Ok(dir) => self.dir = dir,
            Err(e) => self.ui.update(move |s, _| s.lab.session.finish_run(false, "", &format!("No se pudo preparar la carpeta del lab: {e}"))),
        }
        let done = self.rt.block_on(async {
            let s3 = checker::client().await;
            checker::reset(&s3, &node.objectives).await;
            checker::evaluate(&s3, &node.objectives).await
        });
        self.lab_id = id;
        self.ui.update(move |s, _| {
            s.lab.session.update(&done);
            s.lab.running = false;
        });
    }

    fn load_profiles(&self) {
        let cards: Vec<ProfileCard> = self
            .store
            .profiles()
            .unwrap_or_default()
            .into_iter()
            .map(|p| {
                let stars = self.store.load_progress(&p.id).map_or(0, |pr| pr.total_stars());
                ProfileCard { profile: p, stars }
            })
            .collect();
        self.ui.update(move |s, _| {
            s.creating = cards.is_empty();
            s.profiles = Some(cards);
        });
    }

    fn select(&mut self, id: String) {
        let profile = self.store.profiles().unwrap_or_default().into_iter().find(|p| p.id == id);
        let Some(profile) = profile else { return };
        let progress = match self.store.load_progress(&id) {
            Ok(p) => Some(p),
            Err(e) => {
                eprintln!("labcloud: could not read saved progress, it will not be overwritten: {e}");
                None
            }
        };
        let shown = progress.clone().unwrap_or_default();
        self.profile = Some(id);
        self.progress = progress;
        self.ui.update(move |s, _| {
            s.profile = Some(profile);
            s.progress = shown;
            // A completed in-memory lesson must not unlock labs for a different profile.
            s.lesson = LessonView::default();
            s.class_id = None;
            s.class_return = None;
            s.class_review = false;
            s.creating = false;
            s.new_name = TextEdit::new("");
            s.to_map();
        });
        self.load_profiles();
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
                s.screen = Screen::Profiles;
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

fn load_content() -> Content {
    // ponytail: content is read from the source tree; bundle it with the app in the installers phase.
    match course::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("content")) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("labcloud: invalid course content: {e}");
            std::process::exit(1);
        }
    }
}

type Scene = (&'static str, fn(&mut State));

/// Headless snapshots: `cargo run -- --snap out.png --scene <name>`.
const SCENES: &[Scene] = &[
    ("intro", |s| {
        s.intro_started = true;
        s.setup = Some(Report { docker: true, docker_running: true, aws_cli: true, emulator: true });
        s.profiles = Some(Vec::new());
    }),
    ("setup", |s| {
        s.screen = Screen::Setup;
        s.setup = Some(Report { docker: true, docker_running: true, aws_cli: true, emulator: false });
    }),
    ("profiles", |s| {
        s.screen = Screen::Profiles;
        s.profiles = Some(vec![
            ProfileCard { profile: Profile { id: "alex".into(), name: "Alex".into(), color: "#BFE0FB".into() }, stars: 10 },
            ProfileCard { profile: Profile { id: "sam".into(), name: "Sam".into(), color: "#FFE2B8".into() }, stars: 3 },
        ]);
    }),
    ("map", |s| {
        s.profile = Some(Profile { id: "alex".into(), name: "Alex".into(), color: "#BFE0FB".into() });
        s.progress.record("s3-buckets", 1);
        s.to_map();
    }),
    ("class", |s| {
        s.open_class("s3-buckets");
    }),
    ("class-2", |s| {
        s.progress.record("s3-buckets", 0);
        s.progress.record("lab-primer-bucket", 0);
        s.open_class("s3-objetos");
    }),
    ("lesson", |s| {
        s.open_stop("s3-buckets");
        s.lesson.choice = Some(1);
    }),
    ("lesson-wrong", |s| {
        s.open_stop("s3-buckets");
        let content = s.content.clone();
        s.lesson.choice = Some(0);
        s.lesson.session.check(&content.nodes["s3-buckets"].exercises[0], &labcloud::grading::Answer::Choice(0));
    }),
    ("lesson-order", |s| {
        s.open_stop("s3-buckets");
        s.lesson.session.index = 3;
        s.lesson.picked = vec![2];
    }),
    ("lesson-match", |s| {
        s.open_stop("s3-buckets");
        s.lesson.session.index = 4;
        s.lesson.pairs = vec![(0, 0)];
        s.lesson.left = Some(1);
    }),
    ("lesson-done", |s| {
        s.open_stop("s3-buckets");
        s.lesson.session = LessonSession { index: 5, total: 5, errors: 1, mistakes: vec!["¿Qué identifica a un objeto dentro de un bucket?".into()], feedback: None };
    }),
    ("lab", |s| {
        s.open_stop("lab-primer-bucket");
        s.lab.running = false;
        let hints: Vec<String> = s.lab.objectives.iter().map(|(_, h)| h.clone()).collect();
        s.lab.session.submit("aws s3api create-bucket --bucket mi-primer-bucket", &hints);
        s.lab.session.finish_run(true, "{\n    \"Location\": \"/mi-primer-bucket\"\n}", "");
        s.lab.session.update(&[true, false, false]);
    }),
];

fn main() {
    emulator::point_at_emulator();
    let content = Arc::new(load_content());
    let root = Store::default_root();
    let store = Store::new(&root);

    let (tx, rx) = mpsc::channel();
    tx.send(Job::Probe).expect("queue the first probe");
    tx.send(Job::LoadProfiles).expect("queue profile loading");
    let state_content = content.clone();
    let spec = AppSpec {
        scenes: SCENES,
        setup: Some(load_art),
        poll: Some(poll),
        min_size: Some((960.0, 680.0)),
        ..AppSpec::new("Lab Cloud", (1180.0, 760.0), State::default, view::view)
    };
    let app = App::new(spec).init(move || State { jobs: Some(tx), content: state_content, ..State::default() });
    let ui = app.handle();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("tokio runtime");
        let mut worker = Worker { ui, rt, content, store, root, profile: None, progress: None, lab_id: String::new(), dir: PathBuf::new() };
        run(&mut worker, rx);
    });
    app.run();
}

fn run(worker: &mut Worker, jobs: Receiver<Job>) {
    for job in jobs {
        worker.handle(job);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use labcloud::grading::Answer;
    use rune::{Env, Runtime};

    fn state(scene: &str) -> State {
        let mut s = State { content: Arc::new(load_content()), ..State::default() };
        let (_, apply) = SCENES.iter().find(|(name, _)| *name == scene).expect("scene exists");
        apply(&mut s);
        s
    }

    fn render(s: State) -> Runtime<State> {
        let mut rt = Runtime::new(s, view::view, (1180.0, 760.0), Env { system_dark: false, scale: 1.0 });
        rt.frame(0.0);
        rt.frame(1.0 / 60.0);
        rt
    }

    #[test]
    fn every_class_section_has_a_picture() {
        for scene in ["class", "class-2"] {
            let rt = render(state(scene));
            for key in ["class-diagram", "class-example-visual", "class-review-visual"] {
                assert!(rt.find(key).is_some(), "{scene}: missing {key}");
            }
        }
    }

    #[test]
    fn keyboard_alone_starts_the_games_from_a_class() {
        let mut rt = render(state("class"));
        rt.set_tab_visits_controls(true);
        let on_games = std::rc::Rc::new(std::cell::Cell::new(false));
        // Tab through the sidebar and the page until the games button has focus.
        for _ in 0..30 {
            rt.key(rune::Key::Tab.into());
            rt.frame(1.0 / 60.0);
            let flag = on_games.clone();
            rt.update(move |_, tx| flag.set(tx.focused() == Some("class-games")));
            if on_games.get() { break; }
        }
        assert!(on_games.get(), "Tab must reach the games button");
        rt.key(rune::Key::Text(" ".into()).into());
        rt.frame(1.0 / 60.0);
        assert_eq!(rt.state.screen, Screen::Lesson);
        assert_eq!(rt.state.lesson.id, "s3-buckets");
    }

    #[test]
    fn class_gates_practice_and_restores_attempts() {
        let mut s = state("map");
        s.progress = Progress::default();
        s.open_class("s3-buckets");
        assert_eq!(s.screen, Screen::Class);
        s.start_class_lab();
        assert_eq!(s.screen, Screen::Class, "lab requires a completed lesson");
        s.start_class_lesson();
        assert_eq!(s.screen, Screen::Lesson);
        s.lesson.choice = Some(1);
        s.lesson.session.errors = 2;
        s.revisit_class();
        s.return_to_attempt();
        assert_eq!(s.screen, Screen::Lesson);
        assert_eq!(s.lesson.choice, Some(1));
        assert_eq!(s.lesson.session.errors, 2);
    }

    #[test]
    fn class_result_flow_and_lab_revisit_preserve_attempt() {
        let mut s = state("map");
        s.progress = Progress::default();
        s.open_class("s3-objetos");
        assert_eq!(s.screen, Screen::Map, "second class requires the previous lab");
        s.open_class("lab-primer-bucket");
        assert_eq!(s.screen, Screen::Class);
        s.start_class_lesson();
        s.lesson.session.index = s.lesson.session.total;
        let (tx, rx) = mpsc::channel();
        s.jobs = Some(tx);
        s.lesson_to_lab();
        assert_eq!(s.screen, Screen::Lab);
        assert!(matches!(rx.try_recv(), Ok(Job::StartLab(id)) if id == "lab-primer-bucket"));
        s.lab.input = TextEdit::new("aws s3 ls");
        s.lab.session.errors = 2;
        s.progress.record("s3-buckets", 1);
        let before = s.progress.clone();
        s.revisit_class();
        assert_eq!(s.screen, Screen::Class);
        s.start_class_lab();
        assert!(rx.try_recv().is_err(), "revisit cannot start a new lab");
        s.return_to_attempt();
        assert_eq!(s.screen, Screen::Lab);
        assert_eq!(s.lab.input.text, "aws s3 ls");
        assert_eq!(s.lab.session.errors, 2);
        assert_eq!(s.progress, before);
        s.lab.session.done.fill(true);
        s.show_class_review();
        assert_eq!(s.screen, Screen::Class);
        assert!(s.class_review);
        s.return_to_attempt();
        assert!(s.lab.session.complete());
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn class_teaching_and_revisit_controls_render() {
        let rt = render(state("class"));
        assert!(rt.find("class-title").is_some());
        assert!(rt.find("class-diagram").is_some());
        assert!(rt.find("diagram-bucket").is_some());
        assert!(rt.find("class-games").is_some());
        assert!(rt.find("class-lab").is_some());
        assert!(render(state("lesson")).find("lesson-explanation").is_some());
        assert!(render(state("lab")).find("lab-explanation").is_some());
        assert!(render(state("lesson-done")).find("lesson-to-lab").is_some());
    }

    #[test]
    fn the_intro_offers_to_enter_once_checked() {
        let rt = render(state("intro"));
        assert!(rt.find("enter").is_some());
        assert!(rt.find("check-emulator").is_some());
    }

    #[test]
    fn the_setup_screen_offers_a_fix_for_each_missing_tool() {
        let report = Report { docker: false, docker_running: false, aws_cli: false, emulator: false };
        let rt = render(State { screen: Screen::Setup, setup: Some(report), has_brew: true, ..State::default() });
        assert!(rt.find("fix-Docker").is_some());
        assert!(rt.find("fix-AWS CLI").is_some());
        assert!(rt.find("fix-AWS Emulator").is_none(), "the emulator waits for Docker");
        assert!(rt.find("recheck").is_some());
    }

    #[test]
    fn profiles_can_be_picked_or_created() {
        let mut rt = render(state("profiles"));
        assert!(rt.find("profile-alex").is_some());
        assert!(rt.find("new-profile").is_some());
        assert!(rt.find("profile-form").is_none());
        rt.update(|s, _| s.creating = true);
        rt.frame(2.0 / 60.0);
        assert!(rt.find("profile-form").is_some());
        assert!(rt.find("color-0").is_some());
    }

    #[test]
    fn home_lists_every_class_and_the_one_to_continue() {
        let rt = render(state("map"));
        assert!(rt.find("class-1").is_some());
        assert!(rt.find("class-2").is_some());
        assert!(rt.find("continue").is_some());
        assert!(rt.find("continue-class").is_some());
    }

    #[test]
    fn opening_stops_goes_to_the_right_screen() {
        let mut s = state("map");
        s.open_stop("s3-buckets");
        assert_eq!(s.screen, Screen::Lesson);
        assert_eq!(s.lesson.session.total, 5);
        s.open_stop("lab-primer-bucket");
        assert_eq!(s.screen, Screen::Lab);
        assert_eq!(s.lab.objectives.len(), 3);
    }

    #[test]
    fn every_exercise_kind_renders_its_answer_area() {
        for (scene, key) in [("lesson", "option-1"), ("lesson-order", "step-0"), ("lesson-match", "right-0")] {
            let rt = render(state(scene));
            assert!(rt.find(key).is_some(), "{scene}: {key}");
            assert!(rt.find("check").is_some(), "{scene}: check button");
        }
        let mut s = state("lesson");
        s.lesson.session.index = 2;
        let rt = render(s);
        assert!(rt.find("command").is_some());
    }

    #[test]
    fn a_wrong_answer_shows_the_hint_and_a_retry() {
        let rt = render(state("lesson-wrong"));
        assert!(rt.find("feedback-wrong").is_some());
        assert!(rt.find("retry").is_some());
    }

    #[test]
    fn the_answer_is_read_back_in_content_indices() {
        let c = load_content();
        let exs = &c.nodes["s3-buckets"].exercises;
        let mut l = LessonView::new(&c.nodes["s3-buckets"]);
        let display = labcloud::lesson::shuffled(3);
        // Pick the display positions that hold content steps 0, 1, 2.
        l.picked = (0..3).map(|step| display.iter().position(|&d| d == step).unwrap()).collect();
        assert_eq!(view::answer(&exs[3], &l), Some(Answer::Order(vec![0, 1, 2])));
        l.picked.pop();
        assert_eq!(view::answer(&exs[3], &l), None, "incomplete answers cannot be checked");
    }

    #[test]
    fn a_finished_lesson_shows_stars_and_a_way_to_perfect_it() {
        let rt = render(state("lesson-done"));
        assert!(rt.find("lesson-result").is_some());
        assert!(rt.find("perfect").is_some());
        assert!(rt.find("back-to-map").is_some());
    }

    #[test]
    fn the_lab_screen_shows_objectives_terminal_and_a_way_back() {
        let rt = render(state("lab"));
        assert!(rt.find("input").is_some());
        assert!(rt.find("goal-2").is_some());
        assert!(rt.find("close").is_some());
    }
}
