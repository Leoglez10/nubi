//! Everything the UI can ask for, as plain methods over the `nubi` engine. `main.rs` wraps
//! each one in a Tauri command; tests call them directly.

use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use nubi::course::{Content, NodeKind};
use nubi::grading::{self, Answer, Level, Stop};
use nubi::lab;
use nubi::lesson::{self, LessonSession};
use nubi::progress::{Profile, Progress, Store};
use nubi::setup::{self, Action, Need, Os, Report};
use nubi::{checker, emulator};
use serde::Serialize;

pub struct Engine {
    pub content: Content,
    store: Store,
    root: PathBuf,
    state: Mutex<Signed>,
}

/// The signed-in student and the attempts in play.
#[derive(Default)]
struct Signed {
    profile: Option<Profile>,
    /// `None` when the saved progress could not be read: then nothing is saved over it.
    progress: Option<Progress>,
    lesson: Option<(String, LessonSession)>,
    lab: Option<LabRun>,
}

struct LabRun {
    id: String,
    dir: PathBuf,
    session: lab::Session,
    saved: bool,
}

#[derive(Serialize, Debug)]
pub struct ProfileCard {
    pub profile: Profile,
    pub stars: u32,
}

#[derive(Serialize, Debug)]
pub struct StopView {
    pub id: String,
    pub status: Stop,
}

/// What the dashboard shows about the signed-in student.
#[derive(Serialize, Debug)]
pub struct ProgressView {
    pub profile: Profile,
    pub progress: Progress,
    pub level: Level,
    pub stops: Vec<StopView>,
    /// False when the saved progress was unreadable: the app plays but does not save.
    pub saving: bool,
}

#[derive(Serialize)]
pub struct LessonView<'a> {
    pub id: String,
    pub session: &'a LessonSession,
    pub finished: bool,
    pub progress: f32,
    /// Set once a finished attempt is recorded.
    pub saved: Option<ProgressView>,
}

#[derive(Serialize)]
pub struct LabView<'a> {
    pub id: String,
    pub session: &'a lab::Session,
    pub complete: bool,
    /// Set once a completed lab is recorded.
    pub saved: Option<ProgressView>,
}

#[derive(Serialize, Debug)]
pub struct NeedView {
    pub need: Need,
    pub action: Action,
}

#[derive(Serialize, Debug)]
pub struct SetupView {
    pub report: Report,
    pub needs: Vec<NeedView>,
    pub ready: bool,
    pub os: &'static str,
}

impl Engine {
    pub fn new(content: Content, root: PathBuf) -> Engine {
        Engine { content, store: Store::new(&root), root, state: Mutex::default() }
    }

    fn state(&self) -> std::sync::MutexGuard<'_, Signed> {
        // ponytail: a panic while holding the lock leaves plain data; keep serving it.
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }

    // --- Setup ---

    pub async fn probe(&self) -> SetupView {
        let (docker, docker_running, aws_cli) = tokio_blocking(setup::probe_tools).await;
        let emulator = docker_running && emulator::healthy().await;
        let report = Report { docker, docker_running, aws_cli, emulator };
        let brew = tokio_blocking(setup::has_brew).await;
        let os = Os::current();
        let needs = report.needs().into_iter().map(|need| NeedView { need, action: setup::action(need, os, brew) }).collect();
        let os = match os { Os::Mac => "mac", Os::Windows => "windows", Os::Linux => "linux" };
        SetupView { report, needs, ready: report.ready(), os }
    }

    /// Runs the one-click fix. `Ok(Some(note))` is something to tell the student;
    /// `Ok(None)` means done, probe again.
    pub async fn fix(&self, need: Need) -> Result<Option<String>, String> {
        match setup::action(need, Os::current(), tokio_blocking(setup::has_brew).await) {
            Action::Run { program, args } => {
                let out = tokio_blocking(move || Command::new(program).args(&args).output()).await;
                match out {
                    Ok(out) if out.status.success() => {
                        if need == Need::DockerRunning {
                            tokio_blocking(|| wait_until(Duration::from_secs(90), || setup::probe_tools().1)).await;
                        }
                        Ok(None)
                    }
                    Ok(out) => Err(format!("No se pudo completar: {}", last_line(&out.stderr))),
                    Err(e) => Err(format!("No se pudo ejecutar {program}: {e}")),
                }
            }
            Action::OpenUrl(url) => {
                open_url(url);
                Ok(Some("Abrimos la página oficial de descarga. Cuando termines de instalar, pulsa «Comprobar de nuevo».".into()))
            }
            Action::StartEmulator => {
                tokio_blocking(emulator::start).await.map_err(|e| format!("No se pudo iniciar el emulador: {e}"))?;
                let start = Instant::now();
                while start.elapsed() < Duration::from_secs(90) {
                    if emulator::healthy().await {
                        return Ok(None);
                    }
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
                Err(format!("El emulador no respondió a tiempo. Revisa: docker logs {}", emulator::CONTAINER))
            }
            Action::Manual(text) => Ok(Some(text.to_string())),
        }
    }

    // --- Profiles ---

    pub fn profiles(&self) -> Vec<ProfileCard> {
        self.store
            .profiles()
            .unwrap_or_default()
            .into_iter()
            .map(|p| {
                let stars = self.store.load_progress(&p.id).map_or(0, |pr| pr.total_stars());
                ProfileCard { profile: p, stars }
            })
            .collect()
    }

    pub fn create_profile(&self, name: &str, color: &str) -> Result<ProgressView, String> {
        let p = self.store.create_profile(name, color).map_err(|e| format!("No se pudo crear el perfil: {e}"))?;
        self.select_profile(&p.id)
    }

    pub fn select_profile(&self, id: &str) -> Result<ProgressView, String> {
        let profile = self.store.profiles().unwrap_or_default().into_iter().find(|p| p.id == id);
        let profile = profile.ok_or_else(|| "Ese perfil ya no existe.".to_string())?;
        let progress = match self.store.load_progress(id) {
            Ok(p) => Some(p),
            Err(e) => {
                eprintln!("nubi: could not read saved progress, it will not be overwritten: {e}");
                None
            }
        };
        let mut st = self.state();
        *st = Signed { profile: Some(profile), progress, ..Signed::default() };
        Ok(self.view(&st).expect("just signed in"))
    }

    pub fn update_profile(&self, profile: Profile) -> Result<ProgressView, String> {
        self.store.update_profile(&profile).map_err(|e| format!("No se pudieron guardar los cambios: {e}"))?;
        let mut st = self.state();
        if st.profile.as_ref().is_some_and(|p| p.id == profile.id) {
            st.profile = Some(profile);
        }
        self.view(&st).ok_or_else(|| "No hay un perfil abierto.".to_string())
    }

    pub fn delete_profile(&self, id: &str) -> Result<(), String> {
        self.store.delete_profile(id).map_err(|e| format!("No se pudo eliminar el perfil: {e}"))?;
        let mut st = self.state();
        if st.profile.as_ref().is_some_and(|p| p.id == id) {
            *st = Signed::default();
        }
        Ok(())
    }

    pub fn sign_out(&self) {
        *self.state() = Signed::default();
    }

    pub fn progress(&self) -> Option<ProgressView> {
        self.view(&self.state())
    }

    fn view(&self, st: &Signed) -> Option<ProgressView> {
        let profile = st.profile.clone()?;
        let progress = st.progress.clone().unwrap_or_default();
        let order = self.content.order();
        let stars_of = |id: &str| progress.nodes.get(id).map(|r| r.best_stars);
        let stops = order.iter().map(|id| StopView { id: id.to_string(), status: grading::stop_status(&order, id, stars_of) }).collect();
        Some(ProgressView { level: grading::level(progress.total_stars()), stops, saving: st.progress.is_some(), profile, progress })
    }

    /// Records a finished attempt and saves it.
    fn record(&self, st: &mut Signed, id: &str, errors: u32) -> Result<(), String> {
        let (Some(profile), Some(progress)) = (st.profile.as_ref(), st.progress.as_mut()) else { return Ok(()) };
        progress.record(id, errors);
        self.store.save_progress(&profile.id, progress).map_err(|e| format!("No se pudo guardar tu progreso: {e}"))
    }

    // --- Lessons ---

    pub fn lesson_start(&self, id: &str) -> Result<serde_json::Value, String> {
        let node = self.content.nodes.get(id).filter(|n| n.kind == NodeKind::Lesson).ok_or("Esa práctica no existe.")?;
        self.state().lesson = Some((id.to_string(), LessonSession::new(node.exercises.len())));
        self.lesson_view(None)
    }

    pub fn lesson_check(&self, answer: Answer) -> Result<serde_json::Value, String> {
        {
            let mut st = self.state();
            let (id, session) = st.lesson.as_mut().ok_or("No hay una práctica en curso.")?;
            let exercise = self.content.nodes[id.as_str()].exercises.get(session.index).ok_or("La práctica ya terminó.")?;
            session.check(exercise, &answer);
        }
        self.lesson_view(None)
    }

    pub fn lesson_retry(&self) -> Result<serde_json::Value, String> {
        self.state().lesson.as_mut().ok_or("No hay una práctica en curso.")?.1.retry();
        self.lesson_view(None)
    }

    /// On to the next exercise; past the last one the attempt is recorded.
    pub fn lesson_next(&self) -> Result<serde_json::Value, String> {
        let mut st = self.state();
        let (id, session) = st.lesson.as_mut().ok_or("No hay una práctica en curso.")?;
        let was_finished = session.finished();
        session.next();
        let (id, errors, finished) = (id.clone(), session.errors, session.finished());
        let saved = if finished && !was_finished {
            self.record(&mut st, &id, errors)?;
            self.view(&st)
        } else {
            None
        };
        drop(st);
        self.lesson_view(saved)
    }

    fn lesson_view(&self, saved: Option<ProgressView>) -> Result<serde_json::Value, String> {
        let st = self.state();
        let (id, session) = st.lesson.as_ref().ok_or("No hay una práctica en curso.")?;
        let view = LessonView { id: id.clone(), session, finished: session.finished(), progress: session.progress(), saved };
        serde_json::to_value(view).map_err(|e| e.to_string())
    }

    // --- Labs ---

    pub async fn lab_start(&self, id: &str) -> Result<serde_json::Value, String> {
        let node = self.content.nodes.get(id).filter(|n| n.kind == NodeKind::Lab).ok_or("Ese laboratorio no existe.")?;
        let dir = lab::sandbox(&self.root, &node.id, &node.files).map_err(|e| format!("No se pudo preparar la carpeta del lab: {e}"))?;
        let s3 = checker::client().await;
        checker::reset(&s3, &node.objectives).await;
        let done = checker::evaluate(&s3, &node.objectives).await;
        let mut session = lab::Session::new(node.objectives.len());
        session.update(&done);
        self.state().lab = Some(LabRun { id: id.to_string(), dir, session, saved: false });
        self.lab_view(None)
    }

    /// One line typed in the lab terminal: built-ins answer at once, `aws` commands run
    /// against the emulator and the objectives are checked again.
    pub async fn lab_submit(&self, line: &str) -> Result<serde_json::Value, String> {
        let (id, dir, args) = {
            let mut st = self.state();
            let run = st.lab.as_mut().ok_or("No hay un laboratorio en curso.")?;
            let hints: Vec<String> = self.content.nodes[run.id.as_str()].objectives.iter().map(|o| o.hint.clone()).collect();
            (run.id.clone(), run.dir.clone(), run.session.submit(line, &hints))
        };
        let Some(args) = args else { return self.lab_view(None) };
        let (ok, out, err) = tokio_blocking(move || lab::run_aws(&args, &dir)).await;
        let done = checker::evaluate(&checker::client().await, &self.content.nodes[id.as_str()].objectives).await;
        let mut st = self.state();
        let run = st.lab.as_mut().filter(|r| r.id == id).ok_or("El laboratorio cambió mientras corría el comando.")?;
        run.session.finish_run(ok, &out, &err);
        run.session.update(&done);
        let finish = (run.session.complete() && !run.saved).then_some(run.session.errors);
        let saved = match finish {
            Some(errors) => {
                st.lab.as_mut().expect("checked above").saved = true;
                self.record(&mut st, &id, errors)?;
                self.view(&st)
            }
            None => None,
        };
        drop(st);
        self.lab_view(saved)
    }

    /// ↑ (`older`) / ↓ through the commands typed so far.
    pub fn lab_history(&self, older: bool) -> Option<String> {
        let mut st = self.state();
        let session = &mut st.lab.as_mut()?.session;
        let line = if older { session.history_prev() } else { session.history_next() };
        line.map(String::from)
    }

    fn lab_view(&self, saved: Option<ProgressView>) -> Result<serde_json::Value, String> {
        let st = self.state();
        let run = st.lab.as_ref().ok_or("No hay un laboratorio en curso.")?;
        let view = LabView { id: run.id.clone(), session: &run.session, complete: run.session.complete(), saved };
        serde_json::to_value(view).map_err(|e| e.to_string())
    }
}

async fn tokio_blocking<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    tokio::task::spawn_blocking(f).await.expect("blocking task")
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

/// Display order for items to arrange: never the answer order.
pub fn shuffled(n: usize) -> Vec<usize> {
    lesson::shuffled(n)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nubi::course::Exercise;
    use serde_json::Value;

    fn engine(tag: &str) -> (Engine, PathBuf) {
        let root = std::env::temp_dir().join(format!("nubi-engine-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        (Engine::new(nubi::course::embedded().unwrap(), root.clone()), root)
    }

    fn right(e: &Exercise) -> Answer {
        match e {
            Exercise::Choice { answer, .. } => Answer::Choice(*answer),
            Exercise::Command { answers, .. } => Answer::Command(answers[0].clone()),
            Exercise::Order { steps, .. } => Answer::Order((0..steps.len()).collect()),
            Exercise::Match { pairs, .. } => Answer::Match((0..pairs.len()).map(|i| (i, i)).collect()),
        }
    }

    #[test]
    fn a_new_profile_starts_at_the_first_stop() {
        let (e, root) = engine("new");
        let v = e.create_profile("Alex", "#BFE0FB").unwrap();
        assert_eq!(v.profile.name, "Alex");
        assert!(v.saving);
        assert_eq!(v.stops[0].status, Stop::Current);
        assert!(v.stops[1..].iter().all(|s| s.status == Stop::Locked));
        assert_eq!(e.profiles().len(), 1);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn a_finished_practice_is_recorded_and_saved_with_its_errors() {
        let (e, root) = engine("lesson");
        let id = e.create_profile("Alex", "#BFE0FB").unwrap().profile.id;
        let exercises = &e.content.nodes["s3-buckets"].exercises;
        e.lesson_start("s3-buckets").unwrap();
        // One wrong answer first: an error and a mistake, then retry.
        let wrong = e.lesson_check(Answer::Command("nada".into())).unwrap();
        assert_eq!(wrong["session"]["feedback"]["type"], "wrong");
        assert_eq!(wrong["session"]["errors"], 1);
        e.lesson_retry().unwrap();
        let mut last = Value::Null;
        for ex in exercises {
            assert_eq!(e.lesson_check(right(ex)).unwrap()["session"]["feedback"]["type"], "right");
            last = e.lesson_next().unwrap();
        }
        assert_eq!(last["finished"], true);
        assert_eq!(last["saved"]["progress"]["nodes"]["s3-buckets"]["best_errors"], 1);
        // Saved to disk: a fresh engine reads the same stars.
        let again = Engine::new(nubi::course::embedded().unwrap(), root.clone()).select_profile(&id).unwrap();
        assert_eq!(again.progress.nodes["s3-buckets"].best_stars, nubi::grading::stars(1));
        assert_eq!(again.stops[1].status, Stop::Current, "the lab opens after the practice");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn deleting_the_open_profile_signs_out() {
        let (e, root) = engine("delete");
        let mut p = e.create_profile("Alex", "#BFE0FB").unwrap().profile;
        p.name = "Sam".into();
        assert_eq!(e.update_profile(p.clone()).unwrap().profile.name, "Sam");
        e.delete_profile(&p.id).unwrap();
        assert!(e.progress().is_none());
        assert!(e.profiles().is_empty());
        assert!(e.select_profile(&p.id).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn answers_arrive_as_tagged_json() {
        let a: Answer = serde_json::from_str(r#"{"type":"order","value":[2,0,1]}"#).unwrap();
        assert_eq!(a, Answer::Order(vec![2, 0, 1]));
        let m: Answer = serde_json::from_str(r#"{"type":"match","value":[[0,1]]}"#).unwrap();
        assert_eq!(m, Answer::Match(vec![(0, 1)]));
    }

    #[test]
    fn lab_commands_need_a_lab_in_play() {
        let (e, _) = engine("nolab");
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        assert!(rt.block_on(e.lab_submit("aws s3 ls")).is_err());
        assert_eq!(e.lab_history(true), None);
    }

    /// Needs the emulator running: `cargo test -p nubi-desktop -- --include-ignored`.
    #[test]
    #[ignore]
    fn a_lab_rejects_foreign_commands_and_runs_aws() {
        nubi::emulator::point_at_emulator();
        let (e, root) = engine("lab");
        let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
        rt.block_on(async {
            e.lab_start("lab-primer-bucket").await.unwrap();
            let v = e.lab_submit("ls").await.unwrap();
            assert_eq!(v["session"]["lines"].as_array().unwrap().last().unwrap()["kind"], "err");
            let v = e.lab_submit("aws s3api create-bucket --bucket mi-primer-bucket").await.unwrap();
            assert_eq!(v["session"]["done"][0], true);
        });
        assert_eq!(e.lab_history(true).as_deref(), Some("aws s3api create-bucket --bucket mi-primer-bucket"));
        std::fs::remove_dir_all(root).unwrap();
    }
}
