//! Nubi desktop: a Tauri window over the `nubi` engine. The UI lives in `ui/`.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use nubi::course::{self, Content};

/// The whole course, as the UI renders it.
#[tauri::command]
fn course(content: tauri::State<'_, Content>) -> &Content {
    content.inner()
}

fn main() {
    nubi::emulator::point_at_emulator();
    let content = course::embedded().expect("embedded course content is valid");
    tauri::Builder::default()
        .manage(content)
        .invoke_handler(tauri::generate_handler![course])
        .run(tauri::generate_context!())
        .expect("run the Nubi window");
}
