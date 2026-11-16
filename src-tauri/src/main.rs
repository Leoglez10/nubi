//! Nubi desktop: a Tauri window over the `nubi` engine. The UI lives in `ui/`; every
//! command here is a thin wrapper over [`engine::Engine`].

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod engine;

use engine::{Engine, ProfileCard, ProgressView, SetupView};
use nubi::course;
use nubi::grading::Answer;
use nubi::progress::{Profile, Store};
use nubi::setup::Need;
use serde_json::Value;
use tauri::State;

type R<T> = Result<T, String>;

#[tauri::command]
fn course(engine: State<'_, Engine>) -> R<Value> {
    serde_json::to_value(&engine.content).map_err(|e| e.to_string())
}

#[tauri::command]
async fn probe(engine: State<'_, Engine>) -> R<SetupView> {
    Ok(engine.probe().await)
}

#[tauri::command]
async fn fix(engine: State<'_, Engine>, need: Need) -> R<Option<String>> {
    engine.fix(need).await
}

#[tauri::command]
fn profiles(engine: State<'_, Engine>) -> Vec<ProfileCard> {
    engine.profiles()
}

#[tauri::command]
fn create_profile(engine: State<'_, Engine>, name: String, color: String) -> R<ProgressView> {
    engine.create_profile(&name, &color)
}

#[tauri::command]
fn select_profile(engine: State<'_, Engine>, id: String) -> R<ProgressView> {
    engine.select_profile(&id)
}

#[tauri::command]
fn update_profile(engine: State<'_, Engine>, profile: Profile) -> R<ProgressView> {
    engine.update_profile(profile)
}

#[tauri::command]
fn delete_profile(engine: State<'_, Engine>, id: String) -> R<()> {
    engine.delete_profile(&id)
}

#[tauri::command]
fn sign_out(engine: State<'_, Engine>) {
    engine.sign_out()
}

#[tauri::command]
fn progress(engine: State<'_, Engine>) -> Option<ProgressView> {
    engine.progress()
}

#[tauri::command]
fn shuffled(n: usize) -> Vec<usize> {
    engine::shuffled(n)
}

#[tauri::command]
fn lesson_start(engine: State<'_, Engine>, id: String) -> R<Value> {
    engine.lesson_start(&id)
}

#[tauri::command]
fn lesson_check(engine: State<'_, Engine>, answer: Answer) -> R<Value> {
    engine.lesson_check(answer)
}

#[tauri::command]
fn lesson_retry(engine: State<'_, Engine>) -> R<Value> {
    engine.lesson_retry()
}

#[tauri::command]
fn lesson_next(engine: State<'_, Engine>) -> R<Value> {
    engine.lesson_next()
}

#[tauri::command]
async fn lab_start(engine: State<'_, Engine>, id: String) -> R<Value> {
    engine.lab_start(&id).await
}

#[tauri::command]
async fn lab_submit(engine: State<'_, Engine>, line: String) -> R<Value> {
    engine.lab_submit(&line).await
}

#[tauri::command]
fn lab_history(engine: State<'_, Engine>, older: bool) -> Option<String> {
    engine.lab_history(older)
}

fn main() {
    nubi::emulator::point_at_emulator();
    let content = course::embedded().expect("embedded course content is valid");
    tauri::Builder::default()
        .manage(Engine::new(content, Store::default_root()))
        .invoke_handler(tauri::generate_handler![
            course, probe, fix, profiles, create_profile, select_profile, update_profile, delete_profile, sign_out,
            progress, shuffled, lesson_start, lesson_check, lesson_retry, lesson_next, lab_start, lab_submit, lab_history
        ])
        .run(tauri::generate_context!())
        .expect("run the Nubi window");
}
