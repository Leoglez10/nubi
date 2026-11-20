//! Nubi (Lab Cloud) engine: course content, grading, local progress and the lab checker.
//! The Tauri app in `src-tauri/` drives it.

pub mod checker;
pub mod course;
pub mod emulator;
pub mod grading;
pub mod lab;
pub mod lesson;
pub mod progress;
pub mod setup;

pub mod dynamo_lab;
