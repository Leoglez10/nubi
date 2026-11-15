//! Local profiles and progress. Everything lives on the student's machine:
//! `<root>/profiles/<id>/profile.json` and `progress.json`, written atomically.

use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::grading::stars;

/// Best result for one stop. Replaying never lowers it.
#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
pub struct NodeResult {
    pub best_stars: u8,
    pub best_errors: u32,
    pub attempts: u32,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
pub struct Progress {
    pub nodes: BTreeMap<String, NodeResult>,
}

impl Progress {
    /// Records a finished attempt and returns the stop's best result.
    pub fn record(&mut self, id: &str, errors: u32) -> &NodeResult {
        let r = self.nodes.entry(id.to_string()).or_default();
        if r.attempts == 0 || errors < r.best_errors {
            r.best_errors = errors;
            r.best_stars = stars(errors);
        }
        r.attempts += 1;
        r
    }

    pub fn completed(&self, id: &str) -> bool {
        self.nodes.get(id).is_some_and(|r| r.best_stars > 0)
    }

    pub fn total_stars(&self) -> u32 {
        self.nodes.values().map(|r| u32::from(r.best_stars)).sum()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct Profile {
    pub id: String,
    pub name: String,
    /// Avatar cloud color, e.g. "#BFE0FB".
    pub color: String,
    /// The student's motion preference; profiles saved before it existed read as false.
    #[serde(default)]
    pub reduced_motion: bool,
}

pub struct Store {
    root: PathBuf,
}

impl Store {
    pub fn new(root: impl Into<PathBuf>) -> Store {
        Store { root: root.into() }
    }

    /// The OS app-data folder. It keeps the old "LabCloud" name so existing profiles survive the rename to Nubi.
    pub fn default_root() -> PathBuf {
        let env = |k: &str| std::env::var_os(k).map(PathBuf::from);
        let home = env("HOME").or_else(|| env("USERPROFILE")).unwrap_or_default();
        if cfg!(target_os = "macos") {
            home.join("Library/Application Support/LabCloud")
        } else if cfg!(windows) {
            env("APPDATA").unwrap_or(home).join("LabCloud")
        } else {
            env("XDG_DATA_HOME").unwrap_or_else(|| home.join(".local/share")).join("labcloud")
        }
    }

    pub fn profiles(&self) -> io::Result<Vec<Profile>> {
        let dir = self.root.join("profiles");
        let entries = match std::fs::read_dir(&dir) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            other => other?,
        };
        let mut profiles = Vec::new();
        for entry in entries {
            let path = entry?.path().join("profile.json");
            // ponytail: a folder without a readable profile.json is skipped, not fatal.
            if let Ok(p) = read_json::<Profile>(&path) {
                profiles.push(p);
            }
        }
        profiles.sort_by_key(|p| p.name.to_lowercase());
        Ok(profiles)
    }

    pub fn create_profile(&self, name: &str, color: &str) -> io::Result<Profile> {
        let name = valid_name(name)?;
        let base = slug(name);
        let mut id = base.clone();
        let mut n = 2;
        while self.root.join("profiles").join(&id).exists() {
            id = format!("{base}-{n}");
            n += 1;
        }
        let profile = Profile { id, name: name.to_string(), color: color.to_string(), ..Profile::default() };
        let dir = self.dir(&profile.id)?;
        std::fs::create_dir_all(&dir)?;
        write_json(&dir.join("profile.json"), &profile)?;
        Ok(profile)
    }

    /// Saves a changed name, color or preference of an existing profile; its id and progress stay.
    pub fn update_profile(&self, profile: &Profile) -> io::Result<()> {
        let path = self.dir(&profile.id)?.join("profile.json");
        if !path.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "no such profile"));
        }
        let name = valid_name(&profile.name)?.to_string();
        write_json(&path, &Profile { name, ..profile.clone() })
    }

    /// Removes a profile and all its progress. Callers confirm with the student first.
    pub fn delete_profile(&self, id: &str) -> io::Result<()> {
        let dir = self.dir(id)?;
        if !dir.join("profile.json").exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "no such profile"));
        }
        std::fs::remove_dir_all(dir)
    }

    /// A profile with no saved progress yet starts empty. A corrupt file is an error, never
    /// silently replaced, so progress is not lost.
    pub fn load_progress(&self, id: &str) -> io::Result<Progress> {
        match read_json(&self.dir(id)?.join("progress.json")) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Progress::default()),
            other => other,
        }
    }

    pub fn save_progress(&self, id: &str, progress: &Progress) -> io::Result<()> {
        write_json(&self.dir(id)?.join("progress.json"), progress)
    }

    /// One JSON backup with the profile and its progress.
    pub fn export(&self, id: &str) -> io::Result<String> {
        let profile: Profile = read_json(&self.dir(id)?.join("profile.json"))?;
        let backup = Backup { version: 1, profile, progress: self.load_progress(id)? };
        serde_json::to_string_pretty(&backup).map_err(io::Error::other)
    }

    /// Restores a backup as a new profile; never overwrites an existing one.
    pub fn import(&self, backup: &str) -> io::Result<Profile> {
        let b: Backup = serde_json::from_str(backup).map_err(|e| invalid(&e.to_string()))?;
        if b.version != 1 {
            return Err(invalid("unsupported backup version"));
        }
        // The backup's id is not trusted as a path: a fresh one is derived from the name.
        let profile = self.create_profile(&b.profile.name, &b.profile.color)?;
        self.save_progress(&profile.id, &b.progress)?;
        Ok(profile)
    }

    fn dir(&self, id: &str) -> io::Result<PathBuf> {
        let safe = !id.is_empty() && id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
        if !safe {
            return Err(invalid("invalid profile id"));
        }
        Ok(self.root.join("profiles").join(id))
    }
}

#[derive(Serialize, Deserialize)]
struct Backup {
    version: u32,
    profile: Profile,
    progress: Progress,
}

fn valid_name(name: &str) -> io::Result<&str> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 40 {
        return Err(invalid("the name must have 1 to 40 characters"));
    }
    Ok(name)
}

fn invalid(msg: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, msg.to_string())
}

/// "Ána María" → "ana-maria"; names with no latin letters or digits → "perfil".
fn slug(name: &str) -> String {
    let mut out = String::new();
    for c in name.to_lowercase().chars() {
        let c = match c {
            'á' | 'à' | 'ä' | 'â' => 'a',
            'é' | 'è' | 'ë' | 'ê' => 'e',
            'í' | 'ì' | 'ï' | 'î' => 'i',
            'ó' | 'ò' | 'ö' | 'ô' => 'o',
            'ú' | 'ù' | 'ü' | 'û' => 'u',
            'ñ' => 'n',
            'ç' => 'c',
            c => c,
        };
        if c.is_ascii_alphanumeric() {
            out.push(c);
        } else if !out.is_empty() && !out.ends_with('-') {
            out.push('-');
        }
    }
    let out = out.trim_end_matches('-').to_string();
    if out.is_empty() { "perfil".into() } else { out }
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> io::Result<T> {
    let text = std::fs::read_to_string(path)?;
    serde_json::from_str(&text).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("{}: {e}", path.display())))
}

/// Writes to `<file>.tmp`, flushes it to disk, then renames it over `<file>`: a crash leaves
/// either the old file or the new one, never half of one.
fn write_json<T: Serialize>(path: &Path, value: &T) -> io::Result<()> {
    let tmp = path.with_extension("json.tmp");
    let bytes = serde_json::to_vec_pretty(value).map_err(io::Error::other)?;
    {
        let mut f = std::fs::File::create(&tmp)?;
        std::io::Write::write_all(&mut f, &bytes)?;
        f.sync_all()?;
    }
    std::fs::rename(&tmp, path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// A scratch store outside the worktree, removed on drop.
    struct Scratch(PathBuf);
    impl Scratch {
        fn new(name: &str) -> Scratch {
            let dir = std::env::temp_dir().join(format!("labcloud-store-{name}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&dir);
            Scratch(dir)
        }
        fn store(&self) -> Store {
            Store::new(&self.0)
        }
    }
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn editing_a_profile_keeps_its_id_and_progress() {
        let scratch = Scratch::new("edit");
        let store = scratch.store();
        let alex = store.create_profile("Alex", "#BFE0FB").unwrap();
        let mut progress = Progress::default();
        progress.record("s3-buckets", 0);
        store.save_progress(&alex.id, &progress).unwrap();

        let edited = Profile { name: "Alexa".into(), color: "#FFE2B8".into(), reduced_motion: true, ..alex.clone() };
        store.update_profile(&edited).unwrap();
        assert_eq!(store.profiles().unwrap(), vec![edited.clone()]);
        assert_eq!(store.load_progress(&alex.id).unwrap(), progress, "progress survives a rename");

        let blank = Profile { name: "  ".into(), ..edited.clone() };
        assert!(store.update_profile(&blank).is_err(), "an empty name is rejected");
        let missing = Profile { id: "nadie".into(), ..edited };
        assert!(store.update_profile(&missing).is_err(), "editing never creates a profile");
        assert_eq!(store.profiles().unwrap().len(), 1);
    }

    #[test]
    fn deleting_a_profile_removes_only_that_profile() {
        let scratch = Scratch::new("delete");
        let store = scratch.store();
        let alex = store.create_profile("Alex", "#BFE0FB").unwrap();
        let sam = store.create_profile("Sam", "#FFE2B8").unwrap();
        store.save_progress(&alex.id, &Progress::default()).unwrap();
        store.delete_profile(&alex.id).unwrap();
        assert_eq!(store.profiles().unwrap(), vec![sam.clone()]);
        assert!(!scratch.0.join("profiles").join(&alex.id).exists());
        assert!(store.delete_profile("../profiles").is_err(), "ids are never paths");
        assert!(store.delete_profile("nadie").is_err(), "deleting a missing profile reports it");
        assert_eq!(store.profiles().unwrap(), vec![sam]);
    }

    #[test]
    fn profiles_saved_before_the_motion_preference_still_load() {
        let scratch = Scratch::new("legacy");
        let dir = scratch.0.join("profiles").join("alex");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("profile.json"), r##"{"id":"alex","name":"Alex","color":"#BFE0FB"}"##).unwrap();
        let profiles = scratch.store().profiles().unwrap();
        assert_eq!(profiles.len(), 1);
        assert!(!profiles[0].reduced_motion);
    }

    #[test]
    fn replaying_keeps_the_best_result_and_counts_attempts() {
        let mut p = Progress::default();
        assert_eq!(p.record("a", 3), &NodeResult { best_stars: 1, best_errors: 3, attempts: 1 });
        assert_eq!(p.record("a", 0), &NodeResult { best_stars: 3, best_errors: 0, attempts: 2 });
        assert_eq!(p.record("a", 5), &NodeResult { best_stars: 3, best_errors: 0, attempts: 3 }, "a worse replay never lowers it");
    }

    #[test]
    fn total_stars_and_completion_come_from_best_results() {
        let mut p = Progress::default();
        p.record("a", 0);
        p.record("b", 2);
        assert_eq!(p.total_stars(), 5);
        assert!(p.completed("a") && p.completed("b"));
        assert!(!p.completed("c"));
    }

    #[test]
    fn profiles_and_progress_survive_a_restart() {
        let s = Scratch::new("roundtrip");
        let alex = s.store().create_profile("Alex", "#BFE0FB").unwrap();
        let ana = s.store().create_profile("Ána María", "#FFE2B8").unwrap();
        assert_eq!(ana.id, "ana-maria");
        assert!(s.0.join("profiles/alex/profile.json").is_file());

        let mut p = Progress::default();
        p.record("s3-buckets", 1);
        s.store().save_progress(&alex.id, &p).unwrap();

        let again = s.store();
        let names: Vec<_> = again.profiles().unwrap().into_iter().map(|p| p.name).collect();
        assert_eq!(names, vec!["Alex", "Ána María"]);
        assert_eq!(again.load_progress(&alex.id).unwrap(), p);
        assert_eq!(again.load_progress(&ana.id).unwrap(), Progress::default(), "no progress yet");
        assert!(!s.0.join("profiles/alex/progress.json.tmp").exists(), "no temp file left behind");
    }

    #[test]
    fn same_names_get_different_profiles() {
        let s = Scratch::new("dupes");
        let a = s.store().create_profile("Sam", "#fff").unwrap();
        let b = s.store().create_profile("sam", "#fff").unwrap();
        assert_ne!(a.id, b.id);
        assert_eq!(s.store().profiles().unwrap().len(), 2);
    }

    #[test]
    fn bad_names_and_ids_are_rejected() {
        let s = Scratch::new("bad");
        assert_eq!(s.store().create_profile("   ", "#fff").unwrap_err().kind(), io::ErrorKind::InvalidInput);
        assert_eq!(s.store().load_progress("../../etc").unwrap_err().kind(), io::ErrorKind::InvalidInput);
        assert_eq!(s.store().create_profile("日本", "#fff").unwrap().id, "perfil", "names without latin letters still work");
    }

    #[test]
    fn corrupt_progress_is_an_error_not_a_reset() {
        let s = Scratch::new("corrupt");
        let alex = s.store().create_profile("Alex", "#fff").unwrap();
        fs::write(s.0.join("profiles/alex/progress.json"), "{ not json").unwrap();
        assert!(s.store().load_progress(&alex.id).is_err());
        assert_eq!(fs::read_to_string(s.0.join("profiles/alex/progress.json")).unwrap(), "{ not json", "left untouched");
    }

    #[test]
    fn a_backup_restores_on_another_machine_without_overwriting() {
        let from = Scratch::new("export");
        let alex = from.store().create_profile("Alex", "#BFE0FB").unwrap();
        let mut p = Progress::default();
        p.record("s3-buckets", 0);
        from.store().save_progress(&alex.id, &p).unwrap();
        let backup = from.store().export(&alex.id).unwrap();

        let to = Scratch::new("import");
        let restored = to.store().import(&backup).unwrap();
        assert_eq!((restored.name.as_str(), restored.color.as_str()), ("Alex", "#BFE0FB"));
        assert_eq!(to.store().load_progress(&restored.id).unwrap(), p);

        let twice = to.store().import(&backup).unwrap();
        assert_ne!(twice.id, restored.id, "a second import is a new profile");
        assert!(to.store().import("garbage").is_err());
    }
}
